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
    rust_biguint, managed_address
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
    sc_setup.delegate_test(&caller, amount.clone(), false);
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &big_zero);
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &amount);
    sc_setup.check_total_egld_staked(amount.clone());
    sc_setup.check_liquid_supply(amount.clone());

    // undelegate
    sc_setup.undelegate_test(&caller, amount.clone(), big_zero.clone());
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
    sc_setup.delegate_test(&caller, one.clone(), false);
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &big_zero);
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &one);
    sc_setup.check_total_egld_staked(one.clone());
    sc_setup.check_liquid_supply(one.clone());

    // add reserve
    sc_setup.add_reserve_test(&reserver, one.clone());
    sc_setup.check_egld_reserve(one.clone());
    sc_setup.check_available_egld_reserve(one.clone());

    // undelegate now
    sc_setup.undelegate_now_test(&caller, one.clone(), one_minus_fee.clone(), big_zero.clone());
    sc_setup.blockchain_wrapper.check_egld_balance(&caller, &one_minus_fee);
    sc_setup.blockchain_wrapper.check_esdt_balance(&caller, TOKEN_ID, &big_zero);
    sc_setup.check_available_egld_reserve(rest.clone());

    // undelegate all
    sc_setup.check_egld_to_undelegate(one.clone());
    sc_setup.undelegate_all_test(&caller);
    sc_setup.check_egld_to_undelegate(big_zero.clone());
    sc_setup.check_reserve_undelegations_amount(one.clone());

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
    sc_setup.check_user_reserve(managed_address!(&reserver), big_zero.clone());
    sc_setup.check_user_reserve_points(managed_address!(&reserver), big_zero.clone());
}

#[test]
fn reserve_to_user_undelegation_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let big_zero = rust_biguint!(0);
    let ten = exp(10, 18);
    let one = exp(1, 18);

    let delegator1 = sc_setup.setup_new_user(1u64);
    let delegator2 = sc_setup.setup_new_user(2u64);
    let reserver1 = sc_setup.setup_new_user(3u64);
    let reserver2 = sc_setup.setup_new_user(4u64);
    let caller = sc_setup.setup_new_user(5u64);

    // set epoch and balances
    sc_setup.blockchain_wrapper.set_block_epoch(1u64);
    sc_setup.blockchain_wrapper.set_egld_balance(&delegator1, &ten);
    sc_setup.blockchain_wrapper.set_egld_balance(&delegator2, &ten);
    sc_setup.blockchain_wrapper.set_egld_balance(&reserver1, &ten);
    sc_setup.blockchain_wrapper.set_egld_balance(&reserver2, &ten);
    sc_setup.blockchain_wrapper.set_egld_balance(&caller, &one);

    // delegate 5 and add reserves 5
    sc_setup.delegate_test(&delegator1, one.clone(), false);
    sc_setup.delegate_test(&delegator2, one.clone() * 4u64, false);
    sc_setup.add_reserve_test(&reserver1, one.clone() * 2u64);
    sc_setup.add_reserve_test(&reserver2, one.clone() * 3u64);
    // stake = 5, reserve = 5, available reserve = 5

    // undelegate: 1, undelegate now 3
    sc_setup.undelegate_now_test(&delegator1, one.clone(), exp(98u64, 16), big_zero.clone());
    sc_setup.undelegate_all_test(&caller);
    sc_setup.undelegate_now_test(&delegator2, one.clone() * 2u64, exp(196u64, 16), big_zero.clone());
    sc_setup.undelegate_test(&delegator2, one.clone(), big_zero.clone());
    // stake = 1, reserve = 5.06, available reserve = 2.06

    // remove reserves 3.04
    sc_setup.blockchain_wrapper.set_block_epoch(2u64);
    sc_setup.remove_reserve_test(&reserver1, exp(1024u64, 15));
    sc_setup.remove_reserve_test(&reserver2, exp(2016u64, 15));
    // stake = 1, reserve = 2.02, available reserve = 0

    // check delegators balances
    sc_setup.blockchain_wrapper.check_egld_balance(&delegator1, &(exp(998u64, 16)));
    sc_setup.blockchain_wrapper.check_egld_balance(&delegator2, &(exp(796u64, 16)));
    sc_setup.blockchain_wrapper.check_esdt_balance(&delegator1, TOKEN_ID, &big_zero);
    sc_setup.blockchain_wrapper.check_esdt_balance(&delegator2, TOKEN_ID, &one);

    // check egld staked and reserve
    sc_setup.check_total_egld_staked(one.clone());
    sc_setup.check_available_egld_reserve(big_zero.clone());
    sc_setup.check_egld_reserve(exp(202u64, 16));
    sc_setup.check_user_undelegations_order(managed_address!(&reserver2));
    sc_setup.check_user_undelegations_order(managed_address!(&delegator2));
    sc_setup.check_total_undelegations_order();

    // undelegate and withdraw
    sc_setup.blockchain_wrapper.set_block_epoch(3u64);
    sc_setup.undelegate_all_test(&caller);
    sc_setup.blockchain_wrapper.set_block_epoch(12u64);
    sc_setup.withdraw_all_test(&caller);
    sc_setup.compute_withdrawn_test(&caller);
    sc_setup.blockchain_wrapper.set_block_epoch(13u64);
    sc_setup.withdraw_all_test(&caller);
    sc_setup.compute_withdrawn_test(&caller);
    sc_setup.withdraw_test(&delegator2);
    sc_setup.withdraw_test(&reserver2);

    // final checks
    sc_setup.blockchain_wrapper.check_egld_balance(&delegator2, &(exp(896u64, 16)));
    sc_setup.blockchain_wrapper.check_egld_balance(&reserver1, &(exp(9024u64, 15)));
    sc_setup.blockchain_wrapper.check_egld_balance(&reserver2, &(exp(9016u64, 15)));
    sc_setup.check_available_egld_reserve(exp(202u64, 16));
    sc_setup.check_user_reserve(managed_address!(&reserver1), one.clone());
    // sc_setup.check_user_reserve(managed_address!(&reserver2), exp(102u64, 16));
}

