use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(
        "mxsc:output/ect-token.mxsc.json",
        ect_token::ContractBuilder,
    );
    blockchain
}

#[test]
fn deploy_rs() {
    world().run("scenarios/01_deploy.scen.json");
}

#[test]
fn add_reward_and_claim_rs() {
    world().run("scenarios/02_add_reward_and_claim.scen.json");
}

#[test]
fn unauthorized_add_reward_rs() {
    world().run("scenarios/03_unauthorized_add_reward.scen.json");
}
