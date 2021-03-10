use crate::constants::{
    HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, HRP_SAPLING_EXTENDED_SPENDING_KEY,
    HRP_SAPLING_PAYMENT_ADDRESS, NETWORK,
};
use crate::{connect_lightnode, grpc::RawTransaction, Result, SigningCommitments, SigningNonces, SigningShare, Tx, TxIn, TxOut, WalletError, ZECUnit, ACCOUNT, DATA_PATH, TxBin, decode_extended_point, decode_scalar};
use group::GroupEncoding;
use rand::prelude::ThreadRng;
use redjubjub::{get_randomizer, preprocess, RandomizerPackage};
use zcash_client_backend::encoding::{
    decode_extended_full_viewing_key, decode_extended_spending_key, decode_payment_address,
    encode_extended_full_viewing_key,
};
use zcash_client_backend::{
    address::RecipientAddress, data_api::WalletRead, encoding::encode_payment_address,
};
use zcash_client_sqlite::WalletDB;
use zcash_primitives::consensus::{BlockHeight, BranchId};
use zcash_primitives::merkle_tree::IncrementalWitness;
use zcash_primitives::primitives::Diversifier;
use zcash_primitives::sapling::Node;
use zcash_primitives::transaction::builder::Builder;
use zcash_primitives::{
    primitives::Rseed,
    transaction::components::{amount::DEFAULT_FEE, Amount},
};
use zcash_proofs::prover::LocalTxProver;
use zcash_primitives::transaction::Transaction;

pub fn prepare_tx(to_addr: &str, amount: String, unit: &ZECUnit) -> Result<Tx> {
    let satoshis = unit.to_satoshis(&amount);
    let to_addr = RecipientAddress::decode(&NETWORK, to_addr)
        .ok_or(WalletError::Decode(to_addr.to_string()))?;
    let amount = Amount::from_u64(satoshis).expect("Invalid amount");
    let wallet_db = WalletDB::for_path(DATA_PATH, NETWORK)?;
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
        return Err(WalletError::NotEnoughFunds(
            u64::from(selected_value),
            u64::from(target_value),
            unit.clone(),
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
        let d = selected.diversifier.0;
        let fvk = encode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &extfvk);
        let a = u64::from(selected.note_value);
        let (rseed, z212) = match selected.rseed {
            Rseed::BeforeZip212(s) => (s.to_bytes(), false),
            Rseed::AfterZip212(s) => (s, true),
        };
        let mut mp = Vec::<u8>::new();
        selected.witness.write(&mut mp)?;

        tx.inputs.push(TxIn {
            diversifier: hex::encode(d),
            fvk,
            amount: a,
            z212,
            rseed: hex::encode(rseed),
            witness: hex::encode(mp),
            multisigs: vec![],
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
        return Err(WalletError::Submit(r.error_code, r.error_message).into());
    }
    println!("Success! tx id: {}", r.error_message);

    Ok(())
}

pub fn make_commitments(
    my_index: u32,
    mut tx: Tx,
    rng: &mut ThreadRng,
) -> (Tx, Vec<SigningNonces>) {
    let mut tx_nonces: Vec<SigningNonces> = vec![];
    for tx_in in tx.inputs.iter_mut() {
        let (nonces, commitments) = preprocess(1, my_index, rng);
        let (commitments, nonces): (Vec<_>, Vec<_>) = commitments
            .iter()
            .zip(nonces.iter())
            .map(|(c, n)| {
                (
                    SigningCommitments {
                        index: c.index,
                        randomizer: hex::encode(c.randomizer.to_bytes()),
                        hiding: hex::encode(c.hiding.to_bytes()),
                        binding: hex::encode(c.binding.to_bytes()),
                        randomizer_nonce: hex::encode(n.randomizer.to_bytes()),
                    },
                    SigningNonces {
                        randomizer: hex::encode(n.randomizer.to_bytes()),
                        hiding: hex::encode(n.hiding.to_bytes()),
                        binding: hex::encode(n.binding.to_bytes()),
                    },
                )
            })
            .unzip();
        tx_in.multisigs.push(SigningShare {
            index: my_index,
            commitment: commitments[0].clone(),
            signature: None,
        });
        tx_nonces.push(nonces[0].clone());
    }
    (tx, tx_nonces)
}

pub fn pre_multi_sign(spending_key: String, tx: Tx) -> anyhow::Result<TxBin> {
    let extsk = decode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, &spending_key)?
        .ok_or(WalletError::Decode(spending_key.to_string()))?;
    let ovk = extsk.expsk.ovk;
    let prover = LocalTxProver::with_default_location().ok_or(WalletError::Prover)?;
    let height = BlockHeight::from_u32(tx.height as u32);
    let consensus_branch_id = BranchId::for_height(&NETWORK, height);
    let mut builder = Builder::new(NETWORK, height);
    for input in tx.inputs.iter() {
        let mut d = [0u8; 11];
        hex::decode_to_slice(&input.diversifier, &mut d)?;
        let diversifier = Diversifier(d);
        let from =
            decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &input.fvk)?
                .unwrap();
        let pa = from.fvk.vk.to_payment_address(diversifier).unwrap();
        let mut rseed = [0u8; 32];
        hex::decode_to_slice(&input.rseed, &mut rseed)?;
        let rseed = if input.z212 {
            Rseed::AfterZip212(rseed)
        } else {
            Rseed::BeforeZip212(jubjub::Fr::from_bytes(&rseed).unwrap())
        };
        let note = pa.create_note(input.amount, rseed).unwrap();
        let w = hex::decode(&input.witness)?;
        let witness = IncrementalWitness::<Node>::read(&w[..])?;
        let merkle_path = witness.path().unwrap();
        let signing_commitments = input
            .multisigs
            .iter()
            .map(|s| redjubjub::frost::SigningCommitments {
                index: s.index,
                randomizer: decode_extended_point(&s.commitment.randomizer),
                hiding: decode_extended_point(&s.commitment.hiding),
                binding: decode_extended_point(&s.commitment.binding),
            })
            .collect::<Vec<_>>();
        let randomizers = input
            .multisigs
            .iter()
            .map(|s| decode_scalar(&s.commitment.randomizer_nonce))
            .collect::<Vec<_>>();
        let randomizer_package = RandomizerPackage {
            signing_commitments,
            randomizers,
        };
        let alpha = get_randomizer(&randomizer_package);
        builder.add_sapling_spend_multi(extsk.clone(), from.fvk, alpha, diversifier, note, merkle_path)?;
    }
    let output = tx.output.as_ref().unwrap();
    let output_addr = decode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &output.addr)?.unwrap();
    builder.add_sapling_output(
        Some(ovk),
        output_addr,
        Amount::from_u64(output.amount).unwrap(),
        None,
    )?;

    let commitments = tx.inputs.iter().map(|inp| {
        inp.multisigs.iter().map(|s| s.commitment.clone()).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let (tx_data, tx_metadata, sighash) = builder.prepare_multi_sign(consensus_branch_id, &prover)?;
    let tx = Transaction::from_data(tx_data)?;
    let mut tx_bytes: Vec<u8> = vec![];
    tx.write(&mut tx_bytes)?;

    let tx_bin = TxBin {
        bytes: hex::encode(&tx_bytes),
        sighash: hex::encode(&sighash),
        commitments,
        spend_indices: tx_metadata.spend_indices,
        output_indices: tx_metadata.output_indices,
    };

    Ok(tx_bin)
}
