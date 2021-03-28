use crate::{
    connect_lightnode,
    constants::NETWORK,
    grpc::{BlockId, BlockRange, ChainSpec, RawTransaction},
    Result, WalletError, CACHE_PATH, DATA_PATH, MAX_REORG_DEPTH,
};
use anyhow::Context;
use prost::{bytes::BytesMut, Message};
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::PathBuf;
use zcash_client_backend::data_api::{chain::scan_cached_blocks, WalletRead};
use zcash_client_sqlite::{
    chain::init::init_cache_database, wallet::init::init_wallet_db, BlockDB, WalletDB,
};

pub fn init_db(directory_path: &str) -> Result<()> {
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let db_data = WalletDB::for_path(data_path, NETWORK)?;
    init_wallet_db(&db_data)?;

    let cache_path: PathBuf = [directory_path, CACHE_PATH].iter().collect();
    let db_cache = BlockDB::for_path(cache_path)?;
    init_cache_database(&db_cache)?;

    Ok(())
}

pub async fn sync(directory_path: &str, lightnode_url: &str, max_blocks: u32) -> Result<u64> {
    let cache_path: PathBuf = [directory_path, CACHE_PATH].iter().collect();
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let lightnode_url = lightnode_url.to_string();
    let cache_connection = Connection::open(cache_path.clone())?;
    let wallet_db = WalletDB::for_path(data_path.clone(), NETWORK)?;
    let (_, last_bh) = wallet_db
        .block_height_extrema()?
        .ok_or(WalletError::AccountNotInitialized)?;

    let start_height: u64 = cache_connection
        .query_row("SELECT MAX(height) FROM compactblocks", NO_PARAMS, |row| {
            Ok(row.get::<_, u32>(0).map(u64::from).map(|h| h + 1).ok())
        })?
        .unwrap_or_else(|| u64::from(last_bh));
    println!("Starting height: {}", start_height);

    let mut client = connect_lightnode(lightnode_url).await?;
    let latest_block = client
        .get_latest_block(tonic::Request::new(ChainSpec {}))
        .await?
        .into_inner();

    let synced_height =
        (latest_block.height - MAX_REORG_DEPTH).min(start_height.saturating_add(max_blocks as u64));
    let mut blocks = client
        .get_block_range(tonic::Request::new(BlockRange {
            start: Some(BlockId {
                hash: Vec::new(),
                height: start_height,
            }),
            end: Some(BlockId {
                hash: Vec::new(),
                height: synced_height,
            }),
        }))
        .await?
        .into_inner();

    let mut statement =
        cache_connection.prepare("INSERT INTO compactblocks (height, data) VALUES (?, ?)")?;
    while let Some(cb) = blocks.message().await? {
        let mut cb_bytes = BytesMut::with_capacity(cb.encoded_len());
        cb.encode_raw(&mut cb_bytes);
        statement.execute(params![cb.height as u32, cb_bytes.to_vec()])?;
    }

    log::debug!("Synced from {} to {}", start_height, synced_height);

    scan(directory_path)?;

    Ok(synced_height.saturating_sub(start_height))
}

pub fn scan(directory_path: &str) -> Result<()> {
    let cache_path: PathBuf = [directory_path, CACHE_PATH].iter().collect();
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let cache = BlockDB::for_path(cache_path)?;
    let db_read = WalletDB::for_path(data_path, NETWORK)?;
    let mut data = db_read.get_update_ops()?;
    scan_cached_blocks(&NETWORK, &cache, &mut data, None)?;

    log::debug!("Scan completed");
    Ok(())
}

