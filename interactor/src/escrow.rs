use multiversx_sc_snippets::imports::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::state::State;

pub mod proxy {
    use multiversx_sc::proxy_imports::*;
    use ect_escrow::Session;

    pub struct EctEscrowProxy;

    impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EctEscrowProxy
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        type TxProxyMethods = EctEscrowProxyMethods<Env, From, To, Gas>;
        fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
            EctEscrowProxyMethods { wrapped_tx: tx }
        }
    }

    pub struct EctEscrowProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
    }

    #[rustfmt::skip]
    impl<Env, From, Gas> EctEscrowProxyMethods<Env, From, (), Gas>
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
    impl<Env, From, To, Gas> EctEscrowProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        Env::Api: VMApi,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        pub fn set_ect_token_id<Arg0: ProxyArg<TokenIdentifier<Env::Api>>>(
            self,
            token_id: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("setEctTokenId").argument(&token_id).original_result()
        }

        pub fn create_session<
            Arg0: ProxyArg<ManagedAddress<Env::Api>>,
            Arg1: ProxyArg<u64>,
        >(
            self,
            mentor: Arg0,
            deadline: Arg1,
        ) -> TxTypedCall<Env, From, To, (), Gas, u64> {
            self.wrapped_tx
                .raw_call("createSession")
                .argument(&mentor)
                .argument(&deadline)
                .original_result()
        }

        pub fn confirm_completion<Arg0: ProxyArg<u64>>(
            self,
            id: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("confirmCompletion").argument(&id).original_result()
        }

        pub fn dispute_session<
            Arg0: ProxyArg<u64>,
            Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
        >(
            self,
            id: Arg0,
            reason: Arg1,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("disputeSession").argument(&id).argument(&reason).original_result()
        }

        pub fn get_session<Arg0: ProxyArg<u64>>(
            self,
            id: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, Session<Env::Api>> {
            self.wrapped_tx.payment(NotPayable).raw_call("getSession").argument(&id).original_result()
        }
    }
}

pub struct EscrowInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub escrow_code: BytesValue,
    pub state: State,
}

impl EscrowInteract {
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
        let escrow_code = BytesValue::interpret_from(
            "mxsc:../contracts/ect-escrow/output/ect-escrow.mxsc.json",
            &InterpreterContext::default(),
        );
        EscrowInteract {
            interactor,
            wallet_address,
            escrow_code,
            state: State::load(),
        }
    }

    pub async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(300_000_000u64)
            .typed(proxy::EctEscrowProxy)
            .init()
            .code(&self.escrow_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let addr = multiversx_sc_snippets::imports::Bech32Address::from(new_address.clone()).to_string();
        self.state.escrow_contract = Some(Bech32Address::from_bech32_string(addr.clone()));
        println!("[escrow] Deployed at: {addr}");
    }

    /// Set the ECT token ID in the escrow contract
    pub async fn set_ect_token_id(&mut self) {
        let token_id_str = self
            .state
            .ect_token_id
            .as_ref()
            .expect("ECT token ID not set — run `token issue` first");
        let token_id = TokenIdentifier::<StaticApi>::from(token_id_str.as_str());

        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.escrow_address())
            .gas(10_000_000u64)
            .typed(proxy::EctEscrowProxy)
            .set_ect_token_id(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[escrow] ECT token ID set to: {token_id_str}");
    }

    /// Create a mentoring session, locking ECT tokens in the escrow.
    /// Requires the ECT token identifier stored in state (set after `token issue`).
    pub async fn create_session(&mut self, mentor_bech32: &str, amount: u128, deadline_hours: u64) {
        let token_id_str = self
            .state
            .ect_token_id
            .as_ref()
            .expect("ECT token ID not set — run `token issue` first");
        let token_id = TokenIdentifier::<StaticApi>::from(token_id_str.as_str());

        let mentor = Bech32Address::from_bech32_string(mentor_bech32.to_string()).to_address();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let deadline = now + deadline_hours * 3600;

        let payment = EsdtTokenPayment::<StaticApi>::new(
            token_id,
            0,
            BigUint::<StaticApi>::from(amount),
        );

        let session_id = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.escrow_address())
            .gas(20_000_000u64)
            .typed(proxy::EctEscrowProxy)
            .create_session(mentor, deadline)
            .payment(payment)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("[escrow] Session created! ID={session_id}, mentor={mentor_bech32}, amount={amount} ECT, deadline={deadline}");
    }

    /// Query the current state of a session.
    pub async fn get_session(&mut self, id: u64) {
        use ect_escrow::SessionStatus;

        let session = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.escrow_address())
            .gas(5_000_000u64)
            .typed(proxy::EctEscrowProxy)
            .get_session(id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        let status_label = match session.status {
            SessionStatus::Open => "Open",
            SessionStatus::Completed => "Completed",
            SessionStatus::Disputed => "Disputed",
            SessionStatus::Expired => "Expired",
        };
        println!("[escrow] Session #{id}:");
        println!("  status:     {status_label}");
        println!("  amount:     {} ECT (base units)", session.amount);
        println!("  deadline:   {}", session.deadline);
        println!("  created_at: {}", session.created_at);
    }

    pub async fn confirm_completion(&mut self, id: u64) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.escrow_address())
            .gas(10_000_000u64)
            .typed(proxy::EctEscrowProxy)
            .confirm_completion(id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[escrow] Session {id} confirmed as completed");
    }
}
