use multiversx_sc::types::{MultiValueEncoded, TokenIdentifier};
use onedex_mock::logic::{common::CommonLogicModule, swap::SwapLogicModule};

use crate::*;

pub fn add_onedex_initial_liquidity(
    world: &mut ScenarioWorld,
    legld_amount: &num_bigint::BigUint,
    wegld_amount: &num_bigint::BigUint
) {
    let mut onedex_liquidity: Vec<TxESDT> = Vec::new();
    onedex_liquidity.push(TxESDT{
        esdt_token_identifier: BytesValue::from(TOKEN_ID_EXPR),
        nonce: U64Value::from(0),
        esdt_value: BigUintValue::from(legld_amount)
    });
    onedex_liquidity.push(TxESDT{
        esdt_token_identifier: BytesValue::from(WEGLD_ID_EXPR),
        nonce: U64Value::from(0),
        esdt_value: BigUintValue::from(wegld_amount)
    });

    let onedex_whitebox = WhiteboxContract::new(ONEDEX_ADDRESS_EXPR, onedex_mock::contract_obj);
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(ONEDEX_OWNER_ADDRESS_EXPR)
            .multi_esdt_transfer(onedex_liquidity),
        |sc| {
            let pair_id = sc.get_pair_id(&TokenIdentifier::from(TOKEN_ID), &TokenIdentifier::from(WEGLD_ID));
            sc.add_initial_liquidity();
            sc.pair_state(pair_id).set(onedex_mock::state::State::Active);
        }
    );
}

pub fn sell_on_onedex(
    world: &mut ScenarioWorld,
    caller: &str,
    legld_amount: &num_bigint::BigUint,
    min_amount_out: &num_bigint::BigUint,
) {
    let onedex_whitebox = WhiteboxContract::new(ONEDEX_ADDRESS_EXPR, onedex_mock::contract_obj);
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(caller)
            .esdt_transfer(TOKEN_ID_EXPR, 0, legld_amount),
        |sc| {
            let legld_token = TokenIdentifier::from(TOKEN_ID);
            let wegld_token = TokenIdentifier::from(WEGLD_ID);
            let mut path: MultiValueEncoded<DebugApi, TokenIdentifier<DebugApi>> = MultiValueEncoded::new();
            path.push(legld_token.clone());
            path.push(wegld_token.clone());

            sc.swap_multi_tokens_fixed_input(to_managed_biguint(min_amount_out), false, path);
        }
    );
}

pub fn buy_from_onedex(
    world: &mut ScenarioWorld,
    caller: &str,
    wegld_amount: &num_bigint::BigUint,
    min_amount_out: &num_bigint::BigUint,
) {
    let onedex_whitebox = WhiteboxContract::new(ONEDEX_ADDRESS_EXPR, onedex_mock::contract_obj);
    world.whitebox_call(
        &onedex_whitebox,
        ScCallStep::new()
            .from(caller)
            .esdt_transfer(WEGLD_ID_EXPR, 0, wegld_amount),
        |sc| {
            let legld_token = TokenIdentifier::from(TOKEN_ID);
            let wegld_token = TokenIdentifier::from(WEGLD_ID);
            let mut path: MultiValueEncoded<DebugApi, TokenIdentifier<DebugApi>> = MultiValueEncoded::new();
            path.push(wegld_token.clone());
            path.push(legld_token.clone());

            sc.swap_multi_tokens_fixed_input(to_managed_biguint(min_amount_out), false, path);
        }
    );
}
