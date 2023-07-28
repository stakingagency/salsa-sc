multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{common::consts::*, common::errors::*};

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
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

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub enum KnightState {
    Undefined,
    InactiveKnight,
    PendingConfirmation,
    ActiveKnight,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Knight<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub state: KnightState,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct Heir<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub inheritance_epochs: u64,
    pub last_accessed_epoch: u64,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct UserInfo<M: ManagedTypeApi + multiversx_sc::api::StorageMapperApi> {
    pub undelegations: ManagedVec<M, Undelegation<M>>,
    pub reserve: BigUint<M>,
    pub add_reserve_epoch: u64,
    pub delegation: BigUint<M>,
    pub knight: Knight<M>,
    pub knight_users: ManagedVec<M, Knight<M>>,
    pub heir: Heir<M>,
    pub heir_users: ManagedVec<M, Heir<M>>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct ContractInfo<M: ManagedTypeApi + multiversx_sc::api::StorageMapperApi> {
    pub state: State,
    pub liquid_token_id: TokenIdentifier<M>,
    pub liquid_token_supply: BigUint<M>,
    pub total_egld_staked: BigUint<M>,
    pub provider_address: ManagedAddress<M>,
    pub egld_reserve: BigUint<M>,
    pub available_egld_reserve: BigUint<M>,
    pub unbond_period: u64,
    pub undelegate_now_fee: u64,
    pub token_price: BigUint<M>,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub enum Exchange {
    None,
    Onedex,
    Xexchange,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug)]
pub struct LpInfo<M: ManagedTypeApi> {
    pub exchange: Exchange,
    pub liquid_reserve: BigUint<M>,
    pub egld_reserve: BigUint<M>,
    pub lp_supply: BigUint<M>,
    pub lp_token: TokenIdentifier<M>,
    pub lp_balance: BigUint<M>,
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

        self.compute_reserve_points_amount(egld_amount, &egld_reserve, &reserve_points)
    }

    fn compute_reserve_points_amount(&self, egld_amount: &BigUint, egld_reserve: &BigUint, reserve_points: &BigUint) -> BigUint {
        let mut points = egld_amount.clone();
        if egld_reserve > &0 {
            if reserve_points == &0 {
                points += egld_reserve
            } else {
                points = egld_amount * reserve_points / egld_reserve
            }
        };

        points
    }

    #[view(getReserveEgldAmount)]
    fn get_reserve_egld_amount(&self, points_amount: &BigUint) -> BigUint {
        let egld_reserve = self.egld_reserve().get();
        let reserve_points = self.reserve_points().get();

        self.compute_reserve_egld_amount(points_amount, &egld_reserve, &reserve_points)
    }

    fn compute_reserve_egld_amount(&self, points_amount: &BigUint, egld_reserve: &BigUint, reserve_points: &BigUint) -> BigUint {
        if reserve_points > &0 {
            points_amount * egld_reserve / reserve_points
        } else {
            points_amount.clone()
        }
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

        let one = BigUint::from(ONE_EGLD);
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

    #[storage_mapper("egld_in_lp")]
    fn egld_in_lp(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("legld_in_lp")]
    fn legld_in_lp(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("excess_lp_egld")]
    fn excess_lp_egld(&self) -> SingleValueMapper<BigUint>;
    
    #[storage_mapper("excess_lp_legld")]
    fn excess_lp_legld(&self) -> SingleValueMapper<BigUint>;
    
    // custodial liquid staking

    #[view(getLegldInCustody)]
    #[storage_mapper("legld_in_custody")]
    fn legld_in_custody(&self) -> SingleValueMapper<BigUint>;

    #[view(getUserDelegation)]
    #[storage_mapper("user_delegation")]
    fn user_delegation(&self, user: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(getUserKnight)]
    #[storage_mapper("user_knight")]
    fn user_knight(&self, user: &ManagedAddress) -> SingleValueMapper<Knight<Self::Api>>;

    #[view(getKnightUsers)]
    #[storage_mapper("knight_users")]
    fn knight_users(&self, knight: &ManagedAddress) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getUserHeir)]
    #[storage_mapper("user_heir")]
    fn user_heir(&self, user: &ManagedAddress) -> SingleValueMapper<Heir<Self::Api>>;

    #[view(getHeirUsers)]
    #[storage_mapper("heir_users")]
    fn heir_users(&self, heir: &ManagedAddress) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getContractInfo)]
    fn get_contract_info(&self) -> ContractInfo<Self::Api> {
        ContractInfo{
            state: self.state().get(),
            liquid_token_id: self.liquid_token_id().get_token_id(),
            liquid_token_supply: self.liquid_token_supply().get(),
            total_egld_staked: self.total_egld_staked().get(),
            provider_address: self.provider_address().get(),
            egld_reserve: self.egld_reserve().get(),
            available_egld_reserve: self.available_egld_reserve().get(),
            unbond_period: self.unbond_period().get(),
            undelegate_now_fee: self.undelegate_now_fee().get(),
            token_price: self.token_price(),
        }
    }

    #[view(getUserInfo)]
    fn get_user_info(&self, user: &ManagedAddress) -> UserInfo<Self::Api> {
        let user_knight = self.user_knight(user);
        let knight = if user_knight.is_empty() {
            Knight{
                address: ManagedAddress::from(&[0u8; 32]),
                state: KnightState::Undefined,
            }
        } else {
            user_knight.get()
        };

        let mut knight_users: ManagedVec<Self::Api, Knight<Self::Api>> = ManagedVec::new();
        for knight_user in self.knight_users(user).iter() {
            let mut k = self.user_knight(&knight_user).get();
            k.address = knight_user;
            knight_users.push(k);
        }

        let user_heir = self.user_heir(user);
        let heir = if user_heir.is_empty() {
            Heir{
                address: ManagedAddress::from(&[0u8; 32]),
                inheritance_epochs: 0,
                last_accessed_epoch: 0,
            }
        } else {
            user_heir.get()
        };

        let mut heir_users: ManagedVec<Self::Api, Heir<Self::Api>> = ManagedVec::new();
        for heir_user in self.heir_users(user).iter() {
            let mut h = self.user_heir(&heir_user).get();
            h.address = heir_user;
            heir_users.push(h);
        }

        let mut undelegations: ManagedVec<Self::Api, Undelegation<Self::Api>> = ManagedVec::new();
        for node in self.luser_undelegations(user).iter() {
            let undelegation = node.into_value();
            undelegations.push(undelegation);
        }

        UserInfo{
            undelegations,
            reserve: self.get_user_reserve(user),
            add_reserve_epoch: self.add_reserve_epoch(user).get(),
            delegation: self.user_delegation(user).get(),
            knight,
            knight_users,
            heir,
            heir_users,
        }
    }
}
