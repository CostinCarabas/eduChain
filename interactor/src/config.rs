#![allow(unused)]

use serde::Deserialize;
use std::io::Read;

pub const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Real,
    Simulator,
}

/// Source for the wallet private key — never commit the key itself.
/// For TRL-4: use "env" or "file". Vault is stubbed for TRL-5.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "source")]
pub enum KeySource {
    /// Read from env var. Value can be PEM content or a path to a PEM file.
    Env { env_var: String },
    /// Read PEM from filesystem path (dev only, never commit to git).
    File { file_path: String },
    /// TODO(TRL-5): HashiCorp Vault.
    Vault {
        vault_addr: String,
        vault_path: String,
    },
}

impl KeySource {
    pub fn load_pem(&self) -> String {
        match self {
            KeySource::Env { env_var } => {
                let val = std::env::var(env_var)
                    .unwrap_or_else(|_| panic!("Env var '{}' not set", env_var));
                if val.trim_start().starts_with("-----") {
                    val
                } else {
                    std::fs::read_to_string(val.trim())
                        .expect("Cannot read PEM file from path in env var")
                }
            }
            KeySource::File { file_path } => std::fs::read_to_string(file_path)
                .unwrap_or_else(|_| panic!("Cannot read PEM at '{}'", file_path)),
            KeySource::Vault { .. } => {
                // TODO(TRL-5): implement HashiCorp Vault lookup
                unimplemented!("Vault key source is a TRL-5 target")
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub gateway_uri: String,
    pub chain_type: ChainType,
    pub key: KeySource,
}

impl Config {
    pub fn new() -> Self {
        let mut file = std::fs::File::open(CONFIG_FILE).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        toml::from_str(&content).unwrap()
    }

    pub fn chain_simulator_config() -> Self {
        Config {
            gateway_uri: "http://localhost:8085".to_owned(),
            chain_type: ChainType::Simulator,
            key: KeySource::File {
                file_path: "../eduChain.pem".to_owned(),
            },
        }
    }

    pub fn gateway_uri(&self) -> &str {
        &self.gateway_uri
    }

    pub fn use_chain_simulator(&self) -> bool {
        matches!(self.chain_type, ChainType::Simulator)
    }
}
