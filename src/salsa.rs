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
                let ls_amount = self.add_liquidity(&staked_tokens);
                let user_payment = self.mint_liquid_token(ls_amount);
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
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_to_unstake = self.remove_liquidity(&payment.amount);
        self.burn_liquid_token(&payment.amount);
        let delegation_contract = self.provider_address().get();
        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(egld_to_unstake.clone())
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).undelegate_callback(caller, egld_to_unstake),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_callback(
        &self,
        caller: ManagedAddress,
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
                let ls_token_amount = self.add_liquidity(&egld_amount);
                let user_payment = self.mint_liquid_token(ls_token_amount);
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

        let egld_reserve = self.egld_reserve().get();
        let sc_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(sc_balance - egld_reserve >= withdraw_amount, ERROR_NOT_ENOUGH_FUNDS);

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
            .contract(delegation_contract)
            .redelegate_rewards()
            .with_gas_limit(gas_for_async_call)
            .transfer_execute();
    }

    #[endpoint(updateTotalEgldStaked)]
    fn update_total_egld_staked(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let delegation_contract = self.provider_address().get();
        let this_contract = self.blockchain().get_sc_address();
        let gas_for_async_call = self.get_gas_for_async_call();

        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .get_user_active_stake(this_contract)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).update_egld_staked_callback(),
            )
            .call_and_exit()
    }

    #[callback]
    fn update_egld_staked_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(total_stake) => {
                let total_egld_staked = self.total_egld_staked().get();
                require!(
                    total_stake > total_egld_staked,
                    ERROR_NOT_ENOUGH_FUNDS
                );

                self.total_egld_staked()
                    .update(|value| *value = total_stake);
            }
            ManagedAsyncCallResult::Err(_) => {
            }
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
            .transfer_execute();
    }

    #[payable("EGLD")]
    #[endpoint(addReserve)]
    fn add_reserve(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        let reserve_amount = self.call_value().egld_value();

        self.user_reserves().update(|user_reserves| {
            let mut user_found = false;
            for mut user_reserve in user_reserves.into_iter() {
                if user_reserve.address == caller {
                    user_reserve.amount += &reserve_amount;
                    user_found = true;
                    break;
                }
            }

            if !user_found {
                let new_reserve = config::Reserve {
                    address: caller.clone(),
                    amount: reserve_amount.clone(),
                };
                user_reserves.push(new_reserve);
            }
        });

        self.egld_reserve()
            .update(|value| *value += reserve_amount);
    }

    #[endpoint(removeReserve)]
    fn remove_reserve(&self, amount: BigUint) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();

        let sc_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(sc_balance >= amount, ERROR_NOT_ENOUGH_FUNDS);

        let mut user_found = false;
        self.backup_user_reserves().clear();
        self.user_reserves().update(|user_reserves| {
            for mut user_reserve in user_reserves.into_iter() {
                if user_reserve.address == caller {
                    require!(user_reserve.amount >= amount, ERROR_NOT_ENOUGH_FUNDS);

                    user_reserve.amount -= &amount;
                    if user_reserve.amount > 0 {
                        self.backup_user_reserves()
                            .update(|backup_reserves| backup_reserves.push(user_reserve));
                    }
                    user_found = true;
                    break;
                }
            }
        });
        require!(user_found, ERROR_NOT_ENOUGH_FUNDS);

        let reserves = self.backup_user_reserves().take();
        self.user_reserves().set(reserves);
        self.send().direct_egld(&caller, &amount);
        self.egld_reserve()
            .update(|value| *value -= &amount);
    }

    #[payable("*")]
    #[endpoint(unDelegateNow)]
    fn undelegate_now(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        let egld_reserve = self.egld_reserve().get();
        let total_egld_staked = self.total_egld_staked().get();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        let fee = self.undelegate_now_fee().get();
        let caller = self.blockchain().get_caller();
        let delegation_contract = self.provider_address().get();
        let gas_for_async_call = self.get_gas_for_async_call();
        let egld_to_unstake = self.remove_liquidity(&payment.amount);
        let egld_to_unstake_with_fee = egld_to_unstake.clone() - egld_to_unstake.clone() * fee / 10000u32;
        let sc_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(sc_balance >= egld_to_unstake_with_fee, ERROR_NOT_ENOUGH_FUNDS);
        require!(egld_to_unstake_with_fee <= egld_reserve, ERROR_NOT_ENOUGH_FUNDS);
        require!(egld_to_unstake <= total_egld_staked, ERROR_NOT_ENOUGH_FUNDS);

        let remaining = &egld_to_unstake - &egld_to_unstake_with_fee;
        self.backup_user_reserves().clear();
        let original_user_reserves = self.user_reserves().get();
        self.backup_user_reserves().update(|user_reserves| {
            for original_user_reserve in original_user_reserves.into_iter() {
                let new_reserve = config::Reserve {
                    address: original_user_reserve.address,
                    amount: original_user_reserve.amount.clone() + &original_user_reserve.amount * &remaining / &egld_reserve,
                };
                user_reserves.push(new_reserve);
            }
        });

        self.burn_liquid_token(&payment.amount);
        self.delegation_proxy_obj()
            .contract(delegation_contract)
            .undelegate(&egld_to_unstake)
            .with_gas_limit(gas_for_async_call)
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).undelegate_now_callback(caller, egld_to_unstake),
            )
            .call_and_exit()
    }

    #[callback]
    fn undelegate_now_callback(
        &self,
        caller: ManagedAddress,
        egld_to_unstake: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let fee = self.undelegate_now_fee().get();
                let egld_to_unstake_with_fee = egld_to_unstake.clone() - egld_to_unstake.clone() * fee / 10000u32;
                self.send().direct_egld(&caller, &egld_to_unstake_with_fee);
                let remaining = &egld_to_unstake - &egld_to_unstake_with_fee;
                self.egld_reserve()
                    .update(|value| *value += remaining);
                let reserves = self.backup_user_reserves().take();
                self.user_reserves().set(reserves);
            }
            ManagedAsyncCallResult::Err(_) => {
                let ls_token_amount = self.add_liquidity(&egld_to_unstake);
                let user_payment = self.mint_liquid_token(ls_token_amount);
                self.send().direct_esdt(
                    &caller,
                    &user_payment.token_identifier,
                    user_payment.token_nonce,
                    &user_payment.amount,
                );
            }
        }
    }

    fn get_gas_for_async_call(&self) -> u64 {
        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > MIN_GAS_FOR_ASYNC_CALL + MIN_GAS_FOR_CALLBACK,
            ERROR_INSUFFICIENT_GAS
        );

        gas_left - MIN_GAS_FOR_CALLBACK
    }

    fn add_liquidity(&self, new_stake_amount: &BigUint) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        let ls_amount = if total_egld_staked > 0 {
            new_stake_amount * &liquid_token_supply / &total_egld_staked
        } else {
            new_stake_amount.clone()
        };

        require!(ls_amount > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

        self.total_egld_staked()
            .update(|value| *value += new_stake_amount);
        self.liquid_token_supply()
            .update(|value| *value += &ls_amount);

        ls_amount
    }

    fn remove_liquidity(&self, ls_amount: &BigUint) -> BigUint {
        let egld_amount = self.get_egld_amount(ls_amount);
        self.total_egld_staked()
            .update(|value| *value -= &egld_amount);
        self.liquid_token_supply()
            .update(|value| *value -= ls_amount);

        egld_amount
    }

    fn get_egld_amount(&self, ls_token_amount: &BigUint) -> BigUint {
        let total_egld_staked = self.total_egld_staked().get();
        let liquid_token_supply = self.liquid_token_supply().get();
        require!(
            &liquid_token_supply >= ls_token_amount,
            ERROR_NOT_ENOUGH_LIQUID_SUPPLY
        );
        require!(ls_token_amount > &0, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_amount = ls_token_amount * &total_egld_staked / &liquid_token_supply;
        require!(egld_amount > 0u64, ERROR_BAD_PAYMENT_AMOUNT);

        egld_amount
    }

    fn mint_liquid_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.liquid_token_id().mint(amount)
    }

    fn burn_liquid_token(&self, amount: &BigUint) {
        self.liquid_token_id().burn(amount);
    }

    // proxy

    #[proxy]
    fn delegation_proxy_obj(&self) -> delegation_proxy::Proxy<Self::Api>;
}
