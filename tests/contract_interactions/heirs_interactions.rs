use crate::*;
use salsa::{heirs::HeirsModule, *};

use multiversx_sc::codec::multi_types::OptionalValue;
use multiversx_sc_scenario::{scenario_model::{AddressValue, ScCallStep}, ScenarioWorld, WhiteboxContract};

use crate::to_managed_biguint;

pub fn undelegate_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
    user: &str,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool,
    error: &[u8]
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call_check(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .no_expect(),
        |sc| {
            sc.undelegate_heir(
                managed_address!(&AddressValue::from(user).to_address()),
                to_managed_biguint(amount),
                OptionalValue::Some(without_arbitrage)
            );
        },
        |r| {
            assert!(r.result_message.as_bytes() == error);
        }
    );
}

pub fn undelegate_now_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
    user: &str,
    min_amount_out: &num_bigint::BigUint,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.undelegate_now_heir(
                managed_address!(&AddressValue::from(user).to_address()),
                to_managed_biguint(min_amount_out),
                to_managed_biguint(amount),
                OptionalValue::Some(without_arbitrage)
            );
        }
    );
}

pub fn withdraw_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
    user: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.withdraw_heir(
                managed_address!(&AddressValue::from(user).to_address()),
            );
        }
    );
}

pub fn remove_reserve_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
    user: &str,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.remove_reserve_heir(
                managed_address!(&AddressValue::from(user).to_address()),
                to_managed_biguint(amount),
                OptionalValue::Some(without_arbitrage)
            );
        }
    );
}

pub fn set_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
    heir: &str,
    inheritance_epochs: u64,
    error: &[u8]
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call_check(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .no_expect(),
        |sc| {
            sc.set_heir(
                managed_address!(&AddressValue::from(heir).to_address()),
                inheritance_epochs
            );
        },
        |r| {
            assert!(r.result_message.as_bytes() == error);
        }
    );
}

pub fn cancel_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.cancel_heir();
        }
    );
}

pub fn remove_heir_test(
    world: &mut ScenarioWorld,
    caller: &str,
    user: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.remove_heir(
                managed_address!(&AddressValue::from(user).to_address())
            );
        }
    );
}

pub fn update_last_accessed_test(
    world: &mut ScenarioWorld,
    caller: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.update_last_accessed();
        }
    );
}