#[test]
fn merge_undelegations_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let big_zero = rust_biguint!(0);
    let one = exp(1, 18);
    let mut epoch = 1u64;

    let delegator = sc_setup.setup_new_user(1u64);
    let reserver = sc_setup.setup_new_user(2u64);
    let caller = sc_setup.setup_new_user(3u64);

    // set epoch and balances
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.blockchain_wrapper.set_egld_balance(&delegator, &(one.clone() * 250u64));
    sc_setup.blockchain_wrapper.set_egld_balance(&reserver, &(one.clone() * 125u64));
    sc_setup.blockchain_wrapper.set_egld_balance(&caller, &one);

    // delegate and add reserve
    sc_setup.delegate_test(&delegator, one.clone() * 250u64, false);
    sc_setup.add_reserve_test(&reserver, one.clone() * 125u64);

    // undelegate and undelegate now reserve in 15 epochs
    for i in 1u64..16u64 {
        sc_setup.undelegate_test(&delegator, exp(i, 18), big_zero.clone());
        sc_setup.undelegate_now_test(&delegator, exp(i, 18), exp(i * 98u64, 16), big_zero.clone());
        epoch += 1u64;
        sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    }

    // check undelegations lenghts and order
    sc_setup.check_user_undelegations_order(managed_address!(&delegator));
    sc_setup.check_total_undelegations_order();
    sc_setup.check_user_undelegations_length(managed_address!(&delegator), 11);
    sc_setup.check_total_users_undelegations_lengths(11);
    sc_setup.check_reserve_undelegations_lengths(11);

    // undelegate all
    sc_setup.undelegate_all_test(&caller);
    epoch += 10u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.withdraw_all_test(&caller);
    sc_setup.compute_withdrawn_test(&caller);
    sc_setup.withdraw_test(&delegator);

    // final checks
    sc_setup.blockchain_wrapper.check_egld_balance(&delegator, &(exp(2376u64, 17)));
    sc_setup.blockchain_wrapper.check_esdt_balance(&delegator, TOKEN_ID, &(one.clone() * 10u64));
    sc_setup.check_available_egld_reserve(exp(1274, 17));
    sc_setup.check_total_egld_staked(exp(1, 19));
}

