use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(
        "mxsc:output/edu-chain-nft.mxsc.json",
        edu_chain_nft::ContractBuilder,
    );
    blockchain
}

#[test]
fn deploy_rs() {
    world().run("scenarios/01_deploy.scen.json");
}

#[test]
fn add_remove_issuer_rs() {
    world().run("scenarios/02_add_remove_issuer.scen.json");
}

#[test]
fn revoke_not_issuer_rs() {
    world().run("scenarios/03_revoke_certificate.scen.json");
}

#[test]
fn verify_revoked_rs() {
    world().run("scenarios/04_verify_revoked.scen.json");
}

#[test]
fn verify_unknown_rs() {
    world().run("scenarios/05_verify_unknown.scen.json");
}
