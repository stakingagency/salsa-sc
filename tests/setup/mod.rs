use multiversx_sc::types::{ManagedVec, TokenIdentifier};
use multiversx_sc_scenario::{ScenarioWorld, WhiteboxContract};

use onedex_mock::{
    logic::{
        liquidity::LiquidityLogicModule,
        pair::PairLogicModule
    },
    storage::{
        common_storage::CommonStorageModule,
        pair_storage::PairStorageModule
    },
    view::ViewModule
};

use onedex_mock::OneDexMock;
use xexchange_mock::XexchangeMock;
use xstake_mock::XStakeMock;

use crate::*;

pub fn setup_providers(world: &mut ScenarioWorld) {
    let delegation1_whitebox = WhiteboxContract::new(DELEGATION1_ADDRESS_EXPR, delegation_mock::contract_obj);
    let delegation2_whitebox = WhiteboxContract::new(DELEGATION2_ADDRESS_EXPR, delegation_mock::contract_obj);
    let delegation_code = world.code_expression(DELEGATION_PATH_EXPR);

    // deploy providers
    world.whitebox_deploy(
        &delegation1_whitebox,
        ScDeployStep::new()
            .from(DELEGATOR1_ADDRESS_EXPR)
            .code(delegation_code.clone()),
        |sc| {
            sc.init(
                BigUint::from(ONE_EGLD) * DELEGATION1_TOTAL_STAKE,
                DELEGATION1_NODES_COUNT,
                DELEGATION1_FEE,
                DELEGATION1_APR
            )
        }
    );

    world.whitebox_deploy(
        &delegation2_whitebox,
        ScDeployStep::new()
            .from(DELEGATOR2_ADDRESS_EXPR)
            .code(delegation_code),
        |sc| {
            sc.init(
                BigUint::from(ONE_EGLD) * DELEGATION2_TOTAL_STAKE,
                DELEGATION2_NODES_COUNT,
                DELEGATION2_FEE,
                DELEGATION2_APR
            )
        }
    );
}

pub fn setup_wrap_sc(world: &mut ScenarioWorld) {
    let wrap_whitebox = WhiteboxContract::new(WRAP_ADDRESS_EXPR, wrap_mock::contract_obj);
    world.whitebox_call(
        &wrap_whitebox,
        ScCallStep::new()
            .from(WRAP_OWNER_ADDRESS_EXPR),
        |sc| sc.wrapped_egld_token_id().set(managed_token_id!(WEGLD_ID)),
    );
}

pub fn setup_onedex_sc(world: &mut ScenarioWorld) -> usize {
    let onedex_whitebox = WhiteboxContract::new(ONEDEX_ADDRESS_EXPR, onedex_mock::contract_obj);
    let wrap_whitebox = WhiteboxContract::new(WRAP_ADDRESS_EXPR, wrap_mock::contract_obj);
    let mut legld_pair_id: usize = 0;
    let mut rone_pair_id: usize = 0;
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR),
        |sc| {
            sc.init(
                managed_token_id!(WEGLD_ID),
                managed_address!(&Address::from_slice(wrap_whitebox.address_expr.to_address().as_bytes()))
            );
            sc.add_main_pair(managed_token_id!(WEGLD_ID));
        }
    );

    // create legld pair
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR),
        |sc| {
            legld_pair_id = sc.create_pair(managed_token_id!(TOKEN_ID), managed_token_id!(WEGLD_ID));
            sc.pair_lp_token_id(legld_pair_id).set(managed_token_id!(ONEDEX_LP));
            sc.lp_token_pair_id_map().insert(managed_token_id!(ONEDEX_LP), legld_pair_id);
            sc.pair_enabled(legld_pair_id).set(true);
            sc.pair_state(legld_pair_id).set(onedex_mock::state::State::Active);
        }
    );

    // create rone pair
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR),
        |sc| {
            rone_pair_id = sc.create_pair(managed_token_id!(REWARD_TOKEN), managed_token_id!(WEGLD_ID));
            sc.pair_lp_token_id(rone_pair_id).set(managed_token_id!(RONE_LP));
            sc.lp_token_pair_id_map().insert(managed_token_id!(RONE_LP), rone_pair_id);
            sc.pair_enabled(rone_pair_id).set(true);
            sc.pair_state(rone_pair_id).set(onedex_mock::state::State::Active);
        }
    );
    // add rone liquidity
    let rone_liquidity = rust_biguint!(ONE_EGLD) * RONE_LIQUIDITY_EGLD;
    let mut onedex_liquidity: Vec<TxESDT> = Vec::new();
    onedex_liquidity.push(TxESDT{
        esdt_token_identifier: BytesValue::from(REWARD_TOKEN_EXPR),
        nonce: U64Value::from(0),
        esdt_value: BigUintValue::from(&rone_liquidity * RONES_PER_EGLD)
    });
    onedex_liquidity.push(TxESDT{
        esdt_token_identifier: BytesValue::from(WEGLD_ID_EXPR),
        nonce: U64Value::from(0),
        esdt_value: BigUintValue::from(&rone_liquidity)
    });
    world.whitebox_call(
        &wrap_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR)
            .egld_value(&rone_liquidity),
        |sc| {
            _ = sc.wrap_egld()
        }
    );
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR)
            .multi_esdt_transfer(onedex_liquidity),
        |sc| {
            sc.add_initial_liquidity();
            sc.pair_state(rone_pair_id).set(onedex_mock::state::State::Active);
        }
    );

    // check pairs active
    world.whitebox_query(
        &onedex_whitebox, |sc| {
            assert!(sc.view_pair(legld_pair_id).enabled);
            assert_eq!(sc.view_pair(legld_pair_id).state, onedex_mock::state::State::Active);
            assert_eq!(sc.pair_state(legld_pair_id).get(), onedex_mock::state::State::Active);

            assert!(sc.view_pair(rone_pair_id).enabled);
            assert_eq!(sc.view_pair(rone_pair_id).state, onedex_mock::state::State::Active);
            assert_eq!(sc.pair_state(rone_pair_id).get(), onedex_mock::state::State::Active);
        }
    );

    legld_pair_id
}

