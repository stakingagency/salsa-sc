multiversx_sc::imports!();

use crate::common::{config::*, consts::*, errors::*};
use crate::proxies::delegation_proxy::{self};

#[multiversx_sc::module]
pub trait ProvidersModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(addProvider)]
    fn add_provider(self, address: ManagedAddress) {
        require!(!self.is_state_active(), ERROR_ACTIVE);

        require!(
            !self.providers().contains_key(&address),
            ERROR_PROVIDER_ALREADY_ADDED
        );

        let mut provider = self.empty_provider();
        provider.address = address.clone();
        self.providers().insert(address.clone(), provider);

        self.refresh_provider_config(&address);
        self.refresh_provider_stake(&address);
        self.refresh_provider_nodes(&address);
        self.refresh_provider_funds_data(&address);
    }

    fn get_provider(&self, address: &ManagedAddress) -> ProviderConfig<Self::Api> {
        require!(
            self.providers().contains_key(address),
            ERROR_PROVIDER_NOT_FOUND
        );

        self.providers().get(address).unwrap()
    }

    #[only_owner]
    #[endpoint(removeProvider)]
    fn remove_provider(&self, address: &ManagedAddress) {
        let provider = self.get_provider(address);

        let current_nonce = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        if provider.is_active() {
            require!(
                provider.are_funds_up_to_date(current_nonce, current_epoch),
                ERROR_PROVIDER_NOT_UP_TO_DATE
            );
        }

        require!(
            provider.salsa_stake == 0 &&
            provider.salsa_rewards == 0 &&
            provider.salsa_undelegated == 0 &&
            provider.salsa_withdrawable == 0,
            ERROR_PROVIDER_WITH_FUNDS
        );

        self.providers().remove(address);
        if self.providers().is_empty() {
            self.state().set(State::Inactive);
        }
    }

    #[only_owner]
    #[endpoint(setProviderState)]
    fn set_provider_state(&self, address: ManagedAddress, new_state: State) {
        let mut provider = self.get_provider(&address);
        if provider.state != new_state {
            provider.config_last_update_nonce = 0;
            provider.stake_last_update_nonce = 0;
            provider.nodes_last_update_nonce = 0;
            provider.funds_last_update_nonce = 0;
            provider.funds_last_update_epoch = 0;
        }
        provider.state = new_state;
        self.providers().insert(address, provider);
    }

    /**
     * Are Providers Updated - updates all providers infos and returns true if all are up to date and false otherwise
     */
    fn are_providers_updated(&self) -> bool {
        let current_nonce = self.blockchain().get_block_nonce();
        let current_epoch = self.blockchain().get_block_epoch();
        for (address, provider) in self.providers().iter() {
            if !provider.is_active() || provider.is_up_to_date(current_nonce, current_epoch) {
                continue
            }

            if !provider.is_config_up_to_date(current_nonce) {
                self.refresh_provider_config(&address);
            }
            if !provider.is_stake_up_to_date(current_nonce) {
                self.refresh_provider_stake(&address);
            }
            if !provider.are_nodes_up_to_date(current_nonce) {
                self.refresh_provider_nodes(&address);
            }
            if !provider.are_funds_up_to_date(current_nonce, current_epoch) {
                self.refresh_provider_funds_data(&address);
            }

            return false
        }

        true
    }

    // refresh provider data functions

    fn refresh_provider_config(&self, address: &ManagedAddress) {
        self.providers_delegation_proxy_obj()
            .contract(address.clone())
            .get_contract_config()
            .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
            .async_call_promise()
            .with_callback(ProvidersModule::callbacks(self).get_contract_config_callback(address))
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    fn refresh_provider_stake(&self, address: &ManagedAddress) {
        self.providers_delegation_proxy_obj()
            .contract(address.clone())
            .get_total_active_stake()
            .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
            .async_call_promise()
            .with_callback(ProvidersModule::callbacks(self).get_total_active_stake_callback(address))
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    fn refresh_provider_nodes(&self, address: &ManagedAddress) {
        self.providers_delegation_proxy_obj()
            .contract(address.clone())
            .get_all_nodes_states()
            .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
            .async_call_promise()
            .with_callback(ProvidersModule::callbacks(self).get_all_nodes_states_callback(address))
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    fn refresh_provider_funds_data(&self, address: &ManagedAddress) {
        self.providers_delegation_proxy_obj()
            .contract(address.clone())
            .get_delegator_funds_data(self.blockchain().get_sc_address())
            .with_gas_limit(MIN_GAS_FOR_ASYNC_CALL)
            .async_call_promise()
            .with_callback(ProvidersModule::callbacks(self).get_delegator_funds_data_callback(address))
            .with_extra_gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    // callbacks

    #[promises_callback]
    fn get_contract_config_callback(
        &self,
        address: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) {
        let mut provider = self.get_provider(address);
        match result {
            ManagedAsyncCallResult::Ok(config) => {
                require!(config.len() == 10, ERROR_INVALID_SC_RESPONSE);

                let config_items = config.into_vec_of_buffers();
                provider.max_cap = BigUint::from(config_items.get(PROVIDER_CONFIG_MAX_CAP_INDEX).clone_value());
                provider.fee = config_items.get(PROVIDER_CONFIG_FEE_INDEX).parse_as_u64().unwrap_or(0);
                provider.has_cap = config_items.get(PROVIDER_CONFIG_HAS_CAP_INDEX).clone_value() == b"true";
                provider.config_last_update_nonce = self.blockchain().get_block_nonce();
            }
            ManagedAsyncCallResult::Err(_) => {
                provider.config_last_update_nonce = 0;
            }
        }
        self.providers().insert(address.clone(), provider);
    }

    #[promises_callback]
    fn get_total_active_stake_callback(
        &self,
        address: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        let mut provider = self.get_provider(address);
        match result {
            ManagedAsyncCallResult::Ok(stake) => {
                provider.total_stake = stake;
                provider.stake_last_update_nonce = self.blockchain().get_block_nonce();
            }
            ManagedAsyncCallResult::Err(_) => {
                provider.stake_last_update_nonce = 0;
            }
        }
        self.providers().insert(address.clone(), provider);
    }

    #[promises_callback]
    fn get_all_nodes_states_callback(
        &self,
        address: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) {
        let mut provider = self.get_provider(address);
        match result {
            ManagedAsyncCallResult::Ok(nodes_states) => {
                let states = nodes_states.to_arg_buffer();
                let mut count = false;
                let mut staked_nodes = 0;
                for state in states.raw_arg_iter() {
                    let value = state.clone_value();
                    if value == b"staked" {
                        count = true;
                        continue
                    }

                    if count {
                        if value.len() == 96 {
                            staked_nodes += 1;
                        } else {
                            break
                        }
                    }
                }
                provider.staked_nodes = staked_nodes;
                provider.nodes_last_update_nonce = self.blockchain().get_block_nonce();
            }
            ManagedAsyncCallResult::Err(_) => {
                provider.nodes_last_update_nonce = 0;
            }
        }
        self.providers().insert(address.clone(), provider);
    }

    #[promises_callback]
    fn get_delegator_funds_data_callback(
        &self,
        address: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) {
        let mut provider = self.get_provider(address);
        match result {
            ManagedAsyncCallResult::Ok(delegator_funds_data) => {
                require!(delegator_funds_data.len() == 4, ERROR_INVALID_SC_RESPONSE);

                let funds_data = delegator_funds_data.into_vec_of_buffers();
                provider.salsa_stake = BigUint::from(funds_data.get(PROVIDER_FUNDS_DELEGATED_INDEX).clone_value());
                provider.salsa_rewards = BigUint::from(funds_data.get(PROVIDER_FUNDS_REWARDS_INDEX).clone_value());
                provider.salsa_undelegated = BigUint::from(funds_data.get(PROVIDER_FUNDS_UNDELEGATED_INDEX).clone_value());
                provider.salsa_withdrawable = BigUint::from(funds_data.get(PROVIDER_FUNDS_WITHDRAWABLE_INDEX).clone_value());
                provider.funds_last_update_nonce = self.blockchain().get_block_nonce();
                provider.funds_last_update_epoch = self.blockchain().get_block_epoch();
            }
            ManagedAsyncCallResult::Err(_) => {
                provider.funds_last_update_nonce = 0;
                provider.funds_last_update_epoch = 0;
            }
        }
        self.providers().insert(address.clone(), provider);
    }

    // helpers

    fn empty_provider(&self) -> ProviderConfig<Self::Api> {
        ProviderConfig{
            state: State::Active,
            address: ManagedAddress::from(&[0u8; 32]),
            staked_nodes: 0,
            total_stake: BigUint::zero(),
            max_cap: BigUint::zero(),
            has_cap: false,
            fee: 0,
            salsa_stake: BigUint::zero(),
            salsa_undelegated: BigUint::zero(),
            salsa_withdrawable: BigUint::zero(),
            salsa_rewards: BigUint::zero(),
            config_last_update_nonce: 0,
            stake_last_update_nonce: 0,
            nodes_last_update_nonce: 0,
            funds_last_update_nonce: 0,
            funds_last_update_epoch: 0,
        }
    }

    // proxy

    #[proxy]
    fn providers_delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
