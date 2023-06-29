multiversx_sc::imports!();

use crate::{common::{storage_cache::StorageCache, errors::*, config::*, consts::*}, proxies::{onedex_proxy, xexchange_proxy, wrap_proxy}, exchanges::lp_cache::LpCache};

use super::{onedex_cache::OnedexCache, xexchange_cache::XexchangeCache};

#[multiversx_sc::module]
pub trait LpModule:
crate::common::config::ConfigModule
+ crate::helpers::HelpersModule
+ crate::exchanges::arbitrage::ArbitrageModule
+ crate::exchanges::onedex::OnedexModule
+ crate::exchanges::xexchange::XexchangeModule
+ multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(addLP)]
    fn add_lp(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        if !self.is_arbitrage_active() {
            return
        }

        let storage_cache = StorageCache::new(self);
        let mut lp_cache = LpCache::new(self);
        let available_egld_for_lp =
            &lp_cache.excess_lp_egld + &storage_cache.available_egld_reserve - &lp_cache.egld_in_lp;
        if available_egld_for_lp < MIN_EGLD {
            return
        }
        let available_legld_for_lp =
            &lp_cache.excess_lp_legld + &storage_cache.legld_in_custody - &lp_cache.legld_in_lp;
        if available_legld_for_lp < MIN_EGLD {
            return
        }

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

        // find the exchange with the price closest to SALSA's price for lowest IL
        let one = BigUint::from(ONE_EGLD);
        let salsa_price = if (storage_cache.liquid_supply == 0) || (storage_cache.total_stake == 0) {
            one.clone()
        } else {
            &one * &storage_cache.total_stake / &storage_cache.liquid_supply
        };
        let mut min_price_gap = BigUint::zero();
        let mut best_price = BigUint::zero();
        let mut best_exchange = Exchange::None;
        for lp in lps.iter() {
            let price = &lp.egld_reserve * &one / &lp.liquid_reserve;
            let price_gap = if price > salsa_price {
                &price - &salsa_price
            } else {
                &salsa_price - &price
            };
            if min_price_gap > price_gap || best_exchange == Exchange::None {
                min_price_gap = price_gap;
                best_price = price;
                best_exchange = lp.exchange;
            }
        }
        if min_price_gap > MAX_PRICE_GAP || best_exchange == Exchange::None {
            return
        }

        // calculate amounts to add to LP
        let mut egld_to_lp = available_egld_for_lp;
        let mut legld_to_lp = &egld_to_lp / &best_price;
        if legld_to_lp > available_legld_for_lp {
            legld_to_lp = available_legld_for_lp;
            egld_to_lp = &legld_to_lp * &best_price;
        }
        let mut payments :ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        payments.push(EsdtTokenPayment::new(storage_cache.liquid_token_id.clone(), 0, legld_to_lp.clone()));
        payments.push(EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, egld_to_lp.clone()));
        let (old_egld_balance, old_ls_balance) = self.get_sc_balances();

        // wrap eGLD
        self.wrap_lp_proxy_obj()
            .contract(xexchange_cache.wrap_sc_address.clone())
            .wrap_egld()
            .with_egld_transfer(egld_to_lp.clone())
            .execute_on_dest_context::<()>();

        // add to LP
        match best_exchange {
            Exchange::Onedex => {
                self.onedex_lp_proxy_obj()
                    .contract(onedex_cache.sc_address)
                    .add_liquidity(legld_to_lp, egld_to_lp)
                    .with_multi_token_transfer(payments)
                    .execute_on_dest_context::<()>();
            }
            Exchange::Xexchange => {
                self.xexchange_lp_proxy_obj()
                    .contract(xexchange_cache.sc_address)
                    .add_liquidity(legld_to_lp, egld_to_lp)
                    .with_multi_token_transfer(payments)
                    .execute_on_dest_context::<()>();
            }
            Exchange::None => {}
        }

        // unwrap WEGLD
        let wegld_balance =
            self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(storage_cache.wegld_id.clone()), 0);
        if wegld_balance > 0 {
            let payment = EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, wegld_balance);
            self.wrap_lp_proxy_obj()
                .contract(xexchange_cache.wrap_sc_address)
                .unwrap_egld()
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }

        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();
        let mut added_egld = &old_egld_balance - &new_egld_balance;
        let mut added_legld = &old_ls_balance - &new_ls_balance;
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
        lp_cache.egld_in_lp += added_egld;
        lp_cache.legld_in_lp += added_legld;
    }

    fn remove_egld_lp(&self, amount: BigUint, lp_cache: &mut LpCache<Self>) {
        if !self.is_arbitrage_active() {
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

        let storage_cache = StorageCache::new(self);
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

            // find the exchange with the most cheap LEGLD
            let one = BigUint::from(ONE_EGLD);
            let mut best_price = BigUint::zero();
            let mut best_exchange = Exchange::None;
            for lp in lps.iter() {
                let egld_per_lp = &one * &lp.egld_reserve / &lp.lp_supply;
                if best_price < egld_per_lp {
                    best_price = egld_per_lp;
                    best_exchange = lp.exchange;
                }
            }

            if best_exchange == Exchange::None {
                break
            }

            // remove LP
            let mut lp_to_remove = &one * &left_amount / &best_price;
            let mut egld_to_remove = BigUint::zero();
            match best_exchange {
                Exchange::Onedex => {
                    if lp_to_remove > onedex_cache.lp_info.lp_balance {
                        lp_to_remove = onedex_cache.lp_info.lp_balance.clone();
                    }
                    let legld_to_remove =
                        &onedex_cache.lp_info.liquid_reserve * &lp_to_remove / &onedex_cache.lp_info.lp_supply;
                    egld_to_remove =
                        &onedex_cache.lp_info.egld_reserve * &lp_to_remove / &onedex_cache.lp_info.lp_supply;
                    let payment =
                        EsdtTokenPayment::new(onedex_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    onedex_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.onedex_lp_proxy_obj()
                        .contract(onedex_cache.sc_address.clone())
                        .remove_liquidity(legld_to_remove, egld_to_remove.clone(), false)
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::Xexchange => {
                    if lp_to_remove > xexchange_cache.lp_info.lp_balance {
                        lp_to_remove = xexchange_cache.lp_info.lp_balance.clone();
                    }
                    let legld_to_remove =
                        &xexchange_cache.lp_info.liquid_reserve * &lp_to_remove / &xexchange_cache.lp_info.lp_supply;
                    egld_to_remove =
                        &xexchange_cache.lp_info.egld_reserve * &lp_to_remove / &xexchange_cache.lp_info.lp_supply;
                    let payment
                        = EsdtTokenPayment::new(xexchange_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    xexchange_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.xexchange_lp_proxy_obj()
                        .contract(xexchange_cache.sc_address.clone())
                        .remove_liquidity(legld_to_remove, egld_to_remove.clone())
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::None => {}
            }
            left_amount -= &egld_to_remove;
        }

        // unwrap WEGLD
        let wegld_balance =
            self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(storage_cache.wegld_id.clone()), 0);
        if wegld_balance > 0 {
            let payment = EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, wegld_balance);
            self.wrap_lp_proxy_obj()
                .contract(xexchange_cache.wrap_sc_address)
                .unwrap_egld()
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }

        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();
        let removed_egld = &new_egld_balance - &old_egld_balance;
        let removed_legld = &new_ls_balance - &old_ls_balance;
        if lp_cache.egld_in_lp >= removed_egld {
            lp_cache.egld_in_lp -= removed_egld;
        } else {
            let excess = &removed_egld - &lp_cache.egld_in_lp;
            lp_cache.egld_in_lp = BigUint::zero();
            lp_cache.excess_lp_egld += excess;
        }
        if lp_cache.legld_in_lp >= removed_legld {
            lp_cache.legld_in_lp -= removed_legld;
        } else {
            let excess = &removed_legld - &lp_cache.legld_in_lp;
            lp_cache.legld_in_lp = BigUint::zero();
            lp_cache.excess_lp_legld += excess;
        }
    }

    fn remove_legld_lp(&self, amount: BigUint, lp_cache: &mut LpCache<Self>) {
        if !self.is_arbitrage_active() {
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

        let storage_cache = StorageCache::new(self);
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

            // find the exchange with the most expensive LEGLD
            let one = BigUint::from(ONE_EGLD);
            let mut best_price = BigUint::zero();
            let mut best_exchange = Exchange::None;
            for lp in lps.iter() {
                let legld_per_lp = &one * &lp.liquid_reserve / &lp.lp_supply;
                if best_price < legld_per_lp {
                    best_price = legld_per_lp;
                    best_exchange = lp.exchange;
                }
            }

            if best_exchange == Exchange::None {
                break
            }

            // remove LP
            let mut lp_to_remove = &one * &left_amount / &best_price;
            let mut legld_to_remove = BigUint::zero();
            match best_exchange {
                Exchange::Onedex => {
                    if lp_to_remove > onedex_cache.lp_info.lp_balance {
                        lp_to_remove = onedex_cache.lp_info.lp_balance.clone();
                    }
                    let egld_to_remove =
                        &onedex_cache.lp_info.egld_reserve * &lp_to_remove / &onedex_cache.lp_info.lp_supply;
                    legld_to_remove =
                        &onedex_cache.lp_info.liquid_reserve * &lp_to_remove / &onedex_cache.lp_info.lp_supply;
                    let payment =
                        EsdtTokenPayment::new(onedex_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    onedex_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.onedex_lp_proxy_obj()
                        .contract(onedex_cache.sc_address.clone())
                        .remove_liquidity(legld_to_remove.clone(), egld_to_remove, false)
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::Xexchange => {
                    if lp_to_remove > xexchange_cache.lp_info.lp_balance {
                        lp_to_remove = xexchange_cache.lp_info.lp_balance.clone();
                    }
                    let egld_to_remove =
                        &xexchange_cache.lp_info.egld_reserve * &lp_to_remove / &xexchange_cache.lp_info.lp_supply;
                    legld_to_remove =
                        &xexchange_cache.lp_info.liquid_reserve * &lp_to_remove / &xexchange_cache.lp_info.lp_supply;
                    let payment
                        = EsdtTokenPayment::new(xexchange_cache.lp_info.lp_token.clone(), 0, lp_to_remove.clone());
                    xexchange_cache.lp_info.lp_balance -= &lp_to_remove;
                    self.xexchange_lp_proxy_obj()
                        .contract(xexchange_cache.sc_address.clone())
                        .remove_liquidity(legld_to_remove.clone(), egld_to_remove)
                        .with_esdt_transfer(payment)
                        .execute_on_dest_context::<()>();
                }
                Exchange::None => {}
            }
            left_amount -= &legld_to_remove;
        }

        // unwrap WEGLD
        let wegld_balance =
            self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(storage_cache.wegld_id.clone()), 0);
        if wegld_balance > 0 {
            let payment = EsdtTokenPayment::new(storage_cache.wegld_id.clone(), 0, wegld_balance);
            self.wrap_lp_proxy_obj()
                .contract(xexchange_cache.wrap_sc_address)
                .unwrap_egld()
                .with_esdt_transfer(payment)
                .execute_on_dest_context::<()>();
        }

        let (new_egld_balance, new_ls_balance) = self.get_sc_balances();
        let removed_egld = &new_egld_balance - &old_egld_balance;
        let removed_legld = &new_ls_balance - &old_ls_balance;
        if lp_cache.egld_in_lp >= removed_egld {
            lp_cache.egld_in_lp -= removed_egld;
        } else {
            let excess = &removed_egld - &lp_cache.egld_in_lp;
            lp_cache.egld_in_lp = BigUint::zero();
            lp_cache.excess_lp_egld += excess;
        }
        if lp_cache.legld_in_lp >= removed_legld {
            lp_cache.legld_in_lp -= removed_legld;
        } else {
            let excess = &removed_legld - &lp_cache.legld_in_lp;
            lp_cache.legld_in_lp = BigUint::zero();
            lp_cache.excess_lp_legld += excess;
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
