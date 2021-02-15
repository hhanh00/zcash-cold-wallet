use serde::{Deserialize, Serialize};
use zcash_client_backend::wallet::AccountId;

pub const DATA_PATH: &str = "data.sqlite3";
pub const CACHE_PATH: &str = "cache.sqlite3";
pub const LIGHTNODE_URL: &str = "http://127.0.0.1:9067";

pub mod grpc {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}

pub mod keys;
pub mod account;
pub mod chain;
mod checkpoint;
pub mod transact;
pub mod sign;

pub const ACCOUNT: AccountId = AccountId(0);
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

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
    rseed: String,
    witness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    addr: String,
    amount: u64,
    ovk: String,
}

