multiversx_sc::imports!();

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

    #[view(getPairFirstTokenReserve)]
    #[storage_mapper("pair_first_token_reserve")]
    fn pair_first_token_reserve(&self, pair_id: usize) -> SingleValueMapper<BigUint>;

    #[view(getPairSecondTokenReserve)]
    #[storage_mapper("pair_second_token_reserve")]
    fn pair_second_token_reserve(&self, pair_id: usize) -> SingleValueMapper<BigUint>;

    #[view(getTotalFeePercent)]
    #[storage_mapper("total_fee_percent")]
    fn total_fee_percent(&self) -> SingleValueMapper<u64>;
}
