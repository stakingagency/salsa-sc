multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq)]
pub enum State {
    Inactive,
    Active,
    ActiveButNoSwap,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
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

#[multiversx_sc::proxy]
pub trait OneDexProxy {
    #[payable("*")]
    #[endpoint(swapMultiTokensFixedInput)]
    fn swap_multi_tokens_fixed_input(
        &self,
        amount_out_min: BigUint,
        unwrap_required: bool,
        path_args: MultiValueEncoded<TokenIdentifier>,
    );

    #[endpoint(getAmountOut)]
    fn get_amount_out_view(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
        amount_in: BigUint
    ) -> BigUint;

    #[view(viewPair)]
    fn view_pair(&self, pair_id: usize) -> Pair<Self::Api>;

    #[payable("*")]
    #[endpoint(addLiquidity)]
    fn add_liquidity(&self, first_token_amount_min: BigUint, second_token_amount_min: BigUint);

    #[payable("*")]
    #[endpoint(removeLiquidity)]
    fn remove_liquidity(
        &self,
        first_token_amount_min: BigUint,
        second_token_amount_min: BigUint,
        unwrap_required: bool
    );
}
