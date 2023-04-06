multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{
    errors::*,
};

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Undelegation<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub unbond_epoch: u64,
}

#[multiversx_sc::module]
pub trait ConfigModule:
    multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerLiquidToken)]
    fn register_liquid_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        require!(
            !self.is_state_active(),
            ERROR_ACTIVE
        );

        let payment_amount = self.call_value().egld_value();
        self.liquid_token_id().issue_and_set_all_roles(
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        let provider_address = self.provider_address().get();
        require!(
            provider_address != ManagedAddress::zero(),
            ERROR_PROVIDER_NOT_SET
        );

        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    #[only_owner]
    #[endpoint(setProviderAddress)]
    fn set_provider_address(
        &self,
        address: ManagedAddress
    ) {
        require!(
            !self.is_state_active(),
            ERROR_ACTIVE
        );
        
        let provider_address = self.provider_address().get();
        require!(
            provider_address == ManagedAddress::zero(),
            ERROR_PROVIDER_ALREADY_SET
        );

        self.provider_address().set(address);
    }

    #[inline]
    fn is_state_active(&self) -> bool {
        let state = self.state().get();
        state == State::Active
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[view(getLiquidTokenId)]
    #[storage_mapper("liquid_token_id")]
    fn liquid_token_id(&self) -> FungibleTokenMapper<Self::Api>;

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
