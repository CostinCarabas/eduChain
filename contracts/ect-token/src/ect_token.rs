#![no_std]

use multiversx_sc::imports::*;

/// EduChain Token ($ECT) — fungible ESDT for rewards and P2P payments.
/// Manages an internal treasury pool from which rewards are allocated
/// to beneficiaries and claimed by them.
#[multiversx_sc::contract]
pub trait EctToken: multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule {
    // ─── lifecycle ──────────────────────────
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    // ─── Token setup (owner only) ────────────

    /// Issue the $ECT fungible ESDT. Payable in EGLD (system SC fee).
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        decimals: u32,
        initial_supply: BigUint,
    ) {
        let payment = self.call_value().egld();
        self.ect_token_id().issue(
            payment.clone_value(),
            token_name,
            token_ticker,
            initial_supply,
            decimals as usize,
            None,
        );
    }

    /// Grant LocalMint + LocalBurn roles to the SC address.
    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        self.require_token_issued();
        self.ect_token_id().set_local_roles(
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
            None,
        );
    }

    // ─── RBAC: distributor management ────────

    #[only_owner]
    #[endpoint(addDistributor)]
    fn add_distributor(&self, addr: ManagedAddress) {
        self.authorized_distributors().insert(addr);
    }

    #[only_owner]
    #[endpoint(removeDistributor)]
    fn remove_distributor(&self, addr: ManagedAddress) {
        self.authorized_distributors().remove(&addr);
    }

    #[view(isDistributor)]
    fn is_distributor(&self, addr: ManagedAddress) -> bool {
        self.authorized_distributors().contains(&addr)
    }

    // ─── Treasury management ─────────────────

    /// Mint new $ECT and add to the internal treasury pool (only owner).
    #[only_owner]
    #[endpoint(mintRewards)]
    fn mint_rewards(&self, amount: BigUint) {
        self.require_token_issued();
        require!(amount > 0u64, "Amount must be > 0");
        let token_id = self.ect_token_id().get_token_id();
        self.send().esdt_local_mint(&token_id, 0, &amount);
        self.treasury_balance().update(|b| *b += &amount);
    }

    /// Allocate rewards to a beneficiary (moves from treasury to pending balance).
    /// Caller must be an authorized distributor.
    #[endpoint(addReward)]
    fn add_reward(
        &self,
        beneficiary: ManagedAddress,
        amount: BigUint,
        reason: ManagedBuffer,
    ) {
        self.require_authorized_distributor();
        require!(amount > 0u64, "Amount must be > 0");
        let treasury = self.treasury_balance().get();
        require!(amount <= treasury, "Insufficient treasury balance");
        self.pending_balance(&beneficiary)
            .update(|b| *b += &amount);
        self.treasury_balance().update(|b| *b -= &amount);
        self.reward_added_event(&beneficiary, &amount, &reason);
    }

    /// Caller claims their pending $ECT balance.
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        let amount = self.pending_balance(&caller).get();
        require!(amount > 0u64, "No pending rewards");
        self.pending_balance(&caller).clear();
        let token_id = self.ect_token_id().get_token_id();
        self.send().direct_esdt(&caller, &token_id, 0, &amount);
        self.total_distributed().update(|t| *t += &amount);
        self.reward_claimed_event(&caller, &amount);
    }

    /// Direct P2P payment from treasury to receiver (authorized distributors only).
    /// Used by the Escrow SC or backend for direct settlements.
    #[endpoint(payP2P)]
    fn pay_p2p(&self, receiver: ManagedAddress, amount: BigUint) {
        self.require_authorized_distributor();
        require!(amount > 0u64, "Amount must be > 0");
        let treasury = self.treasury_balance().get();
        require!(amount <= treasury, "Insufficient treasury balance");
        let token_id = self.ect_token_id().get_token_id();
        self.treasury_balance().update(|b| *b -= &amount);
        self.send().direct_esdt(&receiver, &token_id, 0, &amount);
        self.transfer_made_event(&receiver, &amount);
    }

    // ─── Views ──────────────────────────────

    #[view(getPendingBalance)]
    fn get_pending_balance(&self, addr: ManagedAddress) -> BigUint {
        self.pending_balance(&addr).get()
    }

    #[view(getTreasuryBalance)]
    fn get_treasury_balance_view(&self) -> BigUint {
        self.treasury_balance().get()
    }

    #[view(getTotalDistributed)]
    fn get_total_distributed(&self) -> BigUint {
        self.total_distributed().get()
    }

    #[view(getTokenId)]
    fn get_token_id(&self) -> TokenIdentifier {
        self.ect_token_id().get_token_id()
    }

    // ─── Helpers ────────────────────────────

    fn require_authorized_distributor(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.authorized_distributors().contains(&caller),
            "Caller is not an authorized distributor"
        );
    }

    fn require_token_issued(&self) {
        require!(!self.ect_token_id().is_empty(), "ECT token not yet issued");
    }

    // Custom callback removed, using FungibleTokenMapper's default callback

    // ─── Events ─────────────────────────────

    #[event("reward_added")]
    fn reward_added_event(
        &self,
        #[indexed] beneficiary: &ManagedAddress,
        #[indexed] amount: &BigUint,
        reason: &ManagedBuffer,  // single data arg
    );

    #[event("reward_claimed")]
    fn reward_claimed_event(
        &self,
        #[indexed] beneficiary: &ManagedAddress,
        amount: &BigUint,
    );

    #[event("transfer_made")]
    fn transfer_made_event(
        &self,
        #[indexed] receiver: &ManagedAddress,
        amount: &BigUint,
    );

    // ─── Storage ────────────────────────────

    #[storage_mapper("ectTokenId")]
    fn ect_token_id(&self) -> FungibleTokenMapper<Self::Api>;

    #[storage_mapper("authorizedDistributors")]
    fn authorized_distributors(&self) -> SetMapper<ManagedAddress>;

    /// Pending (claimable) balance per address.
    #[storage_mapper("pendingBalance")]
    fn pending_balance(&self, addr: &ManagedAddress) -> SingleValueMapper<BigUint>;

    /// Total amount ever distributed (claimed).
    #[storage_mapper("totalDistributed")]
    fn total_distributed(&self) -> SingleValueMapper<BigUint>;

    /// Unallocated treasury pool held by this SC.
    #[storage_mapper("treasuryBalance")]
    fn treasury_balance(&self) -> SingleValueMapper<BigUint>;
}
