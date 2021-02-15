use crate::{
    grpc::{compact_tx_streamer_client::CompactTxStreamerClient, RawTransaction},
    Result, Tx, TxIn, TxOut, ACCOUNT, DATA_PATH,
};
use std::fmt::Formatter;
use zcash_client_backend::{
    address::RecipientAddress, data_api::WalletRead, encoding::encode_payment_address,
};
use zcash_client_sqlite::WalletDB;
use zcash_primitives::{
    consensus::Network::TestNetwork,
    constants::testnet::HRP_SAPLING_PAYMENT_ADDRESS,
    primitives::Rseed,
    transaction::components::{amount::DEFAULT_FEE, Amount},
};

#[derive(Debug, Clone)]
pub enum WalletError {
    NotEnoughFunds(String),
    Zip212NotImplemented, // Need to implement after Canopy
}

impl std::error::Error for WalletError {}
impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn prepare_tx(to_addr: &str, amount: u64) -> Result<Tx> {
    let to_addr = RecipientAddress::decode(&TestNetwork, to_addr).expect("Unable to decode address");
    let amount = Amount::from_u64(amount).expect("Invalid amount");
    let wallet_db = WalletDB::for_path(DATA_PATH, TestNetwork)?;
    let fvks = wallet_db.get_extended_full_viewing_keys()?;
    let extfvk = &fvks[&ACCOUNT];
    let ovk = extfvk.fvk.ovk;

    // Target the next block, assuming we are up-to-date.
    let (height, anchor_height) = wallet_db.get_target_and_anchor_heights()?.unwrap();

    let target_value = amount + DEFAULT_FEE;
    let spendable_notes = wallet_db.select_spendable_notes(ACCOUNT, target_value, anchor_height)?;

    // Confirm we were able to select sufficient value
    let selected_value: Amount = spendable_notes.iter().map(|n| n.note_value).sum();
    if selected_value < target_value {
        return Err(WalletError::NotEnoughFunds(format!(
            "Insufficient balance {:?} < {:?}",
            selected_value, target_value
        ))
        .into());
    }

    let mut tx = Tx {
        height: i64::from(height),
        inputs: Vec::new(),
        output: None,
    };

    // Create the transaction
    for selected in spendable_notes {
        let from = extfvk
            .fvk
            .vk
            .to_payment_address(selected.diversifier)
            .expect("Could not convert viewing key to payment address");

        let d = selected.diversifier.0;
        let paddr = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &from);
        let a = u64::from(selected.note_value);
        let rseed = match selected.rseed {
            Rseed::BeforeZip212(s) => s.to_bytes(),
            _ => return Err(WalletError::Zip212NotImplemented.into()),
        };
        let mut mp = Vec::<u8>::new();
        selected.witness.write(&mut mp)?;

        tx.inputs.push(TxIn {
            diversifier: hex::encode(d),
            addr: paddr,
            amount: a,
            rseed: hex::encode(rseed),
            witness: hex::encode(mp),
        });
    }

    match to_addr {
        RecipientAddress::Shielded(to) => {
            tx.output = Some(TxOut {
                ovk: hex::encode(ovk.0),
                addr: encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &to),
                amount: u64::from(amount),
            });
            // builder.add_sapling_output(Some(ovk), to.clone(), amount, None)?;
        }

        RecipientAddress::Transparent(_) => unimplemented!(),
    }

    Ok(tx)
}

pub async fn submit(lightnode_url: &str, raw_tx: RawTransaction) -> Result<()> {
    let lightnode_url = lightnode_url.to_string();
    let mut client = CompactTxStreamerClient::connect(lightnode_url).await?;
    let r = client.send_transaction(raw_tx).await?;
    println!("{:?}", r.into_inner());

    Ok(())
}
