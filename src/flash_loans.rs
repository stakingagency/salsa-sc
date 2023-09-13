multiversx_sc::imports!();

use crate::{common::{errors::*, consts::*, storage_cache::StorageCache, config::State}, exchanges::lp_cache::LpCache};

#[multiversx_sc::module]
pub trait FlashLoansModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + crate::exchanges::arbitrage::ArbitrageModule
    + crate::exchanges::onedex::OnedexModule
    + crate::exchanges::xexchange::XexchangeModule
    + crate::exchanges::lp::LpModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[endpoint(setFlashLoansActive)]
    fn set_flash_loans_active(&self) {
        self.flash_loans().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setFlashLoansInactive)]
    fn set_flash_loans_inactive(&self) {
        self.flash_loans().set(State::Inactive);
    }

    #[inline]
    fn are_flash_loans_active(&self) -> bool {
        let flash_loans = self.flash_loans().get();
        flash_loans == State::Active
    }

    #[view(getFlashLoansState)]
    #[storage_mapper("flash_loans")]
    fn flash_loans(&self) -> SingleValueMapper<State>;

    /**
     * Flash loan LEGLD
     */
    #[endpoint(flashLoanLEGLD)]
    fn flash_loan_legld(
        &self,
        amount: BigUint,
        address: ManagedAddress,
        function: ManagedBuffer,
        args: MultiValueManagedVec<ManagedBuffer>,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);
        require!(self.are_flash_loans_active(), ERROR_FLASH_LOANS_NOT_ACTIVE);
        require!(amount <= BigUint::from(MAX_LOAN), ERROR_INSUFFICIENT_FUNDS);

        self.mint_liquid_token(amount.clone());
        let (_, old_ls_balance) = self.get_sc_balances();
        let _ = self.send_raw().transfer_esdt_execute(
            &address,
            &self.liquid_token_id().get_token_id(),
            &amount,
            self.blockchain().get_gas_left(),
            &function,
            &ManagedArgBuffer::from(args.into_vec()),
        );
        let (_, new_ls_balance) = self.get_sc_balances();
        require!(new_ls_balance > old_ls_balance, ERROR_LOAN_NOT_RETURNED);

        let profit = new_ls_balance - old_ls_balance;
        let fee = &profit * FLASH_LOAN_FEE / MAX_PERCENT;
        self.burn_liquid_token(&(&amount + &fee));
        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &self.liquid_token_id().get_token_id(),
            0,
            &(profit - fee),
        );
    }

    /**
     * Flash loan eGLD
     */
    #[endpoint(flashLoanEGLD)]
    fn flash_loan_egld(
        &self,
        amount: BigUint,
        address: ManagedAddress,
        function: ManagedBuffer,
        args: MultiValueManagedVec<ManagedBuffer>,
    ) {
        require!(self.is_state_active(), ERROR_NOT_ACTIVE);
        require!(self.are_flash_loans_active(), ERROR_FLASH_LOANS_NOT_ACTIVE);
        require!(amount <= BigUint::from(MAX_LOAN), ERROR_INSUFFICIENT_FUNDS);

        let mut storage_cache = StorageCache::new(self);
        let mut lp_cache = LpCache::new(self);
        let (mut old_balance, _) = self.get_sc_balances();
        if old_balance < amount {
            let egld_to_remove = &amount - &old_balance;
            require!(egld_to_remove <= lp_cache.egld_in_lp, ERROR_INSUFFICIENT_FUNDS);

            self.remove_egld_lp(egld_to_remove, &mut storage_cache, &mut lp_cache);
            (old_balance, _) = self.get_sc_balances();
        }
        require!(amount <= old_balance, ERROR_INSUFFICIENT_FUNDS);

        let _ = self.send_raw().direct_egld_execute(
            &address,
            &amount,
            self.blockchain().get_gas_left(),
            &function,
            &ManagedArgBuffer::from(args.into_vec()),
        );
        let (new_balance, _) = self.get_sc_balances();
        require!(new_balance > old_balance, ERROR_LOAN_NOT_RETURNED);

        let profit = new_balance - old_balance;
        let fee = &profit * FLASH_LOAN_FEE / MAX_PERCENT;
        storage_cache.egld_to_delegate += &fee;
        storage_cache.total_stake += &fee;
        self.send().direct_egld(
            &self.blockchain().get_caller(),
            &(profit - fee),
        );
        self.add_lp(&mut storage_cache, &mut lp_cache);
    }
}
