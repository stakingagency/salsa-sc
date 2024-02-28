use multiversx_sc_scenario::{managed_address, scenario_model::{AddressValue, ScCallStep}, ScenarioWorld, WhiteboxContract};
use salsa::{common::config::State, providers::ProvidersModule};

use crate::*;

pub fn add_provider_test(
    world: &mut ScenarioWorld,
    caller: &str,
    provider: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.add_provider(
                managed_address!(&AddressValue::from(provider).to_address())
            );
        }
    );
}

pub fn remove_provider_test(
    world: &mut ScenarioWorld,
    caller: &str,
    provider: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.remove_provider(
                &managed_address!(&AddressValue::from(provider).to_address())
            );
        }
    );
}

pub fn refresh_provider_test(
    world: &mut ScenarioWorld,
    caller: &str,
    provider: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .gas_limit(GAS_LIMIT_REFRESH_PROVIDER),
            |sc| {
            sc.refresh_provider(
                managed_address!(&AddressValue::from(provider).to_address())
            );
        }
    );
}

pub fn refresh_providers_test(
    world: &mut ScenarioWorld,
    caller: &str
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    let mut up_to_date = false;
    while !up_to_date {
        world.whitebox_call(
            &salsa_whitebox,
            ScCallStep::new()
                .from(caller)
                .gas_limit(GAS_LIMIT_REFRESH_PROVIDERS),
            |sc| {
                up_to_date = sc.refresh_providers();
            },
        );
    }
    check_providers_updated(world);
}

pub fn set_provider_state_test(
    world: &mut ScenarioWorld,
    caller: &str,
    provider: &str,
    new_state: State
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.set_provider_state(
                managed_address!(&AddressValue::from(provider).to_address()),
                new_state
            );
        }
    );
}
