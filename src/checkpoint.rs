use hex_literal::hex;

pub struct Checkpoint {
    pub height: u32,
    pub hash: [u8; 32],
    pub time: u32,
    pub sapling_tree: &'static str,
}

pub fn find_checkpoint(height: u32) -> &'static Checkpoint {
    let c = CHECKPOINTS
        .iter()
        .rev()
        .find(|&c| c.height <= height)
        .or(CHECKPOINTS.first())
        .unwrap();
    c
}

#[cfg(not(feature = "mainnet"))]
const CHECKPOINTS: &[Checkpoint] = &[
    Checkpoint {
    height: 1288000,
    hash: hex!("64a3b1d7b4c33eb0c4602fc6c7861e73238b461fc45bba101de76dfc71131900"),
    time: 1613309569,
    sapling_tree: "01dbad37e1dc4eb00844e8a91267de5e7a3817b7e35fb1fe69eea447381d6e810201de0198e5c4032260fe2f0447c0f61c8b792836ccd834e4862b7dffc5d1cf211e100001fed541d823a65e1b9eafb27bab8ea702f6c3a9207fde7c125a6e7abcca98b16e000001ffcee7525cf6c79c312b1a2a59f41910401f18f15a9980250955012f30150d6d01447cb09c8e624f26825fc973217564342c6b8ca957c693fede28471ab861a11e00000000018469338dcbdf2f7e54bca5bc3e1c5fad4a656f206040436d3d0433a901218b5e016d559de7a1a382349cf97fe01a2fba41a49bb5e3b306d9ff8c2bcc301c731c00000001f08f39275112dd8905b854170b7f247cf2df18454d4fa94e6e4f9320cca05f24011f8322ef806eb2430dc4a7a41c1b344bea5be946efc7b4349c1c9edb14ff9d39",
}];

#[cfg(feature = "mainnet")]
const CHECKPOINTS: &[Checkpoint] = &[
    Checkpoint {
        height: 400_000,
        hash: hex!("b53763de090fbcda49279b300b3a8ffb5d90f003656b27fed533120200000000"),
        time: 1537884717,
        sapling_tree: "000000",
    },
    Checkpoint {
        height: 700_000,
        hash: hex!("dc0d2312d3f09510b1ffe6de3126c76b99771f1be60aa267d157c00000000000"),
        time: 1579598443,
        sapling_tree: "01c6b273aee226912526622b91e48a0ff5caf71f1f47569aff8a1c145102b02328012758ab750e1cb4f933ebca089d23ead6032151a38266aa020ae84557bb61844811016443e86acd06140aa932467bcc7235704cf95081e2e5faaf031112a9abd5f930016d1847eb52f8218773e3d2dd8eb19950dbe693484098d763010d7c338337cf68018117bb5e4ad68438572aaa55cb7d66b4b86b9d8310fbb4e36db7982dcc28591400012c4e84168b1c9a322f6035ddb5989fea843045d22182ee9ce45a6a8f6831954301abf6a411ff1708af6252bf921625f28931c567d92833d7ed2b2b14efd6b06e5001d1f934bce5476ef5d21b384c7dddfcbd8c1f630435acbf26a094bc46757f5d3501e6a69ddf114c92d39370a24e840c46ed42fc54a63986d3aa916a08c2a922c73b0001a626bb2ed07614f7228f79d5fbccf541699895842341602c639ab7516b1c9a1a0000019be74b905f0e99399af0fda6832324ceeeaf57551b11b42c73bcb7cd215ab91400000001d2ea556f49fb934dc76f087935a5c07788000b4e3aae24883adfec51b5f4d260",
    },
    Checkpoint {
        height: 1_000_000,
        hash: hex!("7afc70ac08462aad9eecc504271a524ef2fe7b01203005aef9ef620000000000"),
        time: 1602206541,
        sapling_tree: "01a4d1f92e2c051e039ca80b14a86d35c755d88ff9856a3c562da4ed14f77f5d0e0012000001f1ff712c8269b7eb11df56b23f8263d59bc4bb2bbc449973e1c85f399c433a0401e0e8b56e5d56de16c173d83c2d96d4e2f94ce0cbd323a53434c647deff020c08000129acf59ead19b76e487e47cf1d100e953acedc62afa6b384f91a620321b1585300018179961f79f609e6759032f3466067548244c3fe0bf31d275d1b6595bb2d486401b622d3f80d8231c44483faa2a27e96e06a2e08d099b26d15828e8f0bde0bd42001a8d1f585aeceb5c3f22ffb43014fe89db9f7efc080361d4fa4e8d596ab1224400103ee02ae59c6688dcaadf1c4ff95e7b1a902837e4989a4c4994dce7dac6ecb20014ff8c0fe6bce02ac4ad684996bfa931d61c724015d797642819361d611ebd61201c7ae83949d9502b0eff10618124d335f046e4aae52c19ccad5567feceb342a5200000001b7fc5791e3650729b7e1e38ee8c4ea9da612a07b4bf412cefaffbab7ac74c547011323ddf890bfd7b94fc609b0d191982cb426b8bf4d900d04709a8b9cb1a27625",
    },
    Checkpoint {
        height: 1_148_400,
        hash: hex!("816955e7e89233be8b05c82cc94f756a0d004424f25a4357dd07d10100000000"),
        time: 1613395373,
        sapling_tree: "0167f11aa79d032b42b5600f95a96bb8f6bbbf67e53e4ec58acb2a283a6fb3ee570013000001f4738bd1f4faad8eef2240d0cc36672fbab5f26a2ecc57d19333ef60dc39cd0f01dd3dd5ca0d9adfd000cfb1802e4f99006b18fe95e3c05a66388525de766c736100011adb986ef6bc732cac556bd7d573aacf1ac22a9e53937ed9e919688e5d688269014d5e2ff53d8403a048abf70fdce433363a64edc2ec350ac76284a4fa46269d1101b3dd2e481e524c0b2b019dfa6ed8816da3840bdbba3f279d1f90bb9ae39a1d5f0196166384356c64ed8a69b82794e83210e6a342ac12ec5ec6b96d7844bc49d90301e1f4237f4903abf53e73f07aab71322f6feea6ef89d7b756631951bb1f68d257000113625bb6dc4092d25f906a05fd3e2f1a7c5df03381d0e0760aebc2ad0ecf0f0b01582a57901af4736388425e1bd26fff5b23fe6c74d10b1fd7145c46e57eeed609000116316e325ad5299b583121e42838f9cb432a88a7042cfeca8d36f8f5e86e234f0000000118f64df255c9c43db708255e7bf6bffd481e5c2f38fe9ed8f3d189f7f9cf2644",
}];
