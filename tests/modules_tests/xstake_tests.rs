use salsa::{
    exchanges::lp::LpModule,
    helpers::HelpersModule
};

use crate::*;

use super::lp_tests::*;

#[test]
fn test_xstake() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 500_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    add_liquidity_and_enable_arbitrage_and_lp(&mut world, &mut nonce, &liquidity_amount);
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_xstake_active()
        }
    );

    // add legld in custody and reserves
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true, false);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, false);

    // check if half was added as equal LPs to OneDex and xExchange, then transferred to xStake
    check_egld_balance(&mut world, SALSA_ADDRESS_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, XSTAKE_ADDRESS_EXPR, ONEDEX_LP_EXPR, &(&liquidity_amount / 4_u64));
    check_esdt_balance(&mut world, XSTAKE_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &(&liquidity_amount / 4_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &rust_biguint!(0));

    // disable lp module and check profit
    nonce += BLOCKS_PER_EPOCH * 10;
    set_block_nonce(&mut world, nonce);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_lp_inactive();
            sc.take_lp_profit();
            let (egld_balance, _) = sc.get_sc_balances();
            assert!(egld_balance > to_managed_biguint(&liquidity_amount));
            assert!(sc.reserve_points().get() < sc.egld_reserve().get()); // check reserve profit
            assert!(sc.token_price() > ONE_EGLD); // check legld profit
        }
    );

    // final checks
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &liquidity_amount);
    check_esdt_balance(&mut world, XSTAKE_ADDRESS_EXPR, ONEDEX_LP_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, XSTAKE_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &rust_biguint!(0));
}
