use salsa::{
    SalsaContract,
    exchanges::{
        lp::LpModule,
        arbitrage::ArbitrageModule
    },
    helpers::HelpersModule
};

use self::contract_interactions::{
    onedex_interactions::*,
    xexchange_interactions::*,
    wrap_interactions::*
};

use crate::*;

pub fn add_liquidity_and_enable_arbitrage_and_lp(
    mut world: &mut ScenarioWorld,
    nonce: &mut u64,
    liquidity_amount: &num_bigint::BigUint
) {
    // add initial liquidity on onedex
    set_block_nonce(&mut world, *nonce);
    wrap_egld(&mut world, ONEDEX_OWNER_ADDRESS_EXPR, &liquidity_amount);
    delegate_test(&mut world, ONEDEX_OWNER_ADDRESS_EXPR, &liquidity_amount, false, true);
    delegate_all_test(&mut world);
    add_onedex_initial_liquidity(&mut world, &liquidity_amount, &liquidity_amount);

    // add initial liquidity on xexchange
    *nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, *nonce);
    wrap_egld(&mut world, XEXCHANGE_OWNER_ADDRESS_EXPR, &liquidity_amount);
    delegate_test(&mut world, XEXCHANGE_OWNER_ADDRESS_EXPR, &liquidity_amount, false, true);
    delegate_all_test(&mut world);
    add_xexchange_initial_liquidity(&mut world, &liquidity_amount, &liquidity_amount);

    // enable salsa arbitrage and lp
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_arbitrage_active();
            sc.set_lp_active();
        }
    );
}

fn disable_lp_and_take_profit(
    mut world: &mut ScenarioWorld,
    nonce: &mut u64
) {
    // disable lp module and take profit
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    let mut egld_to_delegate = rust_biguint!(0);
    let mut egld_to_undelegate = rust_biguint!(0);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_lp_inactive();
            assert!(sc.lp_state().get() == State::Inactive);
        },
    );
    world.whitebox_call_check(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .no_expect(),
        |sc| {
            sc.take_lp_profit();
        },
        |_| {}
    );
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.call_reduce_egld_to_delegate_undelegate();
            egld_to_delegate = num_bigint::BigUint::from_bytes_be(sc.egld_to_delegate().get().to_bytes_be().as_slice());
            egld_to_undelegate = num_bigint::BigUint::from_bytes_be(sc.egld_to_undelegate().get().to_bytes_be().as_slice());
        }
    );

    // check if there are no more LPs in SC
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &rust_biguint!(0));

    // delegate and undelegate everything, then check if we suffered from IL
    *nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, *nonce);
    if egld_to_delegate >= rust_biguint!(ONE_EGLD) {
        delegate_all_test(&mut world);
    }
    if egld_to_undelegate >= rust_biguint!(ONE_EGLD) {
        undelegate_all_test(&mut world);
        *nonce += UNBOND_PERIOD * BLOCKS_PER_EPOCH;
        set_block_nonce(&mut world, *nonce);
        withdraw_all_test(&mut world);
        compute_withdrawn_test(&mut world);
    }
}

#[test]
fn test_add_remove_lp() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 500_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    add_liquidity_and_enable_arbitrage_and_lp(&mut world, &mut nonce, &liquidity_amount);

    // add legld in custody and reserves
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true, false);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, false);

    // check if half was added as equal LPs to OneDex and xExchange
    check_egld_balance(&mut world, SALSA_ADDRESS_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &(&liquidity_amount / 4_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &(&liquidity_amount / 4_u64));

    // disable lp module
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_call(
        &salsa_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_lp_inactive();
        }
    );

    // check if legld in custody and reserves are restored
    check_egld_balance(&mut world, SALSA_ADDRESS_EXPR, &(&liquidity_amount));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&liquidity_amount));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &rust_biguint!(0));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &rust_biguint!(0));
}

#[test]
fn test_add_remove_imbalanced_lp() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 500_u64;
    let trade_amount = rust_biguint!(ONE_EGLD) * 100_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    add_liquidity_and_enable_arbitrage_and_lp(&mut world, &mut nonce, &liquidity_amount);

    // add legld in custody and reserves
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true, false);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, false);

    // check if half was added as equal LPs to OneDex and xExchange
    check_egld_balance(&mut world, SALSA_ADDRESS_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &(&liquidity_amount / 4_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &(&liquidity_amount / 4_u64));

    // buy from onedex
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    wrap_egld(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount);
    buy_from_onedex(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount, &rust_biguint!(ONE_EGLD));

    // sell on xexchange
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount, false, true);
    delegate_all_test(&mut world);
    sell_on_xexchange(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount, &rust_biguint!(ONE_EGLD));

    disable_lp_and_take_profit(&mut world, &mut nonce);

    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (egld_balance, legld_balance) = sc.get_sc_balances();
            let amount = to_managed_biguint(&liquidity_amount);
            assert!(egld_balance >= amount);
            assert!(legld_balance == amount);
        }
    );
}

#[test]
fn test_add_remove_lp_legld_high() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 500_u64;
    let trade_amount = rust_biguint!(ONE_EGLD) * 100_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    add_liquidity_and_enable_arbitrage_and_lp(&mut world, &mut nonce, &liquidity_amount);

    // add legld in custody and reserves
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true, false);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, false);

    // check if half was added as equal LPs to OneDex and xExchange
    check_egld_balance(&mut world, SALSA_ADDRESS_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &(&liquidity_amount / 4_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &(&liquidity_amount / 4_u64));

    // buy from onedex
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    wrap_egld(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount);
    buy_from_onedex(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount, &rust_biguint!(ONE_EGLD));

    disable_lp_and_take_profit(&mut world, &mut nonce);

    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (egld_balance, legld_balance) = sc.get_sc_balances();
            let amount = to_managed_biguint(&liquidity_amount);
            assert!(egld_balance >= amount);
            assert!(legld_balance == amount);
        }
    );
}

#[test]
fn test_add_remove_lp_legld_low() {
    let mut world = setup();

    let liquidity_amount = rust_biguint!(ONE_EGLD) * 500_u64;
    let trade_amount = rust_biguint!(ONE_EGLD) * 100_u64;
    let mut nonce = BLOCKS_PER_EPOCH;

    add_liquidity_and_enable_arbitrage_and_lp(&mut world, &mut nonce, &liquidity_amount);

    // add legld in custody and reserves
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, true, false);
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, RESERVER1_ADDRESS_EXPR, &liquidity_amount, false);

    // check if half was added as equal LPs to OneDex and xExchange
    check_egld_balance(&mut world, SALSA_ADDRESS_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, TOKEN_ID_EXPR, &(&liquidity_amount / 2_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, ONEDEX_LP_EXPR, &(&liquidity_amount / 4_u64));
    check_esdt_balance(&mut world, SALSA_ADDRESS_EXPR, XEXCHANGE_LP_EXPR, &(&liquidity_amount / 4_u64));

    // sell on xexchange
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount, false, true);
    delegate_all_test(&mut world);
    sell_on_xexchange(&mut world, DELEGATOR1_ADDRESS_EXPR, &trade_amount, &rust_biguint!(ONE_EGLD));

    disable_lp_and_take_profit(&mut world, &mut nonce);

    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let (egld_balance, legld_balance) = sc.get_sc_balances();
            let amount = to_managed_biguint(&liquidity_amount);
            assert!(egld_balance >= amount);
            assert!(legld_balance == amount);
        }
    );
}
