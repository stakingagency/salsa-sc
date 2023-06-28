multiversx_sc::imports!();

use crate::common::storage_cache::StorageCache;
use crate::common::{errors::*, config::State};
use crate::proxies::wrap_proxy;

use super::onedex_cache::OnedexCache;
use super::xexchange_cache::XexchangeCache;

#[multiversx_sc::module]
pub trait ArbitrageModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::exchanges::onedex::OnedexModule
    + crate::exchanges::xexchange::XexchangeModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setArbitrageActive)]
    fn set_arbitrage_active(&self) {
        require!(!self.provider_address().is_empty(), ERROR_PROVIDER_NOT_SET);
        require!(!self.liquid_token_id().is_empty(), ERROR_TOKEN_NOT_SET);
        require!(!self.wrap_sc().is_empty(), ERROR_WRAP_SC);

        if self.wegld_id().is_empty() {
            let wegld_id: TokenIdentifier = self.egld_wrap_proxy_obj()
                .contract(self.wrap_sc().get())
                .wrapped_egld_token_id()
                .execute_on_dest_context();
            self.wegld_id().set(wegld_id);
        }

        self.arbitrage().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setArbitrageInactive)]
    fn set_arbitrage_inactive(&self) {
        self.arbitrage().set(State::Inactive);
    }

    #[inline]
    fn is_arbitrage_active(&self) -> bool {
        let arbitrage = self.arbitrage().get();
        arbitrage == State::Active
    }

    #[view(getArbitrageState)]
    #[storage_mapper("arbitrage")]
    fn arbitrage(&self) -> SingleValueMapper<State>;

    fn do_arbitrage(
        &self, is_buy: bool, in_amount: BigUint, storage_cache: &mut StorageCache<Self>,
    ) -> (BigUint, BigUint) {
        if !self.is_arbitrage_active() {
            return (BigUint::zero(), BigUint::zero())
        }

        let (mut sold_amount, mut bought_amount) = (BigUint::zero(), BigUint::zero());
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();
        if self.is_onedex_arbitrage_active() {
            let onedex_cache = OnedexCache::new(self);
            let (sold, bought) =
                self.do_arbitrage_on_onedex(is_buy, in_amount.clone(), storage_cache, onedex_cache);
            sold_amount += &sold;
            bought_amount += &bought;
        }
        if self.is_xexchange_arbitrage_active() && in_amount > sold_amount {
            let xexchange_cache = XexchangeCache::new(self);
            let (sold, bought) =
                self.do_arbitrage_on_xexchange(is_buy, in_amount - &sold_amount, storage_cache, xexchange_cache);
            sold_amount += sold;
            bought_amount += bought;
        }

        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();
        if is_buy {
            require!(new_ls_balance >= old_ls_balance, ERROR_ARBITRAGE_ISSUE);

            if new_ls_balance > old_ls_balance {
                let swapped_amount = &new_ls_balance - &old_ls_balance;
                require!(swapped_amount >= bought_amount, ERROR_ARBITRAGE_ISSUE);

                let profit = &swapped_amount - &bought_amount;
                if profit > 0 {
                    self.burn_liquid_token(&profit);
                    storage_cache.liquid_supply -= &profit;
                }
            }
        } else {
            require!(new_egld_balance >= old_egld_balance, ERROR_ARBITRAGE_ISSUE);

            if new_egld_balance > old_egld_balance {
                let swapped_amount = &new_egld_balance - &old_egld_balance;
                require!(swapped_amount >= bought_amount, ERROR_ARBITRAGE_ISSUE);

                let profit = &swapped_amount - &bought_amount;
                self.egld_reserve()
                    .update(|value| *value += &profit);
                self.available_egld_reserve()
                    .update(|value| *value += profit);
            }
        }

        (sold_amount, bought_amount)
    }

    // proxy

    #[proxy]
    fn egld_wrap_proxy_obj(&self) -> wrap_proxy::Proxy<Self::Api>;
}
