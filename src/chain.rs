use crate::checkpoint::CHECKPOINT;
use crate::{
    grpc::{
        compact_tx_streamer_client::CompactTxStreamerClient, BlockId, BlockRange, ChainSpec,
    },
    Result, CACHE_PATH, DATA_PATH,
};
use prost::bytes::BytesMut;
use prost::Message;
use rusqlite::{params, Connection, NO_PARAMS};
use zcash_client_backend::data_api::chain::scan_cached_blocks;
use zcash_client_backend::encoding::decode_extended_full_viewing_key;
use zcash_client_sqlite::{
    chain::init::init_cache_database,
    wallet::init::{init_accounts_table, init_blocks_table, init_wallet_db},
    BlockDB, WalletDB,
};
use zcash_primitives::block::BlockHash;
use zcash_primitives::consensus::{BlockHeight, Network};
use zcash_primitives::constants::testnet::HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY;

pub fn init_db() -> Result<()> {
    let db_data = WalletDB::for_path(DATA_PATH, Network::TestNetwork)?;
    init_wallet_db(&db_data)?;

    let db_cache = BlockDB::for_path(CACHE_PATH)?;
    init_cache_database(&db_cache)?;

    Ok(())
}

pub fn init_account(viewing_key: String) -> Result<()> {
    let db_data = WalletDB::for_path(DATA_PATH, Network::TestNetwork)?;
    let extfvks =
        decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &viewing_key)?
            .expect("Cannot decode viewing key");
    init_accounts_table(&db_data, &[extfvks])?;

    init_blocks_table(
        &db_data,
        BlockHeight::from_u32(CHECKPOINT.height),
        BlockHash::from_slice(&CHECKPOINT.hash),
        CHECKPOINT.time,
        &hex::decode(CHECKPOINT.sapling_tree).unwrap(),
    )?;
    Ok(())
}

pub async fn sync(lightnode_url: &str) -> Result<()> {
    let lightnode_url = lightnode_url.to_string();
    let cache_connection = Connection::open(CACHE_PATH)?;

    let start_height: u64 = cache_connection
        .query_row("SELECT MAX(height) FROM compactblocks", NO_PARAMS, |row| {
            Ok(row.get::<_, u32>(0).map(u64::from).map(|h| h + 1).ok())
        })?
        .unwrap_or(CHECKPOINT.height as u64);
    println!("Starting height: {}", start_height);

    let mut client = CompactTxStreamerClient::connect(lightnode_url).await?;
    let latest_block = client
        .get_latest_block(tonic::Request::new(ChainSpec {}))
        .await?
        .into_inner();

    let mut blocks = client
        .get_block_range(tonic::Request::new(BlockRange {
            start: Some(BlockId {
                hash: Vec::new(),
                height: start_height,
            }),
            end: Some(BlockId {
                hash: Vec::new(),
                height: latest_block.height,
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

    println!("Synced to {}", latest_block.height);

    let cache = BlockDB::for_path(CACHE_PATH)?;
    let db_read = WalletDB::for_path(DATA_PATH, Network::TestNetwork)?;
    let mut data = db_read.get_update_ops()?;
    scan_cached_blocks(&Network::TestNetwork, &cache, &mut data, None)?;

    println!("Scan completed");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::LIGHTNODE_URL;

    #[test]
    fn test_init() -> Result<()> {
        init_db()?;
        init_account("zxviewtestsapling1q07ghkk6qqqqpqyqnt30u2gwd5j47fjldmtyunrm99qmaqhp2j3kpqg6k8mvyferpde3vgwndlumht98q29796a6wjujthsxterqh9sjhscaqsmx3tfc6rkt2k9qrkamzpcc5qcskak8cec6ukqysatjxhgdqthh6qnmd53sqfae8nw4z33uletfstrsf0umxpztc365h7vy4jmyw65q6ns5eqkljsquyldn80ssn6hly86zwkx39qvcvzl5psrhj85vcaln6ylacccxrr0kv".to_string())?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sync() -> Result<()> {
        sync(LIGHTNODE_URL).await
    }
}
