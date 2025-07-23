multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug)]
pub struct Stake<M: ManagedTypeApi> {
    pub stake_id: usize,
    pub owner: ManagedAddress<M>,
    pub stake_tokens: ManagedVec<M, TokenIdentifier<M>>,
    pub stake_ratios: ManagedVec<M, BigUint<M>>,
    pub reward_tokens: ManagedVec<M, TokenIdentifier<M>>,
    pub staked: ManagedVec<M, BigUint<M>>,
    pub rewards: ManagedVec<M, BigUint<M>>,
    pub state: State,
    pub start_nonce: u64,
    pub end_nonce: u64,
    pub rps: ManagedVec<M, BigUint<M>>,
    pub claimable_rewards: ManagedVec<M, BigUint<M>>,
    pub remaining_rewards: ManagedVec<M, BigUint<M>>,
    pub last_rps_update_nonce: u64,
    pub remaining_nonces: u64,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug)]
pub struct UserStake<M: ManagedTypeApi> {
    pub staked: ManagedVec<M, BigUint<M>>,
    pub rewards: ManagedVec<M, BigUint<M>>,
    pub rps: ManagedVec<M, BigUint<M>>,
}
