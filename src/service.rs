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
    fn delegate_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let egld_to_delegate = self.egld_to_delegate().get();
        require!(
            egld_to_delegate >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let last_delegation_block = self.last_delegation_block().get();
        let current_block = self.blockchain().get_block_nonce();
        require!(
            last_delegation_block + MIN_BLOCK_BETWEEN_DELEGATIONS <= current_block,
            ERROR_DELEGATE_TOO_SOON
        );

        self.last_delegation_block().set(current_block);

        let (provider_address, amount, _, _) = self.get_provider_to_delegate_and_amount(&egld_to_delegate);
        if amount == 0 {
            return
        }

        self.egld_to_delegate().set(&egld_to_delegate - &amount);
        let gas_for_async_call = self.get_gas_for_async_call();
        self.service_delegation_proxy_obj()
            .contract(provider_address.clone())
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(amount.clone())
            .async_call()
            .with_callback(
                ServiceModule::callbacks(self).delegate_all_callback(provider_address, amount),
            )
            .call_and_exit()
    }

    #[callback]
    fn delegate_all_callback(
        &self,
        provider_address: ManagedAddress,
        egld_to_delegate: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let mut provider = self.get_provider(&provider_address);
                provider.stake_last_update_nonce = 0;
                self.providers().insert(provider_address, provider);
            }
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_delegate()
                    .update(|value| *value += egld_to_delegate);
            }
        }
    }

    #[endpoint(unDelegateAll)]
    fn undelegate_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let egld_to_undelegate = self.egld_to_undelegate().get();
        require!(
            egld_to_undelegate >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let mut storage_cache = StorageCache::new(self);
        self.reduce_egld_to_delegate_undelegate(&mut storage_cache);
        drop(storage_cache);

        let (_, _, provider_address, amount) = self.get_provider_to_delegate_and_amount(&egld_to_undelegate);
        if amount == 0 {
            return
        }

        self.egld_to_undelegate().set(&egld_to_undelegate - &amount);
        let gas_for_async_call = self.get_gas_for_async_call();
        self.service_delegation_proxy_obj()
            .contract(provider_address.clone())
            .undelegate(amount.clone())
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                ServiceModule::callbacks(self).undelegate_all_callback(provider_address, amount),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_all_callback(
        &self,
        provider_address: ManagedAddress,
        egld_to_undelegate: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let mut provider = self.get_provider(&provider_address);
                provider.stake_last_update_nonce = 0;
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

        let current_nonce = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            if !provider.is_active() || !provider.is_up_to_date(current_nonce, current_epoch) || (provider.salsa_rewards == 0) {
                continue
            }

            let gas_left = self.blockchain().get_gas_left();
            if gas_left < MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK {
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
                let mut provider = self.get_provider(&provider_address);
                provider.salsa_rewards = BigUint::zero();
                self.providers().insert(provider_address, provider);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let current_nonce = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            if !provider.is_active() || !provider.is_up_to_date(current_nonce, current_epoch) || (provider.salsa_withdrawable == 0) {
                continue
            }

            let gas_left = self.blockchain().get_gas_left();
            if gas_left < MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK {
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
        ManagedAddress,
        BigUint,
    ) {
        let mut provider_to_delegate = ManagedAddress::from(&[0u8; 32]);
        let mut provider_to_delegate_nodes: u64 = 0;
        let mut provider_to_undelegate = ManagedAddress::from(&[0u8; 32]);
        let mut provider_to_undelegate_nodes: u64 = 0;
        if !self.are_providers_updated() {
            return (provider_to_delegate, BigUint::zero(), provider_to_undelegate, BigUint::zero())
        }

        let mut min_topup = BigUint::zero();
        let mut max_topup = BigUint::zero();
        let base_stake = BigUint::from(NODE_BASE_STAKE) * ONE_EGLD;
        for (address, provider) in self.providers().iter() {
            if !provider.is_active() {
                continue
            }

            let mut topup = provider.total_stake / (provider.staked_nodes as u64);
            if topup > base_stake {
                topup -= &base_stake;
            } else {
                topup = BigUint::zero();
            }
            if topup < min_topup || min_topup == 0 {
                min_topup = topup.clone();
                provider_to_delegate = address.clone();
                provider_to_delegate_nodes = provider.staked_nodes as u64;
            }
            if topup > max_topup || max_topup == 0 {
                max_topup = topup;
                provider_to_undelegate = address;
                provider_to_undelegate_nodes = provider.staked_nodes as u64;
            }
        }
        let dif_topup = &max_topup - &min_topup;
        let mut delegate_amount = BigUint::zero();
        if provider_to_delegate_nodes > 0 {
            delegate_amount = amount.clone();
            if min_topup != max_topup {
                let mut max_amount = &dif_topup * provider_to_delegate_nodes;
                if max_amount < MIN_EGLD {
                    max_amount = BigUint::from(MIN_EGLD);
                }
                if amount > &max_amount {
                    delegate_amount = max_amount;
                }
            }
        }
        let mut undelegate_amount = BigUint::zero();
        if provider_to_undelegate_nodes > 0 {
            undelegate_amount = amount.clone();
            if min_topup != max_topup {
                let mut max_amount = dif_topup * provider_to_undelegate_nodes;
                if max_amount < MIN_EGLD {
                    max_amount = BigUint::from(MIN_EGLD);
                }
                if amount > &max_amount {
                    undelegate_amount = max_amount;
                }
            }
        }

        (provider_to_delegate, delegate_amount, provider_to_undelegate, undelegate_amount)
    }

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
