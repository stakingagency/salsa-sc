multiversx_sc::imports!();

use crate::common::config::State;
use crate::{common::consts::*, common::errors::*};
use crate::proxies::onedex_proxy;

#[multiversx_sc::module]
pub trait OnedexModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setOnedexArbitrageActive)]
    fn set_onedex_arbitrage_active(&self) {
        require!(
            !self.onedex_pair_id().is_empty(),
            ERROR_ONEDEX_PAIR_ID,
        );
        require!(
            !self.onedex_sc().is_empty(),
            ERROR_ONEDEX_SC,
        );

        self.onedex_arbitrage().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setOnedexArbitrageInactive)]
    fn set_onedex_arbitrage_inactive(&self) {
        self.onedex_arbitrage().set(State::Inactive);
    }

    #[inline]
    fn is_onedex_arbitrage_active(&self) -> bool {
        let arbitrage = self.onedex_arbitrage().get();
        arbitrage == State::Active
    }

    #[view(getOnedexArbitrageState)]
    #[storage_mapper("onedex_arbitrage")]
    fn onedex_arbitrage(&self) -> SingleValueMapper<State>;

    #[storage_mapper("onedex_sc")]
    fn onedex_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("onedex_pair_id")]
    fn onedex_pair_id(&self) -> SingleValueMapper<usize>;

    #[only_owner]
    #[endpoint(setOnedexSC)]
    fn set_onedex_sc(&self, address: ManagedAddress) {
        self.onedex_sc().set(address);
    }

    #[only_owner]
    #[endpoint(setOnedexPairId)]
    fn set_onedex_pair_id(&self, id: usize) {
        self.onedex_pair_id().set(id);
    }

    fn get_onedex_reserves(&self, pair_id: usize) -> (BigUint, BigUint) {
        let onedex_sc_address = self.onedex_sc().get();
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
        if !self.is_onedex_arbitrage_active() {
            return BigUint::zero();
        }

        let onedex_sc_address = self.onedex_sc().get();
        let wegld_token_id = self.wegld_id().get();
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

    fn do_arbitrage_on_onedex(
        &self, in_token: &TokenIdentifier, in_amount: &BigUint, out_amount: &BigUint
    ) -> (BigUint, BigUint) {
        if !self.is_onedex_arbitrage_active() {
            return (BigUint::zero(), BigUint::zero())
        }

        let is_buy = in_token == &self.wegld_id().get();
        let pair_id = self.onedex_pair_id().get();
        let (ls_reserve, egld_reserve) = self.get_onedex_reserves(pair_id);
        let mut amount_to_send_to_onedex = if is_buy {
            self.get_buy_quantity(in_amount.clone(), out_amount.clone(), egld_reserve, ls_reserve)
        } else {
            self.get_sell_quantity(in_amount.clone(), out_amount.clone(), ls_reserve, egld_reserve)
        };
        if amount_to_send_to_onedex < MIN_EGLD {
            return (BigUint::zero(), BigUint::zero())
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
            return (BigUint::zero(), BigUint::zero())
        }
        self.swap_on_onedex(in_token, &amount_to_send_to_onedex, &amount_from_salsa);

        (amount_to_send_to_onedex, amount_from_salsa)
    }

    fn swap_on_onedex(&self, in_token: &TokenIdentifier, in_amount: &BigUint, out_amount: &BigUint) {
        let onedex_sc_address = self.onedex_sc().get();
        let wegld_token_id = self.wegld_id().get();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let mut path: MultiValueEncoded<TokenIdentifier> = MultiValueEncoded::new();
        let is_buy = in_token == &wegld_token_id;
        if is_buy {
            path.push(wegld_token_id);
            path.push(liquid_token_id);
            self.onedex_proxy_obj()
                .contract(onedex_sc_address)
                .swap_multi_tokens_fixed_input(out_amount, false, path)
                .with_egld_transfer(in_amount.clone())
                .execute_on_dest_context::<()>();
        } else {
            path.push(liquid_token_id.clone());
            path.push(wegld_token_id);
            let payment = EsdtTokenPayment::new(liquid_token_id, 0, in_amount.clone());
            self.onedex_proxy_obj()
                .contract(onedex_sc_address)
                .swap_multi_tokens_fixed_input(out_amount, true, path)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
    }

    // proxy

    #[proxy]
    fn onedex_proxy_obj(&self) -> onedex_proxy::Proxy<Self::Api>;
}
