use clap::{Parser, Subcommand};
use rust_interact::{
    anchor::AnchorInteract,
    config::Config,
    e2e,
    escrow::EscrowInteract,
    nft::NftInteract,
    token::TokenInteract,
};

/// EduChain multi-contract interactor CLI (TRL-4 Etapa 2)
#[derive(Parser)]
#[command(name = "rust-interact", about = "EduChain blockchain interactor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy one or all contracts
    Deploy {
        #[arg(value_enum)]
        target: DeployTarget,
    },
    /// NFT certificate operations
    Nft {
        #[command(subcommand)]
        action: NftAction,
    },
    /// ECT token operations
    Token {
        #[command(subcommand)]
        action: TokenAction,
    },
    /// Escrow session operations
    Escrow {
        #[command(subcommand)]
        action: EscrowAction,
    },
    /// Content anchoring operations
    Anchor {
        #[command(subcommand)]
        action: AnchorAction,
    },
    /// Run end-to-end scenario
    E2e {
        #[arg(long, default_value = "default")]
        scenario: String,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum DeployTarget {
    Nft,
    Token,
    Escrow,
    Anchor,
    All,
}

#[derive(Subcommand)]
enum NftAction {
    /// Issue the NFT collection (ESDT system)
    IssueToken {
        #[arg(long)]
        name: String,
        #[arg(long)]
        ticker: String,
    },
    /// Set local roles for the NFT collection
    SetLocalRoles,
    /// Add an authorized issuer
    AddIssuer {
        #[arg(long)]
        addr: String,
    },
    /// Issue a certificate NFT from a JSON fixture file
    Issue {
        #[arg(long)]
        cert_file: String,
    },
    /// Revoke a certificate by nonce
    Revoke {
        #[arg(long)]
        nonce: u64,
        #[arg(long, default_value = "")]
        reason: String,
    },
    /// Verify a certificate status
    Verify {
        #[arg(long)]
        nonce: u64,
    },
}

#[derive(Subcommand)]
enum TokenAction {
    /// Issue the $ECT fungible ESDT
    Issue {
        #[arg(long)]
        name: String,
        #[arg(long)]
        ticker: String,
        #[arg(long)]
        supply: u128,
    },
    /// Add a reward distributor
    AddDistributor {
        #[arg(long)]
        addr: String,
    },
    /// Mint rewards into treasury
    MintRewards {
        #[arg(long)]
        amount: u128,
    },
    /// Add a reward for a beneficiary
    AddReward {
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: u128,
        #[arg(long)]
        reason: String,
    },
    /// Show current treasury balance
    TreasuryBalance,
    /// Show pending (unclaimed) balance for an address
    PendingBalance {
        #[arg(long)]
        address: String,
    },
    /// Claim pending rewards (caller's wallet)
    Claim,
}

#[derive(Subcommand)]
enum EscrowAction {
    /// Set the ECT token ID in the escrow contract
    SetTokenId,
    /// Create a new mentoring session (locks ECT tokens)
    Create {
        #[arg(long)]
        mentor: String,
        #[arg(long)]
        amount: u128,
        #[arg(long, default_value = "72")]
        deadline_hours: u64,
    },
    /// Get the status of a session
    GetSession {
        #[arg(long)]
        id: u64,
    },
    /// Confirm a session as completed (releases funds to mentor)
    Confirm {
        #[arg(long)]
        id: u64,
    },
}

#[derive(Subcommand)]
enum AnchorAction {
    /// Hash a local file and anchor it on-chain
    Put {
        #[arg(long)]
        file: String,
        #[arg(long, default_value = "1")]
        version: u32,
        #[arg(long, default_value = "")]
        metadata_uri: String,
    },
    /// Verify a hash is anchored on-chain (provide SHA-256 as hex)
    Verify {
        #[arg(long)]
        hex: String,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let config = Config::new();

    match &cli.command {
        Commands::Deploy { target } => {
            match target {
                DeployTarget::Nft | DeployTarget::All => {
                    NftInteract::new(&config).await.deploy().await;
                }
                _ => {}
            }
            match target {
                DeployTarget::Token | DeployTarget::All => {
                    TokenInteract::new(&config).await.deploy().await;
                }
                _ => {}
            }
            match target {
                DeployTarget::Escrow | DeployTarget::All => {
                    EscrowInteract::new(&config).await.deploy().await;
                }
                _ => {}
            }
            match target {
                DeployTarget::Anchor | DeployTarget::All => {
                    AnchorInteract::new(&config).await.deploy().await;
                }
                _ => {}
            }
        }

        Commands::Nft { action } => {
            let mut i = NftInteract::new(&config).await;
            match action {
                NftAction::IssueToken { name, ticker } => i.issue_token(name, ticker).await,
                NftAction::SetLocalRoles => i.set_local_roles().await,
                NftAction::AddIssuer { addr } => i.add_issuer(addr).await,
                NftAction::Issue { cert_file } => i.issue_certificate(cert_file).await,
                NftAction::Revoke { nonce, reason } => i.revoke_certificate(*nonce, reason).await,
                NftAction::Verify { nonce } => i.verify_certificate(*nonce).await,
            }
        }

        Commands::Token { action } => {
            let mut i = TokenInteract::new(&config).await;
            match action {
                TokenAction::Issue { name, ticker, supply } => i.issue(name, ticker, *supply).await,
                TokenAction::AddDistributor { addr } => i.add_distributor(addr).await,
                TokenAction::MintRewards { amount } => i.mint_rewards(*amount).await,
                TokenAction::AddReward { to, amount, reason } => {
                    i.add_reward(to, *amount, reason).await
                }
                TokenAction::TreasuryBalance => i.treasury_balance().await,
                TokenAction::PendingBalance { address } => i.pending_balance(address).await,
                TokenAction::Claim => i.claim_rewards().await,
            }
        }

        Commands::Escrow { action } => {
            let mut i = EscrowInteract::new(&config).await;
            match action {
                EscrowAction::SetTokenId => i.set_ect_token_id().await,
                EscrowAction::Create { mentor, amount, deadline_hours } => {
                    i.create_session(mentor, *amount, *deadline_hours).await
                }
                EscrowAction::GetSession { id } => i.get_session(*id).await,
                EscrowAction::Confirm { id } => i.confirm_completion(*id).await,
            }
        }

        Commands::Anchor { action } => {
            let mut i = AnchorInteract::new(&config).await;
            match action {
                AnchorAction::Put { file, version, metadata_uri } => {
                    i.anchor_file(file, *version, metadata_uri).await
                }
                AnchorAction::Verify { hex } => i.verify_anchor(hex).await,
            }
        }

        Commands::E2e { scenario } => {
            println!("Running scenario: {scenario}");
            e2e::run_default_scenario(&config).await;
        }
    }
}
