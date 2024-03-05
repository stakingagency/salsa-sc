multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Debug)]
pub enum State {
    Inactive,
    Active,
    ActiveButNoSwap,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Debug)]
pub struct Pair<M: ManagedTypeApi> {
    pub pair_id: usize,
    pub state: State,
    pub enabled: bool,
    pub owner: ManagedAddress<M>,
    
    pub first_token_id: TokenIdentifier<M>,
    pub second_token_id: TokenIdentifier<M>,
    pub lp_token_id: TokenIdentifier<M>,

    pub lp_token_decimal: usize,

    pub first_token_reserve: BigUint<M>,
    pub second_token_reserve: BigUint<M>,
    pub lp_token_supply: BigUint<M>,

    pub lp_token_roles_are_set: bool
}
