use multiversx_sc_snippets::imports::*;
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::state::State;

pub mod proxy {
    use multiversx_sc::proxy_imports::*;
    use ect_anchor::Anchor;

    pub struct EctAnchorProxy;

    impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EctAnchorProxy
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        type TxProxyMethods = EctAnchorProxyMethods<Env, From, To, Gas>;
        fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
            EctAnchorProxyMethods { wrapped_tx: tx }
        }
    }

    pub struct EctAnchorProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
    }

    #[rustfmt::skip]
    impl<Env, From, Gas> EctAnchorProxyMethods<Env, From, (), Gas>
    where
        Env: TxEnv,
        Env::Api: VMApi,
        From: TxFrom<Env>,
        Gas: TxGas<Env>,
    {
        pub fn init(self) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_deploy().original_result()
        }
    }

    #[rustfmt::skip]
    impl<Env, From, To, Gas> EctAnchorProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        Env::Api: VMApi,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        pub fn anchor_content<
            Arg0: ProxyArg<ManagedByteArray<Env::Api, 32>>,
            Arg1: ProxyArg<u32>,
            Arg2: ProxyArg<ManagedBuffer<Env::Api>>,
        >(
            self,
            hash: Arg0,
            version: Arg1,
            metadata_uri: Arg2,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("anchor")
                .argument(&hash)
                .argument(&version)
                .argument(&metadata_uri)
                .original_result()
        }

        pub fn verify_anchor<Arg0: ProxyArg<ManagedByteArray<Env::Api, 32>>>(
            self,
            hash: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<Anchor<Env::Api>>> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("verifyAnchor")
                .argument(&hash)
                .original_result()
        }
    }
}

pub struct AnchorInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub anchor_code: BytesValue,
    pub state: State,
}

impl AnchorInteract {
    pub async fn new(config: &Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());
        interactor.set_current_dir_from_workspace("eduChain-PoC");
        if config.use_chain_simulator() {
            interactor.generate_blocks_until_epoch(1).await.unwrap();
        }
        let pem = config.key.load_pem();
        let wallet = Wallet::from_pem_file_contents(pem).unwrap();
        let wallet_address = interactor.register_wallet(wallet).await;
        let anchor_code = BytesValue::interpret_from(
            "mxsc:../contracts/ect-anchor/output/ect-anchor.mxsc.json",
            &InterpreterContext::default(),
        );
        AnchorInteract {
            interactor,
            wallet_address,
            anchor_code,
            state: State::load(),
        }
    }

    pub async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(300_000_000u64)
            .typed(proxy::EctAnchorProxy)
            .init()
            .code(&self.anchor_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let addr = multiversx_sc_snippets::imports::Bech32Address::from(new_address.clone()).to_string();
        self.state.anchor_contract = Some(Bech32Address::from_bech32_string(addr.clone()));
        println!("[anchor] Deployed at: {addr}");
    }

    /// Hash a local file and anchor it on-chain.
    pub async fn anchor_file(&mut self, file_path: &str, version: u32, metadata_uri: &str) {
        let bytes = std::fs::read(file_path)
            .unwrap_or_else(|_| panic!("Cannot read file: {}", file_path));
        let hash_bytes: [u8; 32] = Sha256::digest(&bytes).into();
        let hash_hex = hex::encode(hash_bytes);
        println!("[anchor] SHA-256({file_path}) = {hash_hex}");

        let hash_managed = ManagedByteArray::<StaticApi, 32>::new_from_bytes(&hash_bytes);
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.anchor_address())
            .gas(10_000_000u64)
            .typed(proxy::EctAnchorProxy)
            .anchor_content(
                hash_managed,
                version,
                ManagedBuffer::new_from_bytes(metadata_uri.as_bytes()),
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[anchor] Anchored: {metadata_uri}");
    }

    /// Verify that a hash is anchored on-chain and print the anchor record.
    pub async fn verify_anchor(&mut self, hash_hex: &str) {
        let bytes = hex::decode(hash_hex).expect("Invalid hex string for hash");
        assert_eq!(bytes.len(), 32, "Hash must be 32 bytes (64 hex chars)");
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        let hash_managed = ManagedByteArray::<StaticApi, 32>::new_from_bytes(&arr);

        let result = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.anchor_address())
            .gas(5_000_000u64)
            .typed(proxy::EctAnchorProxy)
            .verify_anchor(hash_managed)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        match result {
            OptionalValue::Some(anchor) => {
                println!("[anchor] Hash found on-chain ✔");
                println!("  version:      {}", anchor.version);
                println!("  timestamp:    {}", anchor.timestamp);
                println!("  revoked:      {}", anchor.revoked);
                if !anchor.metadata_uri.is_empty() {
                    let uri_bytes = anchor.metadata_uri.to_boxed_bytes();
                    println!("  metadata_uri: {}", String::from_utf8_lossy(uri_bytes.as_slice()));
                }
            }
            OptionalValue::None => {
                println!("[anchor] Hash NOT found on-chain — not anchored");
            }
        }
    }

    /// Compute SHA-256 of arbitrary bytes (utility for off-chain hash calculation).
    pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
        Sha256::digest(data).into()
    }
}
