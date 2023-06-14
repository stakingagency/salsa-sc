use salsa::SalsaContract;
use salsa::knights::KnightsModule;

use crate::{contract_setup::SalsaContractSetup, to_managed_biguint};

use multiversx_sc_scenario::{
    num_bigint, rust_biguint, managed_address, DebugApi,
};

use multiversx_sc::{
    types::{
        Address,
    },
};

impl<SalsaContractObjBuilder> SalsaContractSetup<SalsaContractObjBuilder>
where
    SalsaContractObjBuilder: 'static + Copy + Fn() -> salsa::ContractObj<DebugApi>,
{
    pub fn set_knight_test(
        &mut self,
        sender: &Address,
        knight: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.set_knight(managed_address!(knight))
            })
            .assert_ok();
    }

    pub fn set_knight_fail_test(
        &mut self,
        sender: &Address,
        knight: &Address,
        error: &str,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.set_knight(managed_address!(knight))
            })
            .assert_user_error(error);
    }

    pub fn cancel_knight_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.cancel_knight()
            })
            .assert_ok();
    }

    pub fn cancel_knight_fail_test(
        &mut self,
        sender: &Address,
        error: &str,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.cancel_knight()
            })
            .assert_user_error(error);
    }

    pub fn activate_knight_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.activate_knight()
            })
            .assert_ok();
    }

    pub fn deactivate_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.deactivate_knight(managed_address!(user))
            })
            .assert_ok();
    }

    pub fn confirm_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.confirm_knight(managed_address!(user))
            })
            .assert_ok();
    }

    pub fn remove_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.remove_knight(managed_address!(user))
            })
            .assert_ok();
    }

    pub fn undelegate_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_knight(managed_address!(user), to_managed_biguint(amount))
            })
            .assert_ok();
    }

    pub fn undelegate_knight_fail_test(
        &mut self,
        knight: &Address,
        user: &Address,
        amount: num_bigint::BigUint,
        error: &str,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_knight(managed_address!(user), to_managed_biguint(amount))
            })
            .assert_user_error(error);
    }

    pub fn undelegate_now_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
        min_amount: num_bigint::BigUint,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_now_knight(managed_address!(user), to_managed_biguint(min_amount), to_managed_biguint(amount))
            })
            .assert_ok();
    }

    pub fn withdraw_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.withdraw_knight(managed_address!(user))
            })
            .assert_ok();
    }

    pub fn remove_reserve_knight_test(
        &mut self,
        knight: &Address,
        user: &Address,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(knight, &self.salsa_wrapper, &big_zero, |sc| {
                sc.remove_reserve_knight(managed_address!(user), to_managed_biguint(amount))
            })
            .assert_ok();
    }
}
