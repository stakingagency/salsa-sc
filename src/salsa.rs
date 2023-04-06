#![no_std]

multiversx_sc::imports!();

pub mod config;
pub mod consts;
pub mod delegation_proxy;
pub mod errors;
pub mod storage;

use storage::Undelegation;

use crate::{config::*, consts::*, errors::*};

#[multiversx_sc::contract]
pub trait SalsaContract<ContractReader>:
    storage::StorageModule
    + config::ConfigModule
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
            .contract(delegation_contract.clone())
            .delegate()
            .with_gas_limit(gas_for_async_call)
            .with_egld_transfer(delegate_amount.clone())
            .async_call()
            .with_callback(
                SalsaContract::callbacks(self).delegate_callback(caller, delegate_amount),
            )
            .call_and_exit()
    }

    // Comment
    // It is not enough to just increase the ls_token_supply with the staked amount
    // You need to first see how much that staked_amount would mean comparing with the current value of the ls_token
    // After that, you compute mint and update the storages accordingly
    // I've added the add_liquidity function and included it in the delegate_callback
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

    // Comment
    // You should not chain 2 or more async calls with callbacks
    // The flow should be like this
    // User provides the ls_tokens, we remove the tokens from the supply and compute the unstake_egld
    // We call the undelegate function with the newly computed unstake_egld amount
    // In case of a succes we save the amount in the storage or send to the user an unstake token with his position
    // After the unbonding period, the user can claim his egld either with the unstake token, or from the saved position in the storage
    // We usually prefer sending a token to the user, to keep the storage of the SC as small as possible, but SC storage is OK also
    // In case of an error, we add the liquidity back in the storage and send the ls_token back to the user
    #[payable("*")]
    #[endpoint(unDelegate)]
    fn undelegate(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let ls_supply = self.liquid_token_supply().get();
        require!(ls_supply > 0, ERROR_NOT_ENOUGH_LIQUID_SUPPLY);

        let payment = self.call_value().single_esdt();
        let liquid_token_id = self.liquid_token_id().get_token_id();
        require!(
            payment.token_identifier == liquid_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );
        require!(payment.amount > 0, ERROR_BAD_PAYMENT_AMOUNT);

        let egld_to_unstake = self.remove_liquidity(&payment.amount);
        self.burn_liquid_token(&payment.amount);

        require!(
            egld_to_unstake >= MIN_EGLD_TO_DELEGATE,
            ERROR_INSUFFICIENT_DELEGATE_AMOUNT
        );

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

    // Comment
    // Not needed for the undelegate endpoint
    // Instead, it will be useful in the compound endpoint
    #[callback]
    fn get_user_stake(
        &self,
        caller: ManagedAddress,
        amount: BigUint,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(user_stake) => {
                // let ls_supply = self.liquid_token_supply().get();
                // let egld_amount = amount.clone() * user_stake / ls_supply;
                // self.burn_liquid_token(&amount);

                // let delegation_contract = self.provider_address().get();
                // let gas_for_async_call = self.get_gas_for_async_call();

                // self.delegation_proxy_obj()
                //     .contract(delegation_contract.clone())
                //     .undelegate(egld_amount.clone())
                //     .with_gas_limit(gas_for_async_call)
                //     .async_call()
                //     .with_callback(SalsaContract::callbacks(self).undelegate_callback(
                //         caller,
                //         egld_amount,
                //     ))
                //     .call_and_exit()
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[callback]
    fn undelegate_callback(
        &self,
        caller: ManagedAddress,
        egld_to_unstake: BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let current_epoch = self.blockchain().get_block_epoch();
                let unbond_epoch = current_epoch + UNBOND_PERIOD;

                let undelegation = Undelegation {
                    address: caller.clone(),
                    amount: egld_to_unstake,
                    unbond_epoch,
                };
                self.user_undelegations(&caller)
                    .update(|undelegations| undelegations.push(undelegation));
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

    // Comment
    // When you undelegate from a staking provider, you receive the entire amount of EGLD, not for one user only
    // So, in the withdraw callback, we check how much tokens were received (and send EGLD to the users from here)
    // The flow would be:
    // Alice undelegates 10 egld (epoch 10), Bob 5 Egld (epoch 10), Josh 10 egld (epoch 11), Mike 5 egld (epoch 11)
    // At epoch 10, Alice withdraws the egld -> the validator will return 15 egld (withdrawn_egld = 15)
    // Alice will receive 10 egld, withdrawn_egld will remain 5
    // Epoch 11, Mike calls the withdraw endpoint -> as we already have 5 egld left, we give the egld to Mike directly, without calling withdraw on the provider
    // Then Josh will call the withdraw endpoint. As withdrawn_egld < 10 (0 at this moment), we withdraw from the validator again
    // withdrawn_egld = 15 again, Josh will receive his egld -> withdrawn_egld = 5
    // Bob comes and withdraws his share, we already have the amount available. He receives the EGLD and withdrawn_egld = 0
    #[endpoint(withdraw)]
    fn withdraw(&self) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);

        let current_epoch = self.blockchain().get_block_epoch();
        let delegation_contract = self.provider_address().get();
        let caller = self.blockchain().get_caller();
        let gas_for_async_call = self.get_gas_for_async_call();

        let user_undelegations = self.user_undelegations(&caller).get();
        let mut remaining_undelegations: ManagedVec<Self::Api, storage::Undelegation<Self::Api>> =
            ManagedVec::new();
        let mut withdraw_amount = BigUint::zero();
        for user_undelegation in &user_undelegations {
            if user_undelegation.unbond_epoch <= current_epoch {
                withdraw_amount += user_undelegation.amount;
            } else {
                remaining_undelegations.push(user_undelegation);
            }
        }
        self.user_undelegations(&caller)
            .set(remaining_undelegations);
        let total_withdrawn_amount = self.total_withdrawn_amount().get();
        if withdraw_amount <= total_withdrawn_amount {
            self.send().direct_egld(&caller, &withdraw_amount);
            self.total_withdrawn_amount()
                .update(|value| *value -= withdraw_amount);
        } else {
            // Comment
            // Here we call the withdraw from the staking provider
            // In the callback we increase again the total_withdrawn_amount, while sending the egld to the user
            self.delegation_proxy_obj()
                .contract(delegation_contract.clone())
                .withdraw()
                .with_gas_limit(gas_for_async_call)
                .async_call()
                .with_callback(SalsaContract::callbacks(self).withdraw_callback(caller))
                .call_and_exit()
        }
    }

    // Comment
    // I would strongly suggest to use the user_undelegations proposed approach
    // That way, you don't need to go through the entire vector, for all the users
    // The suggested implementation can be found in the withdraw endpoint
    #[callback]
    fn withdraw_callback(&self, caller: ManagedAddress) {
        let current_epoch = self.blockchain().get_block_epoch();
        let mut withdraw_amount = BigUint::from(0_u64);
        let n = self.undelegated().len();
        for i in (1..n).rev() {
            let undelegation = self.undelegated().get(i);
            if (undelegation.address == caller) && (undelegation.unbond_epoch <= current_epoch) {
                withdraw_amount = withdraw_amount + undelegation.amount;
            }
        }
        require!(withdraw_amount > 0, ERROR_NOTHING_TO_WITHDRAW);

        let sc_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(sc_balance >= withdraw_amount, ERROR_NOT_ENOUGH_FUNDS);

        self.send().direct_egld(&caller, &withdraw_amount);
        for i in (1..n).rev() {
            let undelegation = self.undelegated().get(i);
            if (undelegation.address == caller) && (undelegation.unbond_epoch <= current_epoch) {
                self.undelegated().swap_remove(i);
            }
        }
    }

    // Comment
    // Here the implementation changes a bit from the other liquid staking implementation
    // Because we do not claim * delegate the rewards, but instead we just call the redelegate_rewards endpoint, we need to make a few changes
    // Our objective here is to always have the total_egld_staked variable up-to-date
    // So to flow would be:
    // 1. We call redelegate_rewards on the delegation_contract
    // 2. After the redelegation is successful, we call getUserActiveStake to get the new total_egld_staked (and we update it)
    // A few observations:
    // I would first add a check to see that the amount from getUserActiveStake > total_egld_staked
    // Also, we should be sure that the withdraw logic would not interfere with this process in any way possible
    // Finally, I'm not sure if chaining these 2 operations would be a good idea, or even if this would work
    // So I would recommend to split the redelegate_rewards() and update_total_egld_staked() in 2 different endpoints
    // That way, the update_total_egld_staked logic could be called multiple times, if needed (for safety purposes)
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

    #[only_owner]
    #[endpoint(setProviderAddress)]
    fn set_provider_address(&self, address: ManagedAddress) {
        require!(!self.is_state_active(), ERROR_ACTIVE);

        self.provider_address().set(address);
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

        let egld_amount = ls_token_amount * &total_egld_staked / ls_token_amount;
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
