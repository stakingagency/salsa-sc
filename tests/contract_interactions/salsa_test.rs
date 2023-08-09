use salsa::SalsaContract;
use salsa::service::ServiceModule;

use crate::consts::*;
use crate::{contract_setup::SalsaContractSetup, to_managed_biguint};

use multiversx_sc_scenario::{
    num_bigint, rust_biguint, DebugApi,
};

use multiversx_sc::{
    types::Address,
    codec::multi_types::OptionalValue
};

impl<SalsaContractObjBuilder> SalsaContractSetup<SalsaContractObjBuilder>
where
    SalsaContractObjBuilder: 'static + Copy + Fn() -> salsa::ContractObj<DebugApi>,
{
    pub fn delegate_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
        custodial: bool,
    ) {
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &amount, |sc| {
                sc.delegate(OptionalValue::Some(custodial), OptionalValue::Some(false));
            })
            .assert_ok();
    }

    pub fn undelegate_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
        undelegate_amount: num_bigint::BigUint, // custodial
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.undelegate(OptionalValue::Some(to_managed_biguint(undelegate_amount)), OptionalValue::Some(false))
            })
            .assert_ok();
    }

    pub fn undelegate_fail_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
        undelegate_amount: num_bigint::BigUint, // custodial
        error: &str,
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.undelegate(OptionalValue::Some(to_managed_biguint(undelegate_amount)), OptionalValue::Some(false))
            })
            .assert_user_error(error);
    }

    pub fn add_to_custody_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.add_to_custody()
            })
            .assert_ok();
    }

    pub fn remove_from_custody_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &big_zero, |sc| {
                sc.remove_from_custody(to_managed_biguint(amount))
            })
            .assert_ok();
    }

    pub fn remove_from_custody_fail_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
        error: &str,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &big_zero, |sc| {
                sc.remove_from_custody(to_managed_biguint(amount))
            })
            .assert_user_error(error);
    }

    pub fn withdraw_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &big_zero, |sc| {
                sc.withdraw()
            })
            .assert_ok();
    }

    pub fn add_reserve_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
    ) {
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &amount, |sc| {
                sc.add_reserve();
            })
            .assert_ok();
    }

    pub fn remove_reserve_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.remove_reserve(to_managed_biguint(amount))
            })
            .assert_ok();
    }

    pub fn undelegate_now_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
        min_amount: num_bigint::BigUint,
        undelegate_amount: num_bigint::BigUint, // custodial
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.undelegate_now(to_managed_biguint(min_amount), OptionalValue::Some(to_managed_biguint(undelegate_amount)), OptionalValue::Some(false))
            })
            .assert_ok();
    }

    pub fn undelegate_now_fail_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
        min_amount: num_bigint::BigUint,
        undelegate_amount: num_bigint::BigUint, // custodial
        error: &str,
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.undelegate_now(to_managed_biguint(min_amount), OptionalValue::Some(to_managed_biguint(undelegate_amount)), OptionalValue::Some(false))
            })
            .assert_user_error(error);
    }

    pub fn undelegate_all_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_all()
            })
            .assert_ok();
    }

    pub fn withdraw_all_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.withdraw_all()
            })
            .assert_ok();
    }

    pub fn compute_withdrawn_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.compute_withdrawn()
            })
            .assert_ok();
    }
}
