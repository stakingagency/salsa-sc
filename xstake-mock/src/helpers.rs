use crate::consts::*;
use crate::storage::*;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HelpersModule:
{
    /**
     * Update RPS
     */
    fn update_rps(&self, stake: &mut Stake<Self::Api>) {
        if stake.remaining_nonces == 0 {
            return
        }

        let mut current_nonce = self.blockchain().get_block_nonce();
        if current_nonce > stake.end_nonce {
            current_nonce = stake.end_nonce
        }
        let elapsed_nonces = current_nonce - stake.last_rps_update_nonce;
        if elapsed_nonces == 0 {
            return
        }

        let staked = stake.staked.get(0).clone_value();
        if staked > 0 {
            for i in 0..stake.reward_tokens.len() {
                let remaining_rewards = stake.remaining_rewards.get(i).clone_value();
                let claimable_rewards = stake.claimable_rewards.get(i).clone_value();
                let new_claimable_rewards =
                    &remaining_rewards * elapsed_nonces / stake.remaining_nonces;
                let new_rps = &new_claimable_rewards * RATIO_MULTIPLIER / &staked;
                let rps = stake.rps.get(i).clone_value();

                _ = stake.rps.set(i, &(rps + new_rps));
                _ = stake.claimable_rewards.set(i, &(claimable_rewards + &new_claimable_rewards));
                _ = stake.remaining_rewards.set(i, &(remaining_rewards - new_claimable_rewards));
            }
        }
        stake.last_rps_update_nonce = current_nonce;
        stake.remaining_nonces -= elapsed_nonces;
    }

    /**
     * Compute user rewards
     */
    fn compute_user_rewards(&self, mut stake: &mut Stake<Self::Api>, user_stake: &mut UserStake<Self::Api>) {
        self.update_rps(&mut stake);
        let staked = user_stake.staked.get(0).clone_value();
        for i in 0..stake.reward_tokens.len() {
            let old_reward = user_stake.rewards.get(i).clone_value();
            let user_rps = user_stake.rps.get(i).clone_value();
            let rps = stake.rps.get(i).clone_value();
            let reward = &staked * &(&rps - &user_rps) / RATIO_MULTIPLIER;
            _ = user_stake.rps.set(i, &rps);
            _ = user_stake.rewards.set(i, &(old_reward + &reward));
        }
    }
}
