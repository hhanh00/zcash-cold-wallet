use crate::{grpc::RawTransaction, Result, Tx};
use jubjub::Fr;
use zcash_client_backend::encoding::{decode_extended_spending_key, decode_payment_address};
use zcash_primitives::constants::testnet::HRP_SAPLING_EXTENDED_SPENDING_KEY;
use zcash_primitives::{
    consensus::{BlockHeight, BranchId, Network::TestNetwork},
    constants::testnet::HRP_SAPLING_PAYMENT_ADDRESS,
    merkle_tree::IncrementalWitness,
    primitives::{Diversifier, Rseed},
    sapling::Node,
    transaction::{builder::Builder, components::Amount},
};
use zcash_proofs::prover::LocalTxProver;

pub fn sign_tx(spending_key: &str, tx: &Tx) -> Result<RawTransaction> {
    let extsk = decode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, &spending_key)?
        .expect("Could not decode spending key");
    let ovk = extsk.expsk.ovk;
    let prover = LocalTxProver::with_default_location().unwrap();
    let height = BlockHeight::from_u32(tx.height as u32);
    let consensus_branch_id = BranchId::for_height(&TestNetwork, height);
    let mut builder = Builder::new(TestNetwork, height);
    for input in tx.inputs.iter() {
        let mut d = [0u8; 11];
        hex::decode_to_slice(&input.diversifier, &mut d)?;
        let diversifier = Diversifier(d);
        let from = decode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &input.addr)?.unwrap();
        let mut rseed = [0u8; 32];
        hex::decode_to_slice(&input.rseed, &mut rseed)?;
        let rseed = Rseed::BeforeZip212(Fr::from_bytes(&rseed).unwrap());
        let note = from.create_note(input.amount, rseed).unwrap();
        let w = hex::decode(&input.witness)?;
        let witness = IncrementalWitness::<Node>::read(&w[..])?;
        let merkle_path = witness.path().unwrap();
        builder.add_sapling_spend(extsk.clone(), diversifier, note, merkle_path)?;
    }
    let output = tx.output.as_ref().unwrap();
    let output_addr = decode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &output.addr)?.unwrap();
    eprintln!("Payment of {} to {}", output.amount, output.addr);
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
