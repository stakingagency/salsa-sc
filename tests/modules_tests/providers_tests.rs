use multiversx_sc_scenario::scenario_model::AddressValue;
use salsa::{common::consts::MAX_PROVIDER_FEE, service::ServiceModule};

use crate::*;

#[test]
fn test_one_provider() {
    let mut world = setup();

    let amount = exp(1000, 18);
    let mut nonce = BLOCKS_PER_EPOCH;

    set_block_nonce(&mut world, nonce);
    remove_provider_test(&mut world, OWNER_ADDRESS_EXPR, DELEGATION2_ADDRESS_EXPR, ERROR_PROVIDER_NOT_UP_TO_DATE);
    refresh_provider_test(&mut world, DELEGATION2_ADDRESS_EXPR);
    remove_provider_test(&mut world, OWNER_ADDRESS_EXPR, DELEGATION2_ADDRESS_EXPR, b"");

    add_provider_test(&mut world, OWNER_ADDRESS_EXPR, DELEGATION2_ADDRESS_EXPR, ERROR_ACTIVE);
    set_state_inactive_test(&mut world);
    add_provider_test(&mut world, OWNER_ADDRESS_EXPR, DELEGATION2_ADDRESS_EXPR, b"");
    set_state_active_test(&mut world);
    set_provider_state_test(&mut world, OWNER_ADDRESS_EXPR, DELEGATION2_ADDRESS_EXPR, State::Inactive);

    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount, false, true);
    delegate_all_test(&mut world);

    let mut rewards = rust_biguint!(0);
    let mut epochs = 0_u64;

    while rewards < rust_biguint!(ONE_EGLD) {
        epochs += 1;
        rewards = &amount * DELEGATION1_APR / MAX_PERCENT * epochs / EPOCHS_IN_YEAR;
        let service_fee = &rewards * SERVICE_FEE / MAX_PERCENT;
        rewards -= service_fee;
    }

    nonce += BLOCKS_PER_EPOCH * epochs;
    set_block_nonce(&mut world, nonce);
    claim_rewards_test(&mut world);
    delegate_all_test(&mut world);
    check_total_egld_staked(&mut world, &(&amount + &rewards));
}

fn get_amount_to_equal_topup(world: &mut ScenarioWorld) -> num_bigint::BigUint {
    refresh_providers_test(world);
    let total_stake1 = get_provider_total_stake(world, DELEGATION1_ADDRESS_EXPR);
    let total_stake2 = get_provider_total_stake(world, DELEGATION2_ADDRESS_EXPR);
    let topup1 = total_stake1 / DELEGATION1_NODES_COUNT - NODE_BASE_STAKE;
    let topup2 = total_stake2 / DELEGATION2_NODES_COUNT - NODE_BASE_STAKE;
    if topup1 > topup2 {
        (topup1 - topup2) * DELEGATION2_NODES_COUNT
    } else {
        (topup2 - topup1) * DELEGATION1_NODES_COUNT
    }
}

#[test]
fn test_two_providers() {
    let mut world = setup();

    let mut nonce = BLOCKS_PER_EPOCH;
    let extra = exp(10, 18);
    let amount1 = get_amount_to_equal_topup(&mut world) + &extra;

    // delegate
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount1, false, true);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &extra);

    // now topups are equal
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &rust_biguint!(0));

    // force delegate to second provider
    let amount2 = get_amount_to_equal_topup(&mut world) + &extra;
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount2, false, true);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &extra);

    // now topups are equal
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &rust_biguint!(0));

    // undelegate
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &amount2, true, b"");
    undelegate_all_test(&mut world);
    check_egld_to_undelegate(&mut world, &(&amount2 - &extra));

    // now topups are equal
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &amount1, true, b"");
    undelegate_all_test(&mut world);

    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_all_test(&mut world);
    check_egld_to_undelegate(&mut world, &rust_biguint!(0));
}

#[test]
fn test_provider_reached_cap() {
    let mut world = setup();

    let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
    world.whitebox_call(
        &delegation1_whitebox,
        ScCallStep::new()
            .from(DELEGATOR1_ADDRESS_EXPR),
        |sc| {
            sc.max_cap().set(BigUint::from(ONE_EGLD) * DELEGATION1_TOTAL_STAKE);
            sc.has_cap().set(true);
        }
    );
    refresh_provider_test(&mut world, DELEGATION1_ADDRESS_EXPR);
    check_provider_has_free_space(&mut world, DELEGATION1_ADDRESS_EXPR, false);
}

#[test]
fn test_provider_fee_too_high() {
    let mut world = setup();

    let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
    world.whitebox_call(
        &delegation1_whitebox,
        ScCallStep::new()
            .from(DELEGATOR1_ADDRESS_EXPR),
        |sc| {
            sc.service_fee().set(MAX_PROVIDER_FEE + 1);
        }
    );
    refresh_provider_test(&mut world, DELEGATION1_ADDRESS_EXPR);
    check_provider_eligible(&mut world, DELEGATION1_ADDRESS_EXPR, false);
}

