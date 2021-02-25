use crate::{
    grpc::{compact_tx_streamer_client::CompactTxStreamerClient, BlockId},
    Result,
};

pub struct Checkpoint {
    pub height: u64,
    pub hash: Vec<u8>,
    pub time: u32,
    pub sapling_tree: String,
}

pub async fn find_checkpoint(lightnode_url: &str, height: u64) -> Result<Checkpoint> {
    let lightnode_url = lightnode_url.to_string();
    let mut client = CompactTxStreamerClient::connect(lightnode_url).await?;
    let tree_state = client.get_tree_state(BlockId { height, hash: Vec::new() }).await?.into_inner();
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
