// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct EduChainProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EduChainProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = EduChainProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        EduChainProxyMethods { wrapped_tx: tx }
    }
}

pub struct EduChainProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> EduChainProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init(
        self,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> EduChainProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn upgrade(
        self,
    ) -> TxTypedUpgrade<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_upgrade()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> EduChainProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn issue_certificate<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg2: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg3: ProxyArg<u16>,
        Arg4: ProxyArg<u16>,
        Arg5: ProxyArg<u64>,
    >(
        self,
        token_name: Arg0,
        student_name: Arg1,
        program_name: Arg2,
        grade: Arg3,
        credit_points: Arg4,
        expiration_timestamp: Arg5,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("issueCertificate")
            .argument(&token_name)
            .argument(&student_name)
            .argument(&program_name)
            .argument(&grade)
            .argument(&credit_points)
            .argument(&expiration_timestamp)
            .original_result()
    }

    pub fn register_user<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        name: Arg0,
        user: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("registerUser")
            .argument(&name)
            .argument(&user)
            .original_result()
    }

    pub fn deposit_rewards(
        self,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("depositRewards")
            .original_result()
    }

    pub fn set_rewards_token_identifier<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_identifier: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setRewardsTokenIdentifier")
            .argument(&token_identifier)
            .original_result()
    }

    /// creation/roles NFT management 
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
}