#[test]
fn test_delegate_to_uneligible_providers() {
    let mut world = setup();

    let mut nonce = BLOCKS_PER_EPOCH;
    let amount = exp(10, 18);

    // set high fees so delegation should not be possible
    let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
    world.whitebox_call(
        &delegation1_whitebox,
        ScCallStep::new()
            .from(DELEGATOR1_ADDRESS_EXPR),
        |sc| {
            sc.service_fee().set(MAX_PROVIDER_FEE + 1);
        }
    );
    let delegation2_whitebox = WhiteboxContract::new(DELEGATION2_ADDRESS_EXPR, delegation_mock::contract_obj);
    world.whitebox_call(
        &delegation2_whitebox,
        ScCallStep::new()
            .from(DELEGATOR2_ADDRESS_EXPR),
        |sc| {
            sc.service_fee().set(MAX_PROVIDER_FEE + 1);
        }
    );

    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount, false, true);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &amount);

    // lowering the fee should make delegation possible
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    world.whitebox_call(
        &delegation1_whitebox,
        ScCallStep::new()
            .from(DELEGATOR1_ADDRESS_EXPR),
        |sc| {
            sc.service_fee().set(MAX_PROVIDER_FEE);
        }
    );
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &rust_biguint!(0));
}

#[test]
fn test_delegate_to_uneligible_provider() {
    let mut world = setup();

    let mut nonce = BLOCKS_PER_EPOCH;
    let amount = exp(10, 18);

    // get eligible provider to delegate
    set_block_nonce(&mut world, nonce);
    let mut provider_to_delegate = Address::zero();
    refresh_providers_test(&mut world);
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (to_delegate, _, _, _) =
                sc.get_provider_to_delegate_and_amount(&to_managed_biguint(&amount));
            provider_to_delegate = to_delegate.to_address();
        }
    );

    // if one provider is eligible to delegate, make it uneligible
    if provider_to_delegate == AddressValue::from(DELEGATION1_ADDRESS_EXPR).to_address() {
        let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
        world.whitebox_call(
            &delegation1_whitebox,
            ScCallStep::new()
                .from(DELEGATOR1_ADDRESS_EXPR),
            |sc| {
                sc.service_fee().set(MAX_PROVIDER_FEE + 1);
            }
        );
    } else {
        let delegation2_whitebox = WhiteboxContract::new(DELEGATION2_ADDRESS_EXPR, delegation_mock::contract_obj);
        world.whitebox_call(
            &delegation2_whitebox,
            ScCallStep::new()
                .from(DELEGATOR2_ADDRESS_EXPR),
            |sc| {
                sc.service_fee().set(MAX_PROVIDER_FEE + 1);
            }
        );
    }

    // check if it is not choosen anymore for delegation
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    refresh_providers_test(&mut world);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (to_delegate, _, _, _) =
                sc.get_provider_to_delegate_and_amount(&to_managed_biguint(&amount));
            assert!(provider_to_delegate != to_delegate.to_address());
        }
    );
}

#[test]
fn test_undelegate_from_uneligible_provider() {
    let mut world = setup();

    let mut nonce = BLOCKS_PER_EPOCH;
    let extra = exp(10, 18);
    let amount1 = get_amount_to_equal_topup(&mut world) + &extra;

    // delegate
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount1, false, true);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &extra);

    // now topups are equal
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &rust_biguint!(0));

    // force delegate to second provider
    let amount2 = get_amount_to_equal_topup(&mut world) + &extra;
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount2, false, true);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &extra);

    // now topups are equal
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_all_test(&mut world);
    check_egld_to_delegate(&mut world, &rust_biguint!(0));

    // get eligible provider to undelegate
    let mut provider_to_undelegate = Address::zero();
    refresh_providers_test(&mut world);
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (_, _, to_undelegate, _) =
                sc.get_provider_to_delegate_and_amount(&to_managed_biguint(&amount2));
            provider_to_undelegate = to_undelegate.to_address();
        }
    );

    // if one provider is eligible to undelegate, make the other uneligible
    if provider_to_undelegate == AddressValue::from(DELEGATION2_ADDRESS_EXPR).to_address() {
        let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
        world.whitebox_call(
            &delegation1_whitebox,
            ScCallStep::new()
                .from(DELEGATOR1_ADDRESS_EXPR),
            |sc| {
                sc.service_fee().set(MAX_PROVIDER_FEE + 1);
            }
        );
    } else {
        let delegation2_whitebox = WhiteboxContract::new(DELEGATION2_ADDRESS_EXPR, delegation_mock::contract_obj);
        world.whitebox_call(
            &delegation2_whitebox,
            ScCallStep::new()
                .from(DELEGATOR2_ADDRESS_EXPR),
            |sc| {
                sc.service_fee().set(MAX_PROVIDER_FEE + 1);
            }
        );
    }

    // check if the other provider is choosen for undelegation
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    refresh_providers_test(&mut world);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (_, _, to_undelegate, _) =
                sc.get_provider_to_delegate_and_amount(&to_managed_biguint(&amount2));
            assert!(provider_to_undelegate != to_undelegate.to_address());
        }
    );
}
