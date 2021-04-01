use crate::constants::{
    COIN_TYPE, HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, HRP_SAPLING_EXTENDED_SPENDING_KEY,
    HRP_SAPLING_PAYMENT_ADDRESS,
};
use crate::Result;
use anyhow::Context;
use bip39::{Language, Mnemonic, Seed};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use zcash_client_backend::encoding::{
    decode_extended_full_viewing_key, decode_extended_spending_key, decode_payment_address,
    encode_extended_full_viewing_key, encode_extended_spending_key, encode_payment_address,
};
use zcash_primitives::zip32::{ChildIndex, ExtendedFullViewingKey, ExtendedSpendingKey};

#[derive(Serialize)]
pub struct Keys {
    pub phrase: String,
    pub derivation_path: String,
    pub spending_key: String,
    pub viewing_key: String,
    pub address: String,
}

pub fn generate_key() -> Result<Keys> {
    let mut entropy = [0u8; 32];
    OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)?;
    let phrase = mnemonic.phrase();
    let seed = Seed::new(&mnemonic, "");
    let master = ExtendedSpendingKey::master(seed.as_bytes());
    let path = [
        ChildIndex::Hardened(32),
        ChildIndex::Hardened(COIN_TYPE),
        ChildIndex::Hardened(0),
    ];
    let extsk = ExtendedSpendingKey::from_path(&master, &path);
    let spending_key = encode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, &extsk);
    let fvk = ExtendedFullViewingKey::from(&extsk);
    let viewing_key = encode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &fvk);
    let (_, payment_address) = extsk.default_address().unwrap();
    let address = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &payment_address);

    Ok(Keys {
        phrase: phrase.to_string(),
        derivation_path: format!("m/32'/{}'/0'", COIN_TYPE),
        spending_key,
        viewing_key,
        address,
    })
}

pub fn check_address(address: &str) -> bool {
    decode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &address).is_ok()
}

pub fn get_viewing_key(secret_key: &str) -> anyhow::Result<String> {
    let sk = decode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, secret_key)?.context("Invalid sk")?;
    let fvk = ExtendedFullViewingKey::from(&sk);
    let fvk = encode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &fvk);
    Ok(fvk)
}

pub fn get_address(viewing_key: &str) -> anyhow::Result<String> {
    let fvk = decode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, viewing_key)?.context("Invalid fvk")?;
    let (_, address) = fvk.default_address().unwrap();
    let address = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &address);
    Ok(address)
}

pub enum KeyType {
    VIEWING_KEY,
    SECRET_KEY,
    UNKNOWN,
}

fn valid_key<T, E>(s: std::result::Result<Option<T>, E>) -> bool {
    match s {
        Err(_) => false,
        Ok(None) => false,
        _ => true,
    }
}

pub fn get_key_type(key: &str) -> KeyType {
    if valid_key(decode_extended_full_viewing_key(
        HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY,
        key,
    )) {
        return KeyType::VIEWING_KEY;
    }
    if valid_key(decode_extended_spending_key(
        HRP_SAPLING_EXTENDED_SPENDING_KEY,
        key,
    )) {
        return KeyType::SECRET_KEY;
    }
    KeyType::UNKNOWN
}
