use crate::constants::{HRP_SAPLING_PAYMENT_ADDRESS, NETWORK};
use crate::{grpc::RawTransaction, Result, Tx, TxIn, TxOut, ACCOUNT, DATA_PATH, WalletError, connect_lightnode, ZECUnit, Opt};
use zcash_client_backend::{
    address::RecipientAddress, data_api::WalletRead, encoding::encode_payment_address,
};
use zcash_client_sqlite::WalletDB;
use zcash_primitives::{
    primitives::Rseed,
    transaction::components::{amount::DEFAULT_FEE, Amount},
};
use crate::sign::sign_tx;
use std::path::PathBuf;

pub fn prepare_tx(directory_path: &str, to_addr: &str, amount: String, unit: &ZECUnit) -> Result<Tx> {
    let data_path: PathBuf = [directory_path, DATA_PATH].iter().collect();
    let satoshis = unit.to_satoshis(&amount);
    let to_addr = RecipientAddress::decode(&NETWORK, to_addr).ok_or_else(|| WalletError::Decode(to_addr.to_string()))?;
    let amount = Amount::from_u64(satoshis).expect("Invalid amount");
    let wallet_db = WalletDB::for_path(data_path, NETWORK)?;
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
        return Err(WalletError::NotEnoughFunds(u64::from(selected_value),
            u64::from(target_value),
            unit.clone()
        )
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
        let (rseed, z212) = match selected.rseed {
            Rseed::BeforeZip212(s) => (s.to_bytes(), false),
            Rseed::AfterZip212(s) => (s, true),
        };
        let mut mp = Vec::<u8>::new();
        selected.witness.write(&mut mp)?;

        tx.inputs.push(TxIn {
            diversifier: hex::encode(d),
            addr: paddr,
            amount: a,
            z212,
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
        }

        RecipientAddress::Transparent(_) => unimplemented!(),
    }

    Ok(tx)
}

pub async fn submit(raw_tx: RawTransaction, lightnode_url: &str) -> Result<()> {
    let mut client = connect_lightnode(lightnode_url.to_string()).await?;
    let r = client.send_transaction(raw_tx).await?.into_inner();

    if r.error_code != 0 {
        return Err(WalletError::Submit(r.error_code, r.error_message).into())
    }
    println!("Success! tx id: {}", r.error_message);

    Ok(())
}

