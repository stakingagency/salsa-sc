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

        self.reduce_egld_to_delegate_undelegate(&mut storage_cache);
        if storage_cache.egld_to_delegate == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        let (provider_address, amount) =
            self.get_provider_to_delegate_and_amount(&storage_cache.egld_to_delegate);
        if amount == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        storage_cache.last_delegation_block = current_block;
        storage_cache.egld_to_delegate -= &amount;
        drop(storage_cache);

        let mut provider = self.get_provider(&provider_address);
        provider.funds_last_update_timestamp = 0;
        provider.funds_last_update_epoch = 0;
        provider.stake_last_update_timestamp = 0;
        self.providers().insert(provider_address.clone(), provider);

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
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let mut provider = self.get_provider(&provider_address);
                provider.salsa_stake += egld_to_delegate;
                self.providers().insert(provider_address, provider);
            }
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_delegate()
                    .update(|value| *value += egld_to_delegate);
            }
        }
    }

    #[endpoint(unDelegateAll)]
    fn undelegate_all(&self) -> BigUint {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let challenge = self.challenge();
        if !challenge.is_empty() {
            require!(challenge.get().status != ChallengeStatus::Failed, ERROR_CHALLENGE_EXISTS);
        }

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

        let (provider_address, amount) =
            self.get_provider_to_undelegate_and_amount(&storage_cache.egld_to_undelegate);
        if amount == 0 {
            drop(storage_cache);
            return BigUint::zero()
        }

        storage_cache.egld_to_undelegate -= &amount;
        drop(storage_cache);

        let mut provider = self.get_provider(&provider_address);
        provider.funds_last_update_timestamp = 0;
        provider.funds_last_update_epoch = 0;
        provider.stake_last_update_timestamp = 0;
        self.providers().insert(provider_address.clone(), provider);

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
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.total_undelegated()
                    .update(|value| *value += egld_to_undelegate);
                let mut provider = self.get_provider(&provider_address);
                provider.salsa_stake -= egld_to_undelegate;
                self.providers().insert(provider_address, provider);
            }
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_undelegate()
                    .update(|value| *value += egld_to_undelegate);
            }
        }
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        if !self.refresh_providers() {
            return
        }

        let current_timestamp = self.blockchain().get_block_timestamp();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            let is_active = provider.is_active();
            let is_up_to_date = provider.is_up_to_date(current_timestamp, current_epoch);
            if !is_active || !is_up_to_date || (provider.salsa_rewards == 0) {
                continue
            }

            if !self.enough_gas_left_for_async_call() {
                break
            }

            let mut provider = self.get_provider(&address);
            provider.funds_last_update_timestamp = 0;
            provider.funds_last_update_epoch = 0;
            provider.salsa_rewards = BigUint::zero();
            self.providers().insert(address.clone(), provider);

            self.service_delegation_proxy_obj()
                .contract(address.clone())
                .claim_rewards()
                .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
                .async_call_promise()
                .with_callback(ServiceModule::callbacks(self).claim_rewards_callback())
                .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();
        }
    }

    #[promises_callback]
    fn claim_rewards_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
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
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self, gas: Option<u64>, providers_to_withdraw_from: MultiValueEncoded<ManagedAddress>) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut gas_for_async_withdraw = gas.unwrap_or(0);
        if gas_for_async_withdraw < MIN_GAS_FOR_ASYNC_CALL {
            gas_for_async_withdraw = MIN_GAS_FOR_ASYNC_CALL;
        }

        if !providers_to_withdraw_from.is_empty() {
            for address in providers_to_withdraw_from.into_iter() {
                if !self.providers().contains_key(&address) {
                    continue
                }

                self.service_delegation_proxy_obj()
                    .contract(address.clone())
                    .withdraw()
                    .with_gas_limit(gas_for_async_withdraw)
                    .async_call_promise()
                    .with_callback(ServiceModule::callbacks(self).withdraw_all_callback(address))
                    .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
                    .register_promise();
            }
            return
        }

        if !self.refresh_providers() {
            return
        }

        let current_timestamp = self.blockchain().get_block_timestamp();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            let is_active = provider.is_active();
            let is_up_to_date = provider.is_up_to_date(current_timestamp, current_epoch);
            if !is_active || !is_up_to_date || (provider.salsa_withdrawable == 0) {
                continue
            }

            if !self.enough_gas_left_for_async_call() {
                break
            }

            self.service_delegation_proxy_obj()
                .contract(address.clone())
                .withdraw()
                .with_gas_limit(gas_for_async_withdraw)
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
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdrawn_amount = self.call_value().egld_value();
                self.total_withdrawn_egld()
                    .update(|value| *value += withdrawn_amount.clone_value());
                let mut provider = self.get_provider(&provider_address);
                provider.salsa_withdrawable = BigUint::zero();
                self.providers().insert(provider_address, provider);
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

    fn get_provider_to_delegate_and_amount(
        &self,
        amount: &BigUint,
    ) -> (
        ManagedAddress,
        BigUint,
    ) {
        if !self.refresh_providers() {
            return (ManagedAddress::zero(), BigUint::zero())
        }

        let mut provider_to_delegate = self.empty_provider();
        let min_amount = self.get_min_delegate_amount(amount.clone());
        let mut min_topup_set = false;
        let mut min_topup = BigUint::zero();
        let mut max_topup = BigUint::zero();
        let base_stake = BigUint::from(NODE_BASE_STAKE) * ONE_EGLD;
        for (_, provider) in self.providers().iter() {
            let has_free_space = !provider.has_cap || (provider.max_cap >= &provider.total_stake + &min_amount);
            if !provider.is_active() || !provider.is_eligible(self.max_provider_fee().get()) || !has_free_space {
                continue
            }

            let mut topup = &provider.total_stake / (provider.staked_nodes as u64);
            if topup > base_stake {
                topup -= &base_stake;
            } else {
                topup = BigUint::zero();
            }
            if topup < min_topup || !min_topup_set {
                min_topup = topup.clone();
                provider_to_delegate = provider.clone();
            }
            if topup > max_topup || !min_topup_set {
                max_topup = topup.clone();
            }
            min_topup_set = true;
        }
        let mut delegate_amount = BigUint::zero();
        if min_topup_set {
            let dif_topup_delegate = &max_topup - &min_topup;
            delegate_amount = amount.clone();
            if dif_topup_delegate > 0 {
                let mut max_amount = &dif_topup_delegate * (provider_to_delegate.staked_nodes as u64);
                if max_amount < MIN_EGLD {
                    max_amount = BigUint::from(MIN_EGLD);
                }
                if max_amount < min_amount {
                    max_amount = min_amount;
                }
                if delegate_amount > max_amount {
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

        (provider_to_delegate.address, delegate_amount)
    }

    fn get_provider_to_undelegate_and_amount(
        &self,
        amount: &BigUint,
    ) -> (
        ManagedAddress,
        BigUint,
    ) {
        if !self.refresh_providers() {
            return (ManagedAddress::zero(), BigUint::zero())
        }

        let mut provider_to_undelegate = self.empty_provider();
        let mut undelegate_amount = BigUint::zero();
        let mut min_topup_set = false;
        let mut max_topup_set = false;
        let mut min_topup = BigUint::zero();
        let mut max_topup = BigUint::zero();
        let base_stake = BigUint::from(NODE_BASE_STAKE) * ONE_EGLD;
        for (_, provider) in self.providers().iter() {
            let mut topup = &provider.total_stake / (provider.staked_nodes as u64);
            if topup > base_stake {
                topup -= &base_stake;
            } else {
                topup = BigUint::zero();
            }
            if topup < min_topup || !min_topup_set {
                min_topup_set = true;
                min_topup = topup.clone();
            }
        }
        for (_, provider) in self.providers().iter() {
            if !provider.is_active() || !provider.is_eligible(self.max_provider_fee().get()) {
                if provider.salsa_stake > 0 {
                    let amount_to_undelegate = self.compute_amount_to_undelegate(
                        amount,
                        &provider.salsa_stake,
                        &BigUint::zero(),
                        &BigUint::zero(),
                        0,
                    );
                    if amount_to_undelegate > 0 {
                        return (provider.address, amount_to_undelegate);
                    }
                }
                continue
            }

            let mut topup = &provider.total_stake / (provider.staked_nodes as u64);
            if topup > base_stake {
                topup -= &base_stake;
            } else {
                topup = BigUint::zero();
            }
            if (topup > max_topup || !max_topup_set) && provider.salsa_stake > 0 {
                let amount_to_undelegate = self.compute_amount_to_undelegate(
                    amount,
                    &provider.salsa_stake,
                    &min_topup,
                    &topup,
                    provider.staked_nodes as u64,
                );
                if amount_to_undelegate > 0 {
                    max_topup_set = true;
                    max_topup = topup;
                    provider_to_undelegate = provider.clone();
                    undelegate_amount = amount_to_undelegate;
                }
            }
        }

        (provider_to_undelegate.address, undelegate_amount)
    }

    fn compute_amount_to_undelegate(
        &self,
        amount: &BigUint,
        salsa_stake: &BigUint,
        min_topup: &BigUint,
        max_topup: &BigUint,
        staked_nodes: u64,
    ) -> BigUint {
        let mut undelegate_amount = amount.clone();
        let diff_topup = max_topup - min_topup;
        if diff_topup > 0 {
            let mut max_amount = diff_topup * staked_nodes;
            if max_amount < MIN_EGLD {
                max_amount = BigUint::from(MIN_EGLD);
            }
            let min_amount = self.get_min_delegate_amount(amount.clone());
            if undelegate_amount > max_amount {
                undelegate_amount = max_amount;
            }
            if undelegate_amount < min_amount {
                undelegate_amount = min_amount;
            }
        }
        if salsa_stake < &undelegate_amount {
            undelegate_amount = salsa_stake.clone();
        }
        let diff = salsa_stake - &undelegate_amount;
        if diff < MIN_EGLD && diff > 0 {
            undelegate_amount = salsa_stake - MIN_EGLD;
        }
        if undelegate_amount < MIN_EGLD {
            undelegate_amount = BigUint::zero();
        }

        undelegate_amount
    }

    fn get_min_delegate_amount(&self, amount: BigUint) -> BigUint {
        let min_amount = BigUint::from(MIN_DELEGATE_AMOUNT);
        let min_amount_percent = &amount * MIN_DELEGATE_PERCENT / MAX_PERCENT;
        let mut max = if min_amount > min_amount_percent {
            min_amount
        } else {
            min_amount_percent
        };
        if max > amount {
            max = amount;
        }

        max
    }

    // proxy

    #[proxy]
    fn service_delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
