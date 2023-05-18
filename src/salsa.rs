#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod consts;
pub mod delegation_proxy;
pub mod errors;

use crate::{config::*, consts::*, errors::*};

#[multiversx_sc::contract]
pub trait SalsaContract<ContractReader>:
    config::ConfigModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self) {
        self.state().set(State::Inactive);
    }

    // endpoints: liquid delegation

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) -> EsdtTokenPayment<Self::Api> {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegate_amount = self.call_value().egld_value();
        require!(
            delegate_amount.clone_value() >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let ls_amount = self.add_liquidity(&delegate_amount, true);

        let caller = self.blockchain().get_caller();
        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();
        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(delegate_amount.clone_value())
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).delegate_callback(caller, delegate_amount.clone_value(), ls_amount),
            )
            .call_and_exit()
    }

    #[callback]
    fn delegate_callback(
        &self,
        caller: ManagedAddress,
        staked_tokens: BigUint,
        liquid_tokens: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let user_payment = self.mint_liquid_token(liquid_tokens);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                self.total_egld_staked()
                    .update(|value| *value -= &staked_tokens);
                self.liquid_token_supply()
                    .update(|value| *value -= liquid_tokens);
                self.send().direct_egld(&caller, &staked_tokens);
            }
        }
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn undelegate(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_to_undelegate = self.remove_liquidity(&payment.amount, true);
        self.burn_liquid_token(&payment.amount);
        self.egld_to_undelegate()
            .update(|value| *value += &egld_to_undelegate);
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = current_epoch + self.unbond_period().get();
        self.add_user_undelegation(egld_to_undelegate, unbond_period);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let caller = self.blockchain().get_caller();
        self.do_withdraw(caller);
    }

    #[endpoint(withdrawForUser)]
    fn withdraw_for_user(&self, user: ManagedAddress) {
        self.do_withdraw(user);
    }

    fn do_withdraw(&self, user: ManagedAddress) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.compute_withdrawn();
        let current_epoch = self.blockchain().get_block_epoch();
        let mut total_user_withdrawn_egld = self.user_withdrawn_egld().get();
        let mut _dummy = 0u64;

        (total_user_withdrawn_egld, _dummy) = self.remove_undelegations(
            total_user_withdrawn_egld,
            current_epoch,
            self.luser_undelegations(&user),
            self.luser_undelegations(&user)
        );
        let withdraw_amount = self.user_withdrawn_egld().get() - &total_user_withdrawn_egld;
        require!(withdraw_amount > 0, ERROR_NOTHING_TO_WITHDRAW);

        self.user_withdrawn_egld()
            .set(total_user_withdrawn_egld);
        self.send().direct_egld(&user, &withdraw_amount);
    }

    // endpoints: reserves

    #[payable("EGLD")]
    #[endpoint(addReserve)]
    fn add_reserve(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        self.add_reserve_epoch(&caller).set(current_epoch);

        let reserve_amount = self.call_value().egld_value();
        require!(
            reserve_amount.clone_value() >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let user_reserve_points = self.get_reserve_points_amount(&reserve_amount);

        self.users_reserve_points(&caller)
            .update(|value| *value += &user_reserve_points);
        self.reserve_points()
            .update(|value| *value += user_reserve_points);

        self.egld_reserve().update(|value| *value += reserve_amount.clone_value());
        self.available_egld_reserve().update(|value| *value += reserve_amount.clone_value());
    }

    #[endpoint(removeReserve)]
    fn remove_reserve(&self, amount: BigUint) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let add_reserve_epoch = self.add_reserve_epoch(&caller).get();
        require!(
            add_reserve_epoch < current_epoch,
            ERROR_REMOVE_RESERVE_TOO_SOON
        );

        if add_reserve_epoch > 0 {
            self.add_reserve_epoch(&caller).clear();
        }
        let old_reserve_points = self.users_reserve_points(&caller).get();
        let old_reserve = self.get_reserve_egld_amount(&old_reserve_points);
        require!(old_reserve > 0, ERROR_USER_NOT_PROVIDER);
        require!(old_reserve >= amount, ERROR_NOT_ENOUGH_FUNDS);

        self.compute_withdrawn();
        
        let mut egld_to_remove = amount.clone();
        let mut points_to_remove = self.get_reserve_points_amount(&egld_to_remove);
        if &old_reserve - &amount < DUST_THRESHOLD {
            // avoid rounding issues
            points_to_remove = old_reserve_points.clone();
            egld_to_remove = old_reserve.clone();
        } else {
            require!(&old_reserve - &amount >= MIN_EGLD, ERROR_DUST_REMAINING);
        }
        self.egld_reserve().update(|value| *value -= &egld_to_remove);

        let available_egld_reserve = self.available_egld_reserve().get();
        // if there is not enough available reserve, move the reserve to user undelegation
        if egld_to_remove > available_egld_reserve {
            let egld_to_move = &egld_to_remove - &available_egld_reserve;
            let (remaining_egld, unbond_epoch) = self.remove_undelegations(
                egld_to_move.clone(),
                MAX_EPOCH,
                self.lreserve_undelegations(),
                self.lreserve_undelegations()
            );
            require!(remaining_egld == 0, ERROR_NOT_ENOUGH_FUNDS);

            self.add_user_undelegation(egld_to_move, unbond_epoch);
            egld_to_remove = available_egld_reserve;
        }
        self.available_egld_reserve()
            .update(|value| *value -= &egld_to_remove);
        self.users_reserve_points(&caller)
            .update(|value| *value -= &points_to_remove);
        self.reserve_points()
            .update(|value| *value -= &points_to_remove);
        self.send().direct_egld(&caller, &egld_to_remove);
    }

    #[payable("*")]
    #[endpoint(unDelegateNow)]
    fn undelegate_now(&self, min_amount_out: BigUint) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let fee = self.undelegate_now_fee().get();
        let caller = self.blockchain().get_caller();
        let total_egld_staked = self.total_egld_staked().get();

        let egld_to_undelegate = self.remove_liquidity(&payment.amount, true);
        self.burn_liquid_token(&payment.amount);
        require!(
            egld_to_undelegate >= MIN_EGLD,
            ERROR_BAD_PAYMENT_AMOUNT
        );

        let available_egld_reserve = self.available_egld_reserve().get();
        let egld_to_undelegate_with_fee =
            egld_to_undelegate.clone() - egld_to_undelegate.clone() * fee / MAX_PERCENT;
        require!(
            egld_to_undelegate_with_fee <= available_egld_reserve,
            ERROR_NOT_ENOUGH_FUNDS
        );
        require!(egld_to_undelegate <= total_egld_staked, ERROR_NOT_ENOUGH_FUNDS);
        require!(
            egld_to_undelegate_with_fee >= min_amount_out,
            ERROR_FEE_CHANGED
        );

        // add to reserve undelegations
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + self.unbond_period().get();

        self.add_undelegation(egld_to_undelegate.clone(), unbond_epoch, self.lreserve_undelegations());

        // update storage
        self.egld_to_undelegate()
            .update(|value| *value += &egld_to_undelegate);
        self.available_egld_reserve()
            .update(|value| *value -= &egld_to_undelegate_with_fee);
        let total_rewards = &egld_to_undelegate - &egld_to_undelegate_with_fee;
        self.egld_reserve()
            .update(|value| *value += &total_rewards);

        self.send().direct_egld(&caller, &egld_to_undelegate_with_fee);
    }

    fn add_undelegation(
        &self,
        amount: BigUint,
        unbond_epoch: u64,
        mut list: LinkedListMapper<Undelegation<Self::Api>>
    ) {
        let new_undelegation = config::Undelegation {
            amount: amount.clone(),
            unbond_epoch,
        };
        let mut found = false;
        for node in list.iter() {
            let node_id = node.get_node_id();
            let mut undelegation = node.into_value();
            if unbond_epoch < undelegation.unbond_epoch {
                list.push_before_node_id(node_id, new_undelegation.clone());
                found = true;
                break
            }
            if unbond_epoch == undelegation.unbond_epoch {
                undelegation.amount += amount;
                list.set_node_value_by_id(node_id, undelegation);
                found = true;
                break
            }
        }
        if !found {
            list.push_back(new_undelegation);
        }

        // merge
        let current_epoch = self.blockchain().get_block_epoch();
        let mut amount_to_merge = BigUint::zero();
        loop {
            let first = match list.front() {
                Some(value) => value,
                None => {
                    break
                }
            };
            let node_id = first.get_node_id();
            let undelegation = first.clone().into_value();
            if current_epoch >= undelegation.unbond_epoch {
                amount_to_merge += undelegation.amount;
                list.remove_node_by_id(node_id);
            } else {
                break
            }
        }
        if amount_to_merge > 0 {
            list.push_front(config::Undelegation {
                amount: amount_to_merge,
                unbond_epoch: current_epoch
            });
        }
    }

    fn add_user_undelegation(&self, amount: BigUint, unbond_epoch: u64) {
        let user = self.blockchain().get_caller();
        self.add_undelegation(amount.clone(), unbond_epoch, self.luser_undelegations(&user));
        self.add_undelegation(amount, unbond_epoch, self.ltotal_user_undelegations());
    }

    fn remove_undelegations(
        &self,
        amount: BigUint,
        ref_epoch: u64,
        list: LinkedListMapper<Undelegation<Self::Api>>,
        mut clone_list: LinkedListMapper<Undelegation<Self::Api>>
    ) -> (BigUint, u64) { // left amount, last epoch
        let mut total_amount = amount;
        let mut last_epoch = 0u64;
        for node in list.iter() {
            let mut modified = false;
            let node_id = node.get_node_id();
            let mut undelegation = node.clone().into_value();
            if undelegation.unbond_epoch <= ref_epoch && total_amount > 0 {
                if total_amount >= undelegation.amount {
                    total_amount -= undelegation.amount;
                    undelegation.amount = BigUint::zero();
                } else {
                    undelegation.amount -= total_amount;
                    total_amount = BigUint::zero();
                    last_epoch = undelegation.unbond_epoch;
                    modified = true;
                }
            }
            if undelegation.amount == 0 {
                clone_list.remove_node_by_id(node_id.clone());
            }
            if modified {
                clone_list.set_node_value_by_id(node_id, undelegation);
            }
        }

        (total_amount, last_epoch)
    }

    // endpoints: service

    #[endpoint(unDelegateAll)]
    fn undelegate_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let egld_to_undelegate = self.egld_to_undelegate().get();
        require!(
            egld_to_undelegate >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        self.egld_to_undelegate().clear();

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();
        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(egld_to_undelegate.clone())
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).undelegate_all_callback(egld_to_undelegate),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_all_callback(
        &self,
        egld_to_undelegate: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            ManagedAsyncCallResult::Err(_) => {
                self.egld_to_undelegate()
                    .update(|value| *value += egld_to_undelegate);
            }
        }
    }

    #[endpoint(compound)]
    fn compound(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let this_contract = self.blockchain().get_sc_address();
        let gas_for_async_call = self.get_gas_for_async_call();
        let claimable_rewards_amount = self.claimable_rewards_amount().get();
        let claimable_rewards_epoch = self.claimable_rewards_epoch().get();
        let current_epoch = self.blockchain().get_block_epoch();

        if claimable_rewards_amount == 0 || claimable_rewards_epoch != current_epoch {
            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .get_claimable_rewards(this_contract)
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).get_claimable_rewards_callback(current_epoch),
                )
                .call_and_exit()
        } else {
            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .redelegate_rewards()
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).compound_callback(claimable_rewards_amount),
                )
                .call_and_exit()
        }
    }

    #[callback]
    fn get_claimable_rewards_callback(
        &self,
        current_epoch: u64,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(total_rewards) => {
                self.claimable_rewards_amount().set(total_rewards);
                self.claimable_rewards_epoch().set(current_epoch);
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[callback]
    fn compound_callback(
        &self,
        claimable_rewards: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.total_egld_staked()
                    .update(|value| *value += claimable_rewards);
                self.claimable_rewards_amount().clear();
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(withdrawAll)]
    fn withdraw_all(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .withdraw()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(SalsaContract::callbacks(self).withdraw_all_callback())
            .call_and_exit()
    }

    #[callback]
    fn withdraw_all_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdrawn_amount = self.call_value().egld_value();
                self.total_withdrawn_egld()
                    .update(|value| *value += withdrawn_amount.clone_value());
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[endpoint(computeWithdrawn)]
    fn compute_withdrawn(&self) {
        let current_epoch = self.blockchain().get_block_epoch();
        let total_withdrawn_egld = self.total_withdrawn_egld().get();

        // compute user undelegations eligible for withdraw
        let (mut left_amount, mut _dummy) = self.remove_undelegations(
            total_withdrawn_egld.clone(),
            current_epoch,
            self.ltotal_user_undelegations(),
            self.ltotal_user_undelegations()
        );
        let withdrawn_for_users = &total_withdrawn_egld - &left_amount;
        self.user_withdrawn_egld()
            .update(|value| *value += &withdrawn_for_users);

        // compute reserve undelegations eligible for withdraw
        (left_amount, _dummy) = self.remove_undelegations(
            left_amount,
            current_epoch,
            self.lreserve_undelegations(),
            self.lreserve_undelegations()
        );
        let withdrawn_for_reserves = &total_withdrawn_egld - &left_amount - &withdrawn_for_users;
        self.available_egld_reserve()
            .update(|value| *value += withdrawn_for_reserves);
        
        self.total_withdrawn_egld()
            .set(&left_amount);
    }

    // helpers

    fn get_gas_for_async_call(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK,
            ERROR_INSUFFICIENT_GAS
        );

        gas_left - MIN_GAS_FOR_CALLBACK
    }

    fn add_liquidity(&self, new_stake_amount: &BigUint, update_storage: bool) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        let ls_amount = if total_egld_staked > 0 {
            if liquid_token_supply == 0 {
                new_stake_amount + &total_egld_staked
            } else {
                new_stake_amount * &liquid_token_supply / &total_egld_staked
            }
        } else {
            new_stake_amount.clone()
        };

        require!(ls_amount > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

        if update_storage {
            self.total_egld_staked()
                .update(|value| *value += new_stake_amount);
            self.liquid_token_supply()
               .update(|value| *value += &ls_amount);
        }

        ls_amount
    }

    fn remove_liquidity(&self, ls_amount: &BigUint, update_storage: bool) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        require!(
            &liquid_token_supply >= ls_amount,
            ERROR_NOT_ENOUGH_LIQUID_SUPPLY
        );
        require!(ls_amount > &0, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_amount = ls_amount * &total_egld_staked / &liquid_token_supply;
        require!(egld_amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        if update_storage {
            self.total_egld_staked()
                .update(|value| *value -= &egld_amount);
            self.liquid_token_supply()
                .update(|value| *value -= ls_amount);
        }

        egld_amount
    }

    fn mint_liquid_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.liquid_token_id().mint(amount)
    }

    fn burn_liquid_token(&self, amount: &BigUint) {
        self.liquid_token_id().burn(amount);
    }

    // proxies

    #[proxy]
    fn delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
