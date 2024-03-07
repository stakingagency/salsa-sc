multiversx_sc::imports!();

use crate::common::storage_cache::StorageCache;
use crate::{common::config::*, common::consts::*, common::errors::*};
use crate::proxies::delegation_proxy::{self};

#[multiversx_sc::module]
pub trait ServiceModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::providers::ProvidersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    // endpoints: service

    #[endpoint(delegateAll)]
    fn delegate_all(&self) -> BigUint {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut storage_cache = StorageCache::new(self);
        require!(
            storage_cache.egld_to_delegate >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let current_block = self.blockchain().get_block_nonce();
        require!(
            storage_cache.last_delegation_block + MIN_BLOCK_BETWEEN_DELEGATIONS <= current_block,
            ERROR_DELEGATE_TOO_SOON
        );

        storage_cache.last_delegation_block = current_block;

        self.reduce_egld_to_delegate_undelegate(&mut storage_cache);
        if storage_cache.egld_to_delegate == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        let (provider_address, amount, _, _) =
            self.get_provider_to_delegate_and_amount(&storage_cache.egld_to_delegate);
        if amount == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        storage_cache.egld_to_delegate -= &amount;
        drop(storage_cache);

        self.service_delegation_proxy_obj()
            .contract(provider_address.clone())
            .delegate()
            .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
            .with_egld_transfer(amount.clone())
            .async_call_promise()
            .with_callback(
                ServiceModule::callbacks(self).delegate_all_callback(provider_address, &amount),
            )
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();

        amount
    }

    #[promises_callback]
    fn delegate_all_callback(
        &self,
        provider_address: ManagedAddress,
        egld_to_delegate: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut provider = self.get_provider(&provider_address);
        provider.funds_last_update_nonce = 0;
        provider.funds_last_update_epoch = 0;
        provider.stake_last_update_nonce = 0;
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_delegate()
                    .update(|value| *value += egld_to_delegate);
            }
        }
        self.providers().insert(provider_address, provider);
    }

    #[endpoint(unDelegateAll)]
    fn undelegate_all(&self) -> BigUint {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut storage_cache = StorageCache::new(self);
        require!(
            storage_cache.egld_to_undelegate >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        self.reduce_egld_to_delegate_undelegate(&mut storage_cache);
        if storage_cache.egld_to_undelegate == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        let (_, _, provider_address, amount) =
            self.get_provider_to_delegate_and_amount(&storage_cache.egld_to_undelegate);
        if amount == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        storage_cache.egld_to_undelegate -= &amount;
        drop(storage_cache);

        self.service_delegation_proxy_obj()
            .contract(provider_address.clone())
            .undelegate(&amount)
            .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
            .async_call_promise()
            .with_callback(
                ServiceModule::callbacks(self).undelegate_all_callback(provider_address, &amount),
            )
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();

        amount
    }

    #[promises_callback]
    fn undelegate_all_callback(
        &self,
        provider_address: ManagedAddress,
        egld_to_undelegate: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut provider = self.get_provider(&provider_address);
        provider.funds_last_update_nonce = 0;
        provider.funds_last_update_epoch = 0;
        provider.stake_last_update_nonce = 0;
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_undelegate()
                    .update(|value| *value += egld_to_undelegate);
            }
        }
        self.providers().insert(provider_address, provider);
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        if !self.refresh_providers() {
            return
        }

        let current_nonce = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            let is_active = provider.is_active();
            let is_up_to_date = provider.is_up_to_date(current_nonce, current_epoch);
            if !is_active || !is_up_to_date || (provider.salsa_rewards == 0) {
                continue
            }

            if !self.enough_gas_left_for_callback() {
                break
            }

            self.service_delegation_proxy_obj()
                .contract(address.clone())
                .claim_rewards()
                .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
                .async_call_promise()
                .with_callback(ServiceModule::callbacks(self).claim_rewards_callback(address))
                .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();
        }
    }

    #[promises_callback]
    fn claim_rewards_callback(
        &self,
        provider_address: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut provider = self.get_provider(&provider_address);
        provider.funds_last_update_nonce = 0;
        provider.funds_last_update_epoch = 0;
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let claimed_amount = self.call_value().egld_value().clone_value();
                let commission = &claimed_amount * self.service_fee().get() / MAX_PERCENT;
                let left_amount = &claimed_amount - &commission;
                self.total_egld_staked()
                    .update(|value| *value += &left_amount);
                self.egld_to_delegate()
                    .update(|value| *value += left_amount);
                self.send().direct_egld(&self.blockchain().get_owner_address(), &commission);
                provider.salsa_rewards = BigUint::zero();
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
        self.providers().insert(provider_address, provider);
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        if !self.refresh_providers() {
            return
        }

        let current_nonce = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            let is_active = provider.is_active();
            let is_up_to_date = provider.is_up_to_date(current_nonce, current_epoch);
            if !is_active || !is_up_to_date || (provider.salsa_withdrawable == 0) {
                continue
            }

            if !self.enough_gas_left_for_callback() {
                break
            }

            self.service_delegation_proxy_obj()
                .contract(address.clone())
                .withdraw()
                .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
                .async_call_promise()
                .with_callback(ServiceModule::callbacks(self).withdraw_all_callback(address))
                .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();
        }
    }

    #[promises_callback]
    fn withdraw_all_callback(
        &self,
        provider_address: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut provider = self.get_provider(&provider_address);
        provider.funds_last_update_nonce = 0;
        provider.funds_last_update_epoch = 0;
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdrawn_amount = self.call_value().egld_value();
                self.total_withdrawn_egld()
                    .update(|value| *value += withdrawn_amount.clone_value());
                provider.salsa_withdrawable = BigUint::zero();
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
        self.providers().insert(provider_address, provider);
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

    fn get_provider_to_delegate_and_amount(
        &self,
        amount: &BigUint,
    ) -> (
        ManagedAddress,
        BigUint,
        ManagedAddress,
        BigUint,
    ) {
        let mut provider_to_delegate = self.empty_provider();
        let mut provider_to_undelegate = self.empty_provider();
        let mut uneligible_provider = self.empty_provider();
        if !self.refresh_providers() {
            return (ManagedAddress::zero(), BigUint::zero(), ManagedAddress::zero(), BigUint::zero())
        }

        let mut min_topup = BigUint::zero();
        let mut max_topup_delegate = BigUint::zero();
        let mut max_topup_undelegate = BigUint::zero();
        let base_stake = BigUint::from(NODE_BASE_STAKE) * ONE_EGLD;
        for (_, provider) in self.providers().iter() {
            if !provider.is_active() {
                continue
            }

            if !provider.is_eligible() {
                if provider.salsa_stake > 0 {
                    uneligible_provider = provider.clone();
                }
                continue
            }

            let mut topup = &provider.total_stake / (provider.staked_nodes as u64);
            if topup > base_stake {
                topup -= &base_stake;
            } else {
                topup = BigUint::zero();
            }
            if provider.has_free_space() {
                if topup < min_topup || min_topup == 0 {
                    min_topup = topup.clone();
                    provider_to_delegate = provider.clone();
                }
                if topup > max_topup_delegate || max_topup_delegate == 0 {
                    max_topup_delegate = topup.clone();
                }
            }
            if (topup > max_topup_undelegate || max_topup_undelegate == 0) && provider.salsa_stake > 0 {
                max_topup_undelegate = topup;
                provider_to_undelegate = provider.clone();
            }
        }
        let mut delegate_amount = BigUint::zero();
        if provider_to_delegate.is_active() {
            if max_topup_delegate < min_topup {
                max_topup_delegate = min_topup.clone();
            }
            let dif_topup_delegate = &max_topup_delegate - &min_topup;
            delegate_amount = amount.clone();
            if dif_topup_delegate > 0 {
                let mut max_amount = &dif_topup_delegate * (provider_to_delegate.staked_nodes as u64);
                if max_amount < MIN_EGLD {
                    max_amount = BigUint::from(MIN_EGLD);
                }
                if amount > &max_amount {
                    delegate_amount = max_amount;
                }
            }
            if provider_to_delegate.has_cap {
                let max_amount = provider_to_delegate.max_cap - provider_to_delegate.total_stake;
                if delegate_amount > max_amount {
                    delegate_amount = max_amount;
                }
            }
            if delegate_amount < MIN_EGLD {
                delegate_amount = BigUint::zero();
            }
        }
        let mut undelegate_amount = BigUint::zero();
        if uneligible_provider.is_active() {
            provider_to_undelegate = uneligible_provider.clone();
            undelegate_amount = if amount > &uneligible_provider.salsa_stake {
                uneligible_provider.salsa_stake
            } else {
                amount.clone()
            };
        } else
        if provider_to_undelegate.is_active() {
            if max_topup_undelegate < min_topup {
                max_topup_undelegate = min_topup.clone();
            }
            let dif_topup_undelegate = &max_topup_undelegate - &min_topup;
            undelegate_amount = amount.clone();
            if dif_topup_undelegate > 0 {
                let mut max_amount = dif_topup_undelegate * (provider_to_undelegate.staked_nodes as u64);
                if max_amount < MIN_EGLD {
                    max_amount = BigUint::from(MIN_EGLD);
                }
                if amount > &max_amount {
                    undelegate_amount = max_amount;
                }
            }
            if provider_to_undelegate.salsa_stake < undelegate_amount {
                undelegate_amount = provider_to_undelegate.salsa_stake.clone();
            }
            let diff = &provider_to_undelegate.salsa_stake - &undelegate_amount;
            if diff < MIN_EGLD && diff > 0 {
                undelegate_amount = provider_to_undelegate.salsa_stake - MIN_EGLD;
            }
            if undelegate_amount < MIN_EGLD {
                undelegate_amount = BigUint::zero();
            }
        }

        (provider_to_delegate.address, delegate_amount, provider_to_undelegate.address, undelegate_amount)
    }

    // proxy

    #[proxy]
    fn service_delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
