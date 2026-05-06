use multiversx_sc_snippets::imports::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

pub const STATE_FILE: &str = "state.toml";

/// Persistent state: addresses of all 4 deployed contracts + ECT token identifier.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub nft_contract: Option<Bech32Address>,
    pub token_contract: Option<Bech32Address>,
    pub escrow_contract: Option<Bech32Address>,
    pub anchor_contract: Option<Bech32Address>,
    /// ECT fungible ESDT token identifier (e.g. "ECT-a1b2c3"), set after `token issue`.
    pub ect_token_id: Option<String>,
}

impl State {
    pub fn load() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn nft_address(&self) -> &Bech32Address {
        self.nft_contract
            .as_ref()
            .expect("NFT contract not deployed — run: interactor deploy nft")
    }

    pub fn token_address(&self) -> &Bech32Address {
        self.token_contract
            .as_ref()
            .expect("Token contract not deployed — run: interactor deploy token")
    }

    pub fn escrow_address(&self) -> &Bech32Address {
        self.escrow_contract
            .as_ref()
            .expect("Escrow contract not deployed — run: interactor deploy escrow")
    }

    pub fn anchor_address(&self) -> &Bech32Address {
        self.anchor_contract
            .as_ref()
            .expect("Anchor contract not deployed — run: interactor deploy anchor")
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}
