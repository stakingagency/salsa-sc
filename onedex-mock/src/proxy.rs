multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait ProxyModule {
    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self);
    
    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self);
}
