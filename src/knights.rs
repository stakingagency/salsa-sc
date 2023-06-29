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
    #[endpoint(setKnight)]
    fn set_knight(&self, knight: ManagedAddress) {
        self.check_is_custodial_delegator();

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

    #[endpoint(cancelKnight)]
    fn cancel_knight(&self) {
        let caller = self.blockchain().get_caller();
        self.check_user_has_knight(&caller);
        self.check_is_knight_pending(&caller);

        let knight = self.user_knight(&caller).get();
        self.user_knight(&caller).clear();
        self.knight_users(&knight.address).swap_remove(&caller);
    }

    #[endpoint(activateKnight)]
    fn activate_knight(&self) {
        let caller = self.blockchain().get_caller();
        self.check_is_custodial_delegator();
        self.check_user_has_knight(&caller);
        require!(
            self.user_knight(&caller).get().state == KnightState::InactiveKnight,
            ERROR_KNIGHT_NOT_CONFIRMED,
        );

        self.user_knight(&caller)
            .update(|knight| knight.state = KnightState::ActiveKnight);
    }

    // knight actions

    #[endpoint(deactivateKnight)]
    fn deactivate_knight(&self, user: ManagedAddress) {
        self.check_user_has_knight(&user);
        self.check_is_knight_for_user(&user);
        self.check_is_knight_active(&user);

        self.user_knight(&user)
            .update(|knight| knight.state = KnightState::InactiveKnight);
    }

    #[endpoint(confirmKnight)]
    fn confirm_knight(&self, user: ManagedAddress) {
        self.check_user_has_knight(&user);
        self.check_is_knight_for_user(&user);
        self.check_is_knight_pending(&user);

        self.user_knight(&user)
            .update(|knight| knight.state = KnightState::InactiveKnight);
    }

    #[endpoint(removeKnight)]
    fn remove_knight(&self, user: ManagedAddress) {
        self.check_user_has_knight(&user);
        self.check_is_knight_for_user(&user);

        self.user_knight(&user).clear();
        let knight = self.blockchain().get_caller();
        self.knight_users(&knight).swap_remove(&user);
    }

    // helpers

    fn check_knight(&self, user: &ManagedAddress) {
        self.check_user_has_knight(&user);
        self.check_is_knight_for_user(&user);
        self.check_is_knight_active(&user);
    }

    fn check_knight_activated(&self, caller: &ManagedAddress) {
        let knight = self.user_knight(&caller);
        if !knight.is_empty() {
            require!(
                knight.get().state != KnightState::ActiveKnight,
                ERROR_KNIGHT_ACTIVE,
            );
        }
    }

    fn check_knight_set(&self, caller: &ManagedAddress) {
        let knight = self.user_knight(&caller);
        require!(knight.is_empty(), ERROR_KNIGHT_SET);
    }

    fn check_is_custodial_delegator(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.user_delegation(&caller).get() > 0,
            ERROR_USER_NOT_DELEGATOR,
        );
    }

    fn check_user_has_knight(&self, user: &ManagedAddress) {
        require!(
            !self.user_knight(user).is_empty(),
            ERROR_KNIGHT_NOT_SET,
        );
    }

    fn check_is_knight_for_user(&self, user: &ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.user_knight(user).get().address,
            ERROR_NOT_KNIGHT_OF_USER,
        );
    }

    fn check_is_knight_active(&self, user: &ManagedAddress) {
        require!(
            self.user_knight(user).get().state == KnightState::ActiveKnight,
            ERROR_KNIGHT_NOT_ACTIVE,
        );
    }

    fn check_is_knight_pending(&self, user: &ManagedAddress) {
        require!(
            self.user_knight(user).get().state == KnightState::PendingConfirmation,
            ERROR_KNIGHT_NOT_PENDING,
        );
    }
}
