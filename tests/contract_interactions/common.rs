use crate::{contract_setup::SalsaContractSetup, to_managed_biguint};
use salsa::config::ConfigModule;

use multiversx_sc::{
    types::{
        Address,
        ManagedAddress
    }
};

use multiversx_sc_scenario::{
    rust_biguint, DebugApi
};

impl<SalsaContractObjBuilder> SalsaContractSetup<SalsaContractObjBuilder>
where
    SalsaContractObjBuilder: 'static + Copy + Fn() -> salsa::ContractObj<DebugApi>,
{
    pub fn setup_new_user(
        &mut self,
        egld_mount: u64
    ) -> Address {
        let big_zero = rust_biguint!(0);
        let new_user = self.blockchain_wrapper.create_user_account(&big_zero);
        
        self.blockchain_wrapper
            .set_egld_balance(&new_user, &Self::exp18(egld_mount));
        
        new_user
    }

    pub fn check_total_egld_staked(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.total_egld_staked().get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_liquid_supply(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.liquid_token_supply().get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_egld_to_undelegate(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.egld_to_undelegate().get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_total_withdrawn_egld(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.total_withdrawn_egld().get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_user_withdrawn_egld(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.user_withdrawn_egld().get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_user_reserve(&mut self, user: ManagedAddress<DebugApi>, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.get_reserve_egld_amount(&sc.users_reserve_points(&user).get()),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_user_reserve_points(&mut self, user: ManagedAddress<DebugApi>, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.users_reserve_points(&user).get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_egld_reserve(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.egld_reserve().get(),
                        to_managed_biguint(amount.clone())
                    );
                    assert_eq!(
                        sc.get_reserve_egld_amount(&sc.reserve_points().get()),
                        to_managed_biguint(amount.clone())
                    );
                }
            ).assert_ok();
    }

    pub fn check_available_egld_reserve(&mut self, amount: num_bigint::BigUint) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.available_egld_reserve().get(),
                        to_managed_biguint(amount)
                    );
                }
            ).assert_ok();
    }

    pub fn check_user_undelegations_length(&mut self, user: ManagedAddress<DebugApi>) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.luser_undelegations(&user).len() <= 11,
                        true
                    );
                }
            ).assert_ok();
    }

    pub fn check_total_undelegations_lengths(&mut self) {
        self.blockchain_wrapper
            .execute_query(
                &self.salsa_wrapper, |sc| {
                    assert_eq!(
                        sc.lreserve_undelegations().len() <= 11,
                        true
                    );
                    assert_eq!(
                        sc.ltotal_user_undelegations().len() <= 11,
                        true
                    );
                }
            ).assert_ok();
    }
}
