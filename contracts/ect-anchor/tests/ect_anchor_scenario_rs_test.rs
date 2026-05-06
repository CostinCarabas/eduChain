use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(
        "mxsc:output/ect-anchor.mxsc.json",
        ect_anchor::ContractBuilder,
    );
    blockchain
}

#[test]
fn deploy_rs() {
    world().run("scenarios/01_deploy.scen.json");
}

#[test]
fn anchor_verify_rs() {
    world().run("scenarios/02_anchor_verify.scen.json");
}

#[test]
fn anchor_duplicate_negative_rs() {
    world().run("scenarios/03_anchor_duplicate_negative.scen.json");
}
