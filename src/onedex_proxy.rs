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

    #[payable("*")]
    #[endpoint(swapMultiTokensFixedOutput)]
    fn swap_multi_tokens_fixed_output(
        &self,
        amount_out_wanted: BigUint,
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
}
