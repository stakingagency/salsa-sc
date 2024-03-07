mod consts;
mod contract_interactions;
mod modules_tests;
mod setup;

use std::ops::Mul;

use consts::*;
use contract_interactions::{
    salsa_interactions::*,
    delegation_interaction::*,
    service_interactions::*,
    knights_interactions::*,
    heirs_interactions::*,
    providers_interactions::*,
    checks::*,
};
use crate::setup::*;
use delegation_mock::{DelegationMock, EPOCHS_IN_YEAR};
use wrap_mock::WrapMock;

use salsa::{
    common::{
        config::{ConfigModule, State},
        consts::{MAX_HEIR_USERS, MAX_KNIGHT_USERS, MAX_PERCENT, NODE_BASE_STAKE},
        errors::*
    },
    exchanges::{
        onedex::OnedexModule,
        xexchange::XexchangeModule
    },
    providers::ProvidersModule
};

use multiversx_sc::{
    storage::mappers::StorageTokenWrapper as _, types::{Address, BigUint}
};

use multiversx_sc_scenario::{
    managed_address, managed_token_id, rust_biguint,
    scenario_model::{
        Account, BigUintValue, BytesValue, ScCallStep, ScDeployStep, SetStateStep, TxESDT, U64Value
    },
    DebugApi, ScenarioWorld, WhiteboxContract
};

pub fn world() -> ScenarioWorld {
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
    blockchain.register_contract(
        WRAP_PATH_EXPR,
        wrap_mock::ContractBuilder,
    );
    blockchain.register_contract(
        ONEDEX_PATH_EXPR,
        onedex_mock::ContractBuilder,
    );
    blockchain.register_contract(
        XEXCHANGE_PATH_EXPR,
        xexchange_mock::ContractBuilder,
    );

    blockchain
}

pub fn setup() -> ScenarioWorld {
    let mut world = world();

    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    let salsa_code = world.code_expression(SALSA_PATH_EXPR);

    let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
    let delegation2_whitebox = WhiteboxContract::new(DELEGATION2_ADDRESS_EXPR, delegation_mock::contract_obj);

    let wrap_whitebox = WhiteboxContract::new(WRAP_ADDRESS_EXPR, wrap_mock::contract_obj);
    let wrap_code = world.code_expression(WRAP_PATH_EXPR);

    let onedex_whitebox = WhiteboxContract::new(ONEDEX_ADDRESS_EXPR, onedex_mock::contract_obj);
    let onedex_code = world.code_expression(ONEDEX_PATH_EXPR);

    let xexchange_whitebox = WhiteboxContract::new(XEXCHANGE_ADDRESS_EXPR, xexchange_mock::contract_obj);
    let xexchange_code = world.code_expression(XEXCHANGE_PATH_EXPR);

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
            .new_address(DELEGATOR2_ADDRESS_EXPR, 1, DELEGATION2_ADDRESS_EXPR)
            .put_account(
                RESERVER1_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(RESERVER1_INITIAL_BALANCE_EXPR)
            )
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
                    .esdt_roles(TOKEN_ID_EXPR, roles.clone())
            )
            .put_account(
                WRAP_OWNER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance("1_000_000_000_000_000_000")
            )
            .new_address(WRAP_OWNER_ADDRESS_EXPR, 1, WRAP_ADDRESS_EXPR)
            .put_account(
                WRAP_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .code(wrap_code)
                    .balance(WRAP_INITIAL_BALANCE_EXPR)
                    .esdt_balance(WEGLD_ID_EXPR, WRAP_INITIAL_BALANCE_EXPR)
                    .owner(WRAP_OWNER_ADDRESS_EXPR)
                    .esdt_roles(WEGLD_ID_EXPR, roles.clone())
            )
            .put_account(
                ONEDEX_OWNER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(ONEDEX_OWNER_INITIAL_BALANCE_EXPR)
            )
            .new_address(ONEDEX_OWNER_ADDRESS_EXPR, 1, ONEDEX_ADDRESS_EXPR)
            .put_account(
                ONEDEX_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .code(onedex_code)
                    .owner(ONEDEX_OWNER_ADDRESS_EXPR)
                    .esdt_roles(ONEDEX_LP_EXPR, roles.clone())
            )
            .put_account(
                XEXCHANGE_OWNER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance(XEXCHANGE_OWNER_INITIAL_BALANCE_EXPR)
            )
            .new_address(XEXCHANGE_OWNER_ADDRESS_EXPR, 1, XEXCHANGE_ADDRESS_EXPR)
            .put_account(
                XEXCHANGE_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .code(xexchange_code)
                    .owner(XEXCHANGE_OWNER_ADDRESS_EXPR)
                    .esdt_roles(XEXCHANGE_LP_EXPR, roles)
            )
    );

    setup_providers(&mut world);
    setup_wrap_sc(&mut world);
    let onedex_pair_id = setup_onedex_sc(&mut world);
    setup_xexchange_sc(&mut world);

    // setup SALSA
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.liquid_token_id().set_token_id(managed_token_id!(TOKEN_ID));
            sc.set_unbond_period(UNBOND_PERIOD);
            sc.set_service_fee(SERVICE_FEE);
            sc.set_undelegate_now_fee(UNDELEGATE_NOW_FEE);
            sc.add_provider(managed_address!(&Address::from_slice(delegation1_whitebox.address_expr.to_address().as_bytes())));
            sc.add_provider(managed_address!(&Address::from_slice(delegation2_whitebox.address_expr.to_address().as_bytes())));
            sc.set_state_active();

            sc.set_wrap_sc(managed_address!(&Address::from_slice(wrap_whitebox.address_expr.to_address().as_bytes())));
            sc.set_onedex_sc(managed_address!(&Address::from_slice(onedex_whitebox.address_expr.to_address().as_bytes())));
            sc.set_onedex_pair_id(onedex_pair_id);
            sc.set_onedex_arbitrage_active();
            sc.set_xexchange_sc(managed_address!(&Address::from_slice(xexchange_whitebox.address_expr.to_address().as_bytes())));
            sc.set_xexchange_arbitrage_active();
        }
    );

    // check salsa active
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
    check_provider_eligible(&mut world, DELEGATION1_ADDRESS_EXPR, true);
    check_provider_eligible(&mut world, DELEGATION2_ADDRESS_EXPR, true);
}

