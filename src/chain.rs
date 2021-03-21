use crate::{
    connect_lightnode,
    constants::NETWORK,
    grpc::{BlockId, BlockRange, ChainSpec},
    Result, WalletError, CACHE_PATH, DATA_PATH, MAX_REORG_DEPTH,
};
use prost::{bytes::BytesMut, Message};
use rusqlite::{params, Connection, NO_PARAMS};
use zcash_client_backend::{
    data_api::{chain::scan_cached_blocks, WalletRead}
};
use zcash_client_sqlite::{
    chain::init::init_cache_database,
    wallet::init::init_wallet_db,
    BlockDB, WalletDB,
};

pub fn init_db() -> Result<()> {
    let db_data = WalletDB::for_path(DATA_PATH, NETWORK)?;
    init_wallet_db(&db_data)?;

    let db_cache = BlockDB::for_path(CACHE_PATH)?;
    init_cache_database(&db_cache)?;

    Ok(())
}

pub async fn sync(lightnode_url: &str) -> Result<()> {
    let lightnode_url = lightnode_url.to_string();
    let cache_connection = Connection::open(CACHE_PATH)?;
    let wallet_db = WalletDB::for_path(DATA_PATH, NETWORK)?;
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

    let synced_height = latest_block.height - MAX_REORG_DEPTH;
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

    println!("Synced to {}", synced_height);

    scan()?;

    Ok(())
}

pub fn scan() -> Result<()> {
    let cache = BlockDB::for_path(CACHE_PATH)?;
    let db_read = WalletDB::for_path(DATA_PATH, NETWORK)?;
    let mut data = db_read.get_update_ops()?;
    scan_cached_blocks(&NETWORK, &cache, &mut data, None)?;

    println!("Scan completed");
    Ok(())
}
