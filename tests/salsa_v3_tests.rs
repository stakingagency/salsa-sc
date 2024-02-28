mod contract_interactions;
pub mod consts;

use std::ops::Mul;

use consts::*;
use contract_interactions::{
    checks::*, providers_interactions::*, salsa_interactions::*, service_interactions::*
};
use delegation_mock::DelegationMock;

use salsa::{
    common::{
        config::{ConfigModule, State},
        consts::MAX_PERCENT
    },
    providers::ProvidersModule
};

use multiversx_sc::{
    storage::mappers::StorageTokenWrapper as _,
    types::{Address, BigUint}
};

use multiversx_sc_scenario::{
    managed_address, managed_token_id, rust_biguint,
    scenario_model::{
        Account, BigUintValue, ScCallStep, ScDeployStep, SetStateStep
    },
    DebugApi, ScenarioWorld, WhiteboxContract
};

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace(".");

    blockchain.register_contract(
        SALSA_PATH_EXPR,
        salsa::ContractBuilder,
    );
    blockchain.register_contract(
        DELEGATION_PATH_EXPR,
        delegation_mock::ContractBuilder,
    );

    blockchain
}

fn setup() -> ScenarioWorld {
    let mut world = world();

    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    let salsa_code = world.code_expression(SALSA_PATH_EXPR);

    let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
    let delegation2_whitebox = WhiteboxContract::new(DELEGATION2_ADDRESS_EXPR, delegation_mock::contract_obj);
    let delegation_code = world.code_expression(DELEGATION_PATH_EXPR);

    let roles = vec![
        "ESDTRoleLocalMint".to_string(),
        "ESDTRoleLocalBurn".to_string(),
        "ESDTRoleLocalTransfer".to_string(),
    ];

    world.set_state_step(
        SetStateStep::new()
            .put_account(
                OWNER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance("1_000_000_000_000_000_000")
            )
            .put_account(
                CALLER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance("1_000_000_000_000_000_000")
            )
            .new_address(OWNER_ADDRESS_EXPR, 1, SALSA_ADDRESS_EXPR)
            .put_account(
                DELEGATOR1_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(DELEGATOR1_INITIAL_BALANCE_EXPR)
            )
            .new_address(DELEGATOR1_ADDRESS_EXPR, 1, DELEGATION1_ADDRESS_EXPR)
            .put_account(
                DELEGATOR2_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(DELEGATOR2_INITIAL_BALANCE_EXPR)
            )
            .put_account(
                RESERVER1_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(RESERVER1_INITIAL_BALANCE_EXPR)
            )
            .new_address(RESERVER1_ADDRESS_EXPR, 1, DELEGATION2_ADDRESS_EXPR)
            .put_account(
                RESERVER2_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(RESERVER2_INITIAL_BALANCE_EXPR)
            )
            .put_account(
                SALSA_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .code(salsa_code.clone())
                    .owner(OWNER_ADDRESS_EXPR)
                    .esdt_roles(TOKEN_ID_EXPR, roles)
            )
    );

    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.liquid_token_id().set_token_id(managed_token_id!(TOKEN_ID)),
    );

    // set unbond period
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.set_unbond_period(UNBOND_PERIOD),
    );

    // set service fee
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.set_service_fee(SERVICE_FEE),
    );

    // set undelegate now fee
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.set_undelegate_now_fee(UNDELEGATE_NOW_FEE),
    );

    // add providers
    world.whitebox_deploy(
        &delegation1_whitebox,
        ScDeployStep::new()
            .from(DELEGATOR1_ADDRESS_EXPR)
            .code(delegation_code.clone()),
        |sc| {
            sc.init(
                BigUint::from(1_000_000_000_000_000_000_u64) * 15_000_u64,
                5_u64,
                1000_u64
            )
        }
    );

    world.whitebox_deploy(
        &delegation2_whitebox,
        ScDeployStep::new()
            .from(RESERVER1_ADDRESS_EXPR)
            .code(delegation_code),
        |sc| {
            sc.init(
                BigUint::from(1_000_000_000_000_000_000_u64) * 30_000_u64,
                10_u64,
                800_u64
            )
        }
    );

    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.add_provider(managed_address!(&Address::from_slice(delegation1_whitebox.address_expr.to_address().as_bytes())));
        }
    );

    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.add_provider(managed_address!(&Address::from_slice(delegation2_whitebox.address_expr.to_address().as_bytes())));
        }
    );

    // set state active
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.set_state_active(),
    );

    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.state().get(), State::Active);
        }
    );

    world
}

