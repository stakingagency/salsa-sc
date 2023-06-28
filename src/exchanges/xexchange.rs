multiversx_sc::imports!();

use crate::common::config::State;
use crate::common::storage_cache::StorageCache;
use crate::{common::consts::*, common::errors::*};
use crate::proxies::xexchange_proxy;
use crate::proxies::wrap_proxy;

use super::xexchange_cache::{XexchangeCache};

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

        if self.xexchange_lp().is_empty() {
            let xexchange_sc_address = self.xexchange_sc().get();
            let lp: TokenIdentifier = self.xexchange_proxy_obj()
                .contract(xexchange_sc_address)
                .get_lp_token_identifier()
                .execute_on_dest_context();
            self.xexchange_lp().set(lp);
        }

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

    #[storage_mapper("xexchange_lp")]
    fn xexchange_lp(&self) -> SingleValueMapper<TokenIdentifier>;

    #[only_owner]
    #[endpoint(setXexchangeSC)]
    fn set_xexchange_sc(&self, address: ManagedAddress) {
        self.xexchange_sc().set(address);
    }

    fn get_xexchange_reserves(&self) -> (BigUint, BigUint, BigUint) {
        let xexchange_sc_address = self.xexchange_sc().get();
        let res: MultiValue3<BigUint, BigUint, BigUint> = self.xexchange_proxy_obj()
            .contract(xexchange_sc_address)
            .get_reserves_and_total_supply()
            .execute_on_dest_context();
        let (ls_reserve, egld_reserve, lp_supply) = res.into_tuple();

        (ls_reserve, egld_reserve, lp_supply)
    }

    fn get_xexchange_amount_out(
        &self,
        is_buy: bool,
        in_amount: &BigUint,
        storage_cache: &StorageCache<Self>,
        xexchange_cache: &XexchangeCache<Self>
    ) -> BigUint {
        let in_token = if is_buy {
            storage_cache.wegld_id.clone()
        } else {
            storage_cache.liquid_token_id.clone()
        };
        self.xexchange_proxy_obj()
            .contract(xexchange_cache.sc_address.clone())
            .get_amount_out_view(in_token, in_amount)
            .execute_on_dest_context()
    }

    fn do_arbitrage_on_xexchange(
        &self, is_buy: bool, in_amount: BigUint, storage_cache: &mut StorageCache<Self>, xexchange_cache: XexchangeCache<Self>,
    ) -> (BigUint, BigUint) {
        let out_amount = self.get_salsa_amount_out(&in_amount, is_buy, storage_cache);
        let amount_to_send_to_xexchange =
            self.get_optimal_quantity(
                in_amount, out_amount, xexchange_cache.egld_reserve.clone(), xexchange_cache.liquid_reserve.clone(), is_buy
            );
        if amount_to_send_to_xexchange < MIN_EGLD {
            return (BigUint::zero(), BigUint::zero())
        }

        let amount_from_xexchange =
            self.get_xexchange_amount_out(is_buy, &amount_to_send_to_xexchange, storage_cache, &xexchange_cache);
        let amount_from_salsa =
            self.get_salsa_amount_out(&amount_to_send_to_xexchange, is_buy, storage_cache);
        if amount_from_xexchange < amount_from_salsa {
            return (BigUint::zero(), BigUint::zero())
        }
        self.swap_on_xexchange(is_buy, &amount_to_send_to_xexchange, &amount_from_salsa, storage_cache, xexchange_cache);

        (amount_to_send_to_xexchange, amount_from_salsa)
    }

    fn swap_on_xexchange(
        &self,
        is_buy: bool,
        in_amount: &BigUint,
        out_amount: &BigUint,
        storage_cache: &StorageCache<Self>,
        xexchange_cache: XexchangeCache<Self>,
    ) {
        if is_buy {
            self.wrap_proxy_obj()
                .contract(xexchange_cache.wrap_sc_address)
                .wrap_egld()
                .with_egld_transfer(in_amount.clone())
                .execute_on_dest_context::<()>();
            let payment =
                EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, in_amount.clone());
            self.xexchange_proxy_obj()
                .contract(xexchange_cache.sc_address)
                .swap_tokens_fixed_input(storage_cache.liquid_token_id.clone(), out_amount)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        } else {
            let mut payment =
                EsdtTokenPayment::new(storage_cache.liquid_token_id.clone(), 0, in_amount.clone());
            self.xexchange_proxy_obj()
                .contract(xexchange_cache.sc_address)
                .swap_tokens_fixed_input(storage_cache.wegld_id.clone(), out_amount)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
            let wegld_balance =
                self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(storage_cache.wegld_id.clone()), 0);
            payment = EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, wegld_balance);
            self.wrap_proxy_obj()
                .contract(xexchange_cache.wrap_sc_address)
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