#[test]
fn test_delegation() {
    let mut world = setup();

    let amount = exp(1, 18);
    let first_user_initial_amount = &BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;
    let mut nonce = BLOCKS_PER_EPOCH;

    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    let mut initial_egld_staked = rust_biguint!(0);
    let mut initial_legld_supply = rust_biguint!(0);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            initial_egld_staked = num_bigint::BigUint::from_bytes_be(sc.total_egld_staked().get().to_bytes_be().as_slice());
            initial_legld_supply = num_bigint::BigUint::from_bytes_be(sc.liquid_token_supply().get().to_bytes_be().as_slice());
        }
    );

    // delegate
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount, false, true);
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &amount);
    check_egld_to_delegate(&mut world, &amount);
    delegate_all_test(&mut world);
    check_total_egld_staked(&mut world, &(&amount + &initial_egld_staked));
    check_liquid_token_supply(&mut world, &(&amount + &initial_legld_supply));

    // undelegate
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &amount, true, b"");
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &rust_biguint!(0));
    check_total_egld_staked(&mut world, &initial_egld_staked);
    check_egld_to_undelegate(&mut world, &amount);

    //undelegate all
    undelegate_all_test(&mut world);

    // withdraw_all
    nonce += UNBOND_PERIOD * BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
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
    let mut nonce = BLOCKS_PER_EPOCH;

    // delegate
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, false, true);
    delegate_all_test(&mut world);

    // add reserve
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &one, true);
    check_egld_reserve(&mut world, &one);
    check_available_egld_reserve(&mut world, &one);

    // undelegate now
    undelegate_now_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &(&one - &fee), &one, true, b"");
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&first_user_initial_amount - &fee));
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &rust_biguint!(0));
    check_available_egld_reserve(&mut world, &fee);

    // undelegate all
    undelegate_all_test(&mut world);
    check_reserve_undelegations(&mut world, &one);

    // withdraw all
    nonce += UNBOND_PERIOD * BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
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
    let mut nonce = BLOCKS_PER_EPOCH;

    // delegate 5 and add reserves 5
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, false, true);
    delegate_test(&mut world, DELEGATOR2_ADDRESS_EXPR, &(&one * 4_u64), false, true);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&one * 2_u64), true);
    add_reserve_test(&mut world, RESERVER2_ADDRESS_EXPR, &(&one * 3_u64), true);
    // stake = 5, reserve = 5, available reserve = 5

    // undelegate: 1, undelegate now 3
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_now_test(
        &mut world,
        false,
        DELEGATOR1_ADDRESS_EXPR,
        &one_minus_fee,
        &one,
        true,
        b""
    );
    undelegate_all_test(&mut world);
    undelegate_now_test(
        &mut world,
        false,
        DELEGATOR2_ADDRESS_EXPR,
        &(&one_minus_fee * 2_u64),
        &(&one * 2u64),
        true,
        b""
    );
    undelegate_test(&mut world, false, DELEGATOR2_ADDRESS_EXPR, &one, true, b"");
    // stake = 1, reserve = 5.06, available reserve = 2.06

    // remove reserves 3.04
    let earned = &fee * 3_u64;
    let earned1 = &earned * 2_u64 / 5_u64;
    let earned2 = &earned - &earned1;
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
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
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_all_test(&mut world);
    nonce += UNBOND_PERIOD * BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    withdraw_all_test(&mut world);
    compute_withdrawn_test(&mut world);
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
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

