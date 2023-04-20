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
        self.busy_reserve_undelegations().set(State::Inactive);
        self.operation().set(Operation::Idle);
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
        
        let operation = self.operation().get();
        require!(operation == Operation::Idle, ERROR_BUSY_COMPOUNDING);

        self.operation().set(Operation::Undelegating);
        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_to_unstake = self.remove_liquidity(&payment.amount);
        require!(
            egld_to_unstake >= MIN_EGLD_TO_DELEGATE,
            ERROR_BAD_PAYMENT_AMOUNT
        );

        self.burn_liquid_token(&payment.amount);
        let delegation_contract = self.provider_address().get();
        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(egld_to_unstake.clone())
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).undelegate_callback(caller, egld_to_unstake),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_callback(
        &self,
        caller: ManagedAddress,
        egld_amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let current_epoch = self.blockchain().get_block_epoch();
                let unbond_epoch = current_epoch + UNBOND_PERIOD;
                let undelegation = config::Undelegation {
                    amount: egld_amount,
                    unbond_epoch,
                };
                self.user_undelegations(&caller)
                    .update(|undelegations| undelegations.push(undelegation));
            }
            ManagedAsyncCallResult::Err(_) => {
                let ls_token_amount = self.add_liquidity(&egld_amount);
                let user_payment = self.mint_liquid_token(ls_token_amount);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
        }
        self.operation().set(Operation::Idle);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        require!(
            self.backup_user_undelegations(&caller).is_empty(),
            ERROR_WITHDRAW_BUSY,
        );

        let current_epoch = self.blockchain().get_block_epoch();

        let user_undelegations = self.user_undelegations(&caller).take();
        self.backup_user_undelegations(&caller)
            .set(user_undelegations.clone());

        let mut remaining_undelegations: ManagedVec<Self::Api, config::Undelegation<Self::Api>> =
            ManagedVec::new();
        let mut withdraw_amount = BigUint::zero();
        for user_undelegation in &user_undelegations {
            if user_undelegation.unbond_epoch <= current_epoch {
                withdraw_amount += user_undelegation.amount;
            } else {
                remaining_undelegations.push(user_undelegation);
            }
        }
        require!(withdraw_amount > 0, ERROR_NOTHING_TO_WITHDRAW);

        self.user_undelegations(&caller)
            .set(remaining_undelegations);

        let total_user_withdrawn_egld = self.user_withdrawn_egld().get();
        if withdraw_amount <= total_user_withdrawn_egld {
            self.send().direct_egld(&caller, &withdraw_amount);
            self.user_withdrawn_egld()
                .update(|value| *value -= withdraw_amount);
            self.backup_user_undelegations(&caller).clear();
        } else {
            let delegation_contract = self.provider_address().get();
            let gas_for_async_call = self.get_gas_for_async_call();
            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .withdraw()
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).withdraw_callback(caller, withdraw_amount),
                )
                .call_and_exit()
        }
    }

    #[callback]
    fn withdraw_callback(
        &self,
        caller: ManagedAddress,
        user_withdraw_amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.compute_withdrawn_amount();
                let user_withdrawn_egld_mapper = self.user_withdrawn_egld();
                let new_total_user_withdrawn_egld = user_withdrawn_egld_mapper.get();
                if user_withdraw_amount <= new_total_user_withdrawn_egld {
                    self.send().direct_egld(&caller, &user_withdraw_amount);
                    user_withdrawn_egld_mapper.update(|value| *value -= user_withdraw_amount);
                    self.backup_user_undelegations(&caller).clear();
                } else {
                    let backup_user_undelegations = self.backup_user_undelegations(&caller).take();
                    self.user_undelegations(&caller)
                        .set(backup_user_undelegations);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                let backup_user_undelegations = self.backup_user_undelegations(&caller).take();
                self.user_undelegations(&caller)
                    .set(backup_user_undelegations);
            }
        }
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

        let mut idx = self.reservers_addresses(caller.clone()).get();
        if idx == 0 {
            self.users_reserves().push(&reserve_amount);
            idx = self.users_reserves().len();
            self.reservers_addresses(caller.clone()).set(idx);
            self.reservers_ids()
                .entry(idx)
                .or_insert(caller);
        } else {
            let old_reserve = self.users_reserves().get(idx);
            self.users_reserves().set(idx, &(&old_reserve + &reserve_amount));
        }

        self.egld_reserve().update(|value| *value += &reserve_amount);
        self.available_egld_reserve().update(|value| *value += reserve_amount);
    }

    #[endpoint(removeReserve)]
    fn remove_reserve(&self, amount: BigUint) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let available_egld_reserve = self.available_egld_reserve().get();
        require!(available_egld_reserve >= amount, ERROR_NOT_ENOUGH_FUNDS);

        let idx = self.reservers_addresses(caller.clone()).get();
        require!(idx > 0, ERROR_USER_NOT_PROVIDER);

        let old_reserve = self.users_reserves().get(idx);
        require!(old_reserve >= amount, ERROR_NOT_ENOUGH_FUNDS);

        if old_reserve > amount {
            self.users_reserves().set(idx, &(&old_reserve - &amount));
        } else {
            let n = self.users_reserves().len();
            let data = self.reservers_ids().get(&n);
            if let Some(moved_address) = data {
                self.reservers_ids()
                    .entry(idx)
                    .and_modify(|value| *value = moved_address.clone());
                self.reservers_ids().remove(&n);
                self.reservers_addresses(caller.clone()).set(0);
                if n != idx {
                    self.reservers_addresses(moved_address).set(idx);
                }
                self.users_reserves().swap_remove(idx);
            } else {
                sc_panic!(ERROR_USER_NOT_PROVIDER);
            }
        }

        self.send().direct_egld(&caller, &amount);
        self.egld_reserve().update(|value| *value -= &amount);
        self.available_egld_reserve().update(|value| *value -= amount);
    }

    #[payable("*")]
    #[endpoint(unDelegateNow)]
    fn undelegate_now(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let operation = self.operation().get();
        require!(operation == Operation::Idle, ERROR_BUSY_COMPOUNDING);

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let egld_reserve = self.egld_reserve().get();
        let available_egld_reserve = self.available_egld_reserve().get();
        let total_egld_staked = self.total_egld_staked().get();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let fee = self.undelegate_now_fee().get();
        let caller = self.blockchain().get_caller();
        let egld_to_unstake = self.remove_liquidity(&payment.amount);
        self.burn_liquid_token(&payment.amount);
        require!(
            egld_to_unstake >= MIN_EGLD_TO_DELEGATE,
            ERROR_BAD_PAYMENT_AMOUNT
        );

        let egld_to_unstake_with_fee =
            egld_to_unstake.clone() - egld_to_unstake.clone() * fee / MAX_PERCENT;
        require!(
            egld_to_unstake_with_fee <= available_egld_reserve,
            ERROR_NOT_ENOUGH_FUNDS
        );
        require!(egld_to_unstake <= total_egld_staked, ERROR_NOT_ENOUGH_FUNDS);

        let total_rewards = &egld_to_unstake - &egld_to_unstake_with_fee;
        let mut distributed_rewards = BigUint::zero();
        let n = self.users_reserves().len();
        let mut i: usize = 0;
        for mut reserve in self.users_reserves().into_iter() {
            i += 1;
            let mut reward = &reserve * &total_rewards / &egld_reserve;
            if i == n {
                reward = &total_rewards - &distributed_rewards;
            }
            reserve += &reward;
            distributed_rewards += reward;
            self.users_reserves().set(i, &reserve);
        }

        self.egld_to_replenish_reserve()
            .update(|value| *value += &egld_to_unstake);
        self.send().direct_egld(&caller, &egld_to_unstake_with_fee);
        self.available_egld_reserve().update(|value| *value -= &egld_to_unstake_with_fee);
        self.egld_reserve().update(|value| *value += &total_rewards);
    }

    // endpoints: service

    #[endpoint(undelegateReserves)]
    fn undelegate_reserves(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);
        require!(!self.is_reserve_undelegations_busy(), ERROR_BUSY_UNDELEGATE_RESERVES);

        let total_egld_to_unstake = self.egld_to_replenish_reserve().get();
        require!(total_egld_to_unstake > 0, ERROR_NOT_ENOUGH_FUNDS);

        let operation = self.operation().get();
        require!(operation == Operation::Idle, ERROR_BUSY_OPERATION);

        self.busy_reserve_undelegations().set(State::Active);

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();
        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(&total_egld_to_unstake)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).undelegate_reserves_callback(total_egld_to_unstake),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_reserves_callback(
        &self,
        egld_to_unstake: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let current_epoch = self.blockchain().get_block_epoch();
                let unbond_epoch = current_epoch + UNBOND_PERIOD;
                let reserve_undelegations = self.reserve_undelegations().get();
                let mut found = false;
                for mut reserve_undelegation in reserve_undelegations.into_iter() {
                    if reserve_undelegation.unbond_epoch == unbond_epoch {
                        reserve_undelegation.amount += &egld_to_unstake;
                        found = true;
                        break;
                    }
                }
                self.reserve_undelegations().set(reserve_undelegations);
                if !found {
                    let undelegation = config::Undelegation {
                        amount: egld_to_unstake.clone(),
                        unbond_epoch,
                    };
                    self.reserve_undelegations()
                        .update(|undelegations| undelegations.push(undelegation));
                }
                self.egld_to_replenish_reserve()
                    .update(|value| *value -= egld_to_unstake);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
        self.busy_reserve_undelegations().set(State::Inactive);
    }

    #[endpoint(compound)]
    fn compound(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);
        require!(!self.is_reserve_undelegations_busy(), ERROR_BUSY_UNDELEGATE_RESERVES);

        let operation = self.operation().get();
        require!(operation == Operation::Idle, ERROR_BUSY_UNDELEGATING);

        let replenish = self.egld_to_replenish_reserve().get();
        require!(replenish == 0, ERROR_REPLENISH_NOT_EMPTY);

        self.operation().set(Operation::Compounding);
        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .redelegate_rewards()
            .with_gas_limit(gas_for_async_call)
            .transfer_execute();
    }

    #[endpoint(updateTotalEgldStaked)]
    fn update_total_egld_staked(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let operation = self.operation().get();
        require!(operation == Operation::Compounding, ERROR_NOT_COMPOUNDING);

        let delegation_contract = self.provider_address().get();
        let this_contract = self.blockchain().get_sc_address();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .get_user_active_stake(this_contract)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(SalsaContract::callbacks(self).update_egld_staked_callback())
            .call_and_exit()
    }

    #[callback]
    fn update_egld_staked_callback(&self, #[call_result] result: ManagedAsyncCallResult<BigUint>) {
        match result {
            ManagedAsyncCallResult::Ok(total_stake) => {
                let total_egld_staked = self.total_egld_staked().get();
                require!(total_stake > total_egld_staked, ERROR_NOT_ENOUGH_FUNDS);

                self.total_egld_staked().set(total_stake);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
        self.operation().set(Operation::Idle);
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
                self.compute_withdrawn_amount();
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
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

    fn compute_withdrawn_amount(&self) {
        let withdrawn_amount = self.call_value().egld_value();
        if withdrawn_amount == 0 {
            return;
        }

        let current_epoch = self.blockchain().get_block_epoch();
        let reserve_undelegations = self.reserve_undelegations().get();
        let mut remaining_reserve_undelegations: ManagedVec<
            Self::Api,
            config::Undelegation<Self::Api>,
        > = ManagedVec::new();
        let mut reserve_withdraw_amount = BigUint::zero();
        for reserve_undelegation in &reserve_undelegations {
            if reserve_undelegation.unbond_epoch <= current_epoch {
                reserve_withdraw_amount += reserve_undelegation.amount;
            } else {
                remaining_reserve_undelegations.push(reserve_undelegation);
            }
        }

        if withdrawn_amount < reserve_withdraw_amount {
            self.user_withdrawn_egld()
                .update(|value| *value += withdrawn_amount);
            return;
        }

        self.reserve_undelegations()
            .set(remaining_reserve_undelegations);

        let user_withdrawn_amount = &withdrawn_amount - &reserve_withdraw_amount;
        self.user_withdrawn_egld()
            .update(|value| *value += user_withdrawn_amount);
        self.available_egld_reserve()
            .update(|value| *value += reserve_withdraw_amount);
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
