multiversx_sc::imports!();

use crate::common::consts::MAX_HEIR_USERS;
use crate::{common::errors::*, common::consts::MIN_INHERITANCE_EPOCHS};
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
            self.user_delegation(&caller).get() > 0,
            ERROR_USER_NOT_DELEGATOR,
        );
        require!(
            inheritance_epochs >= MIN_INHERITANCE_EPOCHS,
            ERROR_LOW_INHERITANCE_EPOCHS,
        );
        require!(
            caller != heir,
            ERROR_INHERIT_YOURSELF,
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

    #[endpoint(removeHeir)]
    fn remove_heir(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.user_has_heir(caller.clone()),
            ERROR_NO_HEIR,
        );

        let heir = self.user_heir(&caller).get();
        self.user_heir(&caller).clear();
        self.heir_users(&heir.address).swap_remove(&caller);
    }

    fn user_has_heir(&self, user: ManagedAddress) -> bool {
        !self.user_heir(&user).is_empty()
    }

    fn update_last_accessed(&self) {
        let caller = self.blockchain().get_caller();
        if !self.user_has_heir(caller.clone()) {
            return
        }

        let current_epoch = self.blockchain().get_block_epoch();
        self.user_heir(&caller)
            .update(|heir| heir.last_accessed_epoch = current_epoch);
    }

    fn check_is_heir_entitled(&self, owner: ManagedAddress) {
        require!(
            self.user_has_heir(owner.clone()),
            ERROR_NO_HEIR,
        );

        let caller = self.blockchain().get_caller();
        let heir = self.user_heir(&owner).get();
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
