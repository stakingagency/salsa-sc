multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    Clone,
    PartialEq,
    Eq,
    Debug,
)]
// Comment
// If you use the user_undelegations approach, which I highly recommend
// Then the address field is no longer needed
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
    #[storage_mapper("liquid_token_supply")]
    fn liquid_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalEgldStaked)]
    #[storage_mapper("total_egld_staked")]
    fn total_egld_staked(&self) -> SingleValueMapper<BigUint>;

    // Comment
    // I recommend you to use this mapper to store the entire withdrawn amount
    #[view(getTotalWithdrawnAmount)]
    #[storage_mapper("total_withdrawn_amount")]
    fn total_withdrawn_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("userUndelegations")]
    fn user_undelegations(
        &self,
        user: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<Undelegation<Self::Api>>>;
}