#[test]
fn test_merge_undelegations() {
    let mut world = setup();

    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;
    let one_minus_fee = &one - &fee;
    let mut nonce = 10u64;
    let delegator1_initial_amount = BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;

    // delegate and add reserve
    set_block_nonce(&mut world, nonce);
    let delegation = 250_u64;
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&one * delegation), false, true);
    delegate_all_test(&mut world);
    let reserve = 125_u64;
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&one * reserve), true);

    // undelegate and undelegate now reserve in 15 epochs
    let n = 15_u64;
    for i in 1_u64..=n {
        undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &(&one * i), true, b"");
        undelegate_now_test(
            &mut world,
            false,
            DELEGATOR1_ADDRESS_EXPR,
            &(&one_minus_fee * i),
            &(&one * i),
            true,
            b""
        );
        nonce += BLOCKS_PER_EPOCH;
        set_block_nonce(&mut world, nonce);
    }

    // check undelegations lenghts and order
    check_user_undelegations_order(&mut world, DELEGATOR1_ADDRESS_EXPR);
    check_total_undelegations_order(&mut world);
    check_user_undelegations_length(&mut world, DELEGATOR1_ADDRESS_EXPR, 11);
    check_total_users_undelegations_lengths(&mut world, 11);
    check_reserve_undelegations_lengths(&mut world, 11);

    // undelegate all
    undelegate_all_test(&mut world);
    nonce += BLOCKS_PER_EPOCH * 10;
    set_block_nonce(&mut world, nonce);
    withdraw_all_test(&mut world);
    compute_withdrawn_test(&mut world);
    withdraw_test(&mut world, DELEGATOR1_ADDRESS_EXPR);

    // final checks
    let factorial = n * (n + 1) / 2;
    let total_fee = &fee * factorial;
    let remaining_delegation = &one * (delegation - factorial * 2_u64);
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&delegator1_initial_amount - &remaining_delegation - &total_fee));
    check_esdt_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, TOKEN_ID_EXPR, &remaining_delegation);
    check_available_egld_reserve(&mut world, &(&one * reserve + &total_fee));
    check_total_egld_staked(&mut world, &remaining_delegation);
}

#[test]
fn test_user_undelegations_order() {
    let mut world = setup();

    let one = exp(1, 18);
    let mut nonce = BLOCKS_PER_EPOCH;

    // delegate
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&one * 100_u64), false, true);
    delegate_all_test(&mut world);

    // undelegate in epochs 3, 1 and 2 (4 times, 2 in the same epoch, so should be merged)
    nonce = BLOCKS_PER_EPOCH * 3;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    nonce = BLOCKS_PER_EPOCH * 2;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    nonce = BLOCKS_PER_EPOCH * 4;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");

    // check undelegations orders and lengths
    check_user_undelegations_order(&mut world, DELEGATOR1_ADDRESS_EXPR);
    check_total_undelegations_order(&mut world);
    check_user_undelegations_length(&mut world, DELEGATOR1_ADDRESS_EXPR, 3);
    check_total_users_undelegations_lengths(&mut world, 3);

    // undelegate in epoch 1, 3, 5, 30 and 15
    nonce = BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    nonce = BLOCKS_PER_EPOCH * 3;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    nonce = BLOCKS_PER_EPOCH * 5;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    nonce = BLOCKS_PER_EPOCH * 30u64;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b""); // should merge the previous
    nonce = BLOCKS_PER_EPOCH * 15u64;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");

    // check undelegations orders, lengths and amount
    check_user_undelegations_order(&mut world, DELEGATOR1_ADDRESS_EXPR);
    check_total_undelegations_order(&mut world);
    check_user_undelegations_length(&mut world, DELEGATOR1_ADDRESS_EXPR, 3);
    check_total_users_undelegations_lengths(&mut world, 3);
    check_user_undelegations(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&one * 9_u64));
    check_total_users_undelegations(&mut world, &(&one * 9_u64));
}

