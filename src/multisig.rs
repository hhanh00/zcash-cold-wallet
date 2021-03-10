use redjubjub::frost::{keygen_with_dealer};
use rand::prelude::*;
use zcash_primitives::primitives::ViewingKey;
use zcash_primitives::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};
use jubjub::{ExtendedPoint, SubgroupPoint};
use group::GroupEncoding;
use group::cofactor::CofactorGroup;
use zcash_client_backend::encoding::{encode_payment_address, encode_extended_full_viewing_key, encode_extended_spending_key};
use crate::constants::{HRP_SAPLING_PAYMENT_ADDRESS, HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, HRP_SAPLING_EXTENDED_SPENDING_KEY};
use crate::create_file;

pub fn multisig_gen(num_signers: u32, threshold: u32) -> anyhow::Result<()> {
    let mut rng = thread_rng();

    let (shares, pubkeys) = keygen_with_dealer(num_signers, threshold, &mut rng).unwrap();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    let sk = ExtendedSpendingKey::master(&seed);
    let spending_key = encode_extended_spending_key(HRP_SAPLING_EXTENDED_SPENDING_KEY, &sk);
    println!("{}", spending_key);

    let mut pubkeys_file = create_file(Some("pubkeys.json".to_string()))?;
    writeln!(pubkeys_file, "{}", serde_json::to_string(&pubkeys)?)?;

    for (i, share) in shares.iter().enumerate() {
        let s = serde_json::to_string(&share)?;
        let mut output = create_file(Some(format!("share-{}.json", i+1)))?;
        write!(output, "{}", s)?;
    }

    let vkbytes: [u8; 32] = pubkeys.group_public.into();
    let evk = ExtendedPoint::from_bytes(&vkbytes).unwrap();
    let ak: SubgroupPoint = evk.into_subgroup().unwrap();
    let mut evk = ExtendedFullViewingKey::from(&sk);
    let vk = ViewingKey { ak, nk: evk.fvk.vk.nk };
    evk.fvk.vk = vk;
    let (_, address) = evk.default_address().unwrap();
    let vk_str = encode_extended_full_viewing_key(HRP_SAPLING_EXTENDED_FULL_VIEWING_KEY, &evk);
    println!("{}", vk_str);
    let pa = encode_payment_address(HRP_SAPLING_PAYMENT_ADDRESS, &address);
    println!("{}", pa);

    Ok(())
}
