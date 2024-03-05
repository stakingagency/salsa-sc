use multiversx_sc_scenario::{ScenarioWorld, WhiteboxContract};

use onedex_mock::{
    OneDexMock,
    logic::pair::PairLogicModule,
    storage::{
        common_storage::CommonStorageModule,
        pair_storage::PairStorageModule
    },
    view::ViewModule,
};

use xexchange_mock::XexchangeMock;

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
    let mut pair_id: usize = 0;
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

    // create onedex pair
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR),
        |sc| {
            pair_id = sc.create_pair(managed_token_id!(TOKEN_ID), managed_token_id!(WEGLD_ID));
            sc.pair_lp_token_id(pair_id).set(managed_token_id!(ONEDEX_LP));
            sc.lp_token_pair_id_map().insert(managed_token_id!(ONEDEX_LP), pair_id);
            sc.pair_enabled(pair_id).set(true);
            sc.pair_state(pair_id).set(onedex_mock::state::State::Active);
        }
    );

    // check onedex pair active
    world.whitebox_query(
        &onedex_whitebox, |sc| {
            assert!(sc.view_pair(pair_id).enabled);
            assert_eq!(sc.view_pair(pair_id).state, onedex_mock::state::State::Active);
            assert_eq!(sc.pair_state(pair_id).get(), onedex_mock::state::State::Active);
        }
    );

    pair_id
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
