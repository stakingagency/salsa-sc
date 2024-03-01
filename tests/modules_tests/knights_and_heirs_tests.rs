use crate::*;

#[test]
fn test_knight() {
    let mut world = setup();

    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;
    let one_minus_fee = &one - &fee;
    const KNIGHT1_ADDRESS_EXPR: &str = "address:knight1";
    const KNIGHT2_ADDRESS_EXPR: &str = "address:knight1";
    world.set_state_step(SetStateStep::new().put_account(KNIGHT1_ADDRESS_EXPR, Account::new()));
    world.set_state_step(SetStateStep::new().put_account(KNIGHT2_ADDRESS_EXPR, Account::new()));

    set_block_nonce(&mut world, BLOCKS_PER_EPOCH);

    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, true, true); // true = custodial
    delegate_all_test(&mut world);

    set_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, KNIGHT1_ADDRESS_EXPR, b"");
    set_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, KNIGHT1_ADDRESS_EXPR, ERROR_KNIGHT_ALREADY_SET);
    cancel_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, b"");

    set_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, KNIGHT2_ADDRESS_EXPR, b"");
    confirm_knight_test(&mut world, KNIGHT2_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);
    cancel_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, ERROR_KNIGHT_NOT_PENDING);
    remove_knight_test(&mut world, &KNIGHT2_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);

    set_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, KNIGHT1_ADDRESS_EXPR, b"");
    undelegate_now_test(
        &mut world,
        true,
        DELEGATOR1_ADDRESS_EXPR,
        &one_minus_fee,
        &one,
        true,
        ERROR_KNIGHT_SET
    );
    confirm_knight_test(&mut world, KNIGHT1_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);
    activate_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR);
    undelegate_test(&mut world, true, DELEGATOR1_ADDRESS_EXPR, &one, true, ERROR_KNIGHT_ACTIVE);

    deactivate_knight_test(&mut world, KNIGHT1_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);
    undelegate_test(&mut world, true, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
}

#[test]
fn test_active_knigth() {
    let mut world = setup();

    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;
    let one_minus_fee = &one - &fee;
    let one_plus_fee = &one + &fee;
    let mut nonce = BLOCKS_PER_EPOCH;
    let delegator1_initial_amount = BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;
    const KNIGHT_ADDRESS_EXPR: &str = "address:knight";
    world.set_state_step(SetStateStep::new().put_account(KNIGHT_ADDRESS_EXPR, Account::new()));

    // set epoch
    set_block_nonce(&mut world, nonce);

    // delegate and add reserve
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&one * 2_u64), true, true); // true = custodial
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, true);

    // set knight, confirm and activate
    set_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, KNIGHT_ADDRESS_EXPR, b"");
    confirm_knight_test(&mut world, KNIGHT_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);
    undelegate_knight_test(&mut world, KNIGHT_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR, &one, true, ERROR_KNIGHT_NOT_ACTIVE);
    activate_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR);

    // undelegate knight, undelegate now knight and remove reserve knight
    undelegate_knight_test(&mut world, KNIGHT_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR, &one, true, b"");
    undelegate_now_knight_test(&mut world, KNIGHT_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR, &one_minus_fee, &one, true);
    undelegate_all_test(&mut world);
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    remove_reserve_knight_test(&mut world, KNIGHT_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR, &one_plus_fee, true);

    // withdraw
    nonce += BLOCKS_PER_EPOCH * 9;
    set_block_nonce(&mut world, nonce);
    withdraw_all_test(&mut world);
    compute_withdrawn_test(&mut world);
    withdraw_knight_test(&mut world, KNIGHT_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);

    // checks
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&delegator1_initial_amount - &one * 3_u64));
    check_egld_balance(&mut world, KNIGHT_ADDRESS_EXPR, &(&one * 3_u64));
}

#[test]
fn test_too_many_knight_users() {
    let mut world = setup();

    let one = exp(1, 18);
    const KNIGHT_ADDRESS_EXPR: &str = "address:knight";
    world.set_state_step(SetStateStep::new().put_account(KNIGHT_ADDRESS_EXPR, Account::new()));
    let mut nonce = BLOCKS_PER_EPOCH;

    for i in 0..MAX_KNIGHT_USERS {
        let new_user_string = "address:".to_owned() + &i.to_string();
        let new_user = new_user_string.as_str();
        world.set_state_step(SetStateStep::new().put_account(new_user, Account::new().balance(&one)));
        delegate_test(&mut world, &new_user, &one, true, true);
        set_block_nonce(&mut world, nonce);
        nonce += BLOCKS_PER_EPOCH;
        delegate_all_test(&mut world);
        set_knight_test(&mut world, &new_user, KNIGHT_ADDRESS_EXPR, b"");
    }

    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, true, true);
    set_block_nonce(&mut world, nonce);
    delegate_all_test(&mut world);
    set_knight_test(&mut world, DELEGATOR1_ADDRESS_EXPR, KNIGHT_ADDRESS_EXPR, ERROR_TOO_MANY_KNIGHT_USERS);
}

