multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{consts::*, errors::*};

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

        self.unbond_period().set(period);
    }

    // delegation

    #[view(getUserUndelegations)]
    #[storage_mapper("user_undelegations")]
    fn user_undelegations(
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

    #[view(getTotalWithdrawnEgld)]
    #[storage_mapper("total_withdrawn_egld")]
    fn total_withdrawn_egld(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalUserUndelegations)] // total user undelegations per epoch
    #[storage_mapper("total_user_undelegations")]
    fn total_user_undelegations(&self) -> SingleValueMapper<ManagedVec<Undelegation<Self::Api>>>;

    #[storage_mapper("users_egld_to_undelegate")]
    fn users_egld_to_undelegate(&self) -> SingleValueMapper<BigUint>;

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
    #[storage_mapper("reserve_undelegations")]
    fn reserve_undelegations(&self) -> SingleValueMapper<ManagedVec<Undelegation<Self::Api>>>;

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

    #[storage_mapper("egld_to_replenish_reserve")]
    fn egld_to_replenish_reserve(&self) -> SingleValueMapper<BigUint>;

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

    // arbitrage

    #[inline]
    fn is_arbitrage_active(&self) -> bool {
        let arbitrage = self.arbitrage().get();
        arbitrage == State::Active
    }

    #[view(getArbitrageState)]
    #[storage_mapper("arbitrage")]
    fn arbitrage(&self) -> SingleValueMapper<State>;

    #[view(getLiquidProfit)]
    #[storage_mapper("liquid_profit")]
    fn liquid_profit(&self) -> SingleValueMapper<BigUint>;

    #[view(getEgldProfit)]
    #[storage_mapper("egld_profit")]
    fn egld_profit(&self) -> SingleValueMapper<BigUint>;

    // onedex

    #[storage_mapper("onedex_fee")]
    fn onedex_fee(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("onedex_pair_id")]
    fn onedex_pair_id(&self) -> SingleValueMapper<usize>;

    #[only_owner]
    #[endpoint(setOnedexPairId)]
    fn set_onedex_pair_id(&self, id: usize) {
        self.onedex_pair_id().set(id);
    }

    // misc

    #[view(getTokenPrice)]
    fn token_price(&self) -> BigUint {
        let staked_egld = self.total_egld_staked().get();
        let token_supply = self.liquid_token_supply().get();

        if token_supply == 0 {
            BigUint::zero()
        } else {
            staked_egld / token_supply
        }
    }
}
