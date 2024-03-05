use multiversx_sc::types::TokenIdentifier;
use xexchange_mock::{
    config::ConfigModule,
    pair_actions::initial_liq::InitialLiquidityModule,
    pair_actions::swap::SwapModule,
};

use crate::*;

pub fn add_xexchange_initial_liquidity(
    world: &mut ScenarioWorld,
    legld_amount: &num_bigint::BigUint,
    wegld_amount: &num_bigint::BigUint
) {
    let mut xexchange_liquidity: Vec<TxESDT> = Vec::new();
    xexchange_liquidity.push(TxESDT{
        esdt_token_identifier: BytesValue::from(TOKEN_ID_EXPR),
        nonce: U64Value::from(0),
        esdt_value: BigUintValue::from(legld_amount)
    });
    xexchange_liquidity.push(TxESDT{
        esdt_token_identifier: BytesValue::from(WEGLD_ID_EXPR),
        nonce: U64Value::from(0),
        esdt_value: BigUintValue::from(wegld_amount)
    });

    let xexchange_whitebox = WhiteboxContract::new(XEXCHANGE_ADDRESS_EXPR, xexchange_mock::contract_obj);
    world.whitebox_call(
        &xexchange_whitebox,
        ScCallStep::new()
            .from(XEXCHANGE_OWNER_ADDRESS_EXPR)
            .multi_esdt_transfer(xexchange_liquidity),
        |sc| {
            sc.add_initial_liquidity();
            sc.state().set(xexchange_mock::config::State::Active);
        }
    );
}

pub fn sell_on_xexchange(
    world: &mut ScenarioWorld,
    caller: &str,
    legld_amount: &num_bigint::BigUint,
    min_amount_out: &num_bigint::BigUint,
) {
    let xexchange_whitebox = WhiteboxContract::new(XEXCHANGE_ADDRESS_EXPR, xexchange_mock::contract_obj);
    world.whitebox_call(
        &xexchange_whitebox,
        ScCallStep::new()
            .from(caller)
            .esdt_transfer(TOKEN_ID_EXPR, 0, legld_amount),
        |sc| {
            let wegld_token = TokenIdentifier::from(WEGLD_ID);
            sc.swap_tokens_fixed_input(wegld_token, to_managed_biguint(min_amount_out));
        }
    );
}

pub fn buy_from_xexchange(
    world: &mut ScenarioWorld,
    caller: &str,
    wegld_amount: &num_bigint::BigUint,
    min_amount_out: &num_bigint::BigUint,
) {
    let xexchange_whitebox = WhiteboxContract::new(XEXCHANGE_ADDRESS_EXPR, xexchange_mock::contract_obj);
    world.whitebox_call(
        &xexchange_whitebox,
        ScCallStep::new()
            .from(caller)
            .esdt_transfer(WEGLD_ID_EXPR, 0, wegld_amount),
        |sc| {
            let legld_token = TokenIdentifier::from(TOKEN_ID);
            sc.swap_tokens_fixed_input(legld_token, to_managed_biguint(min_amount_out));
        }
    );
}
