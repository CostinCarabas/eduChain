use multiversx_sc_snippets::imports::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::state::State;

// ─── Input fixture types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CertFixture {
    pub student_address: String,
    pub token_name: String,
    pub attrs: CertAttrsFixture,
    /// Path to the off-chain JSON-LD metadata file (used when content_hash_hex = "auto")
    pub metadata_offchain_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CertAttrsFixture {
    pub certificate_version: u8,
    pub name: String,
    pub program_name: String,
    pub grade: u16,
    pub credit_points: u16,
    pub creation_timestamp: u64,
    pub expiration_timestamp: u64,
    pub course_id: String,
    pub institution_name: String,
    pub instructor_name: String,
    pub verification_url: String,
    // P1 optional
    pub eqf_level: Option<u8>,
    pub ects_credits: Option<u16>,
    pub language: Option<String>,
    pub achievement_type: Option<String>,
    /// "auto" = compute from metadata_offchain_path; hex string = use directly; absent = no hash
    pub content_hash_hex: Option<String>,
}

// ─── Proxy (manually written — replace with sc-meta generated in CI) ─────────

pub mod proxy {
    use multiversx_sc::proxy_imports::*;
    use edu_chain_nft::{AchievementType, CertificateAttributes};

    pub struct EduChainNftProxy;

    impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EduChainNftProxy
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        type TxProxyMethods = EduChainNftProxyMethods<Env, From, To, Gas>;
        fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
            EduChainNftProxyMethods { wrapped_tx: tx }
        }
    }

    pub struct EduChainNftProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
    }

    #[rustfmt::skip]
    impl<Env, From, Gas> EduChainNftProxyMethods<Env, From, (), Gas>
    where
        Env: TxEnv,
        Env::Api: VMApi,
        From: TxFrom<Env>,
        Gas: TxGas<Env>,
    {
        pub fn init(self) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_deploy()
                .original_result()
        }
    }

    #[rustfmt::skip]
    impl<Env, From, To, Gas> EduChainNftProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        Env::Api: VMApi,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        pub fn upgrade(self) -> TxTypedUpgrade<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_upgrade()
                .original_result()
        }

        pub fn add_issuer<Arg0: ProxyArg<ManagedAddress<Env::Api>>>(
            self,
            addr: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("addIssuer")
                .argument(&addr)
                .original_result()
        }

        pub fn remove_issuer<Arg0: ProxyArg<ManagedAddress<Env::Api>>>(
            self,
            addr: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("removeIssuer")
                .argument(&addr)
                .original_result()
        }

        pub fn is_issuer<Arg0: ProxyArg<ManagedAddress<Env::Api>>>(
            self,
            addr: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("isIssuer")
                .argument(&addr)
                .original_result()
        }

        pub fn issue_token<
            Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
            Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
        >(
            self,
            token_name: Arg0,
            token_ticker: Arg1,
        ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
            self.wrapped_tx
                .raw_call("issueToken")
                .argument(&token_name)
                .argument(&token_ticker)
                .original_result()
        }

        pub fn set_local_roles(
            self,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("setLocalRoles")
                .original_result()
        }

        pub fn issue_certificate<
            Arg0: ProxyArg<ManagedAddress<Env::Api>>,
            Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
            Arg2: ProxyArg<CertificateAttributes<Env::Api>>,
        >(
            self,
            student: Arg0,
            token_name: Arg1,
            attrs: Arg2,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("issueCertificate")
                .argument(&student)
                .argument(&token_name)
                .argument(&attrs)
                .original_result()
        }

        pub fn revoke_certificate<
            Arg0: ProxyArg<u64>,
            Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
        >(
            self,
            nonce: Arg0,
            reason: Arg1,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("revokeCertificate")
                .argument(&nonce)
                .argument(&reason)
                .original_result()
        }

        pub fn verify_certificate<Arg0: ProxyArg<u64>>(
            self,
            nonce: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u32> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("verifyCertificate")
                .argument(&nonce)
                .original_result()
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn parse_achievement_type(s: &str) -> edu_chain_nft::AchievementType {
    use edu_chain_nft::AchievementType;
    match s {
        "Bootcamp" => AchievementType::Bootcamp,
        "Microcredential" => AchievementType::Microcredential,
        "Diploma" => AchievementType::Diploma,
        _ => AchievementType::Course,
    }
}

// ─── Interactor helpers ───────────────────────────────────────────────────────

pub struct NftInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub nft_code: BytesValue,
    pub state: State,
}

impl NftInteract {
    pub async fn new(config: &Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());
        interactor.set_current_dir_from_workspace("eduChain-PoC");
        if config.use_chain_simulator() {
            interactor.generate_blocks_until_epoch(1).await.unwrap();
        }

        let pem_content = config.key.load_pem();
        let wallet = Wallet::from_pem_file_contents(pem_content).unwrap();
        let wallet_address = interactor.register_wallet(wallet).await;

        let nft_code = BytesValue::interpret_from(
            "mxsc:../contracts/edu-chain-nft/output/edu-chain-nft.mxsc.json",
            &InterpreterContext::default(),
        );

