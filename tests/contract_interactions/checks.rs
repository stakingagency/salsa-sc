use multiversx_sc_scenario::{scenario_model::{AddressValue, CheckAccount, CheckStateStep}, ScenarioWorld, WhiteboxContract};

use crate::*;

pub fn check_egld_balance(
    world: &mut ScenarioWorld,
    user: &str,
    amount: &num_bigint::BigUint,
) {
    world.check_state_step(
        CheckStateStep::new().put_account(
            user,
            CheckAccount::new().balance(amount.to_string().as_str())
        )
    );
}

pub fn check_esdt_balance(
    world: &mut ScenarioWorld,
    user: &str,
    token: &str,
    amount: &num_bigint::BigUint,
) {
    world.check_state_step(
        CheckStateStep::new().put_account(
            user,
            CheckAccount::new().esdt_balance(token, amount)
        )
    );
}

pub fn check_total_egld_staked(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.total_egld_staked().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_liquid_token_supply(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.liquid_token_supply().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_total_withdrawn_egld(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.total_withdrawn_egld().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_user_withdrawn_egld(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.user_withdrawn_egld().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_user_reserve(
    world: &mut ScenarioWorld,
    user: &str,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.get_user_reserve(&managed_address!(&AddressValue::from(user).to_address())), to_managed_biguint(amount));
        }
    );
}

pub fn check_user_reserve_points(
    world: &mut ScenarioWorld,
    user: &str,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.users_reserve_points(&managed_address!(&AddressValue::from(user).to_address())).get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_egld_to_delegate(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.egld_to_delegate().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_egld_to_undelegate(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.egld_to_undelegate().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_egld_reserve(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.egld_reserve().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_available_egld_reserve(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.available_egld_reserve().get(), to_managed_biguint(amount));
        }
    );
}

pub fn check_reserve_undelegations(
    world: &mut ScenarioWorld,
    amount: &num_bigint::BigUint,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let mut total = BigUint::zero();
            let undelegations = sc.lreserve_undelegations();
            for node in undelegations.iter() {
                let undelegation = node.into_value();
                total += undelegation.amount;
            }
            assert_eq!(total, to_managed_biguint(amount));
        }
    );
}

pub fn check_user_undelegations(world: &mut ScenarioWorld, user: &str, amount: &num_bigint::BigUint) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
                let mut total = BigUint::zero();
                let undelegations =
                    sc.luser_undelegations(&managed_address!(&AddressValue::from(user).to_address()));
                for node in undelegations.iter() {
                    let undelegation = node.into_value();
                    total += undelegation.amount;
                }
                assert_eq!(total, to_managed_biguint(amount));
            }
        );
}

pub fn check_total_users_undelegations(world: &mut ScenarioWorld, amount: &num_bigint::BigUint) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
                let mut total = BigUint::zero();
                let undelegations = sc.ltotal_user_undelegations();
                for node in undelegations.iter() {
                    let undelegation = node.into_value();
                    total += undelegation.amount;
                }
                assert_eq!(total, to_managed_biguint(amount));
            }
        );
}

pub fn check_user_undelegations_order(
    world: &mut ScenarioWorld,
    user: &str
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let mut last_epoch = 0u64;
            let undelegations =
                sc.luser_undelegations(&managed_address!(&AddressValue::from(user).to_address()));
            for node in undelegations.iter() {
                let undelegation = node.into_value();
                assert_eq!(
                    last_epoch <= undelegation.unbond_epoch,
                    true
                );
                last_epoch = undelegation.unbond_epoch;
            }
        }
    );
}

pub fn check_total_undelegations_order(world: &mut ScenarioWorld) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let mut last_epoch = 0u64;
            let undelegations = sc.ltotal_user_undelegations();
            for node in undelegations.iter() {
                let undelegation = node.into_value();
                assert_eq!(
                    last_epoch <= undelegation.unbond_epoch,
                    true
                );
                last_epoch = undelegation.unbond_epoch;
            }
            last_epoch = 0u64;
            let undelegations = sc.lreserve_undelegations();
            for node in undelegations.iter() {
                let undelegation = node.into_value();
                assert_eq!(
                    last_epoch <= undelegation.unbond_epoch,
                    true
                );
                last_epoch = undelegation.unbond_epoch;
            }
        }
    );
}

pub fn check_user_undelegations_length(world: &mut ScenarioWorld, user: &str, len: usize) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.luser_undelegations(&managed_address!(&AddressValue::from(user).to_address())).len(), len);
        }
    );
}

pub fn check_total_users_undelegations_lengths(world: &mut ScenarioWorld, len: usize) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.ltotal_user_undelegations().len(), len);
        }
    );
}

pub fn check_reserve_undelegations_lengths(world: &mut ScenarioWorld, len: usize) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.lreserve_undelegations().len(), len,);
        }
    );
}

pub fn check_custodial_delegation(world: &mut ScenarioWorld, user: &str, amount: &num_bigint::BigUint) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let delegation =
                sc.user_delegation(&managed_address!(&AddressValue::from(user).to_address())).get();
            assert_eq!(delegation == to_managed_biguint(amount), true);
        }
    );
}

pub fn check_total_custodial_delegation(world: &mut ScenarioWorld, amount: &num_bigint::BigUint) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let delegation = sc.legld_in_custody().get();
            assert_eq!(delegation == to_managed_biguint(amount), true);
        }
    );
}

pub fn check_providers_updated(
    world: &mut ScenarioWorld,
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            assert_eq!(sc.view_providers_updated(), true);
        }
    );
}

pub fn check_provider_active(
    world: &mut ScenarioWorld,
    provider: &str
) {
    let salsa_whitebox = WhiteboxContract::new(SALSA_ADDRESS_EXPR, salsa::contract_obj);
    world.whitebox_query(
        &salsa_whitebox, |sc| {
            let provider_info =
                sc.get_provider(&managed_address!(&AddressValue::from(provider).to_address()));
            assert_eq!(provider_info.is_active(), true);
        }
    );
}
