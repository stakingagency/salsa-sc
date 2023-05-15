multiversx_sc::imports!();

use crate::{common::consts::*, common::errors::*};
use crate::common::config::Undelegation;
use crate::proxies::delegation_proxy;

#[multiversx_sc::module]
pub trait ServiceModule:
    crate::common::config::ConfigModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    // endpoints: service

    #[endpoint(unDelegateAll)]
    fn undelegate_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let egld_to_undelegate = self.egld_to_undelegate().take();
        require!(
            egld_to_undelegate >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();
        self.service_delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(egld_to_undelegate.clone())
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                ServiceModule::callbacks(self).undelegate_all_callback(egld_to_undelegate),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_all_callback(
        &self,
        egld_to_undelegate: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_undelegate()
                    .update(|value| *value += egld_to_undelegate);
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
            self.service_delegation_proxy_obj()
                .contract(delegation_contract)
                .get_claimable_rewards(this_contract)
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    ServiceModule::callbacks(self).get_claimable_rewards_callback(current_epoch),
                )
                .call_and_exit()
        } else {
            self.service_delegation_proxy_obj()
                .contract(delegation_contract)
                .redelegate_rewards()
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    ServiceModule::callbacks(self).compound_callback(),
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
    fn compound_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let claimable_rewards = self.claimable_rewards_amount().take();
                self.total_egld_staked()
                    .update(|value| *value += claimable_rewards);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.service_delegation_proxy_obj()
            .contract(delegation_contract)
            .withdraw()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(ServiceModule::callbacks(self).withdraw_all_callback())
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
            Undelegation<Self::Api>,
        > = ManagedVec::new();
        for mut user_undelegation in user_undelegations.into_iter() {
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
            Undelegation<Self::Api>,
        > = ManagedVec::new();
        for mut reserve_undelegation in reserve_undelegations.into_iter() {
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

   // proxy

    #[proxy]
    fn service_delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}