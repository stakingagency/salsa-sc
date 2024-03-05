use crate::*;

pub fn wrap_egld(
    world: &mut ScenarioWorld,
    caller_address_expr: &str,
    amount: &num_bigint::BigUint
) {
    let wrap_whitebox = WhiteboxContract::new(WRAP_ADDRESS_EXPR, wrap_mock::contract_obj);
    world.whitebox_call(
        &wrap_whitebox,
        ScCallStep::new()
            .from(caller_address_expr)
            .egld_value(amount),
        |sc| {
            sc.wrap_egld();
        }
    );
}

pub fn unwrap_wegld(
    world: &mut ScenarioWorld,
    caller_address_expr: &str,
    amount: &num_bigint::BigUint
) {
    let wrap_whitebox = WhiteboxContract::new(WRAP_ADDRESS_EXPR, wrap_mock::contract_obj);
    world.whitebox_call(
        &wrap_whitebox,
        ScCallStep::new()
            .from(caller_address_expr)
            .esdt_transfer(WEGLD_ID_EXPR, 0, amount),
        |sc| {
            sc.unwrap_egld();
        }
    );
}
