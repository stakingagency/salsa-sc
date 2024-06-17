multiversx_sc::imports!();

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