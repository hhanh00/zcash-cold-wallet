use crate::Result;
use bip39::{Language, Mnemonic, Seed};
use rand::rngs::OsRng;
use rand::RngCore;
use zcash_client_backend::encoding::{encode_extended_full_viewing_key, encode_extended_spending_key, encode_payment_address};
use crate::constants::{HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, HRP_SAPLING_EXTENDED_SPENDING_KEY, HRP_SAPLING_PAYMENT_ADDRESS, COIN_TYPE};
use zcash_primitives::zip32::{ChildIndex, ExtendedSpendingKey, ExtendedFullViewingKey};

pub fn generate_key(output: &mut dyn std::io::Write) -> Result<()> {
    let mut entropy = [0u8; 32];
    OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)?;
    let phrase = mnemonic.phrase();
    writeln!(output, "Seed Phrase: {}", phrase)?;
    let seed = Seed::new(&mnemonic, "");
    let master = ExtendedSpendingKey::master(seed.as_bytes());
    let path = [
        ChildIndex::Hardened(32),
        ChildIndex::Hardened(COIN_TYPE),
        ChildIndex::Hardened(0),
    ];
    writeln!(output, "Derivation Path: m/32'/{}'/0'", COIN_TYPE)?;
    let extsk = ExtendedSpendingKey::from_path(&master, &path);
    let spending_key = encode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, &extsk);
    writeln!(output, "Secret Key: {}", spending_key)?;
    let fvk = ExtendedFullViewingKey::from(&extsk);
    let viewing_key = encode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &fvk);
    writeln!(output, "Viewing Key: {}", viewing_key)?;
    let (_, payment_address) = extsk.default_address().unwrap();
    let address = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &payment_address);
    writeln!(output, "Address: {}", address)?;

    Ok(())
}
