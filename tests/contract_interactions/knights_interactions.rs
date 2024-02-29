use crate::*;
use salsa::{knights::KnightsModule, *};

use multiversx_sc::codec::multi_types::OptionalValue;
use multiversx_sc_scenario::{scenario_model::{AddressValue, ScCallStep}, ScenarioWorld, WhiteboxContract};

use crate::to_managed_biguint;

pub fn undelegate_knight_test(
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
            sc.undelegate_knight(
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

pub fn undelegate_now_knight_test(
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
            sc.undelegate_now_knight(
                managed_address!(&AddressValue::from(user).to_address()),
                to_managed_biguint(min_amount_out),
                to_managed_biguint(amount),
                OptionalValue::Some(without_arbitrage)
            );
        }
    );
}

pub fn withdraw_knight_test(
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
            sc.withdraw_knight(
                managed_address!(&AddressValue::from(user).to_address()),
            );
        }
    );
}

pub fn remove_reserve_knight_test(
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
            sc.remove_reserve_knight(
                managed_address!(&AddressValue::from(user).to_address()),
                to_managed_biguint(amount),
                OptionalValue::Some(without_arbitrage)
            );
        }
    );
}

pub fn set_knight_test(
    world: &mut ScenarioWorld,
    caller: &str,
    knight: &str,
    error: &[u8]
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call_check(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .no_expect(),
        |sc| {
            sc.set_knight(
                managed_address!(&AddressValue::from(knight).to_address()),
            );
        },
        |r| {
            assert!(r.result_message.as_bytes() == error);
        }
    );
}

pub fn cancel_knight_test(
    world: &mut ScenarioWorld,
    caller: &str,
    error: &[u8]
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call_check(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .no_expect(),
        |sc| {
            sc.cancel_knight();
        },
        |r| {
            assert!(r.result_message.as_bytes() == error);
        }
    );
}

pub fn activate_knight_test(
    world: &mut ScenarioWorld,
    caller: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.activate_knight();
        }
    );
}

pub fn deactivate_knight_test(
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
            sc.deactivate_knight(
                managed_address!(&AddressValue::from(user).to_address()),
            );
        }
    );
}

pub fn confirm_knight_test(
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
            sc.confirm_knight(
                managed_address!(&AddressValue::from(user).to_address()),
            );
        }
    );
}

pub fn remove_knight_test(
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
            sc.remove_knight(
                managed_address!(&AddressValue::from(user).to_address()),
            );
        }
    );
}
