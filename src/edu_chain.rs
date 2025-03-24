#![no_std]

const NFT_AMOUNT: u32 = 1;
// const ROYALTIES_MAX: u32 = 10_000;

#[allow(unused_imports)]
use multiversx_sc::{derive_imports::*, imports::*};

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct CertificateAttributes<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub program_name: ManagedBuffer<M>,
    pub grade: u16,
    pub credit_points: u16,
    pub creation_timestamp: u64,
    pub expiration_timestamp: u64,
}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct UserAttributes {
    pub creation_timestamp: u64,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait EduChain {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(issueCertificate)]
    fn issue_certificate(
        &self,
        token_name: ManagedBuffer,
        student_name: ManagedBuffer,
        program_name: ManagedBuffer,
        grade: u16,
        credit_points: u16,
        expiration_timestamp: u64,
    ) -> u64 {
        let nft_token_id = self.nft_token_id().get();

        let serialized_attributes = ManagedBuffer::new();
        let attributes_sha256 = self.crypto().sha256(&serialized_attributes);
        let attributes_hash = attributes_sha256.as_managed_buffer();

        let attributes = CertificateAttributes {
            name: student_name,
            program_name,
            grade,
            credit_points,
            creation_timestamp: self.blockchain().get_block_timestamp(),
            expiration_timestamp,
        };
        let uris = ManagedVec::new();

        let nft_nonce = self.send().esdt_nft_create(
            &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &token_name,
            &BigUint::zero(),
            attributes_hash,
            &attributes,
            &uris,
        );
        nft_nonce
    }

    #[only_owner]
    #[endpoint(registerUser)]
    fn register_user(&self, name: ManagedBuffer, user: ManagedAddress) -> u64 {
        let nft_token_id = self.nft_token_id().get();

        let serialized_attributes = ManagedBuffer::new();
        let attributes_sha256 = self.crypto().sha256(&serialized_attributes);
        let attributes_hash = attributes_sha256.as_managed_buffer();

        let attributes = UserAttributes {
            creation_timestamp: self.blockchain().get_block_timestamp(),
        };
        let uris = ManagedVec::new();

        let nft_nonce = self.send().esdt_nft_create(
            &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &name,
            &BigUint::zero(),
            attributes_hash,
            &attributes,
            &uris,
        );

        self.tx()
            .single_esdt(&nft_token_id, nft_nonce, &BigUint::from(1u64))
            .to(user)
            .transfer();

        nft_nonce
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(depositRewards)]
    fn deposit_rewards(&self) {
        let received_rewards = self.call_value().single_esdt();
        let edu_token = self.edu_token().get();
        require!(
            received_rewards.token_identifier == edu_token,
            "Deposited token not correct"
        );
        self.available_rewards().update(|rewards| {
            *rewards += received_rewards.amount.clone();
        })
    }

    #[only_owner]
    #[endpoint(setRewardsTokenIdentifier)]
    fn set_rewards_token_identifier(&self, token_identifier: TokenIdentifier) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "Provided Token ID not valid!"
        );
        self.edu_token().set(&token_identifier);
    }

    // callbacks

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id.unwrap_esdt());
            }
            ManagedAsyncCallResult::Err(_) => {
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.tx().to(ToCaller).egld(returned.amount).transfer();
                }
            }
        }
    }

    /// creation/roles NFT management

    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        require!(self.nft_token_id().is_empty(), "Token already issued");

        let payment_amount = self.call_value().egld();
        self.send()
            .esdt_system_sc_tx()
            .issue_non_fungible(
                payment_amount.clone(),
                &token_name,
                &token_ticker,
                NonFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_transfer_create_role: true,
                    can_change_owner: false,
                    can_upgrade: false,
                    can_add_special_roles: true,
                },
            )
            .with_callback(self.callbacks().issue_callback())
            .async_call_and_exit()
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_token_issued();

        self.send()
            .esdt_system_sc_tx()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                [EsdtLocalRole::NftCreate][..].iter().cloned(),
            )
            .async_call_and_exit()
    }

    //private
    fn require_token_issued(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
    }

    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("availableRewards")]
    fn available_rewards(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("eduToken")]
    fn edu_token(&self) -> SingleValueMapper<TokenIdentifier>;
}
