multiversx_sc::imports!();

use crate::common::consts::MAX_KNIGHT_USERS;
use crate::{common::errors::*};
use crate::common::config::{Knight, KnightState};

#[multiversx_sc::module]
pub trait KnightsModule:
    crate::common::config::ConfigModule
    + crate::helpers::HelpersModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    // Comment
    // Small observation for FE, knights can only have 9 users
    // If you want 10 -> knight_users.len() <= MAX_KNIGHT_USERS
    #[endpoint(setKnight)]
    fn set_knight(&self, knight: ManagedAddress) {
        self.check_is_delegator();

        let caller = self.blockchain().get_caller();
        require!(
            self.user_knight(&caller).is_empty(),
            ERROR_KNIGHT_ALREADY_SET,
        );
        require!(caller != knight, ERROR_KNIGHT_YOURSELF);

        let new_knight = Knight{
            address: knight.clone(),
            state: KnightState::PendingConfirmation,
        };
        self.user_knight(&caller).set(new_knight);
        let mut knight_users = self.knight_users(&knight);
        require!(knight_users.len() < MAX_KNIGHT_USERS, ERROR_TOO_MANY_KNIGHT_USERS);

        knight_users.insert(caller);
    }

    // Comment
    // I would remove check_is_delegator verifications (it should be only in setKnight and activateKnight)
    // Maybe the user was a delegator but then it removed all custodial funds until the knight was activated
    // He should be able to remove the knight even if there are no custodial funds left
    #[endpoint(cancelKnight)]
    fn cancel_knight(&self) {
        let caller = self.blockchain().get_caller();
        self.check_is_delegator();
        self.check_user_has_knight(caller.clone());
        self.check_is_knight_pending(caller.clone());

        let knight = self.user_knight(&caller).get();
        self.user_knight(&caller).clear();
        self.knight_users(&knight.address).swap_remove(&caller);
    }

    // Comment
    // Move the require inside the update
    #[endpoint(activateKnight)]
    fn activate_knight(&self) {
        let caller = self.blockchain().get_caller();
        self.check_is_delegator();
        self.check_user_has_knight(caller.clone());
        require!(
            self.user_knight(&caller).get().state == KnightState::Inactive,
            ERROR_KNIGHT_NOT_CONFIRMED,
        );

        self.user_knight(&caller)
            .update(|knight| knight.state = KnightState::Active);
    }

    // knight actions

    #[endpoint(deactivateKnight)]
    fn deactivate_knight(&self, user: ManagedAddress) {
        self.check_user_has_knight(user.clone());
        self.check_is_knight_for_user(user.clone());
        self.check_is_knight_active(user.clone());

        self.user_knight(&user)
            .update(|knight| knight.state = KnightState::Inactive);
    }

    // Comment
    // I would change this function a bit, even if it's for a edge case
    // The update to self.knight_users(&knight) should happen here
    // Otherwise, 10 other users may set the knight to their accounts and then lock the knight for other accounts without his confirmation
    // I think a knight should be in knight_users storage only in Active or Inactive states
    // This means removing the knight_users storage update from setKnight endpoint
    // knight_users swap_remove can still be called even if there is no entry in storage
    #[endpoint(confirmKnight)]
    fn confirm_knight(&self, user: ManagedAddress) {
        self.check_user_has_knight(user.clone());
        self.check_is_knight_for_user(user.clone());
        self.check_is_knight_pending(user.clone());

        self.user_knight(&user)
            .update(|knight| knight.state = KnightState::Inactive);
    }

    #[endpoint(removeKnight)]
    fn remove_knight(&self, user: ManagedAddress) {
        self.check_user_has_knight(user.clone());
        self.check_is_knight_for_user(user.clone());

        self.user_knight(&user).clear();
        let knight = self.blockchain().get_caller();
        self.knight_users(&knight).swap_remove(&user);
    }

    // helpers

    // Comment
    // I would change the naming of the function to be more explicit
    // check_is_custodial_delegator
    fn check_is_delegator(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.user_delegation(&caller).get() > 0,
            ERROR_USER_NOT_DELEGATOR,
        );
    }

    // Comment
    // Send reference parameter
    fn check_user_has_knight(&self, user: ManagedAddress) {
        require!(
            !self.user_knight(&user).is_empty(),
            ERROR_KNIGHT_NOT_SET,
        );
    }

    // Comment
    // Send reference parameter
    fn check_is_knight_for_user(&self, user: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.user_knight(&user).get().address,
            ERROR_NOT_KNIGHT_OF_USER,
        );
    }

    // Comment
    // Send reference parameter
    fn check_is_knight_active(&self, user: ManagedAddress) {
        require!(
            self.user_knight(&user).get().state == KnightState::Active,
            ERROR_KNIGHT_NOT_ACTIVE,
        );
    }

    // Comment
    // Send reference parameter
    fn check_is_knight_pending(&self, user: ManagedAddress) {
        require!(
            self.user_knight(&user).get().state == KnightState::PendingConfirmation,
            ERROR_KNIGHT_NOT_PENDING,
        );
    }
}
