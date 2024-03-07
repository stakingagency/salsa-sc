multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::state::State;
use crate::constants::LP_TOKEN_DECIMALS;

#[multiversx_sc::module]
pub trait PairLogicModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::storage::pair_storage::PairStorageModule
    + crate::logic::common::CommonLogicModule
{
    /**
     * Register as creator
     *  Cost: 2 EGLD to enable swap
     */
    #[payable("EGLD")]
    #[endpoint(enableSwap)]
    fn enable_swap(
        &self,
        pair_id: usize
    ) {
        let registering_cost = self.call_value().egld_value().clone_value();

        require!(
            registering_cost == self.registering_cost().get(),
            "not enough registering cost"
        );

        self.require_pair_owner_or_admin(pair_id);

        require!(
            !self.pair_enabled(pair_id).get(),
            "Already enabled"
        );

        self.pair_enabled(pair_id).set(true);
        self.pair_state(pair_id).set(State::Active);

        // transfer registering cost to treasury and burner address
        self.send()
            .direct_egld(
                &self.treasury_address().get(),
                &(registering_cost.clone() / &BigUint::from(2u32))
            );

        self.send()
            .direct_egld(
                &self.burner_address().get(),
                &(registering_cost.clone() - &(registering_cost.clone() / &BigUint::from(2u32)))
            );
    }


    /**
     * Create ESDT-ESDT pair
     *  Constraint: Token Owner Only
     */
    #[endpoint(createPair)]
    fn create_pair(
        &self,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) -> usize {
        let caller = self.blockchain().get_caller();

        // Check if first token and second token is same
        require!(
            first_token_id != second_token_id,
            "First token_id must not be same with Second token_id"
        );

        // token identifier validator
        require!(
            first_token_id.is_valid_esdt_identifier(),
            "First token_id must be ESDT"
        );
        require!(
            second_token_id.is_valid_esdt_identifier(),
            "Second token_id must be ESDT"
        );

        require!(
            self.main_pair_tokens().contains(&second_token_id),
            "Invalid second token id for pair"
        );

        // create new pair_id
        let pair_key = (first_token_id.clone(), second_token_id.clone());
        let reverse_pair_key = (second_token_id.clone(), first_token_id.clone());

        let mut pair_ids = self.pair_ids();
        require!(
            !pair_ids.contains_key(&pair_key) && !pair_ids.contains_key(&reverse_pair_key),
            "Already existing pair"
        );
        let pair_id = self.last_pair_id().get() + 1;
        self.last_pair_id().set(pair_id);
        pair_ids.insert(pair_key, pair_id);

        // save pair params
        self.pair_owner(pair_id).set(&caller);
        self.pair_state(pair_id).set(State::Inactive);

        self.pair_first_token_id(pair_id).set(&first_token_id);
        self.pair_second_token_id(pair_id).set(&second_token_id);

        pair_id
    }


    /**
     * Issue Lp Token for pair
     */
    #[payable("EGLD")]
    #[endpoint(issueLpToken)]
    fn issue_lp_token(
        &self,
        pair_id: usize,
    ) {
        self.require_valid_pair_id(pair_id);
        self.require_pair_owner_or_admin(pair_id);

        let caller = self.blockchain().get_caller();
        let issue_cost = self.call_value().egld_value();

        require!(
            self.pair_lp_token_id(pair_id).is_empty(),
            "LP token is already issued"
        );

        let first_token_id = self.pair_first_token_id(pair_id).get();
        let second_token_id = self.pair_second_token_id(pair_id).get();

        let second_token_ticker = second_token_id.ticker();
        let first_token_ticker = first_token_id.ticker();

        let lp_ticker =
            if first_token_ticker.len() + second_token_ticker.len() <= 10 {
                first_token_ticker.concat(second_token_ticker)
            } else {
                first_token_ticker
                .copy_slice(0, 10 - second_token_ticker.len())
                .unwrap()
                .concat(second_token_ticker)
            };

        let lp_name = ManagedBuffer::from("OneDex")
            .concat(lp_ticker.clone())
            .concat(ManagedBuffer::from("LP"));

        // issue
        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(
                issue_cost.clone_value(),
                &lp_name,
                &lp_ticker,
                &BigUint::zero(), // Initial Supply
                FungibleTokenProperties {
                    num_decimals: LP_TOKEN_DECIMALS,
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_mint: true,
                    can_burn: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(
                self.callbacks()
                .issue_lp_token_callback(
                    &caller,
                    pair_id
                ))
            .call_and_exit()
    }

    #[callback]
    fn issue_lp_token_callback(
        &self,
        caller: &ManagedAddress,
        pair_id: usize,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_identifier) => {
                require!(
                    self.pair_lp_token_id(pair_id).is_empty(),
                    "LP token is already issued"
                );

                self.pair_lp_token_id(pair_id).set(&token_identifier);
                self.lp_token_pair_id_map().insert(token_identifier, pair_id);
            },
            ManagedAsyncCallResult::Err(_) => {
                let issue_cost = self.call_value().egld_value();

                self.send()
                    .direct_egld(caller, &issue_cost);
            },
        }
    }


    /**
     * Set Local Role to mint or burn lp token
     */
    #[endpoint(setLpTokenLocalRoles)]
    fn set_lp_token_local_roles(
        &self,
        pair_id: usize,
    ) {
        self.require_valid_pair_id(pair_id);
        self.require_pair_owner_or_admin(pair_id);

        require!(
            !self.pair_lp_token_id(pair_id).is_empty(),
            "LP token is not issued"
        );

        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.pair_lp_token_id(pair_id).get(),
                roles[..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }


    /**
     * Active Pair
     */
    #[only_owner]
    #[endpoint(setPairActive)]
    fn set_pair_active(
        &self,
        pair_id: usize,
    ) {
        self.pair_state(pair_id).set(State::Active);
    }


    /**
     * No Swap Pair
     */
    #[only_owner]
    #[endpoint(setPairActiveButNoSwap)]
    fn set_pair_no_swaps(
        &self,
        pair_id: usize,
    ) {
        self.pair_state(pair_id).set(State::ActiveButNoSwap);
    }

    #[only_owner]
    #[endpoint(setPairInactive)]
    fn set_pair_inactive(
        &self,
        pair_id: usize,
    ) {
        self.pair_state(pair_id).set(State::Inactive);
    }
}
