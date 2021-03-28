use crate::grpc::compact_tx_streamer_client::CompactTxStreamerClient;
use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use tonic::transport::Certificate;
use zcash_client_backend::wallet::AccountId;

pub const DATA_PATH: &str = "data.sqlite3";
pub const CACHE_PATH: &str = "cache.sqlite3";

pub mod grpc {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}

pub mod account;
pub mod chain;
pub mod checkpoint;
pub mod keys;
pub mod sign;
pub mod transact;

pub const ACCOUNT: AccountId = AccountId(0);
pub use anyhow::Result;
use tonic::transport::{Channel, ClientTlsConfig};
use zcash_client_backend::data_api::wallet::ANCHOR_OFFSET;

#[derive(Debug, Clone)]
pub enum ZECUnit {
    Zat,
    MilliZec,
    Zec,
}

impl FromStr for ZECUnit {
    type Err = std::io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "Zat" => ZECUnit::Zat,
            "MilliZec" => ZECUnit::MilliZec,
            "Zec" => ZECUnit::Zec,
            _ => panic!("Unit must be one of Zat, MilliZec or Zec"),
        })
    }
}

impl std::fmt::Display for ZECUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZECUnit::Zat => write!(f, "zatoshis"),
            ZECUnit::MilliZec => write!(f, "mZEC"),
            ZECUnit::Zec => write!(f, "ZEC"),
        }
    }
}

impl ZECUnit {
    pub fn to_satoshis(&self, amount: &str) -> u64 {
        let u = BigDecimal::from_str(amount).unwrap();
        let r = match self {
            ZECUnit::Zec => u * BigDecimal::from(100_000_000),
            ZECUnit::MilliZec => u * BigDecimal::from(100_000),
            ZECUnit::Zat => u,
        };
        r.to_u64().unwrap()
    }

    pub fn from_satoshis(&self, amount: u64) -> String {
        let u = BigDecimal::from(amount);
        let r = match self {
            ZECUnit::Zec => u / BigDecimal::from(100_000_000),
            ZECUnit::MilliZec => u / BigDecimal::from(100_000),
            ZECUnit::Zat => u,
        };
        r.to_string()
    }
}

pub struct Opt {
    pub lightnode_url: String,
    pub unit: ZECUnit,
}

impl Opt {
    pub fn default() -> Opt {
        Opt {
            lightnode_url: constants::LIGHTNODE_URL.to_string(),
            unit: ZECUnit::Zec,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tx {
    height: i64,
    inputs: Vec<TxIn>,
    output: Option<TxOut>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxIn {
    diversifier: String,
    addr: String,
    amount: u64,
    z212: bool,
    rseed: String,
    witness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    addr: String,
    amount: u64,
    ovk: String,
}

pub const MAX_REORG_DEPTH: u64 = 0u64;

#[derive(Error, Debug, Clone)]
pub enum WalletError {
    #[error("Not enough funds: {} < {} {}", .2.from_satoshis(*.0), .2.from_satoshis(*.1), .2)]
    NotEnoughFunds(u64, u64, ZECUnit),
    #[error("Could not decode {}", .0)]
    Decode(String),
    #[error("Could not create ZKSnark prover. Did you download the parameters?")]
    Prover,
    #[error("Could not parse transaction file")]
    TxParse,
    #[error("Account not initialized. Did you use init-account?")]
    AccountNotInitialized,
    #[error("Failed to submit transaction. Error code {}, Error Message {}", .0, .1)]
    Submit(i32, String),
}

async fn connect_lightnode(lightnode_url: String) -> Result<CompactTxStreamerClient<Channel>> {
    let mut channel = tonic::transport::Channel::from_shared(lightnode_url.clone())?;
    if lightnode_url.starts_with("https") {
        let pem = include_bytes!("ca.pem");
        let ca = Certificate::from_pem(pem);
        let tls = ClientTlsConfig::new().ca_certificate(ca);
        channel = channel.tls_config(tls)?;
    }
    let client = CompactTxStreamerClient::connect(channel).await?;
    Ok(client)
}

#[cfg(not(feature = "mainnet"))]
pub mod constants {
    use zcash_primitives::consensus::Network::{self, TestNetwork};
    use zcash_primitives::constants::testnet;

    pub const NETWORK: Network = TestNetwork;
    pub const HRP_SAPLING_EXTENDED_SPENDING_KEY: &str = testnet::HRP_SAPLING_EXTENDED_SPENDING_KEY;
    pub const HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY: &str =
        testnet::HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY;
    pub const HRP_SAPLING_PAYMENT_ADDRESS: &str = testnet::HRP_SAPLING_PAYMENT_ADDRESS;
    pub const COIN_TYPE: u32 = testnet::COIN_TYPE;
    pub const LIGHTNODE_URL: &str = "https://testnet.lightwalletd.com:9067";
}

#[cfg(feature = "mainnet")]
pub mod constants {
    use zcash_primitives::consensus::Network::{self, MainNetwork};
    use zcash_primitives::constants::mainnet;

    pub const NETWORK: Network = MainNetwork;
    pub const HRP_SAPLING_EXTENDED_SPENDING_KEY: &str = mainnet::HRP_SAPLING_EXTENDED_SPENDING_KEY;
    pub const HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY: &str =
        mainnet::HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY;
    pub const HRP_SAPLING_PAYMENT_ADDRESS: &str = mainnet::HRP_SAPLING_PAYMENT_ADDRESS;
    pub const COIN_TYPE: u32 = mainnet::COIN_TYPE;
    pub const LIGHTNODE_URL: &str = "https://mainnet.lightwalletd.com:9067";
}
