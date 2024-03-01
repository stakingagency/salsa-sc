use crate::*;
use salsa::*;

use multiversx_sc::codec::multi_types::OptionalValue;
use multiversx_sc_scenario::{
    scenario_model::SetStateStep, ScenarioWorld
};

pub fn set_block_nonce(
    world: &mut ScenarioWorld,
    block_nonce_expr: u64
) {
    world.set_state_step(SetStateStep::new().block_nonce(block_nonce_expr));
    world.set_state_step(SetStateStep::new().block_epoch(block_nonce_expr / BLOCKS_PER_EPOCH));
}

pub fn set_state_active_test(world: &mut ScenarioWorld) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_state_active();
        }
    );
}

pub fn set_state_inactive_test(world: &mut ScenarioWorld) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_state_inactive();
        }
    );
}

pub fn delegate_test(
    world: &mut ScenarioWorld,
    caller: &str,
    amount: &num_bigint::BigUint,
    with_custody: bool,
    without_arbitrage: bool
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .egld_value(amount),
        |sc| {
            sc.delegate(with_custody, OptionalValue::Some(without_arbitrage));
        }
    );
}

pub fn undelegate_test(
    world: &mut ScenarioWorld,
    from_custody: bool,
    caller: &str,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool,
    error: &[u8]
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    if from_custody {
        world.whitebox_call_check(
            &salsa_whitebox,
            ScCallStep::new()
                .from(caller)
                .no_expect(),
            |sc| {
                sc.undelegate(
                    OptionalValue::Some(to_managed_biguint(amount)),
                    OptionalValue::Some(without_arbitrage)
                );
            },
            |r| {
                assert!(r.result_message.as_bytes() == error);
            }
        );
    } else {
        world.whitebox_call(
            &salsa_whitebox,
            ScCallStep::new()
                .from(caller)
                .esdt_transfer(TOKEN_ID_EXPR, 0, amount),
            |sc| {
                sc.undelegate(
                    OptionalValue::None,
                    OptionalValue::Some(without_arbitrage)
                );
            }
        );
    }
}

pub fn undelegate_now_test(
    world: &mut ScenarioWorld,
    from_custody: bool,
    caller: &str,
    min_amount_out: &num_bigint::BigUint,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool,
    error: &[u8]
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    if from_custody {
        world.whitebox_call_check(
            &salsa_whitebox,
            ScCallStep::new()
                .from(caller)
                .no_expect(),
            |sc| {
                sc.undelegate_now(
                    to_managed_biguint(min_amount_out),
                    OptionalValue::Some(to_managed_biguint(amount)),
                    OptionalValue::Some(without_arbitrage)
                );
            },
            |r| {
                assert!(r.result_message.as_bytes() == error);
            }
            );
    } else {
        world.whitebox_call(
            &salsa_whitebox,
            ScCallStep::new()
                .from(caller)
                .esdt_transfer(TOKEN_ID_EXPR, 0, amount),
            |sc| {
                sc.undelegate_now(
                    to_managed_biguint(min_amount_out),
                    OptionalValue::None,
                    OptionalValue::Some(without_arbitrage)
                );
            }
        );
    }
}

pub fn withdraw_test(
    world: &mut ScenarioWorld,
    caller: &str,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.withdraw();
        }
    );
}

pub fn add_to_custody_test(
    world: &mut ScenarioWorld,
    caller: &str,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .esdt_transfer(TOKEN_ID_EXPR, 0, amount),
        |sc| {
            sc.add_to_custody(OptionalValue::Some(without_arbitrage));
        }
    );
}

pub fn remove_from_custody_test(
    world: &mut ScenarioWorld,
    caller: &str,
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
            sc.remove_from_custody(to_managed_biguint(amount), OptionalValue::Some(without_arbitrage));
        },
        |r| {
            assert!(r.result_message.as_bytes() == error);
        }
    );
}

pub fn add_reserve_test(
    world: &mut ScenarioWorld,
    caller: &str,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller)
            .egld_value(amount),
        |sc| {
            sc.add_reserve(OptionalValue::Some(without_arbitrage));
        }
    );
}

pub fn remove_reserve_test(
    world: &mut ScenarioWorld,
    caller: &str,
    amount: &num_bigint::BigUint,
    without_arbitrage: bool
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(caller),
        |sc| {
            sc.remove_reserve(to_managed_biguint(amount), OptionalValue::Some(without_arbitrage));
        }
    );
}

pub fn reduce_egld_to_delegate_undelegate_test(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(CALLER_ADDRESS_EXPR),
        |sc| {
            sc.call_reduce_egld_to_delegate_undelegate();
        }
    );
}