pub fn exp(value: u64, e: u32) -> num_bigint::BigUint {
    value.mul(rust_biguint!(10).pow(e))
}

pub fn to_managed_biguint(value: &num_bigint::BigUint) -> BigUint<DebugApi> {
    BigUint::from_bytes_be(&value.to_bytes_be())
}

#[test]
fn test_init() {
    let mut world = setup();
    check_provider_active(&mut world, DELEGATION1_ADDRESS_EXPR);
    check_provider_active(&mut world, DELEGATION2_ADDRESS_EXPR);
}

#[test]
fn test_delegation() {
    let mut world = setup();

    let amount = exp(1, 18);
    let first_user_initial_amount = &BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;

    // delegate
    set_block_nonce(&mut world, 10);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount, false, true);
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &amount);
    check_egld_to_delegate(&mut world, &amount);
    delegate_all_test(&mut world);
    check_total_egld_staked(&mut world, &amount);
    check_liquid_token_supply(&mut world, &amount);

    // undelegate
    set_block_nonce(&mut world, 20);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &amount, true);
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &rust_biguint!(0));
    check_total_egld_staked(&mut world, &rust_biguint!(0));
    check_egld_to_undelegate(&mut world, &amount);

    //undelegate all
    undelegate_all_test(&mut world);

    // withdraw_all
    set_block_nonce(&mut world, 120);
    withdraw_all_test(&mut world);
    check_total_withdrawn_egld(&mut world, &amount);

    // compute withdrawn
    compute_withdrawn_test(&mut world);
    check_total_withdrawn_egld(&mut world, &rust_biguint!(0));
    check_user_withdrawn_egld(&mut world, &amount);

    // withdraw
    withdraw_test(&mut world, DELEGATOR1_ADDRESS_EXPR);
    check_user_withdrawn_egld(&mut world, &rust_biguint!(0));
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &first_user_initial_amount);
}

#[test]
fn test_reserves() {
    let mut world = setup();

    let first_user_initial_amount = BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;
    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;

    // delegate
    set_block_nonce(&mut world, 10);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, false, true);
    delegate_all_test(&mut world);

    // add reserve
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &one, true);
    check_egld_reserve(&mut world, &one);
    check_available_egld_reserve(&mut world, &one);

    // undelegate now
    undelegate_now_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &(&one - &fee), &one, true);
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&first_user_initial_amount - &fee));
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &rust_biguint!(0));
    check_available_egld_reserve(&mut world, &fee);

    // undelegate all
    undelegate_all_test(&mut world);
    check_reserve_undelegations(&mut world, &one);

    // withdraw all
    set_block_nonce(&mut world, 110);
    withdraw_all_test(&mut world);

    // compute withdrawn
    compute_withdrawn_test(&mut world);
    check_available_egld_reserve(&mut world, &(&one + &fee));

    // remove reserve
    remove_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&one + &fee), true);
    check_egld_reserve(&mut world, &rust_biguint!(0));
    check_available_egld_reserve(&mut world, &rust_biguint!(0));
    check_user_reserve(&mut world, RESERVER1_ADDRESS_EXPR, &rust_biguint!(0));
    check_user_reserve_points(&mut world, RESERVER1_ADDRESS_EXPR, &rust_biguint!(0));
}

