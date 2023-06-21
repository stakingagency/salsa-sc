multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{common::consts::*, common::errors::*};

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum UndelegationType {
    UserList,
    TotalUsersList,
    ReservesList,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Undelegation<M: ManagedTypeApi> {
    pub amount: BigUint<M>,
    pub unbond_epoch: u64,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub enum KnightState {
    Inactive,
    PendingConfirmation,
    Active,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Knight<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub state: KnightState,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Heir<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub inheritance_epochs: u64,
    pub last_accessed_epoch: u64,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct UserInfo<M: ManagedTypeApi + multiversx_sc::api::StorageMapperApi> {
    pub undelegations: ManagedVec<M, Undelegation<M>>,
    pub reserve: BigUint<M>,
    pub delegation: BigUint<M>,
    pub knight: ManagedAddress<M>,
    pub heir: ManagedAddress<M>,
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
            payment_amount.clone_value(),
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
        require!(!self.unbond_period().is_empty(), ERROR_UNBOND_PERIOD_NOT_SET);

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

    #[view(getUnbondPeriod)]
    #[storage_mapper("unbond_period")]
    fn unbond_period(&self) -> SingleValueMapper<u64>;

    #[only_owner]
    #[endpoint(setUnbondPeriod)]
    fn set_unbond_period(&self, period: u64) {
        require!(!self.is_state_active(), ERROR_ACTIVE);
        require!(
            period > 0 && period <= MAX_UNBOND_PERIOD,
            ERROR_UNBOND_PERIOD_NOT_SET
        );
        require!(self.unbond_period().get() == 0, ERROR_UNBOND_PERIOD_ALREADY_SET);

        self.unbond_period().set(period);
    }

    // delegation

    #[view(getUserUndelegations)]
    #[storage_mapper("luser_undelegations")]
    fn luser_undelegations(
        &self,
        user: &ManagedAddress,
    ) -> LinkedListMapper<Undelegation<Self::Api>>;

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

    #[view(getTotalWithdrawnEgld)]
    #[storage_mapper("total_withdrawn_egld")]
    fn total_withdrawn_egld(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalUserUndelegations)] // total user undelegations per epoch
    #[storage_mapper("ltotal_user_undelegations")]
    fn ltotal_user_undelegations(&self) -> LinkedListMapper<Undelegation<Self::Api>>;

    #[storage_mapper("egld_to_undelegate")]
    fn egld_to_undelegate(&self) -> SingleValueMapper<BigUint>;

    // reserves

    #[view(getEgldReserve)]
    #[storage_mapper("egld_reserve")]
    fn egld_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getReservePoints)]
    #[storage_mapper("reserve_points")]
    fn reserve_points(&self) -> SingleValueMapper<BigUint>;

    #[view(getAvailableEgldReserve)]
    #[storage_mapper("available_egld_reserve")]
    fn available_egld_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getReserveUndelegations)]
    #[storage_mapper("lreserve_undelegations")]
    fn lreserve_undelegations(&self) -> LinkedListMapper<Undelegation<Self::Api>>;

    #[view(getUsersReservePoints)]
    #[storage_mapper("users_reserve_points")]
    fn users_reserve_points(&self, user: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[only_owner]
    #[endpoint(setUndelegateNowFee)]
    fn set_undelegate_now_fee(&self, new_fee: u64) {
        require!(!self.is_state_active(), ERROR_ACTIVE);
        require!(new_fee < MAX_PERCENT, ERROR_INCORRECT_FEE);

        self.undelegate_now_fee().set(new_fee);
    }

    #[view(getUndelegateNowFee)]
    #[storage_mapper("undelegate_now_fee")]
    fn undelegate_now_fee(&self) -> SingleValueMapper<u64>;

    #[view(getReservePointsAmount)]
    fn get_reserve_points_amount(&self, egld_amount: &BigUint) -> BigUint {
        let egld_reserve = self.egld_reserve().get();
        let reserve_points = self.reserve_points().get();
        let mut user_reserve_points = egld_amount.clone();
        if egld_reserve > 0 {
            if reserve_points == 0 {
                user_reserve_points += egld_reserve
            } else {
                user_reserve_points = egld_amount * &reserve_points / &egld_reserve
            }
        }

        user_reserve_points
    }

    #[view(getReserveEgldAmount)]
    fn get_reserve_egld_amount(&self, points_amount: &BigUint) -> BigUint {
        let egld_reserve = self.egld_reserve().get();
        let reserve_points = self.reserve_points().get();
        let mut user_egld_amount = points_amount.clone();
        if reserve_points > 0 {
            user_egld_amount = points_amount * &egld_reserve / &reserve_points
        }

        user_egld_amount
    }

    #[view(getUserReserve)]
    fn get_user_reserve(&self, user: &ManagedAddress) -> BigUint {
        let user_points = self.users_reserve_points(user).get();
        
        self.get_reserve_egld_amount(&user_points)
    }

    #[storage_mapper("add_reserve_epoch")]
    fn add_reserve_epoch(&self, user: &ManagedAddress) -> SingleValueMapper<u64>;

    // misc

    #[view(getTokenPrice)]
    fn token_price(&self) -> BigUint {
        let staked_egld = self.total_egld_staked().get();
        let token_supply = self.liquid_token_supply().get();

        let one = BigUint::from(1_000_000_000_000_000_000u64);
        if (token_supply == 0) || (staked_egld == 0) {
            one
        } else {
            one * staked_egld / token_supply
        }
    }

    // arbitrage

    #[storage_mapper("wegld_id")]
    fn wegld_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("wrap_sc")]
    fn wrap_sc(&self) -> SingleValueMapper<ManagedAddress<Self::Api>>;

    #[only_owner]
    #[endpoint(setWrapSC)]
    fn set_wrap_sc(&self, address: ManagedAddress<Self::Api>) {
        self.wrap_sc().set(address);
    }

    // custodial liquid staking

    #[view(getLegldInCustody)]
    #[storage_mapper("legld_in_custody")]
    fn legld_in_custody(&self) -> SingleValueMapper<BigUint>;

    // Comment
    // I'd use references as keys, so you don't need to clone the value each time
    // eg. fn user_delegation(&self, user: &ManagedAddress) -> SingleValueMapper<BigUint>;
    // this applies to all storage mappers that have complex types (no need for simple types like u64)
    // Also, I'd rename this to user_custodial_delegation, it is better understandable
    #[view(getUserDelegation)]
    #[storage_mapper("user_delegation")]
    fn user_delegation(&self, user: ManagedAddress) -> SingleValueMapper<BigUint>;

    // Comment
    // Use reference key
    #[view(getUserKnight)]
    #[storage_mapper("user_knight")]
    fn user_knight(&self, user: ManagedAddress) -> SingleValueMapper<Knight<Self::Api>>;

    // Comment
    // Use reference key
    #[view(getKnightUsers)]
    #[storage_mapper("knight_users")]
    fn knight_users(&self, knight: ManagedAddress) -> UnorderedSetMapper<ManagedAddress>;

    // Comment
    // Use reference key
    #[view(getUserHeir)]
    #[storage_mapper("user_heir")]
    fn user_heir(&self, user: ManagedAddress) -> SingleValueMapper<Heir<Self::Api>>;

    // Comment
    // Use reference key
    #[view(getHeirUsers)]
    #[storage_mapper("heir_users")]
    fn heir_users(&self, heir: ManagedAddress) -> UnorderedSetMapper<ManagedAddress>;

    // Comment
    // You can use ManagedAddress::default() for empty addresses
    // If you use references as keys for storage mappers, there's no need for cloning the values each time
    #[view(getUserInfo)]
    fn get_user_info(&self, user: ManagedAddress) -> UserInfo<Self::Api> {
        let user_knight = self.user_knight(user.clone());
        let knight = if user_knight.is_empty() {
            ManagedAddress::from(&[0u8; 32])
        } else {
            user_knight.get().address
        };

        let user_heir = self.user_heir(user.clone());
        let heir = if user_heir.is_empty() {
            ManagedAddress::from(&[0u8; 32])
        } else {
            user_heir.get().address
        };

        let mut undelegations: ManagedVec<Self::Api, Undelegation<Self::Api>> =
            ManagedVec::new();
        for node in self.luser_undelegations(&user).iter() {
            let undelegation = node.into_value();
            undelegations.push(undelegation);
        }

        UserInfo{
            undelegations,
            reserve: self.get_user_reserve(&user),
            delegation: self.user_delegation(user).get(),
            knight,
            heir,
        }
    }
}
