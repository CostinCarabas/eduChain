#![no_std]
#![allow(deprecated)]

use multiversx_sc::{derive_imports::*, imports::*};

pub const CURRENT_CERT_VERSION: u8 = 1;
pub const NFT_AMOUNT: u32 = 1;
pub const BATCH_LIMIT: usize = 50;

// ─────────────────────────────────────────────
//  Types
// ─────────────────────────────────────────────

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy)]
pub enum AchievementType {
    Course,
    Bootcamp,
    Microcredential,
    Diploma,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy, PartialEq)]
pub enum CertificateStatus {
    Unknown,
    Active,
    Revoked,
    Expired,
}

/// Full on-chain certificate metadata — schema v1.0
/// P0 (obligatory) = first 11 fields + certificate_version
/// P1 (recommended, optional) = last 5 fields
/// P2 fields (skills, criteria_url, evidence_url) are stored off-chain,
///   referenced via verification_url and anchored via content_hash.
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct CertificateAttributes<M: ManagedTypeApi> {
    // ── version (always first for forward-compat) ──
    pub certificate_version: u8,            // P0
    // ── P0 mandatory ──
    pub name: ManagedBuffer<M>,             // student display name
    pub program_name: ManagedBuffer<M>,
    pub grade: u16,                         // 0..=100
    pub credit_points: u16,
    pub creation_timestamp: u64,
    pub expiration_timestamp: u64,          // 0 = no expiry
    pub course_id: ManagedBuffer<M>,
    pub institution_name: ManagedBuffer<M>,
    pub instructor_name: ManagedBuffer<M>,
    pub verification_url: ManagedBuffer<M>, // URL to off-chain JSON-LD
    // ── P1 optional ──
    pub eqf_level: Option<u8>,              // 1..=8
    pub ects_credits: Option<u16>,
    pub language: Option<ManagedBuffer<M>>, // ISO 639-1
    pub achievement_type: Option<AchievementType>,
    pub content_hash: Option<ManagedByteArray<M, 32>>, // SHA-256 of off-chain JSON-LD
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────

#[multiversx_sc::contract]
pub trait EduChainNft {
    // ─── lifecycle ──────────────────────────
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    // ─── token setup (owner only) ───────────

    /// Emit the NFT collection on ESDT system SC. Must be called once, payable in EGLD.
    #[only_owner]
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

    /// Grant NftCreate + NftBurn roles to the SC address.
    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_token_issued();
        self.send()
            .esdt_system_sc_tx()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn][..].iter().cloned(),
            )
            .async_call_and_exit()
    }

    // ─── RBAC: issuer management ────────────

    /// Add an authorized issuer address (only owner).
    #[only_owner]
    #[endpoint(addIssuer)]
    fn add_issuer(&self, addr: ManagedAddress) {
        self.authorized_issuers().insert(addr.clone());
        self.issuer_added_event(&addr);
    }

    /// Remove an authorized issuer address (only owner).
    #[only_owner]
    #[endpoint(removeIssuer)]
    fn remove_issuer(&self, addr: ManagedAddress) {
        self.authorized_issuers().remove(&addr);
        self.issuer_removed_event(&addr);
    }

    /// Returns true if addr is an authorized issuer.
    #[view(isIssuer)]
    fn is_issuer(&self, addr: ManagedAddress) -> bool {
        self.authorized_issuers().contains(&addr)
    }

    // ─── Certificate endpoints ───────────────

    /// Issue a single SBT certificate NFT and transfer it to the student wallet.
    /// Returns the NFT nonce.
    #[endpoint(issueCertificate)]
    fn issue_certificate(
        &self,
        student: ManagedAddress,
        token_name: ManagedBuffer,
        attrs: CertificateAttributes<Self::Api>,
    ) -> u64 {
        self.require_authorized_issuer();
        self.require_token_issued();

        require!(attrs.grade <= 100, "Grade must be 0..=100");
        require!(
            attrs.expiration_timestamp == 0
                || attrs.expiration_timestamp > self.blockchain().get_block_timestamp(),
            "Expiration must be in the future or zero"
        );
        if let Some(eqf) = attrs.eqf_level {
            require!(eqf >= 1 && eqf <= 8, "EQF level must be 1..=8");
        }
        require!(
            attrs.certificate_version == CURRENT_CERT_VERSION,
            "Unsupported certificate version"
        );
        require!(
            self.certificate_of(&attrs.course_id, &student).is_empty(),
            "Certificate already issued for this student/course"
        );

        let nonce = self.mint_and_transfer_cert(&student, &token_name, &attrs);

        self.cert_issued_event(
            &student,
            &attrs.course_id,
            nonce,
            &self.blockchain().get_caller(),
            attrs.grade,
            self.blockchain().get_block_timestamp(),
        );

        nonce
    }

    /// Batch issue certificates. Max BATCH_LIMIT per call to stay within gas.
    /// Skips duplicates (returns existing nonce without re-emitting).
    #[endpoint(issueCertificateBatch)]
    fn issue_certificate_batch(
        &self,
        requests: MultiValueEncoded<
            MultiValue3<ManagedAddress, ManagedBuffer, CertificateAttributes<Self::Api>>,
        >,
    ) -> ManagedVec<u64> {
        self.require_authorized_issuer();
        self.require_token_issued();
        require!(
            requests.len() <= BATCH_LIMIT,
            "Batch exceeds maximum of 50 certificates"
        );

        let mut nonces = ManagedVec::new();
        for req in requests.into_iter() {
            let (student, token_name, attrs) = req.into_tuple();
            // Skip duplicate
            let existing = self.certificate_of(&attrs.course_id, &student).get();
            if existing != 0 {
                nonces.push(existing);
                continue;
            }
            require!(attrs.grade <= 100, "Grade must be 0..=100");
            if let Some(eqf) = attrs.eqf_level {
                require!(eqf >= 1 && eqf <= 8, "EQF level must be 1..=8");
            }
            let nonce = self.mint_and_transfer_cert(&student, &token_name, &attrs);
            self.cert_issued_event(
                &student,
                &attrs.course_id,
                nonce,
                &self.blockchain().get_caller(),
                attrs.grade,
                self.blockchain().get_block_timestamp(),
            );
            nonces.push(nonce);
        }
        nonces
    }

    /// Revoke a certificate by nonce. Marks it revoked in storage (soft-revoke).
    #[endpoint(revokeCertificate)]
    fn revoke_certificate(&self, nonce: u64, reason: ManagedBuffer) {
        self.require_authorized_issuer();
        require!(nonce > 0, "Invalid nonce");
        require!(
            !self.revoked_certificates().contains(&nonce),
            "Certificate already revoked"
        );
        self.revoked_certificates().insert(nonce);
        self.revocation_reason(nonce).set(&reason);
        self.cert_revoked_event(nonce, &self.blockchain().get_caller(), &reason);
    }

    // ─── Read-only views ─────────────────────

    /// Returns ACTIVE, REVOKED, EXPIRED, or UNKNOWN.
    #[view(verifyCertificate)]
    fn verify_certificate(&self, nonce: u64) -> CertificateStatus {
        if nonce == 0 {
            return CertificateStatus::Unknown;
        }
        if self.revoked_certificates().contains(&nonce) {
            return CertificateStatus::Revoked;
        }
        // Try reading attributes to check expiry; if token data is absent treat as Unknown
        let token_id = self.nft_token_id().get();
        let token_data = self
            .blockchain()
            .get_esdt_token_data(&self.blockchain().get_sc_address(), &token_id, nonce);

        // esdt_nft_create amount is 1; if nonce was never minted, amount == 0
        // We cannot directly check "existence" in this way from SC; use certificate_of reverse mapping instead.
        // Decode attributes to check expiry
        let attrs_result =
            CertificateAttributes::<Self::Api>::top_decode(token_data.attributes.clone());
        match attrs_result {
            Result::Ok(attrs) => {
                let now = self.blockchain().get_block_timestamp();
                if attrs.expiration_timestamp > 0 && now > attrs.expiration_timestamp {
                    CertificateStatus::Expired
                } else {
                    CertificateStatus::Active
                }
            }
            Result::Err(_) => CertificateStatus::Unknown,
        }
    }

    /// Returns the full CertificateAttributes for a given nonce.
    #[view(getCertificateAttributes)]
    fn get_certificate_attributes(&self, nonce: u64) -> CertificateAttributes<Self::Api> {
        let token_id = self.nft_token_id().get();
        let token_data = self
            .blockchain()
            .get_esdt_token_data(&self.blockchain().get_sc_address(), &token_id, nonce);
        CertificateAttributes::<Self::Api>::top_decode(token_data.attributes)
            .unwrap_or_else(|_| sc_panic!("Failed to decode certificate attributes"))
    }

    /// Returns the NFT nonce for (course_id, student) pair. Returns 0 if not found.
    #[view(getCertificateNonce)]
    fn get_certificate_nonce(
        &self,
        course_id: ManagedBuffer,
        student: ManagedAddress,
    ) -> u64 {
        self.certificate_of(&course_id, &student).get()
    }

    /// Returns the revocation reason for a nonce (empty if not revoked).
    #[view(getRevocationReason)]
    fn get_revocation_reason(&self, nonce: u64) -> ManagedBuffer {
        self.revocation_reason(nonce).get()
    }

    // ─── Internal helpers ────────────────────

    fn require_authorized_issuer(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.authorized_issuers().contains(&caller),
            "Caller is not an authorized issuer"
        );
    }

    fn require_token_issued(&self) {
        require!(!self.nft_token_id().is_empty(), "NFT token not yet issued");
    }

    /// Serialize attributes, compute hash, mint NFT, save reverse mapping, transfer to student.
    fn mint_and_transfer_cert(
        &self,
        student: &ManagedAddress,
        token_name: &ManagedBuffer,
        attrs: &CertificateAttributes<Self::Api>,
    ) -> u64 {
        let token_id = self.nft_token_id().get();

        // ── Fix: hash the serialized attributes, not an empty buffer ──
        let mut serialized = ManagedBuffer::new();
        let _ = attrs.top_encode(&mut serialized);
        let attrs_sha256 = self.crypto().sha256(&serialized);
        let attrs_hash = attrs_sha256.as_managed_buffer();

        let uris = ManagedVec::new();
        let nft_nonce = self.send().esdt_nft_create(
            &token_id,
            &BigUint::from(NFT_AMOUNT),
            token_name,
            &BigUint::zero(), // royalties
            attrs_hash,
            attrs,
            &uris,
        );

        // Transfer NFT to student wallet
        self.tx()
            .to(student)
            .single_esdt(&token_id, nft_nonce, &BigUint::from(NFT_AMOUNT))
            .transfer();

        // Save reverse lookup (course_id, student) → nonce
        self.certificate_of(&attrs.course_id, student).set(nft_nonce);

        nft_nonce
    }

    // ─── Callbacks ──────────────────────────

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                // Refund EGLD if the issue failed
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0u64 {
                    self.tx().to(ToCaller).egld(returned.amount).transfer();
                }
            }
        }
    }

    // ─── Events ─────────────────────────────

    #[event("certificate_issued")]
    fn cert_issued_event(
        &self,
        #[indexed] student: &ManagedAddress,
        #[indexed] course_id: &ManagedBuffer,
        #[indexed] nonce: u64,
        #[indexed] issuer: &ManagedAddress,
        #[indexed] grade: u16,
        timestamp: u64,   // single data arg
    );

    #[event("certificate_revoked")]
    fn cert_revoked_event(
        &self,
        #[indexed] nonce: u64,
        #[indexed] issuer: &ManagedAddress,
        reason: &ManagedBuffer,  // single data arg
    );

    #[event("issuer_added")]
    fn issuer_added_event(&self, #[indexed] addr: &ManagedAddress);

    #[event("issuer_removed")]
    fn issuer_removed_event(&self, #[indexed] addr: &ManagedAddress);

    // ─── Storage ────────────────────────────

    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    /// Set of authorized issuer addresses (RBAC whitelist).
    #[storage_mapper("authorizedIssuers")]
    fn authorized_issuers(&self) -> SetMapper<ManagedAddress>;

    /// Set of revoked certificate nonces (soft-revoke).
    #[storage_mapper("revokedCertificates")]
    fn revoked_certificates(&self) -> SetMapper<u64>;

    /// Revocation reason text per nonce.
    #[storage_mapper("revocationReason")]
    fn revocation_reason(&self, nonce: u64) -> SingleValueMapper<ManagedBuffer>;

    /// (course_id, student) → nft_nonce (0 = not issued).
    #[storage_mapper("certificateOf")]
    fn certificate_of(
        &self,
        course_id: &ManagedBuffer,
        student: &ManagedAddress,
    ) -> SingleValueMapper<u64>;
}