pub fn setup_xexchange_sc(world: &mut ScenarioWorld) {
    let xexchange_whitebox = WhiteboxContract::new(XEXCHANGE_ADDRESS_EXPR, xexchange_mock::contract_obj);
    world.whitebox_call(
        &xexchange_whitebox,
        ScCallStep::new()
            .from(XEXCHANGE_OWNER_ADDRESS_EXPR),
        |sc| {
            sc.init(
                managed_token_id!(TOKEN_ID),
                managed_token_id!(WEGLD_ID)
            );
            sc.set_lp_token_identifier(managed_token_id!(XEXCHANGE_LP));
        }
    );
}

pub fn setup_xstake(world: &mut ScenarioWorld) -> (usize, usize) {
    let xstake_whitebox = WhiteboxContract::new(XSTAKE_ADDRESS_EXPR, xstake_mock::contract_obj);
    let xstake_code = world.code_expression(XSTAKE_PATH_EXPR);
    let mut stake1_id: usize = 0;
    let mut stake2_id: usize = 0;
    world.whitebox_deploy(
        &xstake_whitebox,
        ScDeployStep::new()
            .from(XSTAKE_OWNER_ADDRESS_EXPR)
            .code(xstake_code.clone()),
        |sc| {
            sc.init();

            // create stakes
            let mut stake1_tokens: ManagedVec<DebugApi, TokenIdentifier<DebugApi>> = ManagedVec::new();
            stake1_tokens.push(TokenIdentifier::from(ONEDEX_LP));
            let mut reward_tokens: ManagedVec<DebugApi, TokenIdentifier<DebugApi>> = ManagedVec::new();
            reward_tokens.push(TokenIdentifier::from(REWARD_TOKEN));
            let mut stake_ratios: ManagedVec<DebugApi, BigUint<DebugApi>> = ManagedVec::new();
            stake_ratios.push(BigUint::from(ONE_EGLD));
            let mut stake2_tokens: ManagedVec<DebugApi, TokenIdentifier<DebugApi>> = ManagedVec::new();
            stake2_tokens.push(TokenIdentifier::from(XEXCHANGE_LP));
            stake1_id = sc.create_stake(stake1_tokens, stake_ratios.clone(), reward_tokens.clone());
            stake2_id = sc.create_stake(stake2_tokens, stake_ratios, reward_tokens);
        }
    );
    // add stakes rewards
    world.whitebox_call(
        &xstake_whitebox,
        ScCallStep::new()
            .from(XSTAKE_OWNER_ADDRESS_EXPR)
            .esdt_transfer(REWARD_TOKEN_EXPR, 0, XSTAKE_REWARDS_AMOUNT_EXPR),
        |sc| {
            sc.add_stake_rewards(stake1_id);
        }
    );
    world.whitebox_call(
        &xstake_whitebox,
        ScCallStep::new()
            .from(XSTAKE_OWNER_ADDRESS_EXPR)
            .esdt_transfer(REWARD_TOKEN_EXPR, 0, XSTAKE_REWARDS_AMOUNT_EXPR),
        |sc| {
            sc.add_stake_rewards(stake2_id);
        }
    );
    // setup stakes
    world.whitebox_call(
        &xstake_whitebox,
        ScCallStep::new()
            .from(XSTAKE_OWNER_ADDRESS_EXPR),
        |sc| {
            sc.change_stake_end(stake1_id, BLOCKS_PER_EPOCH * 365);
            sc.change_stake_end(stake2_id, BLOCKS_PER_EPOCH * 365);
            sc.set_stake_state(stake1_id, xstake_mock::storage::State::Active);
            sc.set_stake_state(stake2_id, xstake_mock::storage::State::Active);
        }
    );

    (stake1_id, stake2_id)
}
