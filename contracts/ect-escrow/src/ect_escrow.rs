#![no_std]
#![allow(deprecated)]

use multiversx_sc::{derive_imports::*, imports::*};

// ─────────────────────────────────────────────
//  Types
// ─────────────────────────────────────────────

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy, PartialEq)]
pub enum SessionStatus {
    Open,
    Completed,
    Disputed,
    Expired,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Session<M: ManagedTypeApi> {
    pub student: ManagedAddress<M>,
    pub mentor: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub deadline: u64,
    pub status: SessionStatus,
    pub created_at: u64,
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────

/// P2P escrow for mentoring sessions, denominated in $ECT.
/// Flow: student createSession → student confirmCompletion (or expire after deadline)
///       If disputed: owner resolveDispute(winner).
#[multiversx_sc::contract]
pub trait EctEscrow {
    // ─── lifecycle ──────────────────────────
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    // ─── Setup ──────────────────────────────

    /// Set the $ECT token identifier (owner only).
    #[only_owner]
    #[endpoint(setEctTokenId)]
    fn set_ect_token_id(&self, token_id: TokenIdentifier) {
        require!(
            token_id.is_valid_esdt_identifier(),
            "Invalid token identifier"
        );
        self.ect_token_id().set(&token_id);
    }

    // ─── Session lifecycle ───────────────────

    /// Student creates a new mentoring session, locking $ECT.
    /// Returns the new session ID.
    #[payable("*")]
    #[endpoint(createSession)]
    fn create_session(&self, mentor: ManagedAddress, deadline_timestamp: u64) -> u64 {
        let student = self.blockchain().get_caller();
        require!(student != mentor, "Student and mentor must differ");
        let now: u64 = self.blockchain().get_block_timestamp();
        require!(
            deadline_timestamp > now + 60,
            "Deadline must be at least 60 seconds in the future"
        );

        // Accept only $ECT payment
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == self.ect_token_id().get(),
            "Payment must be in $ECT"
        );
        require!(payment.amount > 0u64, "Amount must be > 0");

        let id = self.next_session_id().get();
        self.next_session_id().set(id + 1);

        self.sessions(id).set(Session {
            student: student.clone(),
            mentor: mentor.clone(),
            amount: payment.amount.clone(),
            deadline: deadline_timestamp,
            status: SessionStatus::Open,
            created_at: now,
        });

        self.session_created_event(id, &student, &mentor, deadline_timestamp, &payment.amount);
        id
    }

    /// Student confirms the mentoring session is complete; funds released to mentor.
    #[endpoint(confirmCompletion)]
    fn confirm_completion(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        let mut session = self.get_open_session(id);
        require!(caller == session.student, "Only the student can confirm");
        self.release_to_mentor(id, &mut session, SessionStatus::Completed);
    }

    /// Student or mentor raises a dispute.
    #[endpoint(disputeSession)]
    fn dispute_session(&self, id: u64, reason: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        let mut session = self.get_open_session(id);
        require!(
            caller == session.student || caller == session.mentor,
            "Only student or mentor can dispute"
        );
        session.status = SessionStatus::Disputed;
        self.sessions(id).set(session.clone());
        self.session_disputed_event(id, &caller, &reason);
    }

    /// Owner resolves a dispute by specifying which party wins.
    #[only_owner]
    #[endpoint(resolveDispute)]
    fn resolve_dispute(&self, id: u64, winner: ManagedAddress) {
        let mut session = self.sessions(id).get();
        require!(
            session.status == SessionStatus::Disputed,
            "Session is not disputed"
        );
        require!(
            winner == session.student || winner == session.mentor,
            "Winner must be student or mentor"
        );
        let amount = session.amount.clone();
        session.status = SessionStatus::Completed;
        self.sessions(id).set(session);
        let token_id = self.ect_token_id().get();
        self.send().direct_esdt(&winner, &token_id, 0, &amount);
        self.session_resolved_event(id, &winner);
    }

    /// Anyone can expire a session after the deadline passes.
    /// Default policy: funds released to mentor (assumed completion).
    #[endpoint(expireSession)]
    fn expire_session(&self, id: u64) {
        let now: u64 = self.blockchain().get_block_timestamp();
        let mut session = self.get_open_session(id);
        require!(now > session.deadline, "Session deadline not yet reached");
        self.release_to_mentor(id, &mut session, SessionStatus::Expired);
    }

    // ─── Views ──────────────────────────────

    #[view(getSession)]
    fn get_session(&self, id: u64) -> Session<Self::Api> {
        self.sessions(id).get()
    }

    #[view(getNextSessionId)]
    fn get_next_session_id(&self) -> u64 {
        self.next_session_id().get()
    }

    // ─── Helpers ────────────────────────────

    fn get_open_session(&self, id: u64) -> Session<Self::Api> {
        require!(!self.sessions(id).is_empty(), "Session not found");
        let session = self.sessions(id).get();
        require!(
            session.status == SessionStatus::Open,
            "Session is not open"
        );
        session
    }

    fn release_to_mentor(
        &self,
        id: u64,
        session: &mut Session<Self::Api>,
        new_status: SessionStatus,
    ) {
        let mentor = session.mentor.clone();
        let amount = session.amount.clone();
        session.status = new_status;
        self.sessions(id).set(session.clone());
        let token_id = self.ect_token_id().get();
        self.send().direct_esdt(&mentor, &token_id, 0, &amount);
        match new_status {
            SessionStatus::Completed => {
                self.session_completed_event(id, &session.student, &mentor);
            }
            SessionStatus::Expired => {
                self.session_expired_event(id, &mentor);
            }
            _ => {}
        }
    }

    // ─── Events ─────────────────────────────

    #[event("session_created")]
    fn session_created_event(
        &self,
        #[indexed] id: u64,
        #[indexed] student: &ManagedAddress,
        #[indexed] mentor: &ManagedAddress,
        #[indexed] deadline: u64,
        amount: &BigUint,  // single data arg
    );

    #[event("session_completed")]
    fn session_completed_event(
        &self,
        #[indexed] id: u64,
        #[indexed] student: &ManagedAddress,
        mentor: &ManagedAddress,  // single data arg
    );

    #[event("session_disputed")]
    fn session_disputed_event(
        &self,
        #[indexed] id: u64,
        #[indexed] initiator: &ManagedAddress,
        reason: &ManagedBuffer,
    );

    #[event("session_resolved")]
    fn session_resolved_event(
        &self,
        #[indexed] id: u64,
        #[indexed] winner: &ManagedAddress,
    );

    #[event("session_expired")]
    fn session_expired_event(
        &self,
        #[indexed] id: u64,
        released_to: &ManagedAddress,
    );

    // ─── Storage ────────────────────────────

    #[storage_mapper("ectTokenId")]
    fn ect_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("nextSessionId")]
    fn next_session_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("sessions")]
    fn sessions(&self, id: u64) -> SingleValueMapper<Session<Self::Api>>;
}
