use salsa::SalsaContract;

use crate::consts::*;
use crate::{contract_setup::SalsaContractSetup, to_managed_biguint};

use multiversx_sc_scenario::{
    num_bigint, rust_biguint, DebugApi,
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
    pub fn delegate_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
    ) {
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &amount, |sc| {
                sc.delegate();
            })
            .assert_ok();
    }

    pub fn undelegate_test(
        &mut self,
        sender: &Address,
        amount: num_bigint::BigUint,
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.undelegate()
            })
            .assert_ok();
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
    ) {
        self.blockchain_wrapper
            .execute_esdt_transfer(sender, &self.salsa_wrapper, TOKEN_ID, 0, &amount, |sc| {
                sc.undelegate_now(to_managed_biguint(min_amount))
            })
            .assert_ok();
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

    // pub fn compound_test(
    //     &mut self,
    //     sender: &Address,
    // ) {
    //     let big_zero = rust_biguint!(0);
    //     self.blockchain_wrapper
    //         .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
    //             sc.compound()
    //         })
    //         .assert_ok();
    // }
}
