multiversx_sc::imports!();

use crate::{common::consts::*, common::errors::*};
use crate::proxies::delegation_proxy;
use crate::common::config::{UndelegationType};

#[multiversx_sc::module]
pub trait ServiceModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
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
                    ServiceModule::callbacks(self).compound_callback(claimable_rewards_amount),
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
                self.claimable_rewards_amount().clear();
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
                    .update(|value| *value += withdrawn_amount.clone_value());
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(computeWithdrawn)]
    fn compute_withdrawn(&self) {
        let current_epoch = self.blockchain().get_block_epoch();
        let total_withdrawn_egld = self.total_withdrawn_egld().get();
        let caller = self.blockchain().get_caller();

        // compute user undelegations eligible for withdraw
        let (mut left_amount, _) = self.remove_undelegations(
            total_withdrawn_egld.clone(),
            current_epoch,
            self.ltotal_user_undelegations(),
            UndelegationType::TotalUsersList,
            caller.clone()
        );
        let withdrawn_for_users = &total_withdrawn_egld - &left_amount;
        self.user_withdrawn_egld()
            .update(|value| *value += &withdrawn_for_users);

        // compute reserve undelegations eligible for withdraw
        (left_amount, _) = self.remove_undelegations(
            left_amount,
            current_epoch,
            self.lreserve_undelegations(),
            UndelegationType::ReservesList,
            caller
        );
        let withdrawn_for_reserves = &total_withdrawn_egld - &left_amount - &withdrawn_for_users;
        self.available_egld_reserve()
            .update(|value| *value += withdrawn_for_reserves);
        
        self.total_withdrawn_egld()
            .set(&left_amount);
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