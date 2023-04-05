use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("file:output/salsa.wasm", salsa::ContractBuilder);
    blockchain
}

#[test]
fn salsa_rs() {
    multiversx_sc_scenario::run_rs("scenarios/salsa.scen.json", world());
}
