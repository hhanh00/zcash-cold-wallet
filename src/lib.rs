use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use zcash_client_backend::wallet::AccountId;
use thiserror::Error;
use crate::grpc::compact_tx_streamer_client::CompactTxStreamerClient;

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
pub mod multisig;

pub const ACCOUNT: AccountId = AccountId(0);
pub use anyhow::Result as Result;
use tonic::transport::{ClientTlsConfig, Channel};
use std::fs::File;
use jubjub::ExtendedPoint;
use group::GroupEncoding;

#[derive(Debug, Clone)]
pub enum ZECUnit {
    Zat,
    MilliZec,
    Zec,
}

impl FromStr for ZECUnit {
    type Err = std::io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Zat" => return Ok(ZECUnit::Zat),
            "MilliZec" => return Ok(ZECUnit::MilliZec),
            "Zec" => return Ok(ZECUnit::Zec),
            _ => panic!("Unit must be one of Zat, MilliZec or Zec"),
        }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Tx {
    height: i64,
    inputs: Vec<TxIn>,
    output: Option<TxOut>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningCommitments {
    index: u32,
    randomizer: String, // ExtendedPoint
    hiding: String,
    binding: String,
    randomizer_nonce: String, // Scalar
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningNonces {
    randomizer: String,
    hiding: String,
    binding: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigningShare {
    pub index: u32,
    pub commitment: SigningCommitments,
    pub signature: Option<[u8; 32]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxIn {
    diversifier: String,
    fvk: String,
    amount: u64,
    z212: bool,
    rseed: String,
    witness: String,
    multisigs: Vec<SigningShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    addr: String,
    amount: u64,
    ovk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxBin {
    bytes: String,
    commitments: Vec<Vec<SigningCommitments>>,
    sighash: String,
    spend_indices: Vec<usize>,
    output_indices: Vec<usize>,
}

pub const MAX_REORG_DEPTH: u64 = 3;

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
        let tls = ClientTlsConfig::new();
        channel = channel.tls_config(tls)?;
    }
    let client = CompactTxStreamerClient::connect(channel).await?;
    Ok(client)
}

pub fn read_from_file(file_name: Option<String>) -> String {
    let mut input: Box<dyn std::io::Read> = match file_name {
        Some(file_name) => Box::new(File::open(file_name).unwrap()),
        None => Box::new(std::io::stdin()),
    };
    let mut s = String::new();
    input.read_to_string(&mut s).unwrap();
    s.trim_end().to_string()

}

pub fn create_file(filename: Option<String>) -> Result<Box<dyn std::io::Write>> {
    let output: Box<dyn std::io::Write> = match filename {
        Some(file_name) => Box::new(File::create(file_name)?),
        None => Box::new(std::io::stdout()),
    };
    Ok(output)
}

pub fn decode_extended_point(s: &str) -> ExtendedPoint {
    let mut b = [0u8; 32];
    hex::decode_to_slice(s, &mut b).unwrap();
    ExtendedPoint::from_bytes(&b).unwrap()
}

pub fn decode_scalar(s: &str) -> jubjub::Fr {
    let mut b = [0u8; 32];
    hex::decode_to_slice(s, &mut b).unwrap();
    jubjub::Fr::from_bytes(&b).unwrap()
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
