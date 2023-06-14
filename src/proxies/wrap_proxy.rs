multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait WrapProxy {
    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self) -> EsdtTokenPayment<Self::Api>;

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self);

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
