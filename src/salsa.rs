#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod consts;
pub mod delegation_proxy;
pub mod errors;

use crate::{config::*, consts::*, errors::*};

#[multiversx_sc::contract]
pub trait SalsaContract<ContractReader>:
    config::ConfigModule
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

        let delegate_amount = self.call_value().egld_value();
        require!(
            delegate_amount >= MIN_EGLD_TO_DELEGATE,
            ERROR_INSUFFICIENT_DELEGATE_AMOUNT
        );

        let caller = self.blockchain().get_caller();
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
                let ls_amount = self.add_liquidity(&staked_tokens);
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

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_to_undelegate = self.remove_liquidity(&payment.amount);
        self.burn_liquid_token(&payment.amount);
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + UNBOND_PERIOD;
        self.users_egld_to_undelegate()
            .update(|value| *value += &egld_to_undelegate);
        self.add_user_undelegation(egld_to_undelegate, unbond_epoch);
    }

    fn add_user_undelegation(&self, amount: BigUint, unbond_epoch: u64) {
        let user = self.blockchain().get_caller();
        let mut user_undelegations = self.user_undelegations(&user).get();
        require!(
            user_undelegations.len() < MAX_USER_UNDELEGATIONS,
            ERROR_TOO_MANY_USER_UNDELEGATIONS
        );

        let undelegation = config::Undelegation {
            amount: amount.clone(),
            unbond_epoch,
        };
        let mut found = false;
        for mut user_undelegation in user_undelegations.into_iter() {
            if user_undelegation.unbond_epoch == unbond_epoch {
                user_undelegation.amount += &amount;
                found = true;
                break;
            }
        }
        if !found {
            user_undelegations.push(undelegation.clone());
        }
        self.user_undelegations(&user).set(user_undelegations);

        let mut total_user_undelegations = self.total_user_undelegations().get();
        found = false;
        for mut total_user_undelegation in total_user_undelegations.into_iter() {
            if total_user_undelegation.unbond_epoch == unbond_epoch {
                total_user_undelegation.amount += &amount;
                found = true;
                break;
            }
        }
        if !found {
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
        let mut remaining_undelegations: ManagedVec<Self::Api, config::Undelegation<Self::Api>> =
            ManagedVec::new();
        let mut withdraw_amount = BigUint::zero();
        let mut overflow = false;
        for user_undelegation in &user_undelegations {
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
            reserve_amount >= MIN_EGLD_TO_DELEGATE,
            ERROR_INSUFFICIENT_RESERVE_AMOUNT
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
        require!(old_reserve >= amount, ERROR_NOT_ENOUGH_FUNDS);

        self.compute_withdrawn();
        
        let mut egld_to_remove = amount.clone();
        let mut points_to_remove = self.get_reserve_points_amount(&egld_to_remove);
        // don't leave dust
        if &old_reserve - &amount < MIN_EGLD_TO_DELEGATE {
            egld_to_remove = old_reserve.clone();
            points_to_remove = old_reserve_points.clone();
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
                config::Undelegation<Self::Api>,
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

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_to_undelegate = self.remove_liquidity(&payment.amount);
        self.burn_liquid_token(&payment.amount);
        require!(
            egld_to_undelegate >= MIN_EGLD_TO_DELEGATE,
            ERROR_BAD_PAYMENT_AMOUNT
        );

        let available_egld_reserve = self.available_egld_reserve().get();
        let total_egld_staked = self.total_egld_staked().get();
        let fee = self.undelegate_now_fee().get();
        let egld_to_undelegate_with_fee =
            egld_to_undelegate.clone() - egld_to_undelegate.clone() * fee / MAX_PERCENT;
        require!(
            egld_to_undelegate_with_fee <= available_egld_reserve,
            ERROR_NOT_ENOUGH_FUNDS
        );
        require!(egld_to_undelegate <= total_egld_staked, ERROR_NOT_ENOUGH_FUNDS);

        // add to reserve undelegations
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + UNBOND_PERIOD;
        let mut reserve_undelegations = self.reserve_undelegations().get();
        let mut found = false;
        for mut reserve_undelegation in reserve_undelegations.into_iter() {
            if reserve_undelegation.unbond_epoch == unbond_epoch {
                reserve_undelegation.amount += &egld_to_undelegate;
                found = true;
                break;
            }
        }
        if !found {
            let undelegation = config::Undelegation {
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

        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &egld_to_undelegate_with_fee);
    }

    // endpoints: service

    #[endpoint(unDelegateAll)]
    fn undelegate_all(&self) {
        let users_egld_to_undelegate = self.users_egld_to_undelegate().get();
        let reserves_egld_to_undelegate = self.egld_to_replenish_reserve().get();
        let total_egld_to_undelegate = &users_egld_to_undelegate + &reserves_egld_to_undelegate;
        require!(
            total_egld_to_undelegate >= MIN_EGLD_TO_DELEGATE,
            ERROR_INSUFFICIENT_DELEGATE_AMOUNT
        );

        self.users_egld_to_undelegate().clear();
        self.egld_to_replenish_reserve().clear();
        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();
        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(total_egld_to_undelegate)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).undelegate_all_callback(users_egld_to_undelegate, reserves_egld_to_undelegate),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_all_callback(
        &self,
        users_egld_to_undelegate: BigUint,
        reserves_egld_to_undelegate: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.users_egld_to_undelegate()
                    .update(|value| *value += users_egld_to_undelegate);
                self.egld_to_replenish_reserve()
                    .update(|value| *value += reserves_egld_to_undelegate);
            }
        }
    }

    #[endpoint(compound)]
    fn compound(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let this_contract = self.blockchain().get_sc_address();
        let gas_for_async_call = self.get_gas_for_async_call();
        let claimable_rewards_amount = self.claimable_rewards_amount().get();
        let claimable_rewards_epoch = self.claimable_rewards_epoch().get();
        let current_epoch = self.blockchain().get_block_epoch();

        if claimable_rewards_amount == 0 || claimable_rewards_epoch != current_epoch {
            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .get_claimable_rewards(this_contract)
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).get_claimable_rewards_callback(current_epoch),
                )
                .call_and_exit()
        } else {
            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .redelegate_rewards()
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).compound_callback(claimable_rewards_amount),
                )
                .call_and_exit()
        }
    }

    #[callback]
    fn get_claimable_rewards_callback(
        &self,
        current_epoch: u64,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(total_rewards) => {
                self.claimable_rewards_amount().set(total_rewards);
                self.claimable_rewards_epoch().set(current_epoch);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[callback]
    fn compound_callback(
        &self,
        claimable_rewards: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.total_egld_staked()
                    .update(|value| *value += claimable_rewards);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
        self.claimable_rewards_amount().clear();
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .withdraw()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(SalsaContract::callbacks(self).withdraw_all_callback())
            .call_and_exit()
    }

    #[callback]
    fn withdraw_all_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdrawn_amount = self.call_value().egld_value();
                self.total_withdrawn_egld()
                    .update(|value| *value += withdrawn_amount);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(computeWithdrawn)]
    fn compute_withdrawn(&self) {
        let current_epoch = self.blockchain().get_block_epoch();
        let mut total_withdrawn_egld = self.total_withdrawn_egld().get();
        let mut users_withdrawn_egld = self.user_withdrawn_egld().get();
        let mut available_egld_reserve = self.available_egld_reserve().get();

        // compute user undelegations eligible for withdraw
        let user_undelegations = self.total_user_undelegations().get();
        let mut remaining_users_undelegations: ManagedVec<
            Self::Api,
            config::Undelegation<Self::Api>,
        > = ManagedVec::new();
        for mut user_undelegation in &user_undelegations {
            if user_undelegation.unbond_epoch <= current_epoch {
                let mut egld_to_unbond = user_undelegation.amount.clone();
                if egld_to_unbond > total_withdrawn_egld {
                    egld_to_unbond = total_withdrawn_egld.clone();
                }
                total_withdrawn_egld -= &egld_to_unbond;
                users_withdrawn_egld += &egld_to_unbond;
                user_undelegation.amount -= &egld_to_unbond;
            }
            if user_undelegation.amount > 0 {
                remaining_users_undelegations.push(user_undelegation);
            }
        }

        self.user_withdrawn_egld().set(users_withdrawn_egld);
        self.total_user_undelegations()
            .set(remaining_users_undelegations);

        // compute reserve undelegations eligible for withdraw
        let reserve_undelegations = self.reserve_undelegations().get();
        let mut remaining_reserve_undelegations: ManagedVec<
            Self::Api,
            config::Undelegation<Self::Api>,
        > = ManagedVec::new();
        for mut reserve_undelegation in &reserve_undelegations {
            if reserve_undelegation.unbond_epoch <= current_epoch {
                let mut egld_to_unbond = reserve_undelegation.amount.clone();
                if egld_to_unbond > total_withdrawn_egld {
                    egld_to_unbond = total_withdrawn_egld.clone();
                }
                total_withdrawn_egld -= &egld_to_unbond;
                available_egld_reserve += &egld_to_unbond;
                reserve_undelegation.amount -= &egld_to_unbond;
            }
            if reserve_undelegation.amount > 0 {
                remaining_reserve_undelegations.push(reserve_undelegation);
            }
        }

        self.available_egld_reserve().set(available_egld_reserve);
        self.reserve_undelegations()
            .set(remaining_reserve_undelegations);
        
        self.total_withdrawn_egld()
            .set(&total_withdrawn_egld);
    }

    // helpers

    fn get_gas_for_async_call(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK,
            ERROR_INSUFFICIENT_GAS
        );

        gas_left - MIN_GAS_FOR_CALLBACK
    }

    fn add_liquidity(&self, new_stake_amount: &BigUint) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        let ls_amount = if total_egld_staked > 0 {
            new_stake_amount * &liquid_token_supply / &total_egld_staked
        } else {
            new_stake_amount.clone()
        };

        require!(ls_amount > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

        self.total_egld_staked()
            .update(|value| *value += new_stake_amount);
        self.liquid_token_supply()
            .update(|value| *value += &ls_amount);

        ls_amount
    }

    fn remove_liquidity(&self, ls_amount: &BigUint) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        require!(
            &liquid_token_supply >= ls_amount,
            ERROR_NOT_ENOUGH_LIQUID_SUPPLY
        );
        require!(ls_amount > &0, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_amount = ls_amount * &total_egld_staked / &liquid_token_supply;
        require!(egld_amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        self.total_egld_staked()
            .update(|value| *value -= &egld_amount);
        self.liquid_token_supply()
            .update(|value| *value -= ls_amount);

        egld_amount
    }

    fn mint_liquid_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.liquid_token_id().mint(amount)
    }

    fn burn_liquid_token(&self, amount: &BigUint) {
        self.liquid_token_id().burn(amount);
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
