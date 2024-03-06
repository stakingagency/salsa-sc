multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::{consts::*, storage::*, helpers};

#[multiversx_sc::module]
pub trait ConfigModule:
    helpers::HelpersModule
{
    #[only_owner]
    #[endpoint(setState)]
    fn set_state(&self, new_state: State) {
        self.state().set(new_state);
    }

    // storage & view functions

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    // stakes

    /**
     * Get stake
     */
    fn get_stake_check_exists(&self, stake_id: usize) -> Stake<Self::Api> {
        let stake = self.stake(stake_id);
        require!(!stake.is_empty(), ERROR_STAKE_NOT_FOUND);

        stake.get()
    }

    /**
     * Get stake state
     */
    #[endpoint(getStakeState)]
    fn get_stake_state(&self, stake: &Stake<Self::Api>) -> State {
        if self.state().get() == State::Inactive {
            return State::Inactive
        }

        if stake.state == State::Active {
            let current_nonce = self.blockchain().get_block_nonce();
            if stake.end_nonce <= current_nonce {
                return State::Inactive
            }
        }

        stake.state
    }

    #[view(getLastStakeID)]
    #[storage_mapper("last_stake_id")]
    fn last_stake_id(&self) -> SingleValueMapper<usize>;

    #[view(getStakes)]
    fn get_stakes(&self) -> ManagedVec<Self::Api, Stake<Self::Api>> {
        let mut stakes: ManagedVec<Self::Api, Stake<Self::Api>> = ManagedVec::new();
        let last_id = self.last_stake_id().get();
        for id in 0..last_id+1 {
            let stake = self.stake(id);
            if !stake.is_empty() {
                let mut stake_value = stake.get();
                stake_value.state = self.get_stake_state(&stake_value);
                stakes.push(stake_value);
            }
        }

        stakes
    }

    #[view(getStake)]
    fn get_stake(&self, id: usize) -> Stake<Self::Api> {
        let mut stake = self.get_stake_check_exists(id);
        stake.state = self.get_stake_state(&stake);

        stake
    }

    #[storage_mapper("STAKE")]
    fn stake(&self, id: usize) -> SingleValueMapper<Stake<Self::Api>>;

    #[view(getStakers)]
    #[storage_mapper("stakers")]
    fn stakers(&self, id: usize) -> UnorderedSetMapper<Self::Api, ManagedAddress<Self::Api>>;

    #[view(getUserStake)]
    fn get_user_stake(&self, id: usize, user: &ManagedAddress) -> UserStake<Self::Api> {
        let mut stake = self.get_stake_check_exists(id);
        let mut user_stake = self.get_or_create_user_stake(id, user);
        self.compute_user_rewards(&mut stake, &mut user_stake);

        user_stake
    }

    #[storage_mapper("user_stake")]
    fn view_user_stake(&self, id: usize, user: &ManagedAddress) -> SingleValueMapper<UserStake<Self::Api>>;

    #[view(getAllUserStakes)]
    #[storage_mapper("all_user_stakes")]
    fn all_user_stakes(&self, user: &ManagedAddress) -> UnorderedSetMapper<usize>;

    fn get_or_create_user_stake(&self, stake_id: usize, user: &ManagedAddress) -> UserStake<Self::Api> {
        let user_stake_storage = self.view_user_stake(stake_id, user);
        if user_stake_storage.is_empty() {
            self.empty_userstake(stake_id)
        } else {
            user_stake_storage.get()
        }
    }

    /**
     * Empty userstake
     */
    fn empty_userstake(&self, stake_id: usize) -> UserStake<Self::Api> {
        let stake = self.get_stake_check_exists(stake_id);
        let mut staked: ManagedVec<Self::Api, BigUint<Self::Api>> = ManagedVec::new();
        for _ in stake.stake_tokens.iter() {
            staked.push(BigUint::zero());
        }
        let mut rewards: ManagedVec<Self::Api, BigUint<Self::Api>> = ManagedVec::new();
        for _ in stake.reward_tokens.iter() {
            rewards.push(BigUint::zero());
        }

        UserStake{
            staked,
            rewards,
            rps: stake.rps,
        }
    }
}
