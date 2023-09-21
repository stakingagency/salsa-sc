multiversx_sc::imports!();

use crate::common::consts::MAX_LOAN;
use crate::common::storage_cache::StorageCache;
use crate::common::{errors::*, config::State};
use crate::exchanges::lp_cache::LpCache;
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

    /**
     * Trigger arbitrage
     */
    #[endpoint(triggerArbitrage)]
    fn trigger_arbitrage(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut storage_cache = StorageCache::new(self);
        let mut lp_cache = LpCache::new(self);
        self.do_arbitrage(&mut storage_cache, &mut lp_cache, OptionalValue::Some(true));
    }

    /**
     * Do arbitrage
     */
    fn do_arbitrage(
        &self,
        storage_cache: &mut StorageCache<Self>,
        lp_cache: &mut LpCache<Self>,
        split_profit: OptionalValue<bool>,
    ) {
        self.do_arbitrage_sell(storage_cache, split_profit.clone());
        self.do_arbitrage_buy(storage_cache, lp_cache, split_profit);
    }

    fn do_arbitrage_sell(
        &self,
        storage_cache: &mut StorageCache<Self>,
        split_profit: OptionalValue<bool>,
    ) {
        if !self.is_arbitrage_active() {
            return
        }

        let do_split_profit = match split_profit {
            OptionalValue::Some(value) => value,
            OptionalValue::None => false
        };
        let in_amount = BigUint::from(MAX_LOAN);
        self.mint_liquid_token(in_amount.clone());
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();
        let (_, bought_amount) =
            self.execute_arbitrage(false, in_amount.clone(), storage_cache);
        let new_egld_balance = self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(new_egld_balance >= old_egld_balance, ERROR_ARBITRAGE_ISSUE);

        let swapped_amount = &new_egld_balance - &old_egld_balance;
        require!(swapped_amount >= bought_amount, ERROR_ARBITRAGE_ISSUE);

        if swapped_amount > 0 {
            storage_cache.egld_to_delegate += &swapped_amount;
            let ls_amount =
                self.add_liquidity(&swapped_amount, true, storage_cache);
            self.mint_liquid_token(ls_amount);
        }

        let new_ls_balance = self.blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(storage_cache.liquid_token_id.clone()), 0);
        require!(new_ls_balance >= old_ls_balance, ERROR_ARBITRAGE_ISSUE);

        let profit = &new_ls_balance - &old_ls_balance;
        if do_split_profit && profit > 0 {
            let half_profit = &profit / 2_u64;
            storage_cache.liquid_supply -= &half_profit;
            self.burn_liquid_token(&(&half_profit + &in_amount));
            self.send().direct_esdt(
                &self.blockchain().get_caller(),
                &storage_cache.liquid_token_id,
                0,
                &(profit - half_profit),
            );
        } else {
            storage_cache.liquid_supply -= &profit;
            self.burn_liquid_token(&(profit + in_amount));
        }
    }

    fn do_arbitrage_buy(
        &self,
        storage_cache: &mut StorageCache<Self>,
        lp_cache: &mut LpCache<Self>,
        split_profit: OptionalValue<bool>,
    ) {
        if !self.is_arbitrage_active() {
            return
        }

        let do_split_profit = match split_profit {
            OptionalValue::Some(value) => value,
            OptionalValue::None => false
        };
        let (old_balance, old_ls_balance) = self.get_sc_balances();
        let in_amount = &storage_cache.available_egld_reserve + &lp_cache.excess_lp_egld;
        let (_, bought_amount) =
            self.execute_arbitrage(true, in_amount, storage_cache);
        let (new_balance, new_ls_balance) = self.get_sc_balances();
        require!(new_ls_balance >= old_ls_balance, ERROR_ARBITRAGE_ISSUE);

        let swapped_amount = &new_ls_balance - &old_ls_balance;
        require!(swapped_amount >= bought_amount, ERROR_ARBITRAGE_ISSUE);

        let sold_amount = old_balance - new_balance;
        if bought_amount > 0 {
            storage_cache.total_stake -= &sold_amount;
            storage_cache.liquid_supply -= &bought_amount;
            let profit = &swapped_amount - &bought_amount;
            if do_split_profit && profit > 0 {
                let half_profit = &profit / 2_u64;
                self.burn_liquid_token(&(&bought_amount + &half_profit));
                storage_cache.liquid_supply -= &half_profit;
                self.send().direct_esdt(
                    &self.blockchain().get_caller(),
                    &storage_cache.liquid_token_id,
                    0,
                    &(profit - half_profit),
                );
            } else {
                self.burn_liquid_token(&swapped_amount);
                storage_cache.liquid_supply -= &profit;
            }
            storage_cache.egld_to_undelegate += &sold_amount;
            storage_cache.available_egld_reserve -= &sold_amount;
            let current_epoch = self.blockchain().get_block_epoch();
            let unbond_epoch = current_epoch + storage_cache.unbond_period;
            self.add_undelegation(sold_amount.clone(), unbond_epoch, self.lreserve_undelegations());
        } else {
            require!(sold_amount == 0, ERROR_ARBITRAGE_ISSUE);
        }
    }

    fn execute_arbitrage(
        &self, is_buy: bool, in_amount: BigUint, storage_cache: &mut StorageCache<Self>,
    ) -> (BigUint, BigUint) {
        if !self.is_arbitrage_active() {
            return (BigUint::zero(), BigUint::zero())
        }

        let (mut sold_amount, mut bought_amount) = (BigUint::zero(), BigUint::zero());
        if self.is_onedex_arbitrage_active() {
            let onedex_cache = OnedexCache::new(self);
            if onedex_cache.is_active {
                let (sold, bought) =
                    self.do_arbitrage_on_onedex(is_buy, in_amount.clone(), storage_cache, onedex_cache);
                sold_amount += &sold;
                bought_amount += &bought;
            }
        }
        if self.is_xexchange_arbitrage_active() && in_amount > sold_amount {
            let xexchange_cache = XexchangeCache::new(self);
            if xexchange_cache.is_active {
                let (sold, bought) =
                    self.do_arbitrage_on_xexchange(is_buy, in_amount - &sold_amount, storage_cache, xexchange_cache);
                sold_amount += sold;
                bought_amount += bought;
            }
        }

        (sold_amount, bought_amount)
    }

    // proxy

    #[proxy]
    fn egld_wrap_proxy_obj(&self) -> wrap_proxy::Proxy<Self::Api>;
}
