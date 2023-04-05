multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Undelegation<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub unbond_epoch: u64,
}

#[multiversx_sc::module]
pub trait StorageModule {

    #[view(getProviderAddress)]
    #[storage_mapper("provider_address")]
    fn provider_address(&self) -> SingleValueMapper<ManagedAddress>;
   
    #[view(getUndelegated)]
    #[storage_mapper("undelegated")]
    fn undelegated(&self) -> VecMapper<Undelegation<Self::Api>>;
    
    #[view(getLiquidTokenSupply)]
    #[storage_mapper("liquid_token_suuply")]
    fn liquid_token_supply(&self) -> SingleValueMapper<BigUint>;
}
