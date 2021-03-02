use crate::constants::NETWORK;
use crate::{
    connect_lightnode,
    grpc::{BlockId, ChainSpec},
    Result,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use zcash_primitives::consensus::{NetworkUpgrade, Parameters};

pub struct Checkpoint {
    pub height: u64,
    pub hash: Vec<u8>,
    pub time: u32,
    pub sapling_tree: String,
}

pub async fn find_checkpoint(lightnode_url: &str, height: u64) -> Result<Checkpoint> {
    let lightnode_url = lightnode_url.to_string();
    let mut client = connect_lightnode(lightnode_url).await?;
    let tree_state = client
        .get_tree_state(BlockId {
            height,
            hash: Vec::new(),
        })
        .await?
        .into_inner();
    let mut hash = hex::decode(tree_state.hash)?;
    hash.reverse();
    let checkpoint = Checkpoint {
        height: tree_state.height,
        hash,
        time: tree_state.time,
        sapling_tree: tree_state.tree,
    };

    Ok(checkpoint)
}

pub async fn find_height(lightnode_url: &str, date: &NaiveDate) -> Result<u64> {
    let mut client = connect_lightnode(lightnode_url.to_string()).await?;
    let mut low: u64 = NETWORK
        .activation_height(NetworkUpgrade::Sapling)
        .unwrap()
        .into();
    let mut high: u64 = client
        .get_latest_block(ChainSpec {})
        .await?
        .into_inner()
        .height;
    let datetime = NaiveDateTime::new(date.clone(), NaiveTime::from_hms(0, 0, 0));
    let timestamp = datetime.timestamp() as u32;

    let height = loop {
        if low >= high {
            break high;
        }
        let mid = (low + high) / 2;
        let block = client
            .get_block(BlockId {
                height: mid,
                hash: Vec::new(),
            })
            .await?
            .into_inner();
        let height = block.height;
        if timestamp == block.time {
            break height;
        } else if timestamp < block.time {
            high = mid - 1;
        } else {
            low = mid + 1;
        }
    };

    Ok(height)
}