        NftInteract {
            interactor,
            wallet_address,
            nft_code,
            state: State::load(),
        }
    }

    pub async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(300_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .init()
            .code(&self.nft_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let addr = multiversx_sc_snippets::imports::Bech32Address::from(new_address.clone()).to_string();
        self.state.nft_contract = Some(Bech32Address::from_bech32_string(addr.clone()));
        println!("[nft] Deployed at: {addr}");
    }

    pub async fn add_issuer(&mut self, issuer_bech32: &str) {
        let issuer = multiversx_sc_snippets::imports::Bech32Address::from_bech32_string(issuer_bech32.to_string()).to_address();
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.nft_address())
            .gas(10_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .add_issuer(issuer)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[nft] Issuer added: {issuer_bech32}");
    }

    /// Issue a certificate NFT from a JSON fixture file.
    pub async fn issue_certificate(&mut self, cert_file_path: &str) {
        use edu_chain_nft::CertificateAttributes;

        // 1. Read and parse fixture
        let content = std::fs::read_to_string(cert_file_path)
            .unwrap_or_else(|_| panic!("Cannot read cert fixture: {}", cert_file_path));
        let fixture: CertFixture = serde_json::from_str(&content)
            .expect("Invalid cert fixture JSON");

        // 2. Resolve content hash
        let content_hash: Option<[u8; 32]> = match fixture.attrs.content_hash_hex.as_deref() {
            Some("auto") => {
                let meta_path = fixture
                    .metadata_offchain_path
                    .as_deref()
                    .expect("metadata_offchain_path is required when content_hash_hex = \"auto\"");
                let meta_bytes = std::fs::read(meta_path)
                    .unwrap_or_else(|_| panic!("Cannot read metadata file: {}", meta_path));
                let hash: [u8; 32] = Sha256::digest(&meta_bytes).into();
                println!("[nft] content_hash SHA-256({meta_path}) = {}", hex::encode(hash));
                Some(hash)
            }
            Some(hex_str) => {
                let bytes = hex::decode(hex_str).expect("Invalid hex in content_hash_hex");
                assert_eq!(bytes.len(), 32, "content_hash_hex must be 64 hex chars (32 bytes)");
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            }
            None => None,
        };

        // 3. Parse student address
        let student_addr = Bech32Address::from_bech32_string(fixture.student_address.clone())
            .to_address();

        // 4. Build CertificateAttributes
        let attrs = CertificateAttributes::<StaticApi> {
            certificate_version: fixture.attrs.certificate_version,
            name: ManagedBuffer::new_from_bytes(fixture.attrs.name.as_bytes()),
            program_name: ManagedBuffer::new_from_bytes(fixture.attrs.program_name.as_bytes()),
            grade: fixture.attrs.grade,
            credit_points: fixture.attrs.credit_points,
            creation_timestamp: fixture.attrs.creation_timestamp,
            expiration_timestamp: fixture.attrs.expiration_timestamp,
            course_id: ManagedBuffer::new_from_bytes(fixture.attrs.course_id.as_bytes()),
            institution_name: ManagedBuffer::new_from_bytes(fixture.attrs.institution_name.as_bytes()),
            instructor_name: ManagedBuffer::new_from_bytes(fixture.attrs.instructor_name.as_bytes()),
            verification_url: ManagedBuffer::new_from_bytes(fixture.attrs.verification_url.as_bytes()),
            eqf_level: fixture.attrs.eqf_level,
            ects_credits: fixture.attrs.ects_credits,
            language: fixture.attrs.language.as_deref()
                .map(|l| ManagedBuffer::new_from_bytes(l.as_bytes())),
            achievement_type: fixture.attrs.achievement_type.as_deref()
                .map(parse_achievement_type),
            content_hash: content_hash
                .map(|h| ManagedByteArray::new_from_bytes(&h)),
        };

        // 5. Call the contract
        let nonce = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.nft_address())
            .gas(20_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .issue_certificate(
                student_addr,
                ManagedBuffer::new_from_bytes(fixture.token_name.as_bytes()),
                attrs,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("[nft] Certificate issued for {}! NFT nonce: {nonce}", fixture.student_address);
    }

    pub async fn revoke_certificate(&mut self, nonce: u64, reason: &str) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.nft_address())
            .gas(10_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .revoke_certificate(nonce, ManagedBuffer::new_from_bytes(reason.as_bytes()))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[nft] Certificate #{nonce} revoked: {reason}");
    }

    pub async fn verify_certificate(&mut self, nonce: u64) {
        let status = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.nft_address())
            .gas(5_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .verify_certificate(nonce)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        let label = match status {
            0 => "Unknown",
            1 => "Active",
            2 => "Revoked",
            3 => "Expired",
            _ => "?",
        };
        println!("[nft] Certificate #{nonce} status: {label} (raw={status})");
    }
    pub async fn issue_token(&mut self, token_name: &str, token_ticker: &str) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.nft_address())
            .gas(100_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .issue_token(
                ManagedBuffer::new_from_bytes(token_name.as_bytes()),
                ManagedBuffer::new_from_bytes(token_ticker.as_bytes()),
            )
            .egld(50_000_000_000_000_000u64) // 0.05 EGLD
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        
        println!("[nft] NFT Token issuance transaction sent. Waiting 15s for async processing...");
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        println!("[nft] NFT collection issued: {token_name} ({token_ticker})");
    }

    pub async fn set_local_roles(&mut self) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.nft_address())
            .gas(100_000_000u64)
            .typed(proxy::EduChainNftProxy)
            .set_local_roles()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[nft] Local roles set for NFT collection");
    }
}
