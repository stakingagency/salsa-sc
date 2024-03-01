multiversx_sc::imports!();

use crate::common::config::KnightState;
use crate::common::consts::*;
use crate::common::errors::*;
use crate::common::config::Heir;

#[multiversx_sc::module]
pub trait HeirsModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(setHeir)]
    fn set_heir(
        &self,
        heir: ManagedAddress,
        inheritance_epochs: u64,
    ) {
        let caller = self.blockchain().get_caller();
        require!(
            (MIN_INHERITANCE_EPOCHS..=MAX_INHERITANCE_EPOCHS).contains(&inheritance_epochs),
            ERROR_WRONG_INHERITANCE_EPOCHS,
        );
        require!(
            caller != heir,
            ERROR_INHERIT_YOURSELF,
        );
        require!(
            self.user_heir(&caller).is_empty(),
            ERROR_HEIR_ALREADY_SET,
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let new_heir = Heir{
            address: heir.clone(),
            inheritance_epochs,
            last_accessed_epoch: current_epoch,
        };
        self.user_heir(&caller).set(new_heir);

        let mut heir_users = self.heir_users(&heir);
        require!(heir_users.len() < MAX_HEIR_USERS, ERROR_TOO_MANY_HEIR_USERS);

        heir_users.insert(caller);
    }

    #[endpoint(cancelHeir)]
    fn cancel_heir(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.user_has_heir(&caller),
            ERROR_NO_HEIR,
        );

        let knight = self.user_knight(&caller);
        if !knight.is_empty() {
            require!(
                knight.get().state == KnightState::PendingConfirmation,
                ERROR_CANCEL_HEIR_WHILE_KNIGHT_SET,
            );
        }

        let heir = self.user_heir(&caller).get();
        self.user_heir(&caller).clear();
        self.heir_users(&heir.address).swap_remove(&caller);
    }

    // heir actions

    #[endpoint(removeHeir)]
    fn remove_heir(&self, user: ManagedAddress) {
        require!(
            self.user_has_heir(&user),
            ERROR_NO_HEIR,
        );

        let user_heir = self.user_heir(&user).get();
        let caller = self.blockchain().get_caller();
        require!(
            caller == user_heir.address,
            ERROR_NOT_HEIR_OF_USER,
        );

        self.user_heir(&user).clear();
        self.heir_users(&caller).swap_remove(&user);
    }

    fn user_has_heir(&self, user: &ManagedAddress) -> bool {
        !self.user_heir(user).is_empty()
    }

    #[endpoint(updateLastAccessed)]
    fn update_last_accessed(&self) {
        let caller = self.blockchain().get_caller();
        if !self.user_has_heir(&caller) {
            return
        }

        let knight = self.user_knight(&caller);
        if !knight.is_empty() && knight.get().state == KnightState::ActiveKnight {
            return
        }

        let current_epoch = self.blockchain().get_block_epoch();
        self.user_heir(&caller)
            .update(|heir| heir.last_accessed_epoch = current_epoch);
    }

    fn check_is_heir_entitled(&self, owner: &ManagedAddress) {
        require!(
            self.user_has_heir(owner),
            ERROR_NO_HEIR,
        );

        let caller = self.blockchain().get_caller();
        let heir = self.user_heir(owner).get();
        require!(
            caller == heir.address,
            ERROR_NOT_HEIR_OF_USER,
        );

        let current_epoch = self.blockchain().get_block_epoch();
        require!(
            current_epoch >= heir.last_accessed_epoch + heir.inheritance_epochs,
            ERROR_HEIR_NOT_YET_ENTITLED,
        );
    }
}
