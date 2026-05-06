#![no_std]
#![allow(deprecated)]

use multiversx_sc::{derive_imports::*, imports::*};

// ─────────────────────────────────────────────
//  Types
// ─────────────────────────────────────────────

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Anchor<M: ManagedTypeApi> {
    pub author: ManagedAddress<M>,
    pub timestamp: u64,
    pub version: u32,
    pub metadata_uri: ManagedBuffer<M>,
    pub revoked: bool,
    pub revocation_reason: ManagedBuffer<M>,
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────

/// Immutable content-anchoring SC.
/// Stores SHA-256 hashes of educational resources (curricula, assignments, etc.)
/// on-chain with author + timestamp, enabling provenance verification.
#[multiversx_sc::contract]
pub trait EctAnchor {
    // ─── lifecycle ──────────────────────────
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    // ─── Anchoring ──────────────────────────

    /// Anchor a content hash. Caller becomes the author.
    /// hash must be a 32-byte SHA-256 value.
    /// metadata_uri points to the off-chain JSON-LD description.
    #[endpoint(anchor)]
    fn anchor_content(
        &self,
        hash: ManagedByteArray<Self::Api, 32>,
        version: u32,
        metadata_uri: ManagedBuffer,
    ) {
        require!(
            self.anchors(&hash).is_empty(),
            "Content hash already anchored"
        );
        let author = self.blockchain().get_caller();
        let now: u64 = self.blockchain().get_block_timestamp();
        self.anchors(&hash).set(Anchor {
            author: author.clone(),
            timestamp: now,
            version,
            metadata_uri: metadata_uri.clone(),
            revoked: false,
            revocation_reason: ManagedBuffer::new(),
        });
        self.anchors_by_author(&author).insert(hash.clone());
        self.content_anchored_event(&hash, &author, version, now, &metadata_uri);
    }

    /// Soft-revoke an anchored content hash. Only the original author can revoke.
    #[endpoint(revokeAnchor)]
    fn revoke_anchor(
        &self,
        hash: ManagedByteArray<Self::Api, 32>,
        reason: ManagedBuffer,
    ) {
        require!(!self.anchors(&hash).is_empty(), "Hash not found");
        let mut anchor_data = self.anchors(&hash).get();
        let caller = self.blockchain().get_caller();
        require!(caller == anchor_data.author, "Only the author can revoke");
        require!(!anchor_data.revoked, "Already revoked");
        anchor_data.revoked = true;
        anchor_data.revocation_reason = reason.clone();
        self.anchors(&hash).set(anchor_data.clone());
        self.content_revoked_event(&hash, &caller, &reason);
    }

    // ─── Views ──────────────────────────────

    /// Returns the Anchor record for a hash, or None if not found.
    #[view(verifyAnchor)]
    fn verify_anchor(
        &self,
        hash: ManagedByteArray<Self::Api, 32>,
    ) -> OptionalValue<Anchor<Self::Api>> {
        if self.anchors(&hash).is_empty() {
            OptionalValue::None
        } else {
            OptionalValue::Some(self.anchors(&hash).get())
        }
    }

    /// Paginated list of hashes anchored by a given author.
    #[view(listAnchorsByAuthor)]
    fn list_anchors_by_author(
        &self,
        author: ManagedAddress,
        from: usize,
        count: usize,
    ) -> MultiValueEncoded<ManagedByteArray<Self::Api, 32>> {
        let mut result = MultiValueEncoded::new();
        for (i, hash) in self.anchors_by_author(&author).iter().enumerate() {
            if i < from {
                continue;
            }
            if result.len() >= count {
                break;
            }
            result.push(hash);
        }
        result
    }

    // ─── Events ─────────────────────────────

    #[event("content_anchored")]
    fn content_anchored_event(
        &self,
        #[indexed] hash: &ManagedByteArray<Self::Api, 32>,
        #[indexed] author: &ManagedAddress,
        #[indexed] version: u32,
        #[indexed] timestamp: u64,
        metadata_uri: &ManagedBuffer,  // single data arg
    );

    #[event("content_revoked")]
    fn content_revoked_event(
        &self,
        #[indexed] hash: &ManagedByteArray<Self::Api, 32>,
        #[indexed] author: &ManagedAddress,
        reason: &ManagedBuffer,
    );

    // ─── Storage ────────────────────────────

    /// hash → Anchor record
    #[storage_mapper("anchors")]
    fn anchors(
        &self,
        hash: &ManagedByteArray<Self::Api, 32>,
    ) -> SingleValueMapper<Anchor<Self::Api>>;

    /// author → set of anchored hashes (for pagination)
    #[storage_mapper("anchorsByAuthor")]
    fn anchors_by_author(
        &self,
        author: &ManagedAddress,
    ) -> SetMapper<ManagedByteArray<Self::Api, 32>>;
}
