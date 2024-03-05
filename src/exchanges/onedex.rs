multiversx_sc::imports!();

use crate::common::config::State;
use crate::common::storage_cache::StorageCache;
use crate::{common::consts::*, common::errors::*};
use crate::proxies::onedex_proxy::{self, Pair, MAX_PERCENTAGE};

use super::onedex_cache::OnedexCache;

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

        if self.onedex_lp_token().is_empty() {
            let pair = self.get_onedex_pair_info();
            self.onedex_lp_token().set(pair.lp_token_id);
        }

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

    #[storage_mapper("onedex_lp_token")]
    fn onedex_lp_token(&self) -> SingleValueMapper<TokenIdentifier>;

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

    fn get_onedex_pair_info(&self) -> Pair<Self::Api> {
        let pair_id = self.onedex_pair_id().get();
        let onedex_sc_address = self.onedex_sc().get();
        let pair: Pair<Self::Api> = self.onedex_proxy_obj()
            .contract(onedex_sc_address)
            .view_pair(pair_id)
            .execute_on_dest_context();

        pair
    }

    fn get_onedex_fee(&self) -> u64 {
        let onedex_sc_address = self.onedex_sc().get();
        let fee: u64 = self.onedex_proxy_obj()
            .contract(onedex_sc_address)
            .total_fee_percent()
            .execute_on_dest_context();

        fee
    }

    fn get_onedex_amount_out(
        &self,
        is_buy: bool,
        in_amount: &BigUint,
        storage_cache: &mut StorageCache<Self>,
        onedex_cache: &OnedexCache<Self>,
    ) -> BigUint {
        let (first_token, second_token) = if is_buy {
            (storage_cache.wegld_id.clone(), storage_cache.liquid_token_id.clone())
        } else {
            (storage_cache.liquid_token_id.clone(), storage_cache.wegld_id.clone())
        };
        self.onedex_proxy_obj()
            .contract(onedex_cache.sc_address.clone())
            .get_amount_out_view(&first_token, &second_token, in_amount)
            .execute_on_dest_context()
    }

    fn do_arbitrage_on_onedex(
        &self,
        is_buy: bool,
        in_amount: BigUint,
        storage_cache: &mut StorageCache<Self>,
        onedex_cache: &OnedexCache<Self>,
    ) -> (BigUint, BigUint) {
        let out_amount = self.get_salsa_amount_out(&in_amount, is_buy, storage_cache);
        let amount_to_send_to_onedex =
            self.get_optimal_quantity(
                is_buy, onedex_cache.fee, MAX_PERCENTAGE, in_amount, out_amount, &onedex_cache.lp_info.egld_reserve, &onedex_cache.lp_info.liquid_reserve,
            );
        if amount_to_send_to_onedex < MIN_EGLD {
            return (BigUint::zero(), BigUint::zero())
        }

        let amount_from_onedex =
            self.get_onedex_amount_out(is_buy, &amount_to_send_to_onedex, storage_cache, onedex_cache);
        let amount_from_salsa =
            self.get_salsa_amount_out(&amount_to_send_to_onedex, is_buy, storage_cache);
        if amount_from_onedex <= amount_from_salsa {
            return (BigUint::zero(), BigUint::zero())
        }

        self.swap_on_onedex(is_buy, &amount_to_send_to_onedex, &amount_from_salsa, storage_cache, onedex_cache);

        (amount_to_send_to_onedex, amount_from_salsa)
    }

    fn swap_on_onedex(&self,
        is_buy: bool,
        in_amount: &BigUint,
        out_amount: &BigUint,
        storage_cache: &mut StorageCache<Self>,
        onedex_cache: &OnedexCache<Self>,
    ) {
        let mut path: MultiValueEncoded<TokenIdentifier> = MultiValueEncoded::new();
        let wegld_id = storage_cache.wegld_id.clone();
        let liquid_token_id = storage_cache.liquid_token_id.clone();
        if is_buy {
            path.push(wegld_id);
            path.push(liquid_token_id);
            self.onedex_proxy_obj()
                .contract(onedex_cache.sc_address.clone())
                .swap_multi_tokens_fixed_input(out_amount, false, path)
                .with_egld_transfer(in_amount.clone())
                .execute_on_dest_context::<()>();
        } else {
            path.push(liquid_token_id.clone());
            path.push(wegld_id);
            let payment =
                EsdtTokenPayment::new(liquid_token_id, 0, in_amount.clone());
            self.onedex_proxy_obj()
                .contract(onedex_cache.sc_address.clone())
                .swap_multi_tokens_fixed_input(out_amount, true, path)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
    }

    // proxy

    #[proxy]
    fn onedex_proxy_obj(&self) -> onedex_proxy::Proxy<Self::Api>;
}