#[test]
fn test_reserve_to_user_undelegation() {
    let mut world = setup();

    let delegator1_initial_amount = BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;
    let delegator2_initial_amount = BigUintValue::from(DELEGATOR2_INITIAL_BALANCE_EXPR).value;
    let reserver1_initial_amount = BigUintValue::from(RESERVER1_INITIAL_BALANCE_EXPR).value;
    let reserver2_initial_amount = BigUintValue::from(RESERVER2_INITIAL_BALANCE_EXPR).value;
    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;
    let one_minus_fee = &one - &fee;
    let one_plus_fee = &one + &fee;

    // delegate 5 and add reserves 5
    set_block_nonce(&mut world, 10);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, false, true);
    delegate_test(&mut world, DELEGATOR2_ADDRESS_EXPR, &(&one * 4_u64), false, true);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&one * 2_u64), true);
    add_reserve_test(&mut world, RESERVER2_ADDRESS_EXPR, &(&one * 3_u64), true);
    // stake = 5, reserve = 5, available reserve = 5

    // undelegate: 1, undelegate now 3
    set_block_nonce(&mut world, 20);
    undelegate_now_test(
        &mut world,
        false,
        DELEGATOR1_ADDRESS_EXPR,
        &one_minus_fee,
        &one,
        true
    );
    undelegate_all_test(&mut world);
    undelegate_now_test(
        &mut world,
        false,
        DELEGATOR2_ADDRESS_EXPR,
        &(&one_minus_fee * 2_u64),
        &(&one * 2u64),
        true
    );
    undelegate_test(&mut world, false, DELEGATOR2_ADDRESS_EXPR, &one, true);
    // stake = 1, reserve = 5.06, available reserve = 2.06

    // remove reserves 3.04
    let earned = &fee * 3_u64;
    let earned1 = &earned * 2_u64 / 5_u64;
    let earned2 = &earned - &earned1;
    set_block_nonce(&mut world, 30);
    remove_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&earned1 + &one), true);
    remove_reserve_test(&mut world, RESERVER2_ADDRESS_EXPR, &(&earned2 + &one * 2_u64), true);
    // stake = 1, reserve = 2.02, available reserve = 0

    // check delegators balances
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&delegator1_initial_amount - &fee));
    check_egld_balance(&mut world, DELEGATOR2_ADDRESS_EXPR, &(&delegator2_initial_amount - &one_plus_fee * 2_u64));
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, DELEGATOR2_ADDRESS_EXPR, TOKEN_ID_EXPR, &one);

    // check egld staked and reserve
    check_total_egld_staked(&mut world, &one);
    check_available_egld_reserve(&mut world, &rust_biguint!(0));
    check_egld_reserve(&mut world, &(&one * 2_u64));
    check_user_undelegations_order(&mut world, RESERVER2_ADDRESS_EXPR);
    check_user_undelegations_order(&mut world, DELEGATOR2_ADDRESS_EXPR);
    check_total_undelegations_order(&mut world);

    // undelegate and withdraw
    set_block_nonce(&mut world, 40);
    undelegate_all_test(&mut world);
    set_block_nonce(&mut world, 130);
    withdraw_all_test(&mut world);
    compute_withdrawn_test(&mut world);
    set_block_nonce(&mut world, 140);
    withdraw_all_test(&mut world);
    compute_withdrawn_test(&mut world);
    withdraw_test(&mut world, DELEGATOR2_ADDRESS_EXPR);
    withdraw_test(&mut world, RESERVER2_ADDRESS_EXPR);

    // final checks
    check_egld_balance(&mut world, DELEGATOR2_ADDRESS_EXPR, &(&delegator2_initial_amount - &one - &fee * 2_u64));
    check_egld_balance(&mut world, RESERVER1_ADDRESS_EXPR, &(&reserver1_initial_amount - &one + &earned1));
    check_egld_balance(&mut world, RESERVER2_ADDRESS_EXPR, &(&reserver2_initial_amount - &one + &earned2));
    check_available_egld_reserve(&mut world, &(&one * 2_u64));
    check_user_reserve(&mut world, RESERVER1_ADDRESS_EXPR, &one);
    check_user_reserve(&mut world, RESERVER2_ADDRESS_EXPR, &one);
}
