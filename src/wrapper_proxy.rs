multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    // #[payable("EGLD")]
    // #[endpoint(wrapEgld)]
    // fn wrapEgld(&self) -> EsdtTokenPayment<Self::Api>;

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self);
}