#[test]
fn user_undelegations_order_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let big_zero = rust_biguint!(0);
    let one = exp(1, 18);
    let mut epoch = 1u64;
    let delegator = sc_setup.setup_new_user(1u64);

    // set epoch and balances
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.blockchain_wrapper.set_egld_balance(&delegator, &exp(100, 18));

    // delegate
    sc_setup.delegate_test(&delegator, exp(100, 18), false);

    // undelegate in epochs 3 and 2 (3 times, 2 in the same epoch, so should be merged)
    epoch = 3u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());
    epoch = 2u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());
    epoch = 4u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());

    // check undelegations orders and lengths
    sc_setup.check_user_undelegations_order(managed_address!(&delegator));
    sc_setup.check_total_undelegations_order();
    sc_setup.check_user_undelegations_length(managed_address!(&delegator), 3);
    sc_setup.check_total_users_undelegations_lengths(3);

    // undelegate in epoch 1, 3, 5, 30 and 15
    epoch = 1u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());
    epoch = 3u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());
    epoch = 5u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());
    epoch = 30u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone()); // should merge the previous
    epoch = 15u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_test(&delegator, one.clone(), big_zero.clone());

    // check undelegations orders, lengths and amount
    sc_setup.check_user_undelegations_order(managed_address!(&delegator));
    sc_setup.check_total_undelegations_order();
    sc_setup.check_user_undelegations_length(managed_address!(&delegator), 3);
    sc_setup.check_total_users_undelegations_lengths(3);
    sc_setup.check_user_undelegations_amount(managed_address!(&delegator), exp(9, 18));
    sc_setup.check_total_users_undelegations_amount(exp(9, 18));
}

#[test]
fn reserve_undelegations_order_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let big_zero = rust_biguint!(0);
    let one = exp(1, 18);
    let one_with_fee = exp(98, 16);
    let mut epoch = 1u64;
    let reserver = sc_setup.setup_new_user(100u64);

    // set epoch
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);

    // delegate and add reserve
    sc_setup.delegate_test(&reserver, exp(50, 18), false);
    sc_setup.add_reserve_test(&reserver, exp(50, 18));

    // undelegate now in epochs 3 and 2 (3 times, 2 in the same epoch, so should be merged)
    epoch = 3u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_now_test(&reserver, one.clone(), one_with_fee.clone(), big_zero.clone());
    epoch = 2u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_now_test(&reserver, one.clone(), one_with_fee.clone(), big_zero.clone());
    sc_setup.undelegate_now_test(&reserver, one.clone(), one_with_fee.clone(), big_zero.clone());

    // check undelegations order, length and amount
    sc_setup.check_total_undelegations_order();
    sc_setup.check_reserve_undelegations_lengths(2);
    sc_setup.check_reserve_undelegations_amount(exp(3, 18));

    // undelegate in epoch 30 and 15
    epoch = 30u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_now_test(&reserver, one.clone(), one_with_fee.clone(), big_zero.clone()); // should merge the previous
    epoch = 15u64;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_now_test(&reserver, one.clone(), one_with_fee.clone(), big_zero.clone());

    // check undelegations order, length and amount
    sc_setup.check_total_undelegations_order();
    sc_setup.check_reserve_undelegations_lengths(3);
    sc_setup.check_reserve_undelegations_amount(exp(5, 18));
}

#[test]
fn knight_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let delegator = sc_setup.setup_new_user(10u64);
    let knight1 = sc_setup.setup_new_user(0u64);
    let knight2 = sc_setup.setup_new_user(0u64);

    sc_setup.delegate_test(&delegator, exp(1, 18), true); // true = custodial

    sc_setup.set_knight_test(&delegator, &knight1);
    sc_setup.set_knight_fail_test(&delegator, &knight1, "Knight already set");
    sc_setup.cancel_knight_test(&delegator);

    sc_setup.set_knight_test(&delegator, &knight2);
    sc_setup.confirm_knight_test(&knight2, &delegator);
    sc_setup.cancel_knight_fail_test(&delegator, "Knight can only be canceled or confirmed while pending confirmation");
    sc_setup.remove_knight_test(&knight2, &delegator);

    sc_setup.set_knight_test(&delegator, &knight1);
    sc_setup.confirm_knight_test(&knight1, &delegator);
    sc_setup.activate_knight_test(&delegator);
    sc_setup.undelegate_fail_test(&delegator, rust_biguint!(0), exp(1, 18), "Knight is active");

    sc_setup.deactivate_knight_test(&knight1, &delegator);
    sc_setup.undelegate_test(&delegator, rust_biguint!(0), exp(1, 18));
}

