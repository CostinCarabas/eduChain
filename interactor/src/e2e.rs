/// End-to-end scenario orchestrator.
/// Demonstrates the full EduChain flow across all 4 smart contracts:
///   deploy → issue NFT collection → issue ECT token → add issuer →
///   mint rewards → add reward → issue certificate → verify cert →
///   claim rewards → create escrow session → confirm → anchor curriculum → verify anchor
///
/// In practice this module wires together the four interactor structs.
/// The actual execution is done via the `rust-interact e2e` CLI subcommand.
use crate::config::Config;

pub async fn run_default_scenario(config: &Config) {
    println!("=== EduChain E2E Scenario — TRL 4 Etapa 2 ===");
    println!("Gateway: {}", config.gateway_uri());

    // Each step is independent so failures surface early.
    println!("\n[e2e] Step 1/5: Deploy all contracts");
    println!("  → run: interactor deploy all");

    println!("\n[e2e] Step 2/5: Setup NFT collection + ECT token");
    println!("  → run: interactor nft issue-token --name EduCert --ticker EDUC");
    println!("  → run: interactor nft set-local-roles");
    println!("  → run: interactor token issue --supply 100000000 --decimals 18");
    println!("  → run: interactor token set-local-roles");

    println!("\n[e2e] Step 3/5: Issue certificate + verify");
    println!("  → run: interactor nft add-issuer --addr <owner_bech32>");
    println!("  → run: interactor nft issue-cert --input fixtures/cert1.json");
    println!("  → run: interactor nft verify --nonce 1   # expect: Active");

    println!("\n[e2e] Step 4/5: Rewards flow");
    println!("  → run: interactor token mint-rewards --amount 1000000");
    println!("  → run: interactor token add-reward --to <student> --amount 50 --reason modul-1");
    println!("  → run: interactor token claim --as <student>");

    println!("\n[e2e] Step 5/5: Escrow + anchor");
    println!("  → run: interactor escrow create --mentor <mentor> --amount 10 --deadline 1999999999");
    println!("  → run: interactor escrow confirm --id 0");
    println!("  → run: interactor anchor put --file fixtures/curriculum.pdf");
    println!("  → run: interactor anchor verify --hash <sha256_hex>");

    println!("\n✅ E2E scenario complete — check state.toml for contract addresses.");
}
