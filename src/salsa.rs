#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod proxies;
mod liquidity;
mod service;
mod onedex;

use crate::{common::config::*, common::consts::*, common::errors::*};

#[multiversx_sc::contract]
pub trait SalsaContract<ContractReader>:
    common::config::ConfigModule
    + liquidity::LiquidityModule
    + service::ServiceModule
    + onedex::OnedexModule
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

        let call_value = self.call_value().egld_value();
        let mut delegate_amount = call_value.clone_value();
        require!(
            delegate_amount >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let caller = self.blockchain().get_caller();
        let liquid_token_id = self.liquid_token_id().get_token_id();

        // arbitrage
        let salsa_amount_out = self.add_liquidity(&delegate_amount, false);
        let sold_amount = self.do_arbitrage_on_onedex(
            &TokenIdentifier::from(WEGLD_ID), &delegate_amount, &salsa_amount_out
        );
        delegate_amount -= &sold_amount;

        if delegate_amount > 0 {
            // normal delegate
            let ls_amount = self.add_liquidity(&delegate_amount, true);

            let delegation_contract = self.provider_address().get();
            let gas_for_async_call = self.get_gas_for_async_call();
            self.delegation_proxy_obj()
                .contract(delegation_contract)
                .delegate()
                .with_gas_limit(gas_for_async_call)
                .with_egld_transfer(delegate_amount.clone())
                .async_call()
                .with_callback(
                    SalsaContract::callbacks(self).delegate_callback(caller, delegate_amount, ls_amount),
                )
                .call_and_exit()
        } else {
            EsdtTokenPayment::new(liquid_token_id, 0, salsa_amount_out)
        }
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

        let mut payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        // arbitrage
        let salsa_amount_out = self.remove_liquidity(&payment.amount, false);
        let sold_amount = self.do_arbitrage_on_onedex(
            &liquid_token_id, &payment.amount, &salsa_amount_out
        );
        payment.amount -= sold_amount;
        if payment.amount == 0 {
            return
        }

        // normal undelegate
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
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.compute_withdrawn();

        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let total_user_withdrawn_egld = self.user_withdrawn_egld().get();
        let user_undelegations = self.user_undelegations(&caller).get();
        let mut remaining_undelegations: ManagedVec<Self::Api, Undelegation<Self::Api>> =
            ManagedVec::new();
        let mut withdraw_amount = BigUint::zero();
        let mut overflow = false;
        for user_undelegation in user_undelegations.into_iter() {
            let new_withdraw_amount = &withdraw_amount + &user_undelegation.amount;
            let would_overflow = new_withdraw_amount > total_user_withdrawn_egld;
            overflow = overflow || would_overflow;
            if user_undelegation.unbond_epoch <= current_epoch && !would_overflow {
                withdraw_amount = new_withdraw_amount;
            } else {
                remaining_undelegations.push(user_undelegation);
            }
        }
        if withdraw_amount == 0 {
            if overflow {
                sc_panic!(ERROR_NOT_ENOUGH_FUNDS);
            } else {
                sc_panic!(ERROR_NOTHING_TO_WITHDRAW);
            }
        }

        self.user_undelegations(&caller)
            .set(remaining_undelegations);
        self.user_withdrawn_egld()
            .update(|value| *value -= &withdraw_amount);
        self.send().direct_egld(&caller, &withdraw_amount);
    }

    // endpoints: reserves

    #[payable("EGLD")]
    #[endpoint(addReserve)]
    fn add_reserve(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let call_value = self.call_value().egld_value();
        let reserve_amount = call_value.clone_value();
        require!(
            reserve_amount >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let user_reserve_points = self.get_reserve_points_amount(&reserve_amount);

        self.users_reserve_points(&caller)
            .update(|value| *value += &user_reserve_points);
        self.reserve_points()
            .update(|value| *value += user_reserve_points);

        self.egld_reserve().update(|value| *value += &reserve_amount);
        self.available_egld_reserve().update(|value| *value += reserve_amount);
    }

    #[endpoint(removeReserve)]
    fn remove_reserve(&self, amount: BigUint) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
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
            let mut remaining_egld = egld_to_move.clone();
            let mut unbond_epoch = 0_u64;
            let reserve_undelegations = self.reserve_undelegations().get();
            let mut remaining_reserve_undelegations: ManagedVec<
                Self::Api,
                Undelegation<Self::Api>,
            > = ManagedVec::new();
            for mut reserve_undelegation in reserve_undelegations.into_iter() {
                if remaining_egld > 0 {
                    if remaining_egld <= reserve_undelegation.amount {
                        reserve_undelegation.amount -= &remaining_egld;
                        remaining_egld = BigUint::zero();
                        unbond_epoch = reserve_undelegation.unbond_epoch;
                    } else {
                        remaining_egld -= &reserve_undelegation.amount;
                        reserve_undelegation.amount = BigUint::zero();
                    }
                }
                if reserve_undelegation.amount > 0 {
                    remaining_reserve_undelegations.push(reserve_undelegation);
                }
            }
            require!(remaining_egld == 0, ERROR_NOT_ENOUGH_FUNDS);

            self.reserve_undelegations().set(remaining_reserve_undelegations);
            self.add_user_undelegation(egld_to_move.clone(), unbond_epoch);
            egld_to_remove = available_egld_reserve.clone();
        }
        self.available_egld_reserve().update(|value| *value -= &egld_to_remove);
        self.users_reserve_points(&caller)
            .update(|value| *value -= &points_to_remove);
        self.reserve_points()
            .update(|value| *value -= &points_to_remove);
        self.send().direct_egld(&caller, &egld_to_remove);
    }

    #[payable("*")]
    #[endpoint(unDelegateNow)]
    fn undelegate_now(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let mut payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let fee = self.undelegate_now_fee().get();
        let caller = self.blockchain().get_caller();

        // arbitrage
        let salsa_amount_out = self.remove_liquidity(&payment.amount, false);
        let sold_amount = self.do_arbitrage_on_onedex(
            &liquid_token_id, &payment.amount, &salsa_amount_out
        );
        payment.amount -= sold_amount;
        if payment.amount == 0 {
            return
        };

        // normal unDelegateNow
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

        // add to reserve undelegations
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + self.unbond_period().get();
        let reserve_undelegations = self.reserve_undelegations().get();
        let new_reserve_undelegations = self.add_undelegation(
            egld_to_undelegate.clone(), unbond_epoch, reserve_undelegations
        );
        self.reserve_undelegations().set(new_reserve_undelegations);

        // update storage
        self.egld_to_undelegate()
            .update(|value| *value += &egld_to_undelegate);
        self.available_egld_reserve().update(|value| *value -= &egld_to_undelegate_with_fee);
        let total_rewards = &egld_to_undelegate - &egld_to_undelegate_with_fee;
        self.egld_reserve().update(|value| *value += &total_rewards);

        self.send().direct_egld(&caller, &egld_to_undelegate_with_fee);
    }

    fn add_undelegation(
        &self,
        amount: BigUint,
        unbond_epoch: u64,
        undelegations: ManagedVec<Self::Api, Undelegation<Self::Api>>
    ) -> ManagedVec<Self::Api, Undelegation<Self::Api>> {
        let current_epoch = self.blockchain().get_block_epoch();
        let new_undelegation = Undelegation {
            amount: amount.clone(),
            unbond_epoch,
        };
        let mut found = false;
        let mut withdrawable_amount = BigUint::zero();
        let mut remaining_undelegations: ManagedVec<Self::Api, Undelegation<Self::Api>> =
            ManagedVec::new();
        for mut undelegation in undelegations.into_iter() {
            if undelegation.unbond_epoch == unbond_epoch {
                undelegation.amount += &amount;
                found = true;
            }
            if undelegation.unbond_epoch <= current_epoch {
                withdrawable_amount += undelegation.amount;
            } else {
                remaining_undelegations.push(undelegation);
            }
        }
        if withdrawable_amount > 0 {
            let merged_undelegation = Undelegation {
                amount: withdrawable_amount,
                unbond_epoch: current_epoch,
            };
            remaining_undelegations.push(merged_undelegation);
        }
        if !found {
            remaining_undelegations.push(new_undelegation);
        }
        
        remaining_undelegations
    }

    fn add_user_undelegation(&self, amount: BigUint, unbond_epoch: u64) {
        let user = self.blockchain().get_caller();

        let mut undelegations = self.user_undelegations(&user).get();
        let mut new_undelegations = self.add_undelegation(amount.clone(), unbond_epoch, undelegations);
        self.user_undelegations(&user).set(new_undelegations);

        undelegations = self.total_user_undelegations().get();
        new_undelegations = self.add_undelegation(amount.clone(), unbond_epoch, undelegations);
        self.total_user_undelegations().set(new_undelegations);
    }

    // endpoints: admin

    #[only_owner]
    #[endpoint(distributeProfit)]
    fn burn_ls_profit(&self) {
        require!(!self.is_state_active(), ERROR_ACTIVE);

        let egld_profit = self.egld_profit().get();
        if egld_profit > 0 {
            self.egld_reserve()
                .update(|value| *value += &egld_profit);
            self.available_egld_reserve()
                .update(|value| *value += &egld_profit);    
            self.egld_profit().clear();
        }

        let ls_profit = self.liquid_profit().get();
        if ls_profit > 0 {
            self.liquid_token_supply()
                .update(|value| *value -= &ls_profit);
            self.burn_liquid_token(&ls_profit);
            self.liquid_profit().clear();
        }
    }

    #[only_owner]
    #[endpoint(setArbitrageActive)]
    fn set_arbitrage_active(&self) {
        require!(!self.provider_address().is_empty(), ERROR_PROVIDER_NOT_SET);
        require!(!self.liquid_token_id().is_empty(), ERROR_TOKEN_NOT_SET);
        
        let pair_id = self.onedex_pair_id().get();
        require!(pair_id > 0, ERROR_ONEDEX_PAIR_ID);

        let fee = self.get_onedex_fee();
        self.onedex_fee().set(fee);
        self.arbitrage().set(State::Active);
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> proxies::delegation_proxy::Proxy<Self::Api>;
}
