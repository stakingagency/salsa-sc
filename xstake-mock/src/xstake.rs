#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod consts;
pub mod assertions;
pub mod helpers;
pub mod storage;
pub mod user;

use crate::consts::*;
use crate::storage::*;

#[multiversx_sc::contract]
pub trait XStakeMock<ContractReader>:
    config::ConfigModule +
    assertions::AssertionsModule +
    helpers::HelpersModule +
    user::UserModule
{
    #[init]
    fn init(&self) {
        self.set_state(State::Active);
    }

    #[upgrade]
    fn upgrade(&self) {}

    /**
     * Create stake
     */
    #[payable("EGLD")]
    #[endpoint(createStake)]
    fn create_stake(
        &self,
        stake_tokens: ManagedVec<TokenIdentifier>,
        stake_ratios: ManagedVec<BigUint>,
        reward_tokens: ManagedVec<TokenIdentifier>,
    ) -> usize {
        self.assert_active();

        let slen = stake_tokens.len();
        let rlen = reward_tokens.len();
        require!(slen > 0, ERROR_NO_STAKE_TOKENS);
        require!(rlen > 0, ERROR_NO_REWARD_TOKENS);
        require!(slen == stake_ratios.len(), ERROR_RATIOS_COUNT_MISMATCH);

        let mut new_stake_tokens: ManagedVec<Self::Api, TokenIdentifier<Self::Api>> = ManagedVec::new();
        let mut new_stake_ratios: ManagedVec<Self::Api, BigUint<Self::Api>> = ManagedVec::new();
        let mut zero_staked: ManagedVec<Self::Api, BigUint<Self::Api>> = ManagedVec::new();
        for i in 0..slen {
            for j in i + 1..slen {
                require!(stake_tokens.get(i) != stake_tokens.get(j), ERROR_DUPLICATE_STAKE_TOKEN);
            }
            let ratio = stake_ratios.get(i).clone_value();
            require!(ratio >= MIN_RATIO, ERROR_RATIO_OUT_OF_BOUNDS);

            new_stake_tokens.push(stake_tokens.get(i).clone_value());
            new_stake_ratios.push(ratio);
            zero_staked.push(BigUint::zero());
        }
        let mut new_reward_tokens: ManagedVec<Self::Api, TokenIdentifier<Self::Api>> = ManagedVec::new();
        let mut zero_rewards: ManagedVec<Self::Api, BigUint<Self::Api>> = ManagedVec::new();
        for i in 0..rlen {
            for j in i+1..rlen {
                require!(reward_tokens.get(i) != reward_tokens.get(j), ERROR_DUPLICATE_REWARD_TOKEN);
            }
            new_reward_tokens.push(reward_tokens.get(i).clone_value());
            zero_rewards.push(BigUint::zero());
        }
        let stake_id = self.last_stake_id().get() + 1;
        self.last_stake_id().set(stake_id);
        let new_stake = Stake{
            stake_id,
            owner: self.blockchain().get_caller(),
            state: State::Inactive,
            stake_tokens: new_stake_tokens,
            stake_ratios: new_stake_ratios,
            reward_tokens: new_reward_tokens,
            staked: zero_staked,
            rewards: zero_rewards.clone(),
            start_nonce: 0,
            end_nonce: 0,
            rps: zero_rewards.clone(),
            claimable_rewards: zero_rewards.clone(),
            remaining_rewards: zero_rewards,
            last_rps_update_nonce: 0,
            remaining_nonces: 0,
        };
        self.stake(stake_id).set(new_stake);

        stake_id
    }

    /**
     * Set stake state
     */
    #[endpoint(setStakeState)]
    fn set_stake_state(&self, stake_id: usize, new_state: State) {
        self.assert_active();
        self.assert_stake_owner(stake_id);

        let mut stake = self.get_stake_check_exists(stake_id);
        if stake.state == new_state {
            return
        }

        if new_state == State::Active {
            let current_nonce = self.blockchain().get_block_nonce();

            require!(stake.end_nonce > 0, ERROR_STAKE_END_NOT_SET);
            require!(stake.end_nonce > current_nonce, ERROR_STAKE_EXPIRED);
            require!(stake.rewards.get(0).clone_value() > 0, ERROR_NO_REWARDS_DEPOSITED);
        }

        stake.state = new_state;
        self.stake(stake_id).set(stake);
    }

    /**
     * Add stake rewards
     */
    #[payable("*")]
    #[endpoint(addStakeRewards)]
    fn add_stake_rewards(&self, stake_id: usize) {
        self.assert_active();
        self.assert_stake_owner(stake_id);

        let mut stake = self.get_stake_check_exists(stake_id);
        self.update_rps(&mut stake);

        let payments = self.call_value().all_esdt_transfers();
        let mut found_tokens = 0;
        for i in 0..stake.reward_tokens.len() {
            let mut rewards = stake.rewards.get(i).clone_value();
            let mut remaining_rewards = stake.remaining_rewards.get(i).clone_value();
            for j in 0..payments.len() {
                let payment = payments.get(j);
                require!(payment.amount > 0, ERROR_ZERO_VALUE_TRANSFER);

                if stake.reward_tokens.get(i).clone_value() == payment.token_identifier {
                    rewards += &payment.amount;
                    remaining_rewards += &payment.amount;
                    found_tokens += 1;
                    break
                }
            }
            _ = stake.rewards.set(i, &rewards);
            _ = stake.remaining_rewards.set(i, &remaining_rewards);
        }
        require!(found_tokens > 0, ERROR_NO_REWARD_ADDED);
        require!(found_tokens == payments.len(), ERROR_UNKNOWN_TOKEN);

        self.stake(stake_id).set(stake);
    }

    /**
     * Withdraw stake rewards
     */
    #[endpoint(withdrawStakeRewards)]
    fn withdraw_stake_rewards(&self, stake_id: usize, payments: ManagedVec<EsdtTokenPayment>) {
        self.assert_active();
        self.assert_stake_owner(stake_id);

        let mut stake = self.get_stake_check_exists(stake_id);
        self.update_rps(&mut stake);

        let mut found_tokens = 0;
        for i in 0..stake.reward_tokens.len() {
            let mut rewards = stake.rewards.get(i).clone_value();
            let mut remaining_rewards = stake.remaining_rewards.get(i).clone_value();
            for j in 0..payments.len() {
                let mut payment = payments.get(j);
                require!(payment.amount > 0, ERROR_ZERO_VALUE_TRANSFER);

                if stake.reward_tokens.get(i).clone_value() == payment.token_identifier {
                    require!(rewards >= payment.amount, ERROR_INSUFFICIENT_FUNDS);
                    if payment.amount > remaining_rewards {
                        payment.amount = remaining_rewards.clone();
                    }
                    rewards -= &payment.amount;
                    remaining_rewards -= &payment.amount;
                    found_tokens += 1;
                    break
                }
            }
            _ = stake.rewards.set(i, &rewards);
            _ = stake.remaining_rewards.set(i, &remaining_rewards);
        }
        require!(found_tokens > 0, ERROR_NO_REWARD_WITHDRAWN);
        require!(found_tokens == payments.len(), ERROR_UNKNOWN_TOKEN);

        self.send().direct_multi(&stake.owner, &payments);
        self.stake(stake_id).set(stake);
    }

    /**
     * Change stake end
     */
    #[endpoint(changeStakeEnd)]
    fn change_stake_end(&self, stake_id: usize, new_end_nonce: u64) {
        self.assert_active();
        self.assert_stake_owner(stake_id);

        let current_nonce = self.blockchain().get_block_nonce();
        require!(new_end_nonce > current_nonce, ERROR_END_NONCE_IN_THE_PAST);

        let mut stake = self.get_stake_check_exists(stake_id);
        self.update_rps(&mut stake);

        if stake.start_nonce > 0 && stake.end_nonce > 0 {
            if stake.end_nonce < new_end_nonce {
                // stake period increased
                stake.remaining_nonces += new_end_nonce - stake.end_nonce;
            } else {
                // stake period decreased
                stake.remaining_nonces -= stake.end_nonce - new_end_nonce;
            }
        }
        stake.end_nonce = new_end_nonce;
        self.stake(stake_id).set(stake);
    }
}
