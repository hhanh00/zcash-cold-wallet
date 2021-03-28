use crate::{checkpoint::find_checkpoint, constants::{HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, NETWORK}, Opt, Result, WalletError, DATA_PATH, ZECUnit};
use rusqlite::{Connection, NO_PARAMS};
use zcash_client_backend::encoding::decode_extended_full_viewing_key;
use zcash_client_sqlite::{
    wallet::init::{init_accounts_table, init_blocks_table},
    WalletDB,
};
use zcash_primitives::{block::BlockHash, consensus::BlockHeight};
use anyhow::Context;
use std::path::PathBuf;
use zcash_primitives::consensus::{Parameters, NetworkUpgrade};
use zcash_client_backend::data_api::WalletRead;

pub fn has_account(directory_path: &str) -> Result<bool> {
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let db_data = WalletDB::for_path(data_path, NETWORK)?;
    let keys = db_data.get_extended_full_viewing_keys()?;
    Ok(!keys.is_empty())
}

pub async fn init_account(directory_path: &str, lightnode_url: &str, viewing_key: &str, height: u64) -> Result<()> {
    let sapling_activation_height: u64 = crate::constants::NETWORK.activation_height(NetworkUpgrade::Sapling).unwrap().into();
    let height = height.max(sapling_activation_height);
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let db_data = WalletDB::for_path(data_path, NETWORK)?;
    let extfvks =
        decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &viewing_key)?
            .ok_or(WalletError::Decode(viewing_key.to_string()))?;
    init_accounts_table(&db_data, &[extfvks]).context("init_accounts_table")?;

    let checkpoint = find_checkpoint(lightnode_url, height).await?;
    init_blocks_table(
        &db_data,
        BlockHeight::from_u32(checkpoint.height as u32),
        BlockHash::from_slice(&checkpoint.hash),
        checkpoint.time,
        &hex::decode(checkpoint.sapling_tree).unwrap(),
    ).context("init_blocks_table")?;
    Ok(())
}

pub fn get_balance(directory_path: &str, unit: ZECUnit) -> Result<u64> {
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let data_connection = Connection::open(data_path)?;
    let balance = data_connection.query_row(
        "SELECT SUM(value) FROM received_notes WHERE spent IS NULL",
        NO_PARAMS,
        |row| row.get(0).or(Ok(0i64)),
    )?;
    let balance_str = unit.from_satoshis(balance as u64);
    println!("Balance: {}", balance_str);

    Ok(balance as u64)
}