#[test]
fn test_too_many_heir_users() {
    let mut world = setup();

    let one = exp(1, 18);
    const HEIR_ADDRESS_EXPR: &str = "address:heir";
    world.set_state_step(SetStateStep::new().put_account(HEIR_ADDRESS_EXPR, Account::new()));
    let mut nonce = BLOCKS_PER_EPOCH;

    for i in 0..MAX_HEIR_USERS {
        let new_user_expr = ["address", &i.to_string()].join(":");
        let new_user = new_user_expr.as_str();
        world.set_state_step(SetStateStep::new().put_account(new_user, Account::new().balance(&one)));
        delegate_test(&mut world, &new_user, &one, true, true);
        set_block_nonce(&mut world, nonce);
        nonce += BLOCKS_PER_EPOCH;
        delegate_all_test(&mut world);
        set_heir_test(&mut world, &new_user, HEIR_ADDRESS_EXPR, 365, b"");
    }

    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, true, true);
    set_block_nonce(&mut world, nonce);
    delegate_all_test(&mut world);
    set_heir_test(&mut world, DELEGATOR1_ADDRESS_EXPR, HEIR_ADDRESS_EXPR, 365, ERROR_TOO_MANY_HEIR_USERS);
    let first_heir_expr = "address:0".to_owned();
    let first_heir = first_heir_expr.as_str();
    remove_heir_test(&mut world, HEIR_ADDRESS_EXPR, first_heir);
    set_heir_test(&mut world, DELEGATOR1_ADDRESS_EXPR, HEIR_ADDRESS_EXPR, 365, b"");
}

#[test]
fn test_entitled_heir() {
    let mut world = setup();

    let one = exp(1, 18);
    let fee = &one * UNDELEGATE_NOW_FEE / MAX_PERCENT;
    let one_minus_fee = &one - &fee;
    let one_plus_fee = &one + &fee;
    let delegator1_initial_amount = BigUintValue::from(DELEGATOR1_INITIAL_BALANCE_EXPR).value;
    const HEIR1_ADDRESS_EXPR: &str = "address:heir1";
    const HEIR2_ADDRESS_EXPR: &str = "address:heir2";
    world.set_state_step(SetStateStep::new().put_account(HEIR1_ADDRESS_EXPR, Account::new()));
    world.set_state_step(SetStateStep::new().put_account(HEIR2_ADDRESS_EXPR, Account::new()));
    let mut nonce = BLOCKS_PER_EPOCH;

    // delegate and add reserve
    set_block_nonce(&mut world, nonce);
    delegate_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&one * 2_u64), true, true); // true = custodial
    delegate_all_test(&mut world);
    add_reserve_test(&mut world, DELEGATOR1_ADDRESS_EXPR, &one, true);

    // set heir
    set_heir_test(&mut world, DELEGATOR1_ADDRESS_EXPR, HEIR2_ADDRESS_EXPR, 365u64, b"");
    cancel_heir_test(&mut world, DELEGATOR1_ADDRESS_EXPR);
    set_heir_test(&mut world, DELEGATOR1_ADDRESS_EXPR, HEIR1_ADDRESS_EXPR, 365u64, b"");

    // update last accessed
    nonce += BLOCKS_PER_EPOCH * 100;
    set_block_nonce(&mut world, nonce);
    update_last_accessed_test(&mut world, DELEGATOR1_ADDRESS_EXPR);

    // undelegate heir, undelegate now heir and remove reserve heir
    nonce += BLOCKS_PER_EPOCH * 364;
    set_block_nonce(&mut world, nonce);
    undelegate_heir_test(
        &mut world,
        HEIR1_ADDRESS_EXPR,
        DELEGATOR1_ADDRESS_EXPR,
        &one,
        true,
        ERROR_HEIR_NOT_YET_ENTITLED
    );

    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    undelegate_heir_test(
        &mut world,
        HEIR1_ADDRESS_EXPR,
        DELEGATOR1_ADDRESS_EXPR,
        &one,
        true,
        b""
    );
    undelegate_now_heir_test(
        &mut world,
        HEIR1_ADDRESS_EXPR,
        DELEGATOR1_ADDRESS_EXPR,
        &one_minus_fee,
        &one,
        true
    );
    undelegate_all_test(&mut world);
    nonce += BLOCKS_PER_EPOCH;
    set_block_nonce(&mut world, nonce);
    remove_reserve_heir_test(&mut world, HEIR1_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR, &one_plus_fee, true);

    // withdraw
    nonce += BLOCKS_PER_EPOCH * 9;
    set_block_nonce(&mut world, nonce);
    withdraw_all_test(&mut world);
    compute_withdrawn_test(&mut world);
    withdraw_heir_test(&mut world, HEIR1_ADDRESS_EXPR, DELEGATOR1_ADDRESS_EXPR);

    // checks
    check_egld_balance(&mut world, DELEGATOR1_ADDRESS_EXPR, &(&delegator1_initial_amount - &one * 3_u64));
    check_egld_balance(&mut world, HEIR1_ADDRESS_EXPR, &(&one * 3_u64));
}
