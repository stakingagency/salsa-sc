#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod proxies;
mod liquidity;
mod service;
mod onedex;

use crate::{common::config::*, common::consts::*, common::errors::*};

#[multiversx_sc::contract]
pub trait SalsaContract<ContractReader>:
    common::config::ConfigModule
    + liquidity::LiquidityModule
    + service::ServiceModule
    + onedex::OnedexModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self) {
        self.state().set(State::Inactive);
    }

    // endpoints: liquid delegation

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) -> EsdtTokenPayment<Self::Api> {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut delegate_amount = self.call_value().egld_value();
        require!(
            delegate_amount >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let caller = self.blockchain().get_caller();
        let liquid_token_id = self.liquid_token_id().get_token_id();

        // arbitrage
        let salsa_amount_out = self.add_liquidity(&delegate_amount, false);
        let sold_amount = self.do_arbitrage_on_onedex(
            &TokenIdentifier::from(WEGLD_ID), &delegate_amount, &salsa_amount_out
        );
        delegate_amount -= sold_amount;

        if delegate_amount > 0 {
            // normal delegate
            let delegation_contract = self.provider_address().get();
            let gas_for_async_call = self.get_gas_for_async_call();

            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .delegate()
                .with_gas_limit(gas_for_async_call)
                .with_egld_transfer(delegate_amount.clone())
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).delegate_callback(caller, delegate_amount),
                )
                .call_and_exit()
        } else {
            EsdtTokenPayment::new(liquid_token_id, 0, salsa_amount_out)
        }
    }

    #[callback]
    fn delegate_callback(
        &self,
        caller: ManagedAddress,
        staked_tokens: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let ls_amount = self.add_liquidity(&staked_tokens, true);
                let user_payment = self.mint_liquid_token(ls_amount);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send().direct_egld(&caller, &staked_tokens);
            }
        }
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn undelegate(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        // arbitrage
        let salsa_amount_out = self.remove_liquidity(&payment.amount, false);
        let sold_amount = self.do_arbitrage_on_onedex(
            &liquid_token_id, &payment.amount, &salsa_amount_out
        );
        payment.amount -= sold_amount;
        if payment.amount == 0 {
            return
        }

        // normal undelegate
        let egld_to_undelegate = self.remove_liquidity(&payment.amount, true);
        self.burn_liquid_token(&payment.amount);
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + self.unbond_period().get();
        self.users_egld_to_undelegate()
            .update(|value| *value += &egld_to_undelegate);
        self.add_user_undelegation(egld_to_undelegate, unbond_epoch);
    }

    fn add_user_undelegation(&self, amount: BigUint, unbond_epoch: u64) {
        let user = self.blockchain().get_caller();
        let mut user_undelegations = self.user_undelegations(&user).get();
        let undelegation = Undelegation {
            amount: amount.clone(),
            unbond_epoch,
        };
        let mut found = false;
        let mut idx = 0;
        for mut user_undelegation in user_undelegations.into_iter() {
            if user_undelegation.unbond_epoch == unbond_epoch {
                user_undelegation.amount += &amount;
                let _ = user_undelegations.set(idx, &user_undelegation);
                found = true;
                break;
            }
            idx += 1;
        }
        if !found {
            require!(
                user_undelegations.len() < MAX_USER_UNDELEGATIONS,
                ERROR_TOO_MANY_UNDELEGATIONS
            );
            user_undelegations.push(undelegation.clone());
        }
        self.user_undelegations(&user).set(user_undelegations);

        let mut total_user_undelegations = self.total_user_undelegations().get();
        found = false;
        idx = 0;
        for mut total_user_undelegation in total_user_undelegations.into_iter() {
            if total_user_undelegation.unbond_epoch == unbond_epoch {
                total_user_undelegation.amount += &amount;
                let _ = total_user_undelegations.set(idx, &total_user_undelegation);
                found = true;
                break;
            }
            idx += 1;
        }
        if !found {
            require!(
                total_user_undelegations.len() < MAX_EPOCH_UNDELEGATIONS,
                ERROR_TOO_MANY_UNDELEGATIONS
            );
            total_user_undelegations.push(undelegation);
        }
        self.total_user_undelegations().set(total_user_undelegations);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.compute_withdrawn();

        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let total_user_withdrawn_egld = self.user_withdrawn_egld().get();
        let user_undelegations = self.user_undelegations(&caller).get();
        let mut remaining_undelegations: ManagedVec<Self::Api, Undelegation<Self::Api>> =
            ManagedVec::new();
        let mut withdraw_amount = BigUint::zero();
        let mut overflow = false;
        for user_undelegation in user_undelegations.into_iter() {
            let new_withdraw_amount = &withdraw_amount + &user_undelegation.amount;
            let would_overflow = new_withdraw_amount > total_user_withdrawn_egld;
            overflow = overflow || would_overflow;
            if user_undelegation.unbond_epoch <= current_epoch && !would_overflow {
                withdraw_amount = new_withdraw_amount;
            } else {
                remaining_undelegations.push(user_undelegation);
            }
        }
        if withdraw_amount == 0 {
            if overflow {
                sc_panic!(ERROR_NOT_ENOUGH_FUNDS);
            } else {
                sc_panic!(ERROR_NOTHING_TO_WITHDRAW);
            }
        }

        self.user_undelegations(&caller)
            .set(remaining_undelegations);
        self.user_withdrawn_egld()
            .update(|value| *value -= &withdraw_amount);
        self.send().direct_egld(&caller, &withdraw_amount);
    }

    // endpoints: reserves

    #[payable("EGLD")]
    #[endpoint(addReserve)]
    fn add_reserve(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let reserve_amount = self.call_value().egld_value();
        require!(
            reserve_amount >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let user_reserve_points = self.get_reserve_points_amount(&reserve_amount);

        self.users_reserve_points(&caller)
            .update(|value| *value += &user_reserve_points);
        self.reserve_points()
            .update(|value| *value += user_reserve_points);

        self.egld_reserve().update(|value| *value += &reserve_amount);
        self.available_egld_reserve().update(|value| *value += reserve_amount);
    }

    #[endpoint(removeReserve)]
    fn remove_reserve(&self, amount: BigUint) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let old_reserve_points = self.users_reserve_points(&caller).get();
        let old_reserve = self.get_reserve_egld_amount(&old_reserve_points);
        require!(old_reserve > 0, ERROR_USER_NOT_PROVIDER);
        require!(old_reserve >= amount, ERROR_NOT_ENOUGH_FUNDS);

        self.compute_withdrawn();
        
        let mut egld_to_remove = amount.clone();
        let mut points_to_remove = self.get_reserve_points_amount(&egld_to_remove);
        if &old_reserve - &amount < DUST_THRESHOLD {
            // avoid rounding issues
            points_to_remove = old_reserve_points.clone();
            egld_to_remove = old_reserve.clone();
        } else {
            require!(&old_reserve - &amount >= MIN_EGLD, ERROR_DUST_REMAINING);
        }

        let available_egld_reserve = self.available_egld_reserve().get();
        // if there is not enough available reserve, move the reserve to user undelegation
        if egld_to_remove > available_egld_reserve {
            let egld_to_move = &egld_to_remove - &available_egld_reserve;
            let mut remaining_egld = egld_to_move.clone();
            let mut unbond_epoch = 0_u64;
            let reserve_undelegations = self.reserve_undelegations().get();
            let mut remaining_reserve_undelegations: ManagedVec<
                Self::Api,
                Undelegation<Self::Api>,
            > = ManagedVec::new();
            for mut reserve_undelegation in reserve_undelegations.into_iter() {
                if remaining_egld > 0 {
                    if remaining_egld <= reserve_undelegation.amount {
                        reserve_undelegation.amount -= &remaining_egld;
                        remaining_egld = BigUint::zero();
                        unbond_epoch = reserve_undelegation.unbond_epoch;
                    } else {
                        remaining_egld -= &reserve_undelegation.amount;
                        reserve_undelegation.amount = BigUint::zero();
                    }
                }
                if reserve_undelegation.amount > 0 {
                    remaining_reserve_undelegations.push(reserve_undelegation);
                }
            }
            require!(remaining_egld == 0, ERROR_NOT_ENOUGH_FUNDS);

            self.reserve_undelegations().set(remaining_reserve_undelegations);
            self.egld_reserve().update(|value| *value -= &egld_to_move);
            self.add_user_undelegation(egld_to_move.clone(), unbond_epoch);
            egld_to_remove = available_egld_reserve.clone();
        }
        self.available_egld_reserve().update(|value| *value -= &egld_to_remove);
        self.egld_reserve().update(|value| *value -= &egld_to_remove);
        self.users_reserve_points(&caller)
            .update(|value| *value -= &points_to_remove);
        self.reserve_points()
            .update(|value| *value -= &points_to_remove);
        self.send().direct_egld(&caller, &egld_to_remove);
    }

    #[payable("*")]
    #[endpoint(unDelegateNow)]
    fn undelegate_now(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let fee = self.undelegate_now_fee().get();
        let caller = self.blockchain().get_caller();

        // arbitrage
        let salsa_amount_out = self.remove_liquidity(&payment.amount, false);
        let sold_amount = self.do_arbitrage_on_onedex(
            &liquid_token_id, &payment.amount, &salsa_amount_out
        );
        payment.amount -= sold_amount;
        if payment.amount == 0 {
            return
        };

        // normal unDelegateNow
        let egld_to_undelegate = self.remove_liquidity(&payment.amount, true);
        self.burn_liquid_token(&payment.amount);
        require!(
            egld_to_undelegate >= MIN_EGLD,
            ERROR_BAD_PAYMENT_AMOUNT
        );

        let available_egld_reserve = self.available_egld_reserve().get();
        let total_egld_staked = self.total_egld_staked().get();
        let egld_to_undelegate_with_fee =
            egld_to_undelegate.clone() - egld_to_undelegate.clone() * fee / MAX_PERCENT;
        require!(
            egld_to_undelegate_with_fee <= available_egld_reserve,
            ERROR_NOT_ENOUGH_FUNDS
        );
        require!(egld_to_undelegate <= total_egld_staked, ERROR_NOT_ENOUGH_FUNDS);

        // add to reserve undelegations
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + self.unbond_period().get();
        let mut reserve_undelegations = self.reserve_undelegations().get();
        let mut found = false;
        let mut idx = 0;
        for mut reserve_undelegation in reserve_undelegations.into_iter() {
            if reserve_undelegation.unbond_epoch == unbond_epoch {
                reserve_undelegation.amount += &egld_to_undelegate;
                let _ = reserve_undelegations.set(idx, &reserve_undelegation);
                found = true;
                break;
            }
            idx += 1;
        }
        if !found {
            require!(
                reserve_undelegations.len() < MAX_EPOCH_UNDELEGATIONS,
                ERROR_TOO_MANY_UNDELEGATIONS
            );
            let undelegation = Undelegation {
                amount: egld_to_undelegate.clone(),
                unbond_epoch,
            };
            reserve_undelegations.push(undelegation);
        }
        self.reserve_undelegations().set(reserve_undelegations);

        // update storage
        self.egld_to_replenish_reserve()
            .update(|value| *value += &egld_to_undelegate);
        self.available_egld_reserve().update(|value| *value -= &egld_to_undelegate_with_fee);
        let total_rewards = &egld_to_undelegate - &egld_to_undelegate_with_fee;
        self.egld_reserve().update(|value| *value += &total_rewards);

        self.send().direct_egld(&caller, &egld_to_undelegate_with_fee);
    }

    // endpoints: admin

    #[only_owner]
    #[endpoint(distributeProfit)]
    fn burn_ls_profit(&self) {
        require!(!self.is_state_active(), ERROR_ACTIVE);

        let egld_profit = self.egld_profit().get();
        if egld_profit > 0 {
            self.egld_reserve()
                .update(|value| *value += &egld_profit);
            self.available_egld_reserve()
                .update(|value| *value += &egld_profit);    
            self.egld_profit().clear();
        }

        let ls_profit = self.liquid_profit().get();
        if ls_profit > 0 {
            self.liquid_token_supply()
                .update(|value| *value -= &ls_profit);
            self.burn_liquid_token(&ls_profit);
            self.liquid_profit().clear();
        }
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> proxies::delegation_proxy::Proxy<Self::Api>;
}
