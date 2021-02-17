use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use zcash_client_backend::wallet::AccountId;

pub const DATA_PATH: &str = "data.sqlite3";
pub const CACHE_PATH: &str = "cache.sqlite3";
pub const LIGHTNODE_URL: &str = "http://127.0.0.1:9067";

pub mod grpc {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}

pub mod account;
pub mod chain;
mod checkpoint;
pub mod keys;
pub mod sign;
pub mod transact;

pub const ACCOUNT: AccountId = AccountId(0);
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

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

pub const MAX_REORG_DEPTH: u64 = 3;

#[derive(Debug, Clone)]
pub enum WalletError {
    NotEnoughFunds(u64, u64, ZECUnit),
    Decode(String),
    Prover,
    TxParse,
    AccountNotInitialized,
}

impl std::error::Error for WalletError {}
impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::NotEnoughFunds(a, b, unit) => write!(
                f,
                "Not enough funds: {} < {} {}",
                unit.from_satoshis(*a),
                unit.from_satoshis(*b),
                unit
            ),
            WalletError::Decode(m) => write!(f, "Could not decode {}", m),
            WalletError::Prover => write!(f, "Could not create ZKSnark prover. Did you download the parameters?"),
            WalletError::TxParse => write!(f, "Could not parse transaction file"),
            WalletError::AccountNotInitialized => write!(f, "Account not initialized. Did you use init-account?"),
        }
    }
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
}
