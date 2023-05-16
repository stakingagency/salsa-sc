use salsa::{*};
use salsa::config::ConfigModule;

use delegation_mock::*;

use multiversx_sc::{
    types::{
        Address,
        EsdtLocalRole
    },
    storage::mappers::StorageTokenWrapper
};

use std::ops::Mul;

use multiversx_sc_scenario::{
    managed_address, managed_token_id, num_bigint, rust_biguint, testing_framework::*, DebugApi,
};

use crate::consts::*;

pub static ESDT_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::Mint,
    EsdtLocalRole::Burn,
    EsdtLocalRole::Transfer,
];

pub struct SalsaContractSetup<SalsaContractObjBuilder>
where
    SalsaContractObjBuilder: 'static + Copy + Fn() -> salsa::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub salsa_wrapper: ContractObjWrapper<salsa::ContractObj<DebugApi>, SalsaContractObjBuilder>
}

impl<SalsaContractObjBuilder> SalsaContractSetup<SalsaContractObjBuilder>
where
    SalsaContractObjBuilder: 'static + Copy + Fn() -> salsa::ContractObj<DebugApi>,
{
    pub fn new(salsa_builder: SalsaContractObjBuilder) -> Self {
        let big_zero = rust_biguint!(0u64);
        let mut blockchain_wrapper = BlockchainStateWrapper::new();

        let owner_address = blockchain_wrapper.create_user_account(&big_zero);
        blockchain_wrapper
            .set_egld_balance(&owner_address, &Self::exp18(1000));

        // deploy SALSA
        let salsa_wrapper = blockchain_wrapper.create_sc_account(
            &big_zero,
            Some(&owner_address),
            salsa_builder,
            SALSA_WASM_PATH,
        );

        // init SALSA
        blockchain_wrapper
            .execute_tx(&owner_address, &salsa_wrapper, &big_zero, |sc|{
                sc.init();
            })
            .assert_ok();

        // create liquid token
        blockchain_wrapper
            .execute_tx(&owner_address, &salsa_wrapper, &big_zero, |sc|{
                sc.liquid_token_id().set_token_id(managed_token_id!(TOKEN_ID));
            })
            .assert_ok();
        
        blockchain_wrapper.set_esdt_local_roles(
            salsa_wrapper.address_ref(),
            TOKEN_ID,
            ESDT_ROLES
        );

        // deploy delegation sc
        let delegation_wrapper = blockchain_wrapper.create_sc_account(
            &big_zero,
            Some(&owner_address),
            delegation_mock::contract_obj,
            "delegation-mock.wasm",
        );

        blockchain_wrapper
            .execute_tx(&owner_address, &delegation_wrapper, &big_zero, |sc| {
                sc.init();
            })
            .assert_ok();

        blockchain_wrapper
            .execute_tx(
                &owner_address,
                &delegation_wrapper,
                &Self::exp18(1000),
                |sc| {
                    sc.deposit_egld();
                },
            )
            .assert_ok();

        // set provider address
        let provider = delegation_wrapper.address_ref().clone();
        blockchain_wrapper
            .execute_tx(&owner_address, &salsa_wrapper, &big_zero, |sc|{
                let provider_address = managed_address!(&provider);
                sc.set_provider_address(provider_address)
            })
            .assert_ok();

        // set unbond period
        blockchain_wrapper
            .execute_tx(&owner_address, &salsa_wrapper, &big_zero, |sc|{
                let unbond_period = 10_u64;
                sc.set_unbond_period(unbond_period)
            })
            .assert_ok();

        // set undelegate now fee
        blockchain_wrapper
            .execute_tx(&owner_address, &salsa_wrapper, &big_zero, |sc|{
                let fee = 200_u64;
                sc.set_undelegate_now_fee(fee)
            })
            .assert_ok();

        // set state active
        blockchain_wrapper
            .execute_tx(&owner_address, &salsa_wrapper, &big_zero, |sc|{
                sc.set_state_active();
            })
            .assert_ok();

        SalsaContractSetup {
            blockchain_wrapper,
            owner_address,
            salsa_wrapper,
        }
    }

    pub fn exp18(value: u64) -> num_bigint::BigUint {
        value.mul(rust_biguint!(10).pow(18))
    }
}
