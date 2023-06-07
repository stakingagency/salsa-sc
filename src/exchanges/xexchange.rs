multiversx_sc::imports!();

use crate::common::config::State;
use crate::{common::consts::*, common::errors::*};
use crate::proxies::xexchange_proxy;

#[multiversx_sc::module]
pub trait XexchangeModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setXexchangeArbitrageActive)]
    fn set_xexchange_arbitrage_active(&self) {
        require!(
            !self.xexchange_sc().is_empty(),
            ERROR_XEXCHANGE_SC,
        );

        self.xexchange_arbitrage().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setXexchangeArbitrageInactive)]
    fn set_xexchange_arbitrage_inactive(&self) {
        self.xexchange_arbitrage().set(State::Inactive);
    }

    #[inline]
    fn is_xexchange_arbitrage_active(&self) -> bool {
        let arbitrage = self.xexchange_arbitrage().get();
        arbitrage == State::Active
    }

    #[view(getXexchangeArbitrageState)]
    #[storage_mapper("xexchange_arbitrage")]
    fn xexchange_arbitrage(&self) -> SingleValueMapper<State>;

    #[storage_mapper("xexchange_sc")]
    fn xexchange_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setXexchangeSC)]
    fn set_xexchange_sc(&self, address: ManagedAddress) {
        self.xexchange_sc().set(address);
    }

    fn get_xexchange_reserves(&self) -> (BigUint, BigUint) {
        let xexchange_sc_address = self.xexchange_sc().get();
        let res: MultiValue3<BigUint, BigUint, BigUint> = self.xexchange_proxy_obj()
            .contract(xexchange_sc_address)
            .get_reserves_and_total_supply()
            .execute_on_dest_context();
        let (ls_reserve, egld_reserve, _) = res.into_tuple();

        (ls_reserve, egld_reserve)
    }

    fn get_xexchange_amount_out(&self, in_token: &TokenIdentifier, in_amount: &BigUint) -> BigUint {
        if !self.is_xexchange_arbitrage_active() {
            return BigUint::zero();
        }

        let xexchange_sc_address = self.xexchange_sc().get();
        self.xexchange_proxy_obj()
            .contract(xexchange_sc_address.clone())
            .get_amount_out_view(in_token, in_amount)
            .execute_on_dest_context()
    }

    fn get_xexchange_buy_quantity(&self, egld_amount: BigUint, ls_amount: BigUint) -> BigUint {
        let (ls_reserve, egld_reserve) = self.get_xexchange_reserves();

        self.get_buy_quantity(egld_amount, ls_amount, egld_reserve, ls_reserve)
    }

    fn get_xexchange_sell_quantity(&self, ls_amount: BigUint, egld_amount: BigUint, ) -> BigUint {
        let (ls_reserve, egld_reserve) = self.get_xexchange_reserves();

        self.get_sell_quantity(ls_amount, egld_amount, ls_reserve, egld_reserve)
    }

    fn do_arbitrage_on_xexchange(
        &self, in_token: &TokenIdentifier, in_amount: &BigUint, out_amount: &BigUint
    ) -> (BigUint, BigUint) {
        if !self.is_xexchange_arbitrage_active() {
            return (BigUint::zero(), BigUint::zero())
        }

        let is_buy = in_token == &self.wegld_id().get();
        let mut amount_to_send_to_xexchange = if is_buy {
            self.get_xexchange_buy_quantity(in_amount.clone(), out_amount.clone())
        } else {
            self.get_xexchange_sell_quantity(in_amount.clone(), out_amount.clone())
        };
        if amount_to_send_to_xexchange < MIN_EGLD {
            return (BigUint::zero(), BigUint::zero())
        }

        let rest = in_amount - &amount_to_send_to_xexchange;
        if rest < MIN_EGLD && rest > 0 {
            amount_to_send_to_xexchange = in_amount - MIN_EGLD;
        }
        let amount_from_xexchange = self.get_xexchange_amount_out(in_token, &amount_to_send_to_xexchange);
        let amount_from_salsa = if is_buy {
            self.add_liquidity(&amount_to_send_to_xexchange, false)
        } else {
            self.remove_liquidity(&amount_to_send_to_xexchange, false)
        };
        if amount_from_xexchange < amount_from_salsa {
            return (BigUint::zero(), BigUint::zero())
        }
        self.swap_on_xexchange(in_token, &amount_to_send_to_xexchange, &amount_from_salsa);

        (amount_to_send_to_xexchange, amount_from_salsa)
    }

    fn swap_on_xexchange(&self, in_token: &TokenIdentifier, in_amount: &BigUint, out_amount: &BigUint) {
        let xexchange_sc_address = self.xexchange_sc().get();
        let wegld_token_id = self.wegld_id().get();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();
        let is_buy = in_token == &wegld_token_id;
        if is_buy {
            self.xexchange_proxy_obj()
                .contract(xexchange_sc_address)
                .swap_tokens_fixed_input(liquid_token_id, out_amount)
                .with_egld_transfer(in_amount.clone())
                .execute_on_dest_context::<()>();
        } else {
            let payment = EsdtTokenPayment::new(liquid_token_id, 0, in_amount.clone());
            self.xexchange_proxy_obj()
                .contract(xexchange_sc_address)
                .swap_tokens_fixed_input(wegld_token_id, out_amount)
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
    fn xexchange_proxy_obj(&self) -> xexchange_proxy::Proxy<Self::Api>;
}
