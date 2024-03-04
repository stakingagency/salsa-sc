use salsa::exchanges::arbitrage::ArbitrageModule;

use crate::*;

use self::contract_interactions::{onedex_interactions::*, wrap_interactions::wrap_egld};

#[test]
fn test_onedex_arbitrage_delegate() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 1_000_u64;
    let delegation_amount = rust_biguint!(ONE_EGLD) * 50_u64;
    let sell_amount = rust_biguint!(ONE_EGLD) * 100_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    // add initial liquidity on onedex
    set_block_nonce(&mut world, nonce);
    wrap_egld(&mut world, ONEDEX_OWNER_ADDRESS_EXPR, &liquidity_amount);
    delegate_test(&mut world, ONEDEX_OWNER_ADDRESS_EXPR, &liquidity_amount, false, true);
    delegate_all_test(&mut world);
    add_onedex_initial_liquidity(&mut world, &liquidity_amount, &liquidity_amount);

    // enable salsa arbitrage
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.set_arbitrage_active()
    );

    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true);

    // sell on onedex
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &sell_amount, false, true);
    sell_on_onedex(&mut world, DELEGATOR1_ADDRESS_EXPR, &sell_amount, &rust_biguint!(ONE_EGLD));

    // now the LEGLD price on onedex is low, so delegate should buy instead of delegating
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &delegation_amount, false, false);

    // check if total staked is less than total delegated
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert!(sc.total_egld_staked().get() < to_managed_biguint(&(&liquidity_amount + &sell_amount + &delegation_amount)));
        }
    );
}

#[test]
fn test_onedex_arbitrage_undelegate() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 1_000_u64;
    let undelegation_amount = rust_biguint!(ONE_EGLD) * 50_u64;
    let buy_amount = rust_biguint!(ONE_EGLD) * 100_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    // add initial liquidity on onedex
    set_block_nonce(&mut world, nonce);
    wrap_egld(&mut world, ONEDEX_OWNER_ADDRESS_EXPR, &liquidity_amount);
    delegate_test(&mut world, ONEDEX_OWNER_ADDRESS_EXPR, &liquidity_amount, false, true);
    delegate_all_test(&mut world);
    add_onedex_initial_liquidity(&mut world, &liquidity_amount, &liquidity_amount);

    // enable salsa arbitrage
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| sc.set_arbitrage_active()
    );

    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true, true);

    // buy from onedex
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    wrap_egld(&mut world, DELEGATOR1_ADDRESS_EXPR, &buy_amount);
    buy_from_onedex(&mut world, DELEGATOR1_ADDRESS_EXPR, &buy_amount, &rust_biguint!(ONE_EGLD));

    // now the LEGLD price on onedex is low, so delegate should buy instead of delegating
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_test(&mut world, false, DELEGATOR1_ADDRESS_EXPR, &undelegation_amount, false, b"");

    // check if total staked is larger than total delegated
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert!(sc.total_egld_staked().get() > to_managed_biguint(&(&liquidity_amount * 2_u64 - &undelegation_amount)));
        }
    );
}