#[test]
fn test_reserve_undelegations_order() {
    let mut world = setup();

    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;
    let one_minus_fee = &one - &fee;
    let mut nonce = BLOCKS_PER_EPOCH;

    // delegate and add reserve
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&one * 50_u64), false, true);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &(&one * 50_u64), true);

    // undelegate now in epochs 3 and 2 (3 times, 2 in the same epoch, so should be merged)
    nonce = BLOCKS_PER_EPOCH * 3;
    set_block_nonce(&mut world, nonce);
    undelegate_now_test(&mut world, false, RESERVER1_ADDRESS_EXPR, &one_minus_fee, &one, true, b"");
    nonce = BLOCKS_PER_EPOCH * 2;
    set_block_nonce(&mut world, nonce);
    undelegate_now_test(&mut world, false, RESERVER1_ADDRESS_EXPR, &one_minus_fee, &one, true, b"");
    undelegate_now_test(&mut world, false, RESERVER1_ADDRESS_EXPR, &one_minus_fee, &one, true, b"");

    // check undelegations order, length and amount
    check_total_undelegations_order(&mut world);
    check_reserve_undelegations_lengths(&mut world, 2);
    check_reserve_undelegations(&mut world, &(&one * 3_u64));

    // undelegate in epoch 30 and 15
    nonce = BLOCKS_PER_EPOCH * 30;
    set_block_nonce(&mut world, nonce);
    undelegate_now_test(&mut world, false, RESERVER1_ADDRESS_EXPR, &one_minus_fee, &one, true, b""); // should merge the previous
    nonce = BLOCKS_PER_EPOCH * 15;
    set_block_nonce(&mut world, nonce);
    undelegate_now_test(&mut world, false, RESERVER1_ADDRESS_EXPR, &one_minus_fee, &one, true, b"");

    // check undelegations order, length and amount
    check_total_undelegations_order(&mut world);
    check_reserve_undelegations_lengths(&mut world, 3);
    check_reserve_undelegations(&mut world, &(&one * 5_u64));
}

#[test]
fn test_custodial_delegation() {
    let mut world = setup();

    let one = exp(1, 18);
    const KNIGHT_ADDRESS_EXPR: &str = "address:knight";
    world.set_state_step(SetStateStep::new().put_account(KNIGHT_ADDRESS_EXPR, Account::new()));
    const HEIR_ADDRESS_EXPR: &str = "address:heir";
    world.set_state_step(SetStateStep::new().put_account(HEIR_ADDRESS_EXPR, Account::new()));
    const DELEGATOR_ADDRESS_EXPR: &str = "address:delegator";
    world.set_state_step(
        SetStateStep::new()
            .put_account(
                DELEGATOR_ADDRESS_EXPR,
                Account::new()
                    .balance(&one * 10_u64)
                    .esdt_balance(TOKEN_ID_EXPR, &one * 10_u64)
            )
    );
    set_block_nonce(&mut world, BLOCKS_PER_EPOCH);

    delegate_test(&mut world, DELEGATOR_ADDRESS_EXPR, &one, true, true);
    delegate_all_test(&mut world);
    add_to_custody_test(&mut world, DELEGATOR_ADDRESS_EXPR, &(&one * 4_u64), true);

    set_knight_test(&mut world, DELEGATOR_ADDRESS_EXPR, KNIGHT_ADDRESS_EXPR, b"");
    remove_from_custody_test(&mut world, DELEGATOR_ADDRESS_EXPR, &(&one * 3_u64), true, ERROR_KNIGHT_SET);
    cancel_knight_test(&mut world, DELEGATOR_ADDRESS_EXPR, b"");
    remove_from_custody_test(&mut world, DELEGATOR_ADDRESS_EXPR, &(&one * 9_u64 / 2_u64), true, ERROR_DUST_REMAINING);
    set_heir_test(&mut world, DELEGATOR_ADDRESS_EXPR, HEIR_ADDRESS_EXPR, 365, b"");
    remove_from_custody_test(&mut world, DELEGATOR_ADDRESS_EXPR, &(&one * 3_u64), true, b"");

    check_custodial_delegation(&mut world, DELEGATOR_ADDRESS_EXPR, &(&one * 2_u64));
    check_total_custodial_delegation(&mut world, &(&one * 2_u64));
    check_egld_balance(&mut world, DELEGATOR_ADDRESS_EXPR, &(&one * 9_u64));
    check_esdt_balance(&mut world, DELEGATOR_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&one * 9_u64));
}

#[test]
fn test_undelegate_predelegated() {
    let mut world = setup();

    let amount = exp(1, 18);
    let delegator1_initial_amount = BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;
    set_block_nonce(&mut world, BLOCKS_PER_EPOCH);

    // delegate + undelegate
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &amount, false, true);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &amount, true, b"");
    reduce_egld_to_delegate_undelegate_test(&mut world);

    // compute withdrawn
    set_block_nonce(&mut world, BLOCKS_PER_EPOCH * 11);
    compute_withdrawn_test(&mut world);

    // withdraw
    withdraw_test(&mut world, DELEGATOR1_ADDRESS_EXPR);
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &delegator1_initial_amount);
}
