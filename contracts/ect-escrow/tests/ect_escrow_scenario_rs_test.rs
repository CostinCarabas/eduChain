use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(
        "mxsc:output/ect-escrow.mxsc.json",
        ect_escrow::ContractBuilder,
    );
    blockchain
}

#[test]
fn deploy_rs() {
    world().run("scenarios/01_deploy.scen.json");
}

#[test]
fn session_create_confirm_rs() {
    world().run("scenarios/02_session_create_confirm.scen.json");
}

#[test]
fn session_dispute_resolve_rs() {
    world().run("scenarios/03_session_dispute_resolve.scen.json");
}
