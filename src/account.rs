use crate::{
    checkpoint::find_checkpoint,
    constants::{HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, NETWORK},
    Opt, Result, WalletError, DATA_PATH,
};
use rusqlite::{Connection, NO_PARAMS};
use zcash_client_backend::encoding::decode_extended_full_viewing_key;
use zcash_client_sqlite::{
    wallet::init::{init_accounts_table, init_blocks_table},
    WalletDB,
};
use zcash_primitives::{block::BlockHash, consensus::BlockHeight};
use anyhow::Context;

pub async fn init_account(lightnode_url: &str, viewing_key: String, height: u64) -> Result<()> {
    let db_data = WalletDB::for_path(DATA_PATH, NETWORK)?;
    let extfvks =
        decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &viewing_key)?
            .ok_or(WalletError::Decode(viewing_key))?;
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

pub fn get_balance(opts: &Opt) -> Result<()> {
    let data_connection = Connection::open(DATA_PATH)?;
    let balance = data_connection.query_row(
        "SELECT SUM(value) FROM received_notes WHERE spent IS NULL",
        NO_PARAMS,
        |row| row.get(0).or(Ok(0i64)),
    )?;
    let balance = opts.unit.from_satoshis(balance as u64);
    println!("Balance: {}", balance);

    Ok(())
}
