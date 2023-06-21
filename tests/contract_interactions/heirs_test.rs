use salsa::SalsaContract;
use salsa::heirs::HeirsModule;

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
    pub fn set_heir_test(
        &mut self,
        sender: &Address,
        heir: &Address,
        inheritance_epochs: u64,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.set_heir(managed_address!(heir), inheritance_epochs)
            })
            .assert_ok();
    }

    pub fn set_heir_fail_test(
        &mut self,
        sender: &Address,
        heir: &Address,
        inheritance_epochs: u64,
        error: &str,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.set_heir(managed_address!(heir), inheritance_epochs)
            })
            .assert_user_error(error);
    }

    pub fn remove_heir_test(
        &mut self,
        sender: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(sender, &self.salsa_wrapper, &big_zero, |sc| {
                sc.remove_heir()
            })
            .assert_ok();
    }

    pub fn undelegate_heir_test(
        &mut self,
        heir: &Address,
        user: &Address,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(heir, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_heir(managed_address!(user), to_managed_biguint(amount))
            })
            .assert_ok();
    }

    pub fn undelegate_heir_fail_test(
        &mut self,
        heir: &Address,
        user: &Address,
        amount: num_bigint::BigUint,
        error: &str,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(heir, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_heir(managed_address!(user), to_managed_biguint(amount))
            })
            .assert_user_error(error);
    }

    pub fn undelegate_now_heir_test(
        &mut self,
        heir: &Address,
        user: &Address,
        min_amount: num_bigint::BigUint,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(heir, &self.salsa_wrapper, &big_zero, |sc| {
                sc.undelegate_now_heir(managed_address!(user), to_managed_biguint(min_amount), to_managed_biguint(amount))
            })
            .assert_ok();
    }

    pub fn withdraw_heir_test(
        &mut self,
        heir: &Address,
        user: &Address,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(heir, &self.salsa_wrapper, &big_zero, |sc| {
                sc.withdraw_heir(managed_address!(user))
            })
            .assert_ok();
    }

    pub fn remove_reserve_heir_test(
        &mut self,
        heir: &Address,
        user: &Address,
        amount: num_bigint::BigUint,
    ) {
        let big_zero = rust_biguint!(0);
        self.blockchain_wrapper
            .execute_tx(heir, &self.salsa_wrapper, &big_zero, |sc| {
                sc.remove_reserve_heir(managed_address!(user), to_managed_biguint(amount))
            })
            .assert_ok();
    }
}
