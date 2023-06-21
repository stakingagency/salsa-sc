multiversx_sc::imports!();

use crate::common::config::State;
use crate::{common::consts::*, common::errors::*};
use crate::proxies::xexchange_proxy;
use crate::proxies::wrap_proxy;

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

    fn do_arbitrage_on_xexchange(
        &self, in_token: &TokenIdentifier, in_amount: &BigUint
    ) -> (BigUint, BigUint) {
        // Comment
        // No need for the extra check
        if !self.is_xexchange_arbitrage_active() {
            return (BigUint::zero(), BigUint::zero())
        }

        let is_buy = in_token == &self.wegld_id().get();
        let out_amount = if is_buy {
            self.add_liquidity(&in_amount, false)
        } else {
            self.remove_liquidity(&in_amount, false)
        };
        let (ls_reserve, egld_reserve) = self.get_xexchange_reserves();
        let mut amount_to_send_to_xexchange = if is_buy {
            self.get_buy_quantity(in_amount.clone(), out_amount.clone(), egld_reserve, ls_reserve)
        } else {
            self.get_sell_quantity(in_amount.clone(), out_amount.clone(), ls_reserve, egld_reserve)
        };

        // Comment
        // Like in the OneDex arbitrage function, you can have this checks and updates in the get_buy_quantity function directly
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
        if in_token == &wegld_token_id {
            self.wrap_proxy_obj()
                .contract(self.wrap_sc().get())
                .wrap_egld()
                .with_egld_transfer(in_amount.clone())
                .execute_on_dest_context::<()>();
            let payment = EsdtTokenPayment::new(wegld_token_id.clone(), 0, in_amount.clone());
            self.xexchange_proxy_obj()
                .contract(xexchange_sc_address)
                .swap_tokens_fixed_input(liquid_token_id, out_amount)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        } else {
            let mut payment = EsdtTokenPayment::new(liquid_token_id, 0, in_amount.clone());
            self.xexchange_proxy_obj()
                .contract(xexchange_sc_address)
                .swap_tokens_fixed_input(wegld_token_id.clone(), out_amount)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
            let wegld_balance =
                self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(wegld_token_id.clone()), 0);
            payment = EsdtTokenPayment::new(wegld_token_id, 0, wegld_balance);
            self.wrap_proxy_obj()
                .contract(self.wrap_sc().get())
                .unwrap_egld()
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
    }

    // proxies

    #[proxy]
    fn xexchange_proxy_obj(&self) -> xexchange_proxy::Proxy<Self::Api>;

    #[proxy]
    fn wrap_proxy_obj(&self) -> wrap_proxy::Proxy<Self::Api>;
}
