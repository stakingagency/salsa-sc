multiversx_sc::imports!();

use crate::{
    common::{
        storage_cache::StorageCache,
        errors::*,
        config::*,
        consts::*
    },
    proxies::{onedex_proxy, xexchange_proxy, wrap_proxy},
    exchanges::lp_cache::LpCache};

use super::{onedex_cache::OnedexCache, xexchange_cache::XexchangeCache};

#[multiversx_sc::module]
pub trait LpModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::exchanges::arbitrage::ArbitrageModule
    + crate::exchanges::onedex::OnedexModule
    + crate::exchanges::xexchange::XexchangeModule
    + crate::exchanges::xstake::XStakeModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setLpActive)]
    fn set_lp_active(&self) {
        require!(self.is_arbitrage_active(), ERROR_ARBITRAGE_NOT_ACTIVE);

        self.lp_state().set(State::Active);

        let mut storage_cache = StorageCache::new(self);
        let mut lp_cache = LpCache::new(self);

        self.do_arbitrage(&mut storage_cache, &mut lp_cache, OptionalValue::None);
        self.add_lp(&mut storage_cache, &mut lp_cache);
    }

    #[only_owner]
    #[endpoint(setLpInactive)]
    fn set_lp_inactive(&self) {
        if !self.is_lp_active() {
            return
        }

        let mut storage_cache = StorageCache::new(self);
        let mut lp_cache = LpCache::new(self);

        self.remove_all_lps(&mut storage_cache, &mut lp_cache);
        self.lp_state().set(State::Inactive);
    }

    #[inline]
    fn is_lp_active(&self) -> bool {
        let lp = self.lp_state().get();
        lp == State::Active
    }

    #[view(getLpState)]
    #[storage_mapper("lp_state")]
    fn lp_state(&self) -> SingleValueMapper<State>;

    /**
     * Add LPs
     */
    fn add_lp(&self, storage_cache: &mut StorageCache<Self>, lp_cache: &mut LpCache<Self>) {
        if !self.is_lp_active() || !self.is_arbitrage_active() {
            return
        }

        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut available_egld_for_lp = self.compute_egld_available_for_lp(storage_cache, lp_cache);
        if available_egld_for_lp == 0 {
            return
        }

        let mut available_legld_for_lp =
            &lp_cache.excess_lp_legld + &storage_cache.legld_in_custody - &lp_cache.legld_in_lp;
        if available_legld_for_lp < MIN_EGLD {
            return
        }

        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();

        // get the list of available exchanges
        let mut lps: ManagedVec<Self::Api, LpInfo<Self::Api>> = ManagedVec::new();
        let onedex_cache = OnedexCache::new(self);
        let xexchange_cache = XexchangeCache::new(self);
        if self.is_onedex_arbitrage_active() && onedex_cache.lp_info.liquid_reserve > 0 {
            lps.push(onedex_cache.lp_info);
        }
        if self.is_xexchange_arbitrage_active() && xexchange_cache.lp_info.liquid_reserve > 0 {
            lps.push(xexchange_cache.lp_info);
        }

        let num_exchanges = BigUint::from(lps.len());
        available_egld_for_lp /= &num_exchanges;
        available_legld_for_lp /= &num_exchanges;

        let one = BigUint::from(ONE_EGLD);
        let salsa_price = if (storage_cache.liquid_supply == 0) || (storage_cache.total_stake == 0) {
            one.clone()
        } else {
            &one * &storage_cache.total_stake / &storage_cache.liquid_supply
        };
        for lp in lps.iter() {
            let price = &lp.egld_reserve * &one / &lp.liquid_reserve;
            let price_gap = if price > salsa_price {
                &price * MAX_PERCENT / &salsa_price - MAX_PERCENT
            } else {
                &salsa_price * MAX_PERCENT / &price - MAX_PERCENT
            };
            if price_gap > MAX_PRICE_GAP {
                continue
            }

            // calculate amounts to add to LP
            let mut egld_to_lp = available_egld_for_lp.clone();
            let mut legld_to_lp = &one * &egld_to_lp / &price;
            if legld_to_lp > available_legld_for_lp {
                legld_to_lp = available_legld_for_lp.clone();
                egld_to_lp = &legld_to_lp * &price / &one;
            }
            let mut payments :ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
            payments.push(EsdtTokenPayment::new(storage_cache.liquid_token_id.clone(), 0, legld_to_lp));
            payments.push(EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, egld_to_lp.clone()));

            // wrap eGLD
            self.wrap_lp_proxy_obj()
                .contract(xexchange_cache.wrap_sc_address.clone())
                .wrap_egld()
                .with_egld_transfer(egld_to_lp)
                .execute_on_dest_context::<()>();

            // add to LP
            match lp.exchange {
                Exchange::Onedex => {
                    self.onedex_lp_proxy_obj()
                        .contract(onedex_cache.sc_address.clone())
                        .add_liquidity(BigUint::from(1u64), BigUint::from(1u64))
                        .with_multi_token_transfer(payments)
                        .execute_on_dest_context::<()>();
                }
                Exchange::Xexchange => {
                    self.xexchange_lp_proxy_obj()
                        .contract(xexchange_cache.sc_address.clone())
                        .add_liquidity(BigUint::from(1u64), BigUint::from(1u64))
                        .with_multi_token_transfer(payments)
                        .execute_on_dest_context::<()>();
                }
                Exchange::None => {}
            }
        }

        self.unwrap_all_wegld();
        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();

        let added_egld = &old_egld_balance - &new_egld_balance;
        let added_legld = &old_ls_balance - &new_ls_balance;
        self.add_lp_update_storage(storage_cache, lp_cache, added_egld, added_legld);
    }

    /**
     * Remove eGLD LPs
     */
    fn remove_egld_lp(&self, amount: BigUint, storage_cache: &mut StorageCache<Self>, lp_cache: &mut LpCache<Self>) {
        if !self.is_lp_active() || !self.is_arbitrage_active() {
            return
        }

        require!(amount <= &lp_cache.excess_lp_egld + &lp_cache.egld_in_lp, ERROR_INSUFFICIENT_FUNDS);

        let mut left_amount = amount;
        if lp_cache.excess_lp_egld > 0 {
            if left_amount > lp_cache.excess_lp_egld {
                left_amount -= &lp_cache.excess_lp_egld;
                lp_cache.excess_lp_egld = BigUint::zero();
            } else {
                lp_cache.excess_lp_egld -= &left_amount;
                return
            }
        }

        let mut onedex_cache = OnedexCache::new(self);
        let mut xexchange_cache = XexchangeCache::new(self);
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();

        loop {
            // get the list of available exchanges
            let mut lps: ManagedVec<Self::Api, LpInfo<Self::Api>> = ManagedVec::new();
            if self.is_onedex_arbitrage_active() && onedex_cache.lp_info.lp_balance > 0 {
                lps.push(onedex_cache.lp_info.clone());
            }
            if self.is_xexchange_arbitrage_active() && xexchange_cache.lp_info.lp_balance > 0 {
                lps.push(xexchange_cache.lp_info.clone());
            }

            let (best_exchange, mut lp_to_remove) =
                self.get_exchange_with_cheap_legld(lps, &left_amount);
            if best_exchange == Exchange::None || left_amount == 0 {
                break
            }

            // remove LP
            let mut egld_to_remove = left_amount.clone();
            match best_exchange {
                Exchange::Onedex => {
                    if lp_to_remove > onedex_cache.lp_info.lp_balance {
                        lp_to_remove = onedex_cache.lp_info.lp_balance.clone();
                        egld_to_remove = &onedex_cache.lp_info.egld_reserve * &lp_to_remove / &onedex_cache.lp_info.lp_supply;
                    }
                    let payment =
                        EsdtTokenPayment::new(onedex_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    onedex_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.remove_xstake(1, lp_to_remove, onedex_cache.lp_info.lp_token.clone());
                    self.onedex_lp_proxy_obj()
                        .contract(onedex_cache.sc_address.clone())
                        .remove_liquidity(BigUint::from(1u64), BigUint::from(1u64), false)
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::Xexchange => {
                    if lp_to_remove > xexchange_cache.lp_info.lp_balance {
                        lp_to_remove = xexchange_cache.lp_info.lp_balance.clone();
                        egld_to_remove = &xexchange_cache.lp_info.egld_reserve * &lp_to_remove / &xexchange_cache.lp_info.lp_supply;
                    }
                    let payment
                        = EsdtTokenPayment::new(xexchange_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    xexchange_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.remove_xstake(2, lp_to_remove, xexchange_cache.lp_info.lp_token.clone());
                    self.xexchange_lp_proxy_obj()
                        .contract(xexchange_cache.sc_address.clone())
                        .remove_liquidity(BigUint::from(1u64), BigUint::from(1u64))
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::None => {}
            }
            left_amount = if egld_to_remove > left_amount {
                BigUint::zero()
            } else {
                &left_amount - &egld_to_remove
            };
        }

        self.unwrap_all_wegld();
        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();

        let removed_egld = &new_egld_balance - &old_egld_balance;
        let removed_legld = &new_ls_balance - &old_ls_balance;
        self.remove_lp_update_storage(storage_cache, lp_cache, removed_egld, removed_legld);
    }

    /**
     * Remove LEGLD LPs
     */
    fn remove_legld_lp(&self, amount: BigUint, storage_cache: &mut StorageCache<Self>, lp_cache: &mut LpCache<Self>) {
        if !self.is_lp_active() || !self.is_arbitrage_active() {
            return
        }

        require!(amount <= &lp_cache.excess_lp_legld + &lp_cache.legld_in_lp, ERROR_INSUFFICIENT_FUNDS);

        let mut left_amount = amount;
        if lp_cache.excess_lp_legld > 0 {
            if left_amount > lp_cache.excess_lp_legld {
                left_amount -= &lp_cache.excess_lp_legld;
                lp_cache.excess_lp_legld = BigUint::zero();
            } else {
                lp_cache.excess_lp_legld -= &left_amount;
                return
            }
        }

        let mut onedex_cache = OnedexCache::new(self);
        let mut xexchange_cache = XexchangeCache::new(self);
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();

        loop {
            // get the list of available exchanges
            let mut lps: ManagedVec<Self::Api, LpInfo<Self::Api>> = ManagedVec::new();
            if self.is_onedex_arbitrage_active() && onedex_cache.lp_info.lp_balance > 0 {
                lps.push(onedex_cache.lp_info.clone());
            }
            if self.is_xexchange_arbitrage_active() && xexchange_cache.lp_info.lp_balance > 0 {
                lps.push(xexchange_cache.lp_info.clone());
            }

            let (best_exchange, mut lp_to_remove) =
                self.get_exchange_with_expensive_legld(lps, &left_amount);
            if best_exchange == Exchange::None || left_amount == 0 {
                break
            }

            // remove LP
            let mut legld_to_remove = left_amount.clone();
            match best_exchange {
                Exchange::Onedex => {
                    if lp_to_remove > onedex_cache.lp_info.lp_balance {
                        lp_to_remove = onedex_cache.lp_info.lp_balance.clone();
                        legld_to_remove = &onedex_cache.lp_info.liquid_reserve * &lp_to_remove / &onedex_cache.lp_info.lp_supply;
                    }
                    let payment =
                        EsdtTokenPayment::new(onedex_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    onedex_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.remove_xstake(1, lp_to_remove, onedex_cache.lp_info.lp_token.clone());
                    self.onedex_lp_proxy_obj()
                        .contract(onedex_cache.sc_address.clone())
                        .remove_liquidity(BigUint::from(1u64), BigUint::from(1u64), false)
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::Xexchange => {
                    if lp_to_remove > xexchange_cache.lp_info.lp_balance {
                        lp_to_remove = xexchange_cache.lp_info.lp_balance.clone();
                        legld_to_remove = &xexchange_cache.lp_info.liquid_reserve * &lp_to_remove / &xexchange_cache.lp_info.lp_supply;
                    }
                    let payment
                        = EsdtTokenPayment::new(xexchange_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    xexchange_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.remove_xstake(2, lp_to_remove, xexchange_cache.lp_info.lp_token.clone());
                    self.xexchange_lp_proxy_obj()
                        .contract(xexchange_cache.sc_address.clone())
                        .remove_liquidity(BigUint::from(1u64), BigUint::from(1u64))
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::None => {}
            }
            left_amount = if legld_to_remove > left_amount {
                BigUint::zero()
            } else {
                &left_amount - &legld_to_remove
            };
        }

        self.unwrap_all_wegld();
        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();

        let removed_egld = &new_egld_balance - &old_egld_balance;
        let removed_legld = &new_ls_balance - &old_ls_balance;
        self.remove_lp_update_storage(storage_cache, lp_cache, removed_egld, removed_legld);
    }

    /**
     * Take LPs profit
     */
    #[only_owner]
    #[endpoint(takeLpProfit)]
    fn take_lp_profit(&self) {
        let mut storage_cache = StorageCache::new(self);
        let mut lp_cache = LpCache::new(self);

        self.remove_all_lps(&mut storage_cache, &mut lp_cache);

        let have_excess = lp_cache.excess_lp_egld > 0 || lp_cache.excess_lp_legld > 0;
        let lp_empty = lp_cache.egld_in_lp == 0 && lp_cache.legld_in_lp == 0;
        require!(have_excess && lp_empty, ERROR_INSUFFICIENT_FUNDS);

        if lp_cache.excess_lp_egld > 0 {
            storage_cache.egld_reserve += &lp_cache.excess_lp_egld;
            storage_cache.available_egld_reserve += &lp_cache.excess_lp_egld;
            lp_cache.excess_lp_egld = BigUint::zero();
        }
        if lp_cache.excess_lp_legld > 0 {
            self.burn_liquid_token(&lp_cache.excess_lp_legld);
            storage_cache.liquid_supply -= &lp_cache.excess_lp_legld;
            lp_cache.excess_lp_legld = BigUint::zero();
        }

        self.add_lp(&mut storage_cache, &mut lp_cache);
    }

    // helpers

    fn remove_all_lps(&self, storage_cache: &mut StorageCache<Self>, lp_cache: &mut LpCache<Self>) {
        let mut onedex_cache = OnedexCache::new(self);
        let mut xexchange_cache = XexchangeCache::new(self);

        self.do_arbitrage(storage_cache, lp_cache, OptionalValue::None);

        let (old_balance, old_ls_balance) = self.get_sc_balances();
        if self.is_onedex_arbitrage_active() && onedex_cache.lp_info.lp_balance > 0 {
            let payment =
                EsdtTokenPayment::new(onedex_cache.lp_info.lp_token.clone(), 0, onedex_cache.lp_info.lp_balance.clone());
            onedex_cache.lp_info.lp_balance = BigUint::zero();
            self.onedex_lp_proxy_obj()
                .contract(onedex_cache.sc_address.clone())
                .remove_liquidity(BigUint::from(1u64), BigUint::from(1u64), false)
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
        if self.is_xexchange_arbitrage_active() && xexchange_cache.lp_info.lp_balance > 0 {
            let payment
                = EsdtTokenPayment::new(xexchange_cache.lp_info.lp_token.clone(), 0, xexchange_cache.lp_info.lp_balance.clone());
            xexchange_cache.lp_info.lp_balance -= BigUint::zero();
            self.xexchange_lp_proxy_obj()
                .contract(xexchange_cache.sc_address.clone())
                .remove_liquidity(BigUint::from(1u64), BigUint::from(1u64))
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }

        self.unwrap_all_wegld();
        let (new_balance, new_ls_balance) = self.get_sc_balances();

        let removed_egld = new_balance - old_balance;
        let removed_legld = new_ls_balance - old_ls_balance;
        self.remove_lp_update_storage(storage_cache, lp_cache, removed_egld, removed_legld);

        if lp_cache.excess_lp_egld > 0 && lp_cache.legld_in_lp > 0 {
            let ls_amount =
                self.add_liquidity(&lp_cache.excess_lp_egld, true, storage_cache);
            self.mint_liquid_token(ls_amount.clone());
            storage_cache.egld_to_delegate += &lp_cache.excess_lp_egld;
            lp_cache.excess_lp_egld = BigUint::zero();
            if ls_amount > lp_cache.legld_in_lp {
                lp_cache.excess_lp_legld += &ls_amount - &lp_cache.legld_in_lp;
                lp_cache.legld_in_lp = BigUint::zero();
            } else {
                lp_cache.legld_in_lp -= ls_amount;
            }
        }

        if lp_cache.excess_lp_legld > 0 && lp_cache.egld_in_lp > 0 {
            let egld_amount =
                self.remove_liquidity(&lp_cache.excess_lp_legld, true, storage_cache);
            self.burn_liquid_token(&lp_cache.excess_lp_legld);
            storage_cache.egld_to_undelegate += &egld_amount;
            let current_epoch = self.blockchain().get_block_epoch();
            let unbond_epoch = current_epoch + storage_cache.unbond_period;
            self.add_undelegation(egld_amount.clone(), unbond_epoch, self.lreserve_undelegations());
            lp_cache.excess_lp_legld = BigUint::zero();
            if lp_cache.egld_in_lp > egld_amount {
                lp_cache.egld_in_lp -= egld_amount;
            } else {
                storage_cache.egld_reserve += &egld_amount - &lp_cache.egld_in_lp;
                lp_cache.egld_in_lp = BigUint::zero();
            }
        }
    }

    fn add_lp_update_storage(
        &self,
        storage_cache: &mut StorageCache<Self>,
        lp_cache: &mut LpCache<Self>,
        mut added_egld: BigUint,
        mut added_legld: BigUint,
    ) {
        if added_egld >= lp_cache.excess_lp_egld {
            added_egld -= &lp_cache.excess_lp_egld;
            lp_cache.excess_lp_egld = BigUint::zero();
        } else {
            lp_cache.excess_lp_egld -= &added_egld;
            added_egld = BigUint::zero();
        }
        if added_legld >= lp_cache.excess_lp_legld {
            added_legld -= &lp_cache.excess_lp_legld;
            lp_cache.excess_lp_legld = BigUint::zero();
        } else {
            lp_cache.excess_lp_legld -= &added_legld;
            added_legld = BigUint::zero();
        }
        lp_cache.egld_in_lp += &added_egld;
        lp_cache.legld_in_lp += added_legld;
        storage_cache.available_egld_reserve -= added_egld;
    }

    fn remove_lp_update_storage(
        &self,
        storage_cache: &mut StorageCache<Self>,
        lp_cache: &mut LpCache<Self>,
        removed_egld: BigUint,
        removed_legld: BigUint,
    ) {
        if removed_egld > lp_cache.egld_in_lp {
            lp_cache.excess_lp_egld += &removed_egld - &lp_cache.egld_in_lp;
            storage_cache.available_egld_reserve += &lp_cache.egld_in_lp;
            lp_cache.egld_in_lp = BigUint::zero();
        } else {
            storage_cache.available_egld_reserve += &removed_egld;
            lp_cache.egld_in_lp -= removed_egld;
        }
        if removed_legld > lp_cache.legld_in_lp {
            lp_cache.excess_lp_legld += &removed_legld - &lp_cache.legld_in_lp;
            lp_cache.legld_in_lp = BigUint::zero();
        } else {
            lp_cache.legld_in_lp -= removed_legld;
        }
    }

    fn compute_egld_available_for_lp(
        &self,
        storage_cache: &mut StorageCache<Self>,
        lp_cache: &mut LpCache<Self>,
    ) -> BigUint {
        let half_reserve = &storage_cache.egld_reserve / 2_u64;
        if lp_cache.egld_in_lp >= half_reserve {
            return BigUint::zero()
        }

        let left_from_half = &half_reserve - &lp_cache.egld_in_lp;
        let mut available = &storage_cache.available_egld_reserve + &lp_cache.excess_lp_egld;
        if available > left_from_half {
            available = left_from_half;
        }
        if available < MIN_EGLD {
            return BigUint::zero()
        }

        available
    }

    fn get_exchange_with_cheap_legld(
        &self,
        lps: ManagedVec<Self::Api, LpInfo<Self::Api>>,
        amount: &BigUint,
    ) -> (Exchange, BigUint) {
        let one = BigUint::from(ONE_EGLD);
        let mut best_price = BigUint::zero();
        let mut best_exchange = Exchange::None;
        let mut lp_to_remove = BigUint::zero();
        for lp in lps.iter() {
            let egld_per_lp = &one * &lp.egld_reserve / &lp.lp_supply;
            if best_price > egld_per_lp || best_price == 0 {
                best_price = egld_per_lp;
                best_exchange = lp.exchange;
                lp_to_remove = amount * &lp.lp_supply / &lp.egld_reserve;
                lp_to_remove += BigUint::from(1u64);
            }
        }

        (best_exchange, lp_to_remove)
    }

    fn get_exchange_with_expensive_legld(
        &self,
        lps: ManagedVec<Self::Api, LpInfo<Self::Api>>,
        amount: &BigUint,
    ) -> (Exchange, BigUint) {
        let one = BigUint::from(ONE_EGLD);
        let mut best_price = BigUint::zero();
        let mut best_exchange = Exchange::None;
        let mut lp_to_remove = BigUint::zero();
        for lp in lps.iter() {
            let legld_per_lp = &one * &lp.liquid_reserve / &lp.lp_supply;
            if best_price > legld_per_lp || best_price == 0 {
                best_price = legld_per_lp;
                best_exchange = lp.exchange;
                lp_to_remove = amount * &lp.lp_supply / &lp.liquid_reserve;
                lp_to_remove += BigUint::from(1u64);
            }
        }

        (best_exchange, lp_to_remove)
    }

    fn unwrap_all_wegld(&self) {
        let wegld_id = self.wegld_id().get();
        let wrap_sc_address = self.wrap_sc().get();
        let wegld_balance =
            self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(wegld_id.clone()), 0);
        if wegld_balance > 0 {
            let payment = EsdtTokenPayment::new(wegld_id, 0, wegld_balance);
            self.wrap_lp_proxy_obj()
                .contract(wrap_sc_address)
                .unwrap_egld()
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }
    }

    // proxies

    #[proxy]
    fn onedex_lp_proxy_obj(&self) -> onedex_proxy::Proxy<Self::Api>;

    #[proxy]
    fn xexchange_lp_proxy_obj(&self) -> xexchange_proxy::Proxy<Self::Api>;

    #[proxy]
    fn wrap_lp_proxy_obj(&self) -> wrap_proxy::Proxy<Self::Api>;
}
