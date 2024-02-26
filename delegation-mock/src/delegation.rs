#![no_std]

multiversx_sc::imports!();

pub type Epoch = u64;
pub const MAX_PERCENTAGE: u64 = 100_000;
pub const APY: u64 = 10_000; //10%
pub const EPOCHS_IN_YEAR: u64 = 365;
pub const UNBOND_PERIOD: u64 = 10;

#[multiversx_sc::contract]
pub trait DelegationMock<ContractReader> {
    #[init]
    fn init(&self,
        total_stake: BigUint,
        nodes_count: u64,
        service_fee: u64
    ) {
        self.egld_token_supply().set(total_stake);
        self.nodes_count().set(nodes_count);
        self.service_fee().set(service_fee);
    }

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) {
        let caller = self.blockchain().get_caller();
        let payment_amount = self.call_value().egld_value();
        self.address_deposit(&caller)
            .update(|value| *value += payment_amount.clone_value());
        self.egld_token_supply()
            .update(|value| *value += payment_amount.clone_value());
    }

    #[endpoint(unDelegate)]
    fn undelegate(&self, egld_to_undelegate: BigUint) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let total_deposit = self.address_deposit(&caller).get();
        require!(
            egld_to_undelegate > 0 && egld_to_undelegate <= total_deposit,
            "Invalid undelegate amount"
        );
        self.address_deposit(&caller)
            .update(|value| *value -= &egld_to_undelegate);
        self.address_undelegate_amount(&caller)
            .update(|value| *value += &egld_to_undelegate);
        self.address_undelegate_epoch(&caller)
            .set(current_epoch + UNBOND_PERIOD);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let withdraw_epoch = self.address_undelegate_epoch(&caller).get();
        let withdraw_amount = self.address_undelegate_amount(&caller).get();

        require!(withdraw_amount > 0, "No amount to withdraw");
        require!(
            withdraw_epoch > 0 && current_epoch >= withdraw_epoch,
            "Cannot withdraw yet"
        );

        self.egld_token_supply()
            .update(|value| *value -= &withdraw_amount);
        self.address_undelegate_epoch(&caller).clear();
        self.address_undelegate_amount(&caller).clear();

        self.send_raw().async_call_raw(
            &caller,
            &withdraw_amount,
            &ManagedBuffer::new(),
            &ManagedArgBuffer::new(),
        );
    }

    #[endpoint(reDelegateRewards)]
    fn redelegate_rewards(&self) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let last_claim_epoch = self.address_last_claim_epoch(&caller).get();
        let total_deposit = self.address_deposit(&caller).get();

        if current_epoch > last_claim_epoch {
            let rewards = (total_deposit * APY / MAX_PERCENTAGE)
                * (current_epoch - last_claim_epoch) / EPOCHS_IN_YEAR;
            if rewards > 0u64 {
                self.address_deposit(&caller)
                    .update(|value| *value += &rewards);
                // self.egld_token_supply()
                //     .update(|value| *value += payment_amount.clone_value());
                self.address_last_claim_epoch(&caller).set(current_epoch);
            }
        }
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let last_claim_epoch = self.address_last_claim_epoch(&caller).get();
        let total_deposit = self.address_deposit(&caller).get();

        if current_epoch > last_claim_epoch {
            let rewards = (total_deposit * APY / MAX_PERCENTAGE)
                * (current_epoch - last_claim_epoch) / EPOCHS_IN_YEAR;
            if rewards > 0u64 {
                self.address_last_claim_epoch(&caller).set(current_epoch);
                self.send_raw().async_call_raw(
                    &caller,
                    &rewards,
                    &ManagedBuffer::new(),
                    &ManagedArgBuffer::new(),
                );
            }
        }
    }

    #[endpoint(getClaimableRewards)]
    fn get_claimable_rewards(&self) -> BigUint {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let last_claim_epoch = self.address_last_claim_epoch(&caller).get();
        let total_deposit = self.address_deposit(&caller).get();

        if current_epoch > last_claim_epoch {
            (total_deposit * APY / MAX_PERCENTAGE)
                * (current_epoch - last_claim_epoch) / EPOCHS_IN_YEAR
        } else {
            BigUint::zero()
        }
    }

    #[endpoint(getUserActiveStake)]
    fn get_user_active_stake(&self) -> BigUint {
        let caller = self.blockchain().get_caller();

        self.address_deposit(&caller).get()
    }

    #[endpoint(getDelegatorFundsData)]
    fn get_delegator_funds_data(&self, address: ManagedAddress) -> MultiValueEncoded<ManagedBuffer> {
        let current_epoch = self.blockchain().get_block_epoch();
        let last_claim_epoch = self.address_last_claim_epoch(&address).get();
        let delegated = self.address_deposit(&address).get();

        let rewards = if current_epoch > last_claim_epoch {
            (&delegated * APY / MAX_PERCENTAGE)
                * (current_epoch - last_claim_epoch) / EPOCHS_IN_YEAR
        } else {
            BigUint::zero()
        };
        let undelegated = self.address_undelegate_amount(&address).get();
        let withdraw_epoch = self.address_undelegate_epoch(&address).get();
        let mut unbondable = BigUint::zero();
        if withdraw_epoch > 0 && current_epoch >= withdraw_epoch {
            unbondable = undelegated.clone();
        }

        let mut result: MultiValueEncoded<ManagedBuffer> = MultiValueEncoded::new();
        result.push(delegated.to_bytes_be_buffer());
        result.push(rewards.to_bytes_be_buffer());
        result.push(undelegated.to_bytes_be_buffer());
        result.push(unbondable.to_bytes_be_buffer());

        result
    }

    #[endpoint(getContractConfig)]
    fn get_contract_config(&self) -> MultiValueEncoded<ManagedBuffer> {
        let mut result: MultiValueEncoded<ManagedBuffer> = MultiValueEncoded::new();
        result.push(BigUint::zero().to_bytes_be_buffer()); // owner
        result.push(BigUint::from(self.service_fee().get()).to_bytes_be_buffer()); // service fee
        result.push(BigUint::zero().to_bytes_be_buffer()); // max cap
        result.push(BigUint::zero().to_bytes_be_buffer()); // initial owner funds
        result.push(BigUint::zero().to_bytes_be_buffer()); // automatic activation
        result.push(ManagedBuffer::from("false")); // has cap
        result.push(BigUint::zero().to_bytes_be_buffer()); // changeable fee
        result.push(BigUint::zero().to_bytes_be_buffer()); // check cap on redelegate
        result.push(BigUint::zero().to_bytes_be_buffer()); // created nonce
        result.push(BigUint::zero().to_bytes_be_buffer()); // unbond period

        result
    }

    #[endpoint(getAllNodeStates)]
    fn get_all_nodes_states(&self) -> MultiValueEncoded<ManagedBuffer> {
        let mut result: MultiValueEncoded<ManagedBuffer> = MultiValueEncoded::new();
        result.push(ManagedBuffer::from("staked"));
        for _ in 0..self.nodes_count().get() {
            result.push(ManagedBuffer::from("012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345"));
        }

        result
    }

    #[endpoint(getTotalActiveStake)]
    fn get_total_active_stake(&self) -> BigUint {
        self.egld_token_supply().get()
    }

    #[storage_mapper("egldTokenSupply")]
    fn egld_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("addressDeposit")]
    fn address_deposit(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("addressLastClaim")]
    fn address_last_claim_epoch(&self, address: &ManagedAddress) -> SingleValueMapper<Epoch>;

    #[storage_mapper("addressUndelegateAmount")]
    fn address_undelegate_amount(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("addressUndelegateEpoch")]
    fn address_undelegate_epoch(&self, address: &ManagedAddress) -> SingleValueMapper<Epoch>;

    #[storage_mapper("nodes_count")]
    fn nodes_count(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("service_fee")]
    fn service_fee(&self) -> SingleValueMapper<u64>;
}
