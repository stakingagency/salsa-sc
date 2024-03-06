use crate::{config, assertions, helpers, consts::*};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UserModule:
    config::ConfigModule +
    helpers::HelpersModule +
    assertions::AssertionsModule
{
    /**
     * User stake
     */
    #[payable("*")]
    #[endpoint(userStake)]
    fn user_stake(&self, stake_id: usize) {
        self.assert_stake_active(stake_id);

        let mut stake = self.get_stake_check_exists(stake_id);
        if stake.start_nonce == 0 {
            stake.start_nonce = self.blockchain().get_block_nonce();
            stake.last_rps_update_nonce = stake.start_nonce;
            stake.remaining_nonces = stake.end_nonce - stake.start_nonce;
        }

        let payments = self.call_value().all_esdt_transfers();
        require!(stake.stake_tokens.len() == payments.len(), ERROR_TOKEN_PAYMENTS_COUNT);

        let caller = self.blockchain().get_caller();
        let mut found_tokens = 0;
        let mut min_ratio = BigUint::zero();
        for i in 0..stake.stake_tokens.len() {
            for j in 0..payments.len() {
                let payment = payments.get(j);
                require!(payment.amount > 0, ERROR_ZERO_VALUE_TRANSFER);

                if stake.stake_tokens.get(i).clone_value() == payment.token_identifier {
                    let ratio = &payment.amount * RATIO_MULTIPLIER / stake.stake_ratios.get(i).clone_value();
                    if ratio < min_ratio || min_ratio == 0 {
                        min_ratio = ratio;
                    }
                    found_tokens += 1;
                    break
                }
            }
        }
        require!(found_tokens == payments.len(), ERROR_UNKNOWN_TOKEN);

        let mut refunds: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        let mut user_stake = self.get_or_create_user_stake(stake_id, &caller);
        self.compute_user_rewards(&mut stake, &mut user_stake);
        for i in 0..stake.stake_tokens.len() {
            let mut staked = stake.staked.get(i).clone_value();
            for j in 0..payments.len() {
                let mut payment = payments.get(j);
                if stake.stake_tokens.get(i).clone_value() == payment.token_identifier {
                    let new_stake = &stake.stake_ratios.get(i).clone_value() * &min_ratio / RATIO_MULTIPLIER;
                    payment.amount -= &new_stake;
                    if payment.amount > 0 {
                        refunds.push(payment);
                    }
                    staked += &new_stake;
                    _ = user_stake.staked.set(i, &(user_stake.staked.get(i).clone_value() + new_stake));
                    break
                }
            }
            _ = stake.staked.set(i, &staked);
        }
        if !refunds.is_empty() {
            self.send().direct_multi(&caller, &refunds);
        }
        self.view_user_stake(stake_id, &caller).set(user_stake);
        self.all_user_stakes(&caller).insert(stake_id);
        self.stakers(stake_id).insert(caller);
        self.stake(stake_id).set(&stake);
    }

    /**
     * User unstake
     */
    #[endpoint(userUnstake)]
    fn user_unstake(&self, stake_id: usize, payments: ManagedVec<EsdtTokenPayment>) {
        self.assert_active();

        let mut stake = self.get_stake_check_exists(stake_id);
        require!(stake.stake_tokens.len() == payments.len(), ERROR_TOKEN_PAYMENTS_COUNT);

        let caller = self.blockchain().get_caller();
        let mut found_tokens = 0;
        let mut min_ratio = BigUint::zero();
        for i in 0..stake.stake_tokens.len() {
            for j in 0..payments.len() {
                let payment = payments.get(j);
                require!(payment.amount > 0, ERROR_ZERO_VALUE_TRANSFER);

                if stake.stake_tokens.get(i).clone_value() == payment.token_identifier {
                    let ratio = &payment.amount * RATIO_MULTIPLIER / stake.stake_ratios.get(i).clone_value();
                    if ratio < min_ratio || min_ratio == 0 {
                        min_ratio = ratio;
                    }
                    found_tokens += 1;
                    break
                }
            }
        }
        require!(found_tokens == payments.len(), ERROR_UNKNOWN_TOKEN);

        let mut unstake_payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        let mut user_stake = self.get_or_create_user_stake(stake_id, &caller);
        self.compute_user_rewards(&mut stake, &mut user_stake);
        let mut user_still_has_funds = false;
        for i in 0..stake.stake_tokens.len() {
            let mut staked = stake.staked.get(i).clone_value();
            for j in 0..payments.len() {
                let mut payment = payments.get(j);
                if stake.stake_tokens.get(i).clone_value() == payment.token_identifier {
                    let unstake = &stake.stake_ratios.get(i).clone_value() * &min_ratio / RATIO_MULTIPLIER;
                    require!(staked >= unstake, ERROR_NOT_ENOUGH_STAKE);

                    payment.amount = unstake.clone();
                    unstake_payments.push(payment);
                    staked -= &unstake;
                    let new_user_stake = user_stake.staked.get(i).clone_value() - unstake;
                    if new_user_stake > 0 {
                        user_still_has_funds = true;
                    }
                    _ = user_stake.staked.set(i, &new_user_stake);
                    break
                }
            }
            _ = stake.staked.set(i, &staked);
        }
        self.stake(stake_id).set(&stake);
        self.send().direct_multi(&caller, &unstake_payments);
        if user_still_has_funds {
            self.view_user_stake(stake_id, &caller).set(user_stake);
        } else {
            // send rewards
            let mut user_has_rewards = false;
            let mut claim_payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
            for i in 0..stake.reward_tokens.len() {
                let reward = user_stake.rewards.get(i).clone_value();
                if reward == 0 {
                    continue
                }
    
                user_has_rewards = true;
                let token = stake.reward_tokens.get(i).clone_value();
                claim_payments.push(EsdtTokenPayment::new(token, 0, reward));
                _ = user_stake.rewards.set(i, &BigUint::zero());
            }
            if user_has_rewards {
                self.send().direct_multi(&caller, &claim_payments);
            }

            // clear storage
            self.view_user_stake(stake_id, &caller).clear();
            self.stakers(stake_id).swap_remove(&caller);
            self.all_user_stakes(&caller).swap_remove(&stake_id);
        }
    }

    /**
     * Claim rewards
     */
    #[endpoint(claimRewards)]
    fn claim_rewards(&self, stake_id: usize) {
        self.assert_active();

        let caller = self.blockchain().get_caller();
        let mut stake = self.get_stake_check_exists(stake_id);
        let mut user_stake = self.get_or_create_user_stake(stake_id, &caller);
        self.compute_user_rewards(&mut stake, &mut user_stake);
        let mut user_has_rewards = false;
        let mut claim_payments: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        for i in 0..stake.reward_tokens.len() {
            let reward = user_stake.rewards.get(i).clone_value();
            if reward == 0 {
                continue
            }

            user_has_rewards = true;
            let token = stake.reward_tokens.get(i).clone_value();
            claim_payments.push(EsdtTokenPayment::new(token, 0, reward));
            _ = user_stake.rewards.set(i, &BigUint::zero());
        }
        require!(user_has_rewards, ERROR_NOTHING_TO_CLAIM);

        self.send().direct_multi(&caller, &claim_payments);
        self.view_user_stake(stake_id, &caller).set(user_stake);
        self.stake(stake_id).set(stake);
    }
}
