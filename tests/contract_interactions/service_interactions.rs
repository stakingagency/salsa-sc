use multiversx_sc_scenario::{scenario_model::ScCallStep, ScenarioWorld, WhiteboxContract};
use salsa::service::ServiceModule;

use crate::*;

pub fn delegate_all_test(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    refresh_providers_test(world);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .gas_limit(GAS_LIMIT_DELEGATE_ALL),
        |sc| {
            sc.delegate_all();
        }
    );
    // check_egld_to_delegate(world, &rust_biguint!(0));
}

pub fn undelegate_all_test(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    refresh_providers_test(world);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .gas_limit(GAS_LIMIT_UNDELEGATE_ALL),
        |sc| {
            sc.undelegate_all();
        }
    );
    // check_egld_to_undelegate(world, &rust_biguint!(0));
}

pub fn claim_rewards_test(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    refresh_providers_test(world);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .gas_limit(GAS_LIMIT_CLAIM_REWARDS),
        |sc| {
            sc.claim_rewards();
        }
    );
}

pub fn withdraw_all_test(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    refresh_providers_test(world);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR)
            .gas_limit(GAS_LIMIT_WITHDRAW_ALL),
        |sc| {
            sc.withdraw_all();
        }
    );
}

pub fn compute_withdrawn_test(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR),
        |sc| {
            sc.compute_withdrawn();
        }
    );
}
