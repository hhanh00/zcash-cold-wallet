use crate::constants::{
    HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, HRP_SAPLING_EXTENDED_SPENDING_KEY,
    HRP_SAPLING_PAYMENT_ADDRESS, NETWORK,
};
use crate::{
    decode_extended_point, decode_scalar, grpc::RawTransaction, Opt, Result,
    SigningNonces, Tx, TxBin, WalletError,
};
use jubjub::Fr;
use redjubjub::{sign, SharePackage, SignatureShare, aggregate, PublicKeyPackage};
use zcash_client_backend::encoding::{
    decode_extended_full_viewing_key, decode_extended_spending_key, decode_payment_address,
};
use zcash_primitives::transaction::Transaction;
use zcash_primitives::{
    consensus::{BlockHeight, BranchId},
    merkle_tree::IncrementalWitness,
    primitives::{Diversifier, Rseed},
    sapling::Node,
    transaction::{builder::Builder, components::Amount},
};
use zcash_proofs::prover::LocalTxProver;

pub fn sign_tx(spending_key: &str, tx: &Tx, opts: &Opt) -> Result<RawTransaction> {
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
            Rseed::BeforeZip212(Fr::from_bytes(&rseed).unwrap())
        };
        let note = pa.create_note(input.amount, rseed).unwrap();
        let w = hex::decode(&input.witness)?;
        let witness = IncrementalWitness::<Node>::read(&w[..])?;
        let merkle_path = witness.path().unwrap();
        builder.add_sapling_spend(extsk.clone(), diversifier, note, merkle_path)?;
    }
    let output = tx.output.as_ref().unwrap();
    let output_addr = decode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &output.addr)?.unwrap();
    eprintln!(
        "Payment of {} {} to {}",
        opts.unit.from_satoshis(output.amount),
        opts.unit,
        output.addr
    );
    builder.add_sapling_output(
        Some(ovk),
        output_addr,
        Amount::from_u64(output.amount).unwrap(),
        None,
    )?;
    let (tx, _) = builder.build(consensus_branch_id, &prover)?;
    let mut raw_tx = vec![];
    tx.write(&mut raw_tx)?;

    let raw_tx = RawTransaction {
        data: raw_tx,
        height: 0,
    };

    Ok(raw_tx)
}

pub fn multi_sign_one(
    tx_bin: TxBin,
    nonces: &[SigningNonces],
    share: SharePackage,
) -> anyhow::Result<Vec<SignatureShare>> {
    let mut sighash = [0u8; 32];
    hex::decode_to_slice(tx_bin.sighash, &mut sighash)?;
    let nonces = nonces.iter().map(|n| redjubjub::SigningNonces {
        randomizer: decode_scalar(&n.randomizer),
        hiding: decode_scalar(&n.hiding),
        binding: decode_scalar(&n.binding),
    }).collect::<Vec<_>>();
    let mut shares: Vec<SignatureShare> = vec![];
    for (commitments, nonces) in tx_bin.commitments.iter().zip(nonces.iter()) {
        let signing_commitments = commitments
            .iter()
            .map(|c| redjubjub::SigningCommitments {
                index: c.index,
                randomizer: decode_extended_point(&c.randomizer),
                hiding: decode_extended_point(&c.hiding),
                binding: decode_extended_point(&c.binding),
            })
            .collect::<Vec<_>>();
        let signing_package = redjubjub::SigningPackage {
            message: &sighash,
            signing_commitments,
            randomized: true,
        };

        let signature_share = sign(&signing_package, nonces, &share).unwrap();
        shares.push(signature_share);
    }

    Ok(shares)
}

pub fn combine(tx_bin: TxBin, pubkeys: &PublicKeyPackage, signatures: &[Vec<SignatureShare>]) -> anyhow::Result<RawTransaction> {
    let tx_bytes = hex::decode(tx_bin.bytes)?;
    let mut tx = Transaction::read(tx_bytes.as_slice())?;

    let mut sighash = [0u8; 32];
    hex::decode_to_slice(tx_bin.sighash, &mut sighash)?;

    for (i, commitments) in tx_bin.commitments.iter().enumerate() {
        let pubkeys = pubkeys.clone();
        pubkeys.group_public.check();
        let signing_commitments = commitments
            .iter()
            .map(|c| redjubjub::SigningCommitments {
                index: c.index,
                randomizer: decode_extended_point(&c.randomizer),
                hiding: decode_extended_point(&c.hiding),
                binding: decode_extended_point(&c.binding),
            })
            .collect::<Vec<_>>();
        let signing_package = redjubjub::SigningPackage {
            message: &sighash,
            signing_commitments,
            randomized: true,
        };
        let signing_shares = signatures.iter().map(|sigs| sigs[i].clone()).collect::<Vec<_>>();

        let random_pubkeys = pubkeys.randomize(&signing_package).unwrap();
        random_pubkeys.group_public.check();

        let signature = aggregate(&signing_package, &signing_shares, &random_pubkeys).unwrap();

        random_pubkeys.group_public.verify(&sighash, &signature)?;

        let signature = zcash_primitives::redjubjub::Signature {
            rbar: signature.r_bytes,
            sbar: signature.s_bytes,
        };
        tx.shielded_spends[i].spend_auth_sig = Some(signature);
    }

    let mut raw_tx = vec![];
    tx.write(&mut raw_tx)?;

    let raw_tx = RawTransaction {
        data: raw_tx,
        height: 0,
    };

    Ok(raw_tx)
}