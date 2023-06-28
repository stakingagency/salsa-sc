#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod proxies;
pub mod helpers;
pub mod service;
pub mod exchanges;
pub mod knights;
pub mod heirs;

use crate::{common::config::*, common::consts::*, common::errors::*};

#[multiversx_sc::contract]
pub trait SalsaContract<ContractReader>:
    common::config::ConfigModule
    + helpers::HelpersModule
    + service::ServiceModule
    + exchanges::arbitrage::ArbitrageModule
    + exchanges::onedex::OnedexModule
    + exchanges::xexchange::XexchangeModule
    + knights::KnightsModule
    + heirs::HeirsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self) {
        // self.state().set(State::Inactive);
    }

    // endpoints: liquid delegation

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(
        &self,
        with_custody: OptionalValue<bool>,
    ) -> EsdtTokenPayment<Self::Api> {
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let amount = self.call_value().egld_value();
        let mut delegate_amount = amount.clone_value();
        require!(
            delegate_amount >= MIN_EGLD,
            ERROR_INSUFFICIENT_AMOUNT
        );

        let custodial = match with_custody {
            OptionalValue::Some(value) => value,
            OptionalValue::None => false
        };

        // check if caller is non-payable sc
        // TODO: once v0.42 is out, use get_code_metadata
        let caller = self.blockchain().get_caller();
        if self.blockchain().is_smart_contract(&caller) && !custodial {
            self.send().direct_egld(&caller, &BigUint::zero());
        }

        // arbitrage
        let (sold_amount, bought_amount) =
            self.do_arbitrage(&self.wegld_id().get(), delegate_amount.clone());

        let liquid_token_id = self.liquid_token_id().get_token_id();
        if bought_amount > 0 {
            if custodial {
                self.user_delegation(&caller)
                    .update(|value| *value += &bought_amount);
                self.legld_in_custody()
                    .update(|value| *value += bought_amount);
            } else {
                self.send().direct_esdt(
                    &caller,
                    &liquid_token_id,
                    0,
                    &bought_amount,
                );
            }
        }

        delegate_amount -= &sold_amount;
        if delegate_amount == 0 {
            return EsdtTokenPayment::new(liquid_token_id, 0, sold_amount)
        }

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
                SalsaContract::callbacks(self).delegate_callback(caller, custodial, delegate_amount, ls_amount),
            )
            .call_and_exit()
    }

    #[callback]
    fn delegate_callback(
        &self,
        caller: ManagedAddress,
        custodial: bool,
        staked_tokens: BigUint,
        liquid_tokens: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let user_payment = self.mint_liquid_token(liquid_tokens);
                if custodial {
                    self.user_delegation(&caller)
                        .update(|value| *value += &user_payment.amount);
                    self.legld_in_custody()
                        .update(|value| *value += user_payment.amount);
                } else {
                    self.send().direct_esdt(
                        &caller,
                        &user_payment.token_identifier,
                        0,
                        &user_payment.amount,
                    );
                }
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
    fn undelegate(
        &self,
        undelegate_amount: OptionalValue<BigUint>,
    ) {
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let amount = match undelegate_amount {
            OptionalValue::Some(value) => value,
            OptionalValue::None => BigUint::zero()
        };
        let caller = self.blockchain().get_caller();
        self.check_knight_activated(&caller);
        self.do_undelegate(caller, amount);
    }

    fn do_undelegate(
        &self,
        caller: ManagedAddress,
        undelegate_amount: BigUint,
    ) {
        let (payment_token, mut payment_amount) = self.call_value().egld_or_single_fungible_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        if payment_amount > 0 {
            require!(
                payment_token == liquid_token_id,
                ERROR_BAD_PAYMENT_TOKEN
            );
        }
        payment_amount += &undelegate_amount;
        require!(
            payment_amount > 0u64,
            ERROR_BAD_PAYMENT_AMOUNT,
        );

        if undelegate_amount > 0 {
            let delegated_funds = self.user_delegation(&caller).get();
            require!(
                delegated_funds >= undelegate_amount,
                ERROR_INSUFFICIENT_FUNDS,
            );

            self.user_delegation(&caller).set(&delegated_funds - &undelegate_amount);
            self.legld_in_custody()
                .update(|value| *value -= &undelegate_amount);
        }

        // arbitrage
        if self.user_knight(&caller).is_empty() {
            let (sold_amount, bought_amount) =
                self.do_arbitrage(&liquid_token_id, payment_amount.clone());
            if bought_amount > 0 {
                self.send().direct_egld(&caller, &bought_amount);
            }
            payment_amount -= sold_amount;
            if payment_amount == 0 {
                return
            }
        }

        // normal undelegate
        let egld_to_undelegate = self.remove_liquidity(&payment_amount, true);
        self.burn_liquid_token(&payment_amount);
        self.egld_to_undelegate()
            .update(|value| *value += &egld_to_undelegate);
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = current_epoch + self.unbond_period().get();
        self.add_user_undelegation(caller, egld_to_undelegate, unbond_period);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let user = self.blockchain().get_caller();
        self.check_knight_activated(&user);
        self.do_withdraw(&user, &user);
    }

    fn do_withdraw(
        &self,
        user: &ManagedAddress,
        receiver: &ManagedAddress,
    ) {
        self.compute_withdrawn();
        let current_epoch = self.blockchain().get_block_epoch();
        let mut total_user_withdrawn_egld = self.user_withdrawn_egld().get();

        (total_user_withdrawn_egld, _) = self.remove_undelegations(
            total_user_withdrawn_egld,
            current_epoch,
            self.luser_undelegations(&user),
            UndelegationType::UserList,
            user.clone()
        );
        let withdraw_amount = self.user_withdrawn_egld().get() - &total_user_withdrawn_egld;
        require!(withdraw_amount > 0, ERROR_NOTHING_TO_WITHDRAW);

        if self.user_delegation(&user).get() == 0 {
            let knight = self.user_knight(&user);
            if !knight.is_empty() {
                self.knight_users(&knight.get().address).swap_remove(user);
                knight.clear();
            }
            let heir = self.user_heir(&user);
            if !heir.is_empty() {
                self.heir_users(&heir.get().address).swap_remove(user);
                heir.clear();
            }
        }

        self.user_withdrawn_egld()
            .set(total_user_withdrawn_egld);
        self.send().direct_egld(&receiver, &withdraw_amount);
    }

    // endpoints: custody

    #[payable("*")]
    #[endpoint(addToCustody)]
    fn add_to_custody(&self) {
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let (payment_token, payment_amount) = self.call_value().egld_or_single_fungible_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(payment_token == liquid_token_id, ERROR_BAD_PAYMENT_TOKEN);

        let caller = self.blockchain().get_caller();
        self.user_delegation(&caller)
            .update(|value| *value += &payment_amount);
        self.legld_in_custody()
            .update(|value| *value += payment_amount);
    }

    #[endpoint(removeFromCustody)]
    fn remove_from_custody(&self, amount: BigUint) {
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        self.check_knight_set(&caller);

        let delegation = self.user_delegation(&caller).take();
        require!(amount <= delegation, ERROR_INSUFFICIENT_FUNDS);
        require!(&delegation - &amount >= MIN_EGLD || delegation == amount, ERROR_DUST_REMAINING);
        require!(delegation > amount || self.user_heir(&caller).is_empty(), ERROR_HEIR_SET);

        let liquid_token_id = self.liquid_token_id().get_token_id();
        self.send().direct_esdt(
            &caller,
            &liquid_token_id,
            0,
            &amount,
        );
        if delegation > amount {
            self.user_delegation(&caller).set(&delegation - &amount);
        }
        self.legld_in_custody().update(|value| *value -= amount);
    }

    // endpoints: reserves

    #[payable("EGLD")]
    #[endpoint(addReserve)]
    fn add_reserve(&self) {
        self.update_last_accessed();
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
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        self.check_knight_activated(&caller);
        self.do_remove_reserve(caller.clone(), caller, amount);
    }

    fn do_remove_reserve(
        &self,
        caller: ManagedAddress,
        receiver: ManagedAddress,
        amount: BigUint,
    ) {
        let current_epoch = self.blockchain().get_block_epoch();
        let add_reserve_epoch = self.add_reserve_epoch(&caller).take();
        require!(
            add_reserve_epoch < current_epoch,
            ERROR_REMOVE_RESERVE_TOO_SOON
        );

        let old_reserve_points = self.users_reserve_points(&caller).get();
        let old_reserve = self.get_reserve_egld_amount(&old_reserve_points);
        require!(old_reserve > 0, ERROR_USER_NOT_PROVIDER);
        require!(old_reserve >= amount, ERROR_NOT_ENOUGH_FUNDS);

        self.compute_withdrawn();

        let mut egld_to_remove = amount.clone();
        let mut points_to_remove = self.get_reserve_points_amount(&egld_to_remove) + 1u64;
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
            let unbond_period = self.unbond_period().get();
            let egld_to_move = &egld_to_remove - &available_egld_reserve;
            let (remaining_egld, unbond_epoch) = self.remove_undelegations(
                egld_to_move.clone(),
                &current_epoch + &unbond_period,
                self.lreserve_undelegations(),
                UndelegationType::ReservesList,
                caller.clone()
            );
            require!(remaining_egld == 0, ERROR_NOT_ENOUGH_FUNDS);

            self.add_user_undelegation(caller.clone(), egld_to_move, unbond_epoch);
            egld_to_remove = available_egld_reserve;
        }
        self.available_egld_reserve()
            .update(|value| *value -= &egld_to_remove);
        self.users_reserve_points(&caller)
            .update(|value| *value -= &points_to_remove);
        self.reserve_points()
            .update(|value| *value -= &points_to_remove);
        self.send().direct_egld(&receiver, &egld_to_remove);
    }

    #[payable("*")]
    #[endpoint(unDelegateNow)]
    fn undelegate_now(
        &self,
        min_amount_out: BigUint,
        undelegate_amount: OptionalValue<BigUint>,
    ) {
        self.update_last_accessed();
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let amount = match undelegate_amount {
            OptionalValue::Some(value) => value,
            OptionalValue::None => BigUint::zero()
        };
        let caller = self.blockchain().get_caller();
        self.check_knight_set(&caller);
        self.do_undelegate_now(caller.clone(), caller, min_amount_out, amount);
    }

    fn do_undelegate_now(
        &self,
        caller: ManagedAddress,
        receiver: ManagedAddress,
        min_amount_out: BigUint,
        undelegate_amount: BigUint,
    ) {
        if undelegate_amount > 0 {
            let delegated_funds = self.user_delegation(&caller).get();
            require!(
                delegated_funds >= undelegate_amount,
                ERROR_INSUFFICIENT_FUNDS,
            );

            self.user_delegation(&caller).set(&delegated_funds - &undelegate_amount);
            self.legld_in_custody()
                .update(|value| *value -= &undelegate_amount);
        }

        let (payment_token, mut payment_amount) = self.call_value().egld_or_single_fungible_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        if payment_amount > 0 {
            require!(
                payment_token == liquid_token_id,
                ERROR_BAD_PAYMENT_TOKEN
            );
        }
        payment_amount += undelegate_amount;
        require!(
            payment_amount > 0u64,
            ERROR_BAD_PAYMENT_AMOUNT,
        );

        let fee = self.undelegate_now_fee().get();
        let caller = self.blockchain().get_caller();
        let total_egld_staked = self.total_egld_staked().get();

        // arbitrage
        let (sold_amount, bought_amount) =
            self.do_arbitrage(&liquid_token_id, payment_amount.clone());
        if bought_amount > 0 {
            self.send().direct_egld(&caller, &bought_amount);
        }
        payment_amount -= sold_amount;
        if payment_amount == 0 {
            return
        };

        // normal unDelegateNow
        let egld_to_undelegate = self.remove_liquidity(&payment_amount, true);
        self.burn_liquid_token(&payment_amount);
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

        self.send().direct_egld(&receiver, &egld_to_undelegate_with_fee);
    }

    fn add_user_undelegation(
        &self,
        user: ManagedAddress,
        amount: BigUint,
        unbond_epoch: u64,
    ) {
        self.add_undelegation(amount.clone(), unbond_epoch, self.luser_undelegations(&user));
        self.add_undelegation(amount, unbond_epoch, self.ltotal_user_undelegations());
    }

    // endpoints: knights

    #[endpoint(unDelegateKnight)]
    fn undelegate_knight(
        &self,
        user: ManagedAddress,
        undelegate_amount: BigUint,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_knight(&user);

        self.do_undelegate(user, undelegate_amount);
    }

    #[endpoint(unDelegateNowKnight)]
    fn undelegate_now_knight(
        &self,
        user: ManagedAddress,
        min_amount_out: BigUint,
        undelegate_amount: BigUint,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_knight(&user);

        let knight = self.blockchain().get_caller();
        self.do_undelegate_now(user, knight, min_amount_out, undelegate_amount);
    }

    #[endpoint(withdrawKnight)]
    fn withdraw_knight(&self, user: ManagedAddress) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_knight(&user);

        let knight = self.blockchain().get_caller();
        self.do_withdraw(&user, &knight);
    }

    #[endpoint(removeReserveKnight)]
    fn remove_reserve_knight(
        &self,
        user: ManagedAddress,
        amount: BigUint,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_knight(&user);

        let knight = self.blockchain().get_caller();
        self.do_remove_reserve(user, knight, amount);
    }

    // endpoints: heirs

    #[endpoint(unDelegateHeir)]
    fn undelegate_heir(
        &self,
        user: ManagedAddress,
        undelegate_amount: BigUint,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_is_heir_entitled(&user);

        self.do_undelegate(user, undelegate_amount);
    }

    #[endpoint(unDelegateNowHeir)]
    fn undelegate_now_heir(
        &self,
        user: ManagedAddress,
        min_amount_out: BigUint,
        undelegate_amount: BigUint,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_is_heir_entitled(&user);

        let heir = self.blockchain().get_caller();
        self.do_undelegate_now(user, heir, min_amount_out, undelegate_amount);
    }

    #[endpoint(withdrawHeir)]
    fn withdraw_heir(&self, user: ManagedAddress) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_is_heir_entitled(&user);

        let heir = self.blockchain().get_caller();
        self.do_withdraw(&user, &heir);
    }

    #[endpoint(removeReserveHeir)]
    fn remove_reserve_heir(
        &self,
        user: ManagedAddress,
        amount: BigUint,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        self.check_is_heir_entitled(&user);

        let heir = self.blockchain().get_caller();
        self.do_remove_reserve(user, heir, amount);
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> proxies::delegation_proxy::Proxy<Self::Api>;
}
