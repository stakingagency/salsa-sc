multiversx_sc::imports!();

use crate::{common::consts::*, common::errors::*};
use crate::proxies::onedex_proxy;

#[multiversx_sc::module]
pub trait OnedexModule:
    crate::common::config::ConfigModule
    + crate::liquidity::LiquidityModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn get_sc_balances(&self) -> (BigUint, BigUint) {
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let ls_balance = self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(liquid_token_id.clone()), 0);
        let balance = self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);

        (balance, ls_balance)
    }

    // onedex

    #[storage_mapper("onedex_fee")]
    fn onedex_fee(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("onedex_pair_id")]
    fn onedex_pair_id(&self) -> SingleValueMapper<usize>;

    #[only_owner]
    #[endpoint(setOnedexPairId)]
    fn set_onedex_pair_id(&self, id: usize) {
        self.onedex_pair_id().set(id);
    }

    fn get_onedex_fee(&self) -> u64 {
        let onedex_sc_address = ManagedAddress::from(ONEDEX_SC);
        self.onedex_proxy_obj()
            .contract(onedex_sc_address.clone())
            .total_fee_percent()
            .execute_on_dest_context()
    }

    fn get_onedex_reserves(&self, pair_id: usize) -> (BigUint, BigUint) {
        let onedex_sc_address = ManagedAddress::from(ONEDEX_SC);
        let ls_reserve: BigUint = self.onedex_proxy_obj()
            .contract(onedex_sc_address.clone())
            .pair_first_token_reserve(pair_id)
            .execute_on_dest_context();
        let egld_reserve: BigUint = self.onedex_proxy_obj()
            .contract(onedex_sc_address.clone())
            .pair_second_token_reserve(pair_id)
            .execute_on_dest_context();

        (ls_reserve, egld_reserve)
    }

    fn get_onedex_amount_out(&self, in_token: &TokenIdentifier, in_amount: &BigUint) -> BigUint {
        if !self.is_arbitrage_active() {
            return BigUint::zero();
        }

        let onedex_sc_address = ManagedAddress::from(ONEDEX_SC);
        let wegld_token_id = TokenIdentifier::from(WEGLD_ID);
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let (first_token, second_token) = if in_token == &wegld_token_id {
            (wegld_token_id, liquid_token_id)
        } else {
            (liquid_token_id, wegld_token_id)
        };
        self.onedex_proxy_obj()
            .contract(onedex_sc_address.clone())
            .get_amount_out_view(&first_token, &second_token, in_amount)
            .execute_on_dest_context()
    }

    fn get_onedex_buy_quantity(&self, uegld_amount: BigUint, uls_amount: BigUint) -> BigUint {
        require!(uls_amount > 0, ERROR_INSUFFICIENT_AMOUNT);

        let fee = self.onedex_fee().get();
        require!(fee > 0, ERROR_FEE_ZERO);

        let pair_id = self.onedex_pair_id().get();
        let (uls_reserve, uegld_reserve) = self.get_onedex_reserves(pair_id);

        let ls_amount = BigInt::from_biguint(Sign::Plus, uls_amount);
        let egld_amount = BigInt::from_biguint(Sign::Plus, uegld_amount.clone());
        let ls_reserve = BigInt::from_biguint(Sign::Plus, uls_reserve);
        let egld_reserve = BigInt::from_biguint(Sign::Plus, uegld_reserve);
        let imax = BigInt::from_biguint(Sign::Plus, BigUint::from(MAX_PERCENT));
        let ifee = BigInt::from_biguint(Sign::Plus, BigUint::from(MAX_PERCENT - fee));

        let mut b = ifee.clone() * ls_reserve * egld_amount / ls_amount / imax.clone();
        b = egld_reserve - b;
        let mut x = BigInt::from_biguint(Sign::Plus, b.magnitude()) - b;
        if x < 0 {
            return BigUint::zero()
        }
        x = x * imax / ifee;

        let opt_x = x.into_big_uint();
        let mut ux = match opt_x.into_option() {
            Some(value) => value,
            None => BigUint::zero(),
        };
        ux /= 2_u64;
        if ux > uegld_amount {
            uegld_amount
        } else {
            ux * ARBITRAGE_RATIO / MAX_PERCENT
        }
    }

    fn get_onedex_sell_quantity(&self, uls_amount: BigUint, uegld_amount: BigUint, ) -> BigUint {
        require!(uegld_amount > 0, ERROR_INSUFFICIENT_AMOUNT);

        let fee = self.onedex_fee().get();
        let pair_id = self.onedex_pair_id().get();
        let (uls_reserve, uegld_reserve) = self.get_onedex_reserves(pair_id);

        let ls_amount = BigInt::from_biguint(Sign::Plus, uls_amount.clone());
        let egld_amount = BigInt::from_biguint(Sign::Plus, uegld_amount);
        let ls_reserve = BigInt::from_biguint(Sign::Plus, uls_reserve);
        let egld_reserve = BigInt::from_biguint(Sign::Plus, uegld_reserve);
        let imax = BigInt::from_biguint(Sign::Plus, BigUint::from(MAX_PERCENT));
        let ifee = BigInt::from_biguint(Sign::Plus, BigUint::from(MAX_PERCENT - fee));

        let mut b = ifee * egld_reserve * ls_amount / egld_amount / imax;
        b = ls_reserve - b;
        let x = BigInt::from_biguint(Sign::Plus, b.magnitude()) - b;
        if x < 0 {
            return BigUint::zero()
        }
        
        let opt_x = x.into_big_uint();
        let mut ux = match opt_x.into_option() {
            Some(value) => value,
            None => BigUint::zero(),
        };
        ux /= 2_u64;
        if ux > uls_amount {
            uls_amount
        } else {
            ux * ARBITRAGE_RATIO / MAX_PERCENT
        }
    }

    fn do_arbitrage_on_onedex(
        &self, in_token: &TokenIdentifier, in_amount: &BigUint, out_amount: &BigUint
    ) -> BigUint {
        if !self.is_arbitrage_active() {
            return BigUint::zero()
        }

        let caller = self.blockchain().get_caller();
        let mut is_buy = false;
        if in_token == &TokenIdentifier::from(WEGLD_ID) {
            is_buy = true;
        }
        let mut amount_to_send_to_onedex = if is_buy {
            self.get_onedex_buy_quantity(in_amount.clone(), out_amount.clone())
        } else {
            self.get_onedex_sell_quantity(in_amount.clone(), out_amount.clone())
        };
        if amount_to_send_to_onedex < MIN_EGLD {
            return BigUint::zero()
        }

        let rest = in_amount - &amount_to_send_to_onedex;
        if rest < MIN_EGLD && rest > 0 {
            amount_to_send_to_onedex = in_amount - MIN_EGLD;
        }
        let amount_from_onedex = self.get_onedex_amount_out(in_token, &amount_to_send_to_onedex);
        let amount_from_salsa = if is_buy {
            self.add_liquidity(&amount_to_send_to_onedex, false)
        } else {
            self.remove_liquidity(&amount_to_send_to_onedex, false)
        };
        if amount_from_onedex < amount_from_salsa {
            return BigUint::zero()
        }
        self.swap_on_onedex(in_token, &amount_to_send_to_onedex, &amount_from_salsa);
        if is_buy {
            let liquid_token_id = self.liquid_token_id().get_token_id();
            self.send().direct_esdt(
                &caller,
                &liquid_token_id,
                0,
                &amount_from_salsa,
            );
        } else {
            self.send().direct_egld(&caller, &amount_from_salsa);
        }

        amount_to_send_to_onedex
    }

    fn swap_on_onedex(&self, in_token: &TokenIdentifier, in_amount: &BigUint, out_amount: &BigUint) {
        let onedex_sc_address = ManagedAddress::from(ONEDEX_SC);
        let wegld_token_id = TokenIdentifier::from(WEGLD_ID);
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let mut path: MultiValueEncoded<TokenIdentifier> = MultiValueEncoded::new();
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();
        let mut is_buy = true;
        if in_token == &wegld_token_id {
            path.push(wegld_token_id);
            path.push(liquid_token_id);
            self.onedex_proxy_obj()
                .contract(onedex_sc_address)
                .swap_multi_tokens_fixed_input(out_amount, false, path)
                .with_egld_transfer(in_amount.clone())
                .execute_on_dest_context::<()>();
        } else {
            is_buy = false;
            path.push(liquid_token_id.clone());
            path.push(wegld_token_id);
            let payment = EsdtTokenPayment::new(liquid_token_id, 0, in_amount.clone());
            self.onedex_proxy_obj()
                .contract(onedex_sc_address)
                .swap_multi_tokens_fixed_input(out_amount, true, path)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();
        if is_buy {
            require!(new_ls_balance >= old_ls_balance, ERROR_ARBITRAGE_ISSUE);

            let swapped_amount = &new_ls_balance - &old_ls_balance;
            require!(&swapped_amount >= out_amount, ERROR_ARBITRAGE_ISSUE);

            let profit = &swapped_amount - out_amount;
            self.liquid_profit()
                .update(|value| *value += profit);
        } else {
            require!(new_egld_balance >= old_egld_balance, ERROR_ARBITRAGE_ISSUE);

            let swapped_amount = &new_egld_balance - &old_egld_balance;
            require!(&swapped_amount >= out_amount, ERROR_ARBITRAGE_ISSUE);

            let profit = swapped_amount - out_amount;
            self.egld_profit()
                .update(|value| *value += profit);
        }
    }

    // proxy

    #[proxy]
    fn onedex_proxy_obj(&self) -> onedex_proxy::Proxy<Self::Api>;
}
