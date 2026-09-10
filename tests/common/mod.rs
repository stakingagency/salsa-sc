use multiversx_sc::contract_base::{CallableContract, ContractBase};
use multiversx_sc_scenario::{
    executor::debug::ContractDebugWhiteboxLambda,
    imports::*,
    scenario::{run_vm::ScenarioVMRunner, ScenarioRunner},
    scenario_model::{ScCallStep, ScDeployStep},
    DebugApi, ScenarioWorld, WhiteboxContract,
};
use multiversx_sc_scenario::multiversx_chain_vm::host::context::{TxFunctionName, TxResult};

/// Minimal stand-in for the old whitebox `TxResult` so existing
/// `|r| r.result_message.as_bytes() == error` callbacks keep compiling.
pub struct WhiteboxTxResult {
    pub result_message: String,
}

pub trait WhiteboxCompat {
    fn whitebox_query<ContractObj, F>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        f: F,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj);

    fn whitebox_call<ContractObj, F>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        sc_call_step: ScCallStep,
        f: F,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj);

    fn whitebox_call_check<ContractObj, F, C>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        sc_call_step: ScCallStep,
        f: F,
        check_result: C,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj),
        C: FnOnce(WhiteboxTxResult);

    fn whitebox_deploy<ContractObj, F>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        sc_deploy_step: ScDeployStep,
        f: F,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj);
}

impl WhiteboxCompat for ScenarioWorld {
    fn whitebox_query<ContractObj, F>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        f: F,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj),
    {
        self.query()
            .to(&whitebox_contract.address_expr.value)
            .whitebox(whitebox_contract.contract_obj_builder, f);
        self
    }

    fn whitebox_call<ContractObj, F>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        sc_call_step: ScCallStep,
        f: F,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj),
    {
        let tx_result = run_whitebox_call(self, whitebox_contract, sc_call_step, f);
        tx_result.assert_ok();
        self
    }

    fn whitebox_call_check<ContractObj, F, C>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        sc_call_step: ScCallStep,
        f: F,
        check_result: C,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj),
        C: FnOnce(WhiteboxTxResult),
    {
        let tx_result = run_whitebox_call(self, whitebox_contract, sc_call_step, f);
        check_result(WhiteboxTxResult {
            result_message: tx_result.result_message,
        });
        self
    }

    fn whitebox_deploy<ContractObj, F>(
        &mut self,
        whitebox_contract: &WhiteboxContract<ContractObj>,
        sc_deploy_step: ScDeployStep,
        f: F,
    ) -> &mut Self
    where
        ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
        F: FnOnce(ContractObj),
    {
        let contract_obj = (whitebox_contract.contract_obj_builder)();
        let tx_result = with_vm_runner(self, |vm_runner| {
            let (_, tx_result) = vm_runner.perform_sc_deploy_lambda(
                &sc_deploy_step,
                ContractDebugWhiteboxLambda::new(TxFunctionName::WHITEBOX_INIT, || {
                    f(contract_obj);
                }),
            );
            tx_result
        });
        tx_result.assert_ok();
        self
    }
}

/// The unified `tx().whitebox()` API does not accept `.gas()`, so it always uses
/// the 5M scenario default. That is below the view/nodes promise gas checks and
/// makes `refresh_providers_test` spin forever. Run the lambda through the
/// debugger with the original `ScCallStep` (including gas_limit) instead.
fn run_whitebox_call<ContractObj, F>(
    world: &mut ScenarioWorld,
    whitebox_contract: &WhiteboxContract<ContractObj>,
    sc_call_step: ScCallStep,
    f: F,
) -> TxResult
where
    ContractObj: ContractBase<Api = DebugApi> + CallableContract + 'static,
    F: FnOnce(ContractObj),
{
    let mut sc_call_step = sc_call_step.to(&whitebox_contract.address_expr);
    if sc_call_step.tx.function.is_empty() {
        sc_call_step.tx.function = TxFunctionName::WHITEBOX_CALL.to_string();
    }

    let contract_obj = (whitebox_contract.contract_obj_builder)();
    with_vm_runner(world, |vm_runner| {
        vm_runner.perform_sc_call_lambda(
            &sc_call_step,
            ContractDebugWhiteboxLambda::new(TxFunctionName::WHITEBOX_CALL, || {
                f(contract_obj);
            }),
        )
    })
}

fn with_vm_runner<R>(world: &mut ScenarioWorld, f: impl FnOnce(&mut ScenarioVMRunner) -> R) -> R {
    let mut out = None;
    let mut f = Some(f);
    world.for_each_runner_mut(|runner| {
        if let Some(callback) = f.take() {
            // First ScenarioRunner is always ScenarioVMRunner (see ScenarioWorld::for_each_runner_mut).
            let vm_runner =
                unsafe { &mut *(runner as *mut dyn ScenarioRunner as *mut ScenarioVMRunner) };
            out = Some(callback(vm_runner));
        }
    });
    out.expect("whitebox calls require the debugger backend")
}
