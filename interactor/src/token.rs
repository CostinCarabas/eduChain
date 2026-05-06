use multiversx_sc_snippets::imports::*;
use std::time::Duration;

use crate::config::Config;
use crate::state::State;

pub mod proxy {
    use multiversx_sc::proxy_imports::*;

    pub struct EctTokenProxy;

    impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EctTokenProxy
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        type TxProxyMethods = EctTokenProxyMethods<Env, From, To, Gas>;
        fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
            EctTokenProxyMethods { wrapped_tx: tx }
        }
    }

    pub struct EctTokenProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
    }

    #[rustfmt::skip]
    impl<Env, From, Gas> EctTokenProxyMethods<Env, From, (), Gas>
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
    impl<Env, From, To, Gas> EctTokenProxyMethods<Env, From, To, Gas>
    where
        Env: TxEnv,
        Env::Api: VMApi,
        From: TxFrom<Env>,
        To: TxTo<Env>,
        Gas: TxGas<Env>,
    {
        pub fn issue_token<
            Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
            Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
            Arg2: ProxyArg<u32>,
            Arg3: ProxyArg<BigUint<Env::Api>>,
        >(
            self,
            token_name: Arg0,
            token_ticker: Arg1,
            decimals: Arg2,
            initial_supply: Arg3,
        ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
            self.wrapped_tx
                .raw_call("issueToken")
                .argument(&token_name)
                .argument(&token_ticker)
                .argument(&decimals)
                .argument(&initial_supply)
                .original_result()
        }

        pub fn set_local_roles(self) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("setLocalRoles").original_result()
        }

        pub fn add_distributor<Arg0: ProxyArg<ManagedAddress<Env::Api>>>(
            self,
            addr: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("addDistributor").argument(&addr).original_result()
        }

        pub fn mint_rewards<Arg0: ProxyArg<BigUint<Env::Api>>>(
            self,
            amount: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("mintRewards").argument(&amount).original_result()
        }

        pub fn add_reward<
            Arg0: ProxyArg<ManagedAddress<Env::Api>>,
            Arg1: ProxyArg<BigUint<Env::Api>>,
            Arg2: ProxyArg<ManagedBuffer<Env::Api>>,
        >(
            self,
            beneficiary: Arg0,
            amount: Arg1,
            reason: Arg2,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx
                .payment(NotPayable)
                .raw_call("addReward")
                .argument(&beneficiary)
                .argument(&amount)
                .argument(&reason)
                .original_result()
        }

        pub fn get_pending_balance<Arg0: ProxyArg<ManagedAddress<Env::Api>>>(
            self,
            addr: Arg0,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
            self.wrapped_tx.payment(NotPayable).raw_call("getPendingBalance").argument(&addr).original_result()
        }

        pub fn get_treasury_balance(
            self,
        ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
            self.wrapped_tx.payment(NotPayable).raw_call("getTreasuryBalance").original_result()
        }

        pub fn claim_rewards(self) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
            self.wrapped_tx.payment(NotPayable).raw_call("claimRewards").original_result()
        }

        pub fn get_token_id(self) -> TxTypedCall<Env, From, To, NotPayable, Gas, TokenIdentifier<Env::Api>> {
            self.wrapped_tx.payment(NotPayable).raw_call("getTokenId").original_result()
        }
    }
}

pub struct TokenInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub token_code: BytesValue,
    pub state: State,
}

impl TokenInteract {
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
        let token_code = BytesValue::interpret_from(
            "mxsc:../contracts/ect-token/output/ect-token.mxsc.json",
            &InterpreterContext::default(),
        );
        TokenInteract {
            interactor,
            wallet_address,
            token_code,
            state: State::load(),
        }
    }

    pub async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(300_000_000u64)
            .typed(proxy::EctTokenProxy)
            .init()
            .code(&self.token_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let addr = multiversx_sc_snippets::imports::Bech32Address::from(new_address.clone()).to_string();
        self.state.token_contract = Some(Bech32Address::from_bech32_string(addr.clone()));
        println!("[token] Deployed at: {addr}");
    }

    pub async fn issue(&mut self, name: &str, ticker: &str, supply: u128) {
        // Issue token
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(100_000_000u64)
            .typed(proxy::EctTokenProxy)
            .issue_token(
                ManagedBuffer::new_from_bytes(name.as_bytes()),
                ManagedBuffer::new_from_bytes(ticker.as_bytes()),
                18u32,
                BigUint::<StaticApi>::from(supply),
            )
            .egld(50_000_000_000_000_000u64) // 0.05 EGLD
            .run()
            .await;

        println!("[token] Token issuance transaction sent. Waiting 15s for async processing...");
        tokio::time::sleep(Duration::from_secs(15)).await;

        println!("[token] Token issued: {name} ({ticker}) with supply {supply}");

        println!("[token] Setting local roles (Mint/Burn) for contract...");
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(100_000_000u64)
            .typed(proxy::EctTokenProxy)
            .set_local_roles()
            .run()
            .await;

        println!("[token] Local roles set for contract");

        // Fetch and persist the token identifier for later use (e.g., escrow payments)
        let token_id = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(5_000_000u64)
            .typed(proxy::EctTokenProxy)
            .get_token_id()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        let token_id_str = token_id.to_string();
        self.state.ect_token_id = Some(token_id_str.clone());
        println!("[token] ECT token identifier: {token_id_str}");
    }

    pub async fn add_distributor(&mut self, distributor: &str) {
        let addr = multiversx_sc_snippets::imports::Bech32Address::from_bech32_string(distributor.to_string()).to_address();
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(10_000_000u64)
            .typed(proxy::EctTokenProxy)
            .add_distributor(addr)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[token] Distributor added: {distributor}");
    }

    pub async fn mint_rewards(&mut self, amount: u128) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(20_000_000u64)
            .typed(proxy::EctTokenProxy)
            .mint_rewards(BigUint::<StaticApi>::from(amount))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[token] Minted {amount} rewards into treasury");
    }

    pub async fn add_reward(&mut self, beneficiary: &str, amount: u128, reason: &str) {
        let addr = multiversx_sc_snippets::imports::Bech32Address::from_bech32_string(beneficiary.to_string()).to_address();
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(10_000_000u64)
            .typed(proxy::EctTokenProxy)
            .add_reward(
                addr,
                BigUint::<StaticApi>::from(amount),
                ManagedBuffer::new_from_bytes(reason.as_bytes()),
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[token] Reward {amount} added for {beneficiary}: {reason}");
    }

    pub async fn treasury_balance(&mut self) {
        let balance = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(5_000_000u64)
            .typed(proxy::EctTokenProxy)
            .get_treasury_balance()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[token] Treasury balance: {balance} ECT (base units)");
    }

    pub async fn pending_balance(&mut self, address: &str) {
        let addr = Bech32Address::from_bech32_string(address.to_string()).to_address();
        let balance = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(5_000_000u64)
            .typed(proxy::EctTokenProxy)
            .get_pending_balance(addr)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[token] Pending balance for {address}: {balance} ECT (base units)");
    }

    pub async fn claim_rewards(&mut self) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.token_address())
            .gas(20_000_000u64)
            .typed(proxy::EctTokenProxy)
            .claim_rewards()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
        println!("[token] Rewards claimed to caller wallet");
    }
}
