use multiversx_sc_scenario::{managed_address, rust_biguint, scenario_model::AddressValue, ScenarioWorld, WhiteboxContract};
use salsa::{common::config::ConfigModule, providers::ProvidersModule};

use crate::*;

pub fn get_provider_total_stake(world: &mut ScenarioWorld, provider_address_expr: &str) -> num_bigint::BigUint {
    let mut result = rust_biguint!(0);
    refresh_provider_test(world, provider_address_expr);
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let provider_address = managed_address!(&AddressValue::from(provider_address_expr).to_address());
            assert!(sc.view_provider_updated(&provider_address));

            let provider = sc.get_provider(&provider_address);
            result = num_bigint::BigUint::from_bytes_be(provider.total_stake.to_bytes_be().as_slice());
        }
    );

    result
}
