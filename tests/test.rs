mod contract_setup;
mod contract_interactions;
mod consts;

use consts::*;
use contract_setup::*;
use multiversx_sc_scenario::{
    DebugApi
};

use std::ops::Mul;

use multiversx_sc::{
    types::{
        BigUint,
    },
};

use multiversx_sc_scenario::{
    rust_biguint
};

#[test]
fn init_test() {
    let _ = SalsaContractSetup::new(salsa::contract_obj);
}

// delegate -> undelegate
#[test]
fn delegation_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let big_zero = rust_biguint!(0);
    let caller = sc_setup.setup_new_user(1u64);
    let amount = exp(1, 18);
    sc_setup.blockchain_wrapper.set_block_epoch(1u64);
    sc_setup.blockchain_wrapper.set_egld_balance(&caller, &amount);

    // delegate
    sc_setup.delegate_test(&caller, amount.clone());
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &big_zero);
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &amount);
    sc_setup.check_total_egld_staked(amount.clone());
    sc_setup.check_liquid_supply(amount.clone());

    // undelegate
    sc_setup.undelegate_test(&caller, amount.clone());
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &big_zero);
    sc_setup.check_total_egld_staked(big_zero.clone());
    sc_setup.check_egld_to_undelegate(amount.clone());

    // undelegate all
    sc_setup.undelegate_all_test(&caller);
    sc_setup.check_egld_to_undelegate(big_zero.clone());

    // withdraw all
    sc_setup.blockchain_wrapper.set_block_epoch(11u64);
    sc_setup.withdraw_all_test(&caller);
    sc_setup.check_total_withdrawn_egld(amount.clone());

    // compute withdrawn
    sc_setup.compute_withdrawn_test(&caller);
    sc_setup.check_total_withdrawn_egld(big_zero.clone());
    sc_setup.check_user_withdrawn_egld(amount.clone());

    // withdraw
    sc_setup.withdraw_test(&caller);
    sc_setup.check_user_withdrawn_egld(big_zero.clone());
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &amount);
}

#[test]
fn reserves_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let big_zero = rust_biguint!(0);
    let reserver = sc_setup.setup_new_user(1u64);
    let caller = sc_setup.setup_new_user(2u64);
    let one = exp(1, 18);
    let one_plus_fee = exp(102, 16);
    let one_minus_fee = exp(98, 16);
    let rest = exp(2, 16);
    sc_setup.blockchain_wrapper.set_block_epoch(1u64);
    sc_setup.blockchain_wrapper.set_egld_balance(&reserver, &one);
    sc_setup.blockchain_wrapper.set_egld_balance(&caller, &one);

    // delegate
    sc_setup.delegate_test(&caller, one.clone());
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &big_zero);
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &one);
    sc_setup.check_total_egld_staked(one.clone());
    sc_setup.check_liquid_supply(one.clone());

    // add reserve
    sc_setup.add_reserve_test(&reserver, one.clone());
    sc_setup.check_egld_reserve(one.clone());
    sc_setup.check_available_egld_reserve(one.clone());

    // undelegate now
    sc_setup.undelegate_now_test(&caller, one.clone());
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &one_minus_fee);
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &big_zero);
    sc_setup.check_available_egld_reserve(rest.clone());

    // undelegate all
    sc_setup.check_egld_to_undelegate(one.clone());
    sc_setup.undelegate_all_test(&caller);
    sc_setup.check_egld_to_undelegate(big_zero.clone());

    // withdraw all
    sc_setup.blockchain_wrapper.set_block_epoch(11u64);
    sc_setup.withdraw_all_test(&caller);
    sc_setup.check_total_withdrawn_egld(one.clone());

    // compute withdrawn
    sc_setup.compute_withdrawn_test(&caller);
    sc_setup.check_total_withdrawn_egld(big_zero.clone());
    sc_setup.check_available_egld_reserve(one_plus_fee.clone());

    // remove reserve
    sc_setup.remove_reserve_test(&reserver, one_plus_fee);
    sc_setup.check_egld_reserve(big_zero.clone());
    sc_setup.check_available_egld_reserve(big_zero.clone());
}

pub fn exp(value: u64, e: u32) -> num_bigint::BigUint {
    value.mul(rust_biguint!(10).pow(e))
}

pub fn to_managed_biguint(value: num_bigint::BigUint) -> BigUint<DebugApi> {
    BigUint::from_bytes_be(&value.to_bytes_be())
}
