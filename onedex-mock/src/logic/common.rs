multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::state::State;

#[multiversx_sc::module]
pub trait CommonLogicModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::storage::pair_storage::PairStorageModule
{
    /**
     * Check if pair_id is valid
     */
    #[inline]
    fn require_valid_pair_id(
        &self,
        pair_id: usize,
    ) {
        require!(
            pair_id > 0 && pair_id <= self.last_pair_id().get(),
            "Invalid pair_id"
        );
    }

    /**
     * Check if caller is pair owner or admin
     */
    #[inline]
    fn require_pair_owner_or_admin(
        &self,
        pair_id: usize,
    ) {
        let caller = self.blockchain().get_caller();
        let creator = self.pair_owner(pair_id).get();
        require!(
            caller == creator || caller == self.blockchain().get_owner_address(),
            "caller must be owner of given pair or admin"
        );
    }

    /**
     * Get Pair_id from Token_ids
     */
    #[inline]
    fn get_pair_id(
        &self,
        token_in: &TokenIdentifier,
        token_out: &TokenIdentifier,
    ) -> usize {
        let pair_ids = self.pair_ids();
        if pair_ids.contains_key(&(token_in.clone(), token_out.clone())) {
            pair_ids.get(&(token_in.clone(), token_out.clone())).unwrap()
        } else if pair_ids.contains_key(&(token_out.clone(), token_in.clone())) {
            pair_ids.get(&(token_out.clone(), token_in.clone())).unwrap()
        } else {
            sc_panic!("pair of given tokens does not exist");
        }
    }

    /**
     * Check the pair status for swap
     */
    #[inline]
    fn require_pair_active_swap(
        &self,
        pair_id: usize,
    ) {
        require!(
            self.pair_state(pair_id).get() == State::Active,
            "state of pair must be Active"
        );
    }

    #[inline]
    fn require_pair_active(
        &self,
        pair_id: usize,
    ) {
        require!(
            self.pair_state(pair_id).get() == State::Active || self.pair_state(pair_id).get() == State::ActiveButNoSwap,
            "state of pair must be Active or ActiveButNoSwap"
        );
    }


    #[inline]
    fn require_pair_is_ready(
        &self,
        pair_id: usize,
    ) {
        self.require_valid_pair_id(pair_id);

        require!(
            !self.pair_first_token_id(pair_id).is_empty(),
            "first_token_id is not set"
        );
        require!(
            !self.pair_second_token_id(pair_id).is_empty(),
            "second_token_id is not set"
        );

        require!(
            !self.pair_lp_token_id(pair_id).is_empty(),
            "LP token is not issued"
        );

        let roles = self
            .blockchain()
            .get_esdt_local_roles(&self.pair_lp_token_id(pair_id).get());

        require!(
            roles.has_role(&EsdtLocalRole::Mint),
            "Smart Contract does not have LP token local mint role"
        );
        require!(
            roles.has_role(&EsdtLocalRole::Burn),
            "Smart Contract does not have LP token local burn role"
        );

        require!(
            self.pair_first_token_reserve(pair_id).get() != BigUint::zero() ,
            "first_token_reserve must not be zero"
        );
        require!(
            self.pair_second_token_reserve(pair_id).get() != BigUint::zero() ,
            "second_token_reserve must not be zero"
        );
    }
}