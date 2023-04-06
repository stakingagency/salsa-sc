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

    // endpoints

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) -> EsdtTokenPayment<Self::Api> {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegate_amount = self.call_value().egld_value();
        require!(
            delegate_amount >= MIN_EGLD_TO_DELEGATE,
            ERROR_INSUFFICIENT_DELEGATE_AMOUNT
        );

        let caller = self.blockchain().get_caller();
        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(delegate_amount.clone())
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).delegate_callback(caller, delegate_amount),
            )
            .call_and_exit()
    }

    #[callback]
    fn delegate_callback(
        &self,
        caller: ManagedAddress,
        staked_tokens: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let user_payment = self.mint_liquid_token(staked_tokens);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                self.send().direct_egld(&caller, &staked_tokens);
            }
        }
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn undelegate(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let payment = self.call_value().single_esdt();
        let amount = payment.amount;
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let delegation_contract = self.provider_address().get();
        let this_contract = self.blockchain().get_sc_address();
        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call2();
        self.burn_liquid_token(&amount);

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .get_user_active_stake(this_contract)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).get_user_stake(caller, amount),
            )
            .call_and_exit()
    }

    #[callback]
    fn get_user_stake(
        &self,
        caller: ManagedAddress,
        amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(user_stake) => {
                let ls_supply = self.liquid_token_supply().get();
                let egld_amount = amount.clone() * user_stake / (ls_supply + amount.clone());

                let delegation_contract = self.provider_address().get();
                let gas_for_async_call = self.get_gas_for_async_call();

                self.delegation_proxy_obj()
                    .contract(delegation_contract)
                    .undelegate(egld_amount.clone())
                    .with_gas_limit(gas_for_async_call)
                    .async_call()
                    .with_callback(
                        SalsaContract::callbacks(self).undelegate_callback(caller, amount, egld_amount),
                    )
                    .call_and_exit()
            }
            ManagedAsyncCallResult::Err(_) => {
                let user_payment = self.mint_liquid_token(amount);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
        }
    }

    #[callback]
    fn undelegate_callback(
        &self,
        caller: ManagedAddress,
        amount: BigUint,
        egld_amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let current_epoch = self.blockchain().get_block_epoch();
                let unbond_epoch = current_epoch + UNBOND_PERIOD;
                let undelegation = config::Undelegation {
                    address: caller.clone(),
                    amount: egld_amount,
                    unbond_epoch,
                };
                self.user_undelegations(&caller)
                    .update(|undelegations| undelegations.push(undelegation));
            }
            ManagedAsyncCallResult::Err(_) => {
                let user_payment = self.mint_liquid_token(amount);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
        }
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .withdraw()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).withdraw_callback(caller),
            )
            .call_and_exit()
    }

    #[callback]
    fn withdraw_callback(&self, caller: ManagedAddress) {
        let current_epoch = self.blockchain().get_block_epoch();
        let user_undelegations = self.user_undelegations(&caller).get();
        let mut remaining_undelegations: ManagedVec<Self::Api, config::Undelegation<Self::Api>> =
            ManagedVec::new();
        let mut withdraw_amount = BigUint::zero();
        for user_undelegation in &user_undelegations {
            if user_undelegation.unbond_epoch <= current_epoch {
                withdraw_amount += user_undelegation.amount;
            } else {
                remaining_undelegations.push(user_undelegation);
            }
        }
        require!(withdraw_amount > 0, ERROR_NOTHING_TO_WITHDRAW);

        let sc_balance = self
            .blockchain().
            get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(sc_balance >= withdraw_amount, ERROR_NOT_ENOUGH_FUNDS);

        self.send().direct_egld(&caller, &withdraw_amount);
        self.user_undelegations(&caller)
            .set(remaining_undelegations);
    }

    #[endpoint(compound)]
    fn compound(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract.clone())
            .redelegate_rewards()
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .call_and_exit()
    }

    fn get_gas_for_async_call(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK,
            ERROR_INSUFFICIENT_GAS
        );

        gas_left - MIN_GAS_FOR_CALLBACK
    }

    fn get_gas_for_async_call2(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > 2 * (MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK),
            ERROR_INSUFFICIENT_GAS
        );

        gas_left - 2 * MIN_GAS_FOR_CALLBACK - MIN_GAS_FOR_ASYNC_CALL
    }

    fn mint_liquid_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        let supply = self.liquid_token_supply().get();
        self.liquid_token_supply().set(supply + amount.clone());
        self.liquid_token_id().mint(amount)
    }

    fn burn_liquid_token(&self, amount: &BigUint) {
        let supply = self.liquid_token_supply().get();
        self.liquid_token_supply().set(supply - amount.clone());
        self.liquid_token_id().burn(amount);
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
