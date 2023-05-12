multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self);
}
