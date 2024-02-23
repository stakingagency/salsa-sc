pub mod consts;

use consts::*;

use salsa::{common::config::ProxyTrait, ProxyTrait as _};

use multiversx_sc::{codec::multi_types::OptionalValue, types::Address};

use multiversx_sc_scenario::{
    api::StaticApi,
    scenario_model::{
        Account, AddressValue, CheckAccount, CheckStateStep, ScCallStep, ScDeployStep, ScQueryStep,
        SetStateStep, TxExpect,
    },
    ContractInfo, ScenarioWorld,
};

use num_bigint::BigUint;

type SalsaContract = ContractInfo<salsa::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace(".");

    blockchain.register_contract(
        SALSA_PATH_EXPR,
        salsa::ContractBuilder,
    );
    blockchain
}

struct SalsaTestState {
    world: ScenarioWorld,
    salsa_contract: SalsaContract,
    first_user_address: Address,
    second_user_address: Address,
}

impl SalsaTestState {
    fn new() -> Self {
        let mut world = world();

        world.set_state_step(
            SetStateStep::new()
                .put_account(
                    OWNER_ADDRESS_EXPR,
                    Account::new()
                        .nonce(1)
                        .balance("1_000_000_000_000_000_000")
                )
                .new_address(OWNER_ADDRESS_EXPR, 1, SALSA_ADDRESS_EXPR)
                .put_account(
                    FIRST_USER_ADDRESS_EXPR,
                    Account::new()
                        .nonce(1)
                        .balance("100_000_000_000_000_000_000")
                )
                .put_account(
                    SECOND_USER_ADDRESS_EXPR,
                    Account::new()
                        .nonce(1)
                        .balance("100_000_000_000_000_000_000")
                ),
        );

        let salsa_contract =
            SalsaContract::new(SALSA_ADDRESS_EXPR);

        let first_user_address = AddressValue::from(FIRST_USER_ADDRESS_EXPR).to_address();
        let second_user_address = AddressValue::from(SECOND_USER_ADDRESS_EXPR).to_address();

        Self {
            world,
            salsa_contract,
            first_user_address,
            second_user_address,
        }
    }

    fn deploy(&mut self) -> &mut Self {
        let salsa_code = self.world.code_expression(SALSA_PATH_EXPR);

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(salsa_code)
                .call(self.salsa_contract.init()),
        );

        // register liquid token
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .egld_value(BigUint::from(50_000_000_000_000_000_u64))
                .call(self.salsa_contract.register_liquid_token(TOKEN_NAME, TOKEN_TICKER, TOKEN_DECIMALS)),
        );

        // set unbond period
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .call(self.salsa_contract.set_unbond_period(UNBOND_PERIOD)),
        );

        // set service fee
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .call(self.salsa_contract.set_service_fee(SERVICE_FEE)),
        );

        // set undelegate now fee
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .call(self.salsa_contract.set_undelegate_now_fee(UNDELEGATE_NOW_FEE)),
        );

        // TODO: add provider

        // set state active
        self.world.sc_call(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .call(self.salsa_contract.set_state_active()),
        );

        self
    }

    fn delegate(&mut self, address: &str, amount: &str, with_custody: bool, without_arbitrage: OptionalValue<bool>) -> &mut Self {
        self.world.sc_call(
            ScCallStep::new()
                .from(address)
                .egld_value(amount)
                .call(self.salsa_contract.delegate(with_custody, without_arbitrage)),
        );

        self
    }
}

#[test]
fn test_init() {
    let mut state = SalsaTestState::new();
    state.deploy();
}
