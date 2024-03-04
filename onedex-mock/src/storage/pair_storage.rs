multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::state::State;

#[multiversx_sc::module]
pub trait PairStorageModule {
    /**
     * Pair Owner
     *  pair_id -> owner address
     */
    #[view(getPairOwner)]
    #[storage_mapper("pair_owner")]
    fn pair_owner(&self, pair_id: usize) -> SingleValueMapper<ManagedAddress>;

    /**
     * State
     *  pair_id -> Inactive or Active or ActiveButNoSwap
     */
    #[view(getPairState)]
    #[storage_mapper("pair_state")]
    fn pair_state(&self, pair_id: usize) -> SingleValueMapper<State>;

    /**
     * Enable Swap
     */
    #[view(getPairEnabled)]
    #[storage_mapper("pair_enabled")]
    fn pair_enabled(&self, pair_id: usize) -> SingleValueMapper<bool>;


    /**
     * Pair first token_id
     *  pair_id -> first token_id
     */
    #[view(getPairFirstTokenId)]
    #[storage_mapper("pair_first_token_id")]
    fn pair_first_token_id(&self, pair_id: usize) -> SingleValueMapper<TokenIdentifier>;


    /**
     * Second token_id
     *  pair_id -> second token_id
     */
    #[view(getPairSecondTokenId)]
    #[storage_mapper("pair_second_token_id")]
    fn pair_second_token_id(&self, pair_id: usize) -> SingleValueMapper<TokenIdentifier>;


    /**
     * First token Reserver
     */
    #[view(getPairFirstTokenReserve)]
    #[storage_mapper("pair_first_token_reserve")]
    fn pair_first_token_reserve(&self, pair_id: usize) -> SingleValueMapper<BigUint>;

    /**
     * Second token Reserver
     */
    #[view(getPairSecondTokenReserve)]
    #[storage_mapper("pair_second_token_reserve")]
    fn pair_second_token_reserve(&self, pair_id: usize) -> SingleValueMapper<BigUint>;

    /**
     * Lp Token Id
     */
    #[view(getPairLpTokenId)]
    #[storage_mapper("pair_lp_token_id")]
    fn pair_lp_token_id(&self, pair_id: usize) -> SingleValueMapper<TokenIdentifier>;

    /**
     * Lp Token Total Supply
     */
    #[view(getPairLpTokenTotalSupply)]
    #[storage_mapper("pair_lp_token_supply")]
    fn pair_lp_token_supply(&self, pair_id: usize) -> SingleValueMapper<BigUint>;
}