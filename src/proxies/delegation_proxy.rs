multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
pub struct DelegationContractConfig<M: ManagedTypeApi> {
    pub owner: ManagedAddress<M>,
    pub service_fee: u64,
    pub max_cap: BigUint<M>,
    pub initial_funds: BigUint<M>,
    pub automatic_activation: ManagedBuffer<M>,
    pub with_cap: ManagedBuffer<M>,
    pub changeable_fee: ManagedBuffer<M>,
    pub check_cap_on_redelegate: ManagedBuffer<M>,
    pub created_nonce: u64,
    pub unbond_period: u64,
}

#[multiversx_sc::proxy]
pub trait DelegationProxy {
    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self);

    #[endpoint(unDelegate)]
    fn undelegate(&self, egld_amount: &BigUint);

    #[endpoint(withdraw)]
    fn withdraw(&self);

    #[endpoint(claimRewards)]
    fn claim_rewards(&self);

    #[endpoint(getDelegatorFundsData)]
    fn get_delegator_funds_data(&self, address: ManagedAddress);

    #[endpoint(getContractConfig)]
    fn get_contract_config(&self);

    #[endpoint(getTotalActiveStake)]
    fn get_total_active_stake(&self);

    #[endpoint(getAllNodeStates)]
    fn get_all_nodes_states(&self);
}