#[test]
fn active_knigth_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let one = exp(1, 18);
    let one_with_fee = exp(98, 16);
    let delegator = sc_setup.setup_new_user(10u64);
    let knight = sc_setup.setup_new_user(0u64);
    let mut epoch = 1u64;

    // set epoch
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);

    // delegate and add reserve
    sc_setup.delegate_test(&delegator, exp(2, 18), true); // true = custodial
    sc_setup.add_reserve_test(&delegator, one.clone());

    // set knight, confirm and activate
    sc_setup.set_knight_test(&delegator, &knight);
    sc_setup.confirm_knight_test(&knight, &delegator);
    sc_setup.activate_knight_test(&delegator);

    // undelegate knight, undelegate now knight and remove reserve knight
    sc_setup.undelegate_knight_test(&knight, &delegator, one.clone());
    sc_setup.undelegate_now_knight_test(&knight, &delegator, one_with_fee, one.clone());
    sc_setup.undelegate_all_test(&delegator);
    epoch += 1;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.remove_reserve_knight_test(&knight, &delegator, exp(102, 16));

    // withdraw
    epoch += 9;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.withdraw_all_test(&delegator);
    sc_setup.compute_withdrawn_test(&delegator);
    sc_setup.withdraw_knight_test(&knight, &delegator);

    // checks
    sc_setup.blockchain_wrapper.check_egld_balance(&delegator, &exp(7, 18));
    sc_setup.blockchain_wrapper.check_egld_balance(&knight, &exp(3, 18));
}

#[test]
fn entitled_heir_test() {
    let _ = DebugApi::dummy();

    let mut sc_setup = SalsaContractSetup::new(salsa::contract_obj);
    let one = exp(1, 18);
    let one_with_fee = exp(98, 16);
    let delegator = sc_setup.setup_new_user(10u64);
    let heir = sc_setup.setup_new_user(0u64);
    let heir2 = sc_setup.setup_new_user(0u64);
    let mut epoch = 1u64;

    // set epoch
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);

    // delegate and add reserve
    sc_setup.delegate_test(&delegator, exp(2, 18), true); // true = custodial
    sc_setup.add_reserve_test(&delegator, one.clone());

    // set heir
    sc_setup.set_heir_test(&delegator, &heir2, 365u64);
    sc_setup.remove_heir_test(&delegator);
    sc_setup.set_heir_test(&delegator, &heir, 365u64);

    // undelegate heir, undelegate now heir and remove reserve heir
    epoch += 365;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.undelegate_heir_test(&heir, &delegator, one.clone());
    sc_setup.undelegate_now_heir_test(&heir, &delegator, one_with_fee, one.clone());
    sc_setup.undelegate_all_test(&delegator);
    epoch += 1;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.remove_reserve_heir_test(&heir, &delegator, exp(102, 16));

    // withdraw
    epoch += 9;
    sc_setup.blockchain_wrapper.set_block_epoch(epoch);
    sc_setup.withdraw_all_test(&delegator);
    sc_setup.compute_withdrawn_test(&delegator);
    sc_setup.withdraw_heir_test(&heir, &delegator);

    // checks
    sc_setup.blockchain_wrapper.check_egld_balance(&delegator, &exp(7, 18));
    sc_setup.blockchain_wrapper.check_egld_balance(&heir, &exp(3, 18));
}

pub fn exp(value: u64, e: u32) -> num_bigint::BigUint {
    value.mul(rust_biguint!(10).pow(e))
}

pub fn to_managed_biguint(value: num_bigint::BigUint) -> BigUint<DebugApi> {
    BigUint::from_bytes_be(&value.to_bytes_be())
}
