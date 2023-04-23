multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{consts::MAX_PERCENT, errors::*};

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Undelegation<M: ManagedTypeApi> {
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
        require!(!self.is_state_active(), ERROR_ACTIVE);
        require!(self.liquid_token_id().is_empty(), ERROR_TOKEN_ALREADY_SET);

        let payment_amount = self.call_value().egld_value();
        self.liquid_token_id().issue_and_set_all_roles(
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[view(getLiquidTokenId)]
    #[storage_mapper("liquid_token_id")]
    fn liquid_token_id(&self) -> FungibleTokenMapper<Self::Api>;

    #[view(getLiquidTokenSupply)]
    #[storage_mapper("liquid_token_supply")]
    fn liquid_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        require!(!self.provider_address().is_empty(), ERROR_PROVIDER_NOT_SET);
        require!(!self.liquid_token_id().is_empty(), ERROR_TOKEN_NOT_SET);

        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    #[inline]
    fn is_state_active(&self) -> bool {
        let state = self.state().get();
        state == State::Active
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[only_owner]
    #[endpoint(setProviderAddress)]
    fn set_provider_address(self, address: ManagedAddress) {
        require!(!self.is_state_active(), ERROR_ACTIVE);

        require!(
            self.provider_address().is_empty(),
            ERROR_PROVIDER_ALREADY_SET
        );

        self.provider_address().set(address);
    }

    #[view(getProviderAddress)]
    #[storage_mapper("provider_address")]
    fn provider_address(&self) -> SingleValueMapper<ManagedAddress>;

    // delegation

    #[view(getUserUndelegations)]
    #[storage_mapper("user_undelegations")]
    fn user_undelegations(
        &self,
        user: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<Undelegation<Self::Api>>>;

    #[storage_mapper("backup_user_undelegations")]
    fn backup_user_undelegations(
        &self,
        user: &ManagedAddress,
    ) -> SingleValueMapper<ManagedVec<Undelegation<Self::Api>>>;

    #[view(getTotalEgldStaked)]
    #[storage_mapper("total_egld_staked")]
    fn total_egld_staked(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("claimable_rewards_amount")]
    fn claimable_rewards_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("claimable_rewards_epoch")]
    fn claimable_rewards_epoch(&self) -> SingleValueMapper<u64>;

    #[view(getUserWithdrawnEgld)]
    #[storage_mapper("user_withdrawn_egld")]
    fn user_withdrawn_egld(&self) -> SingleValueMapper<BigUint>;

    // reserves

    #[view(getEgldReserve)]
    #[storage_mapper("egld_reserve")]
    fn egld_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getAvailableEgldReserve)]
    #[storage_mapper("available_egld_reserve")]
    fn available_egld_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getReserveUndelegations)]
    #[storage_mapper("reserve_undelegations")]
    fn reserve_undelegations(&self) -> SingleValueMapper<ManagedVec<Undelegation<Self::Api>>>;

    #[storage_mapper("reservers_ids")]
    fn reservers_ids(&self) -> MapMapper<usize, ManagedAddress>;

    #[view(getReserverID)]
    #[storage_mapper("reservers_addresses")]
    fn reservers_addresses(&self, user: ManagedAddress) -> SingleValueMapper<usize>;

    #[view(getUsersReserves)]
    #[storage_mapper("users_reserves")]
    fn users_reserves(&self) -> VecMapper<BigUint>;

    #[view(getUserReserveByAddress)]
    fn get_user_reserve_by_address(&self, user: ManagedAddress) -> BigUint {
        let id = self.reservers_addresses(user).get();

        self.users_reserves().get(id)
    }

    #[only_owner]
    #[endpoint(setUndelegateNowFee)]
    fn set_undelegate_now_fee(&self, new_fee: u64) {
        require!(new_fee < MAX_PERCENT, ERROR_INCORRECT_FEE);

        self.undelegate_now_fee().set(new_fee);
    }

    #[view(getUndelegateNowFee)]
    #[storage_mapper("undelegate_now_fee")]
    fn undelegate_now_fee(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("egld_to_replenish_reserve")]
    fn egld_to_replenish_reserve(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("backup_egld_to_replenish_reserve")]
    fn backup_egld_to_replenish_reserve(&self) -> SingleValueMapper<BigUint>;

    // misc

    #[view(getTokenPrice)]
    fn token_price(&self) -> BigUint {
        let staked_egld = self.total_egld_staked().get();
        let token_supply = self.liquid_token_supply().get();

        staked_egld / token_supply
    }
}
