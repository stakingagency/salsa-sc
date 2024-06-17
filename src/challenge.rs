use crate::common::{config::*, consts::*, errors::*, storage_cache::StorageCache};
use crate::proxies::delegation_proxy;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ChallengeModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::providers::ProvidersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(createChallenge)]
    fn create_challenge(&self) {
        require!(self.challenge().is_empty(), ERROR_CHALLENGE_EXISTS);

        self.challenge().set(
            Challenge{
                end_epoch: self.blockchain().get_block_epoch() + CHALLENGE_DURATION,
                target_undelegated: self.total_undelegation_requested().get(),
                status: ChallengeStatus::Pending,
            }
        );
    }

    #[endpoint(updateChallenge)]
    fn update_challenge(&self) {
        require!(!self.challenge().is_empty(), ERROR_NO_CHALLENGE);

        let mut challenge = self.challenge().get();
        let current_epoch = self.blockchain().get_block_epoch();
        if current_epoch >= challenge.end_epoch + MIN_DURATION_USERS_CAN_UNDELEGATE {
            self.challenge().clear();
            return;
        }

        if challenge.status == ChallengeStatus::Pending && current_epoch >= challenge.end_epoch {
            if self.total_undelegated().get() >= challenge.target_undelegated {
                self.challenge().clear();
            } else {
                challenge.status = ChallengeStatus::Failed;
                self.challenge().set(challenge);
            }
        }
    }

    #[endpoint(canEmergentlyUndelegate)]
    fn can_emergently_undelegate(&self) -> bool {
        let challenge = self.challenge();
        if challenge.is_empty() {
            return false;
        }

        challenge.get().status == ChallengeStatus::Failed
    }

    #[endpoint(emergentlyUndelegate)]
    fn emergently_undelegate(&self, provider_address: ManagedAddress, gas: Option<u64>) {
        let mut gas_for_async_undelegate = gas.unwrap_or(0);
        if gas_for_async_undelegate < MIN_GAS_FOR_ASYNC_CALL {
            gas_for_async_undelegate = MIN_GAS_FOR_ASYNC_CALL;
        }

        require!(self.can_emergently_undelegate(), ERROR_CAN_NOT_UNDELEGATE);

        let mut storage_cache = StorageCache::new(self);
        let provider = self.get_provider(&provider_address);
        let amount = if storage_cache.egld_to_undelegate.clone() < provider.salsa_stake {
            storage_cache.egld_to_undelegate.clone()
        } else {
            provider.salsa_stake
        };
        storage_cache.egld_to_undelegate = BigUint::zero();
        drop(storage_cache);
        self.challenge_delegation_proxy_obj()
            .contract(provider_address)
            .undelegate(&amount)
            .with_gas_limit(gas_for_async_undelegate)
            .async_call_promise()
            .with_callback(
                ChallengeModule::callbacks(self).emergently_undelegate_all_callback(&amount),
            )
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    #[promises_callback]
    fn emergently_undelegate_all_callback(
        &self,
        egld_to_undelegate: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.total_undelegated()
                    .update(|value| *value += egld_to_undelegate);
            }
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_undelegate()
                    .update(|value| *value += egld_to_undelegate);
            }
        }
    }

    // proxy

    #[proxy]
    fn challenge_delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
