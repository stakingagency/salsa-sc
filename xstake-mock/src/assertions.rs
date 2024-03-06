use crate::{config, helpers, consts::*, storage::*};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AssertionsModule:
    config::ConfigModule +
    helpers::HelpersModule
{
    /**
     * Assert active
     */
    fn assert_active(&self) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
    }

    /**
     * Assert stake active
     */
    fn assert_stake_active(&self, stake_id: usize) {
        let stake = self.get_stake_check_exists(stake_id);
        let current_nonce = self.blockchain().get_block_nonce();
        require!(stake.end_nonce > 0, ERROR_STAKE_END_NOT_SET);
        require!(stake.end_nonce > current_nonce, ERROR_STAKE_EXPIRED);

        let state = self.get_stake_state(&stake);
        require!(state == State::Active, ERROR_STAKE_NOT_ACTIVE);
    }

    /**
     * Assert stake owner
     */
    fn assert_stake_owner(&self, stake_id: usize) {
        let caller = self.blockchain().get_caller();
        let stake = self.get_stake_check_exists(stake_id);
        require!(caller == stake.owner, ERROR_NOT_STAKE_OWNER);
    }
}
