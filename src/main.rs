use anyhow::Context;
use chrono::NaiveDate;
use clap::Clap;
use rand::thread_rng;
use redjubjub::{PublicKeyPackage, SharePackage, SignatureShare};
use zcash_coldwallet::multisig::multisig_gen;
use zcash_coldwallet::sign::{combine, multi_sign_one, sign_tx};
use zcash_coldwallet::transact::{make_commitments, pre_multi_sign, submit};
use zcash_coldwallet::{
    account::{get_balance, init_account},
    chain::{init_db, sync},
    checkpoint::find_height,
    constants::LIGHTNODE_URL,
    create_file,
    grpc::RawTransaction,
    keys::generate_key,
    read_from_file,
    transact::prepare_tx,
    Opt, Result, SigningNonces, Tx, TxBin, WalletError, ZECUnit,
};

#[derive(Clap)]
struct ZCashColdWallet {
    #[clap(short, long)]
    lightnode_url: Option<String>,
    #[clap(short, long, default_value = "Zec")]
    unit: ZECUnit,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    Generate {
        output_filename: Option<String>,
    },
    InitDb,
    InitAccount {
        viewing_key: String,
        birth_day: Option<NaiveDate>,
    },
    GetBalance,
    Sync,
    PrepareTx {
        recipient_addr: String,
        amount: String,
        output_filename: Option<String>,
    },
    MultiSigGen {
        num_sigs: u32,
        threshold: u32,
    },
    MultiSigPrepare {
        my_index: u32,
        tx_json_filename: Option<String>,
        nonces_filename: Option<String>,
    },
    MultiSigPreSign {
        spending_key: String,
        tx_json_filename: Option<String>,
    },
    MultiSigSign {
        nonce_filename: String,
        secret_share_filename: String,
        signature_filename: String,
        tx_json_filename: Option<String>,
    },
    MultiSigCombine {
        pubkeys_filename: String,
        tx_json_filename: String,
        raw_tx_filename: String,
        signature_filename: Vec<String>,
    },
    Sign {
        spending_key: String,
        tx_json_filename: Option<String>,
        output_filename: Option<String>,
    },
    Submit {
        raw_tx_file: Option<String>,
    },
}

/*
quick help:

generate -> seed, derivation_path, secret_key, viewing_key, address
init_db
init_account viewing_key
sync
getbalance -> balance
prepare_tx recipient_addr amount -> tx_json
sign secret_key tx_json -> raw_tx_bytes
submit raw_tx_bytes
 */

#[tokio::main]
async fn main() -> Result<()> {
    let mut rng = thread_rng();
    let mut prog_opt = Opt {
        lightnode_url: LIGHTNODE_URL.to_string(),
        unit: ZECUnit::Zec,
    };
    let opts = ZCashColdWallet::parse();
    let cmd = opts.cmd;
    if let Some(lightnode_url) = opts.lightnode_url {
        prog_opt.lightnode_url = lightnode_url;
    }
    prog_opt.unit = opts.unit;

    match cmd {
        Command::Generate { output_filename } => {
            let mut output = create_file(output_filename)?;
            generate_key(&mut output)?
        }
        Command::InitDb => init_db()?,
        Command::InitAccount {
            viewing_key,
            birth_day,
        } => {
            let birth_height = if let Some(birth_day) = birth_day {
                find_height(&prog_opt.lightnode_url, &birth_day).await?
            } else {
                u64::MAX
            };
            init_account(&prog_opt.lightnode_url, viewing_key, birth_height).await?
        }
        Command::Sync => sync(&prog_opt.lightnode_url).await?,
        Command::GetBalance => get_balance(&prog_opt)?,
        Command::PrepareTx {
            amount,
            recipient_addr,
            output_filename,
        } => {
            let mut output = create_file(output_filename)?;
            let tx = prepare_tx(&recipient_addr, amount, &prog_opt.unit)?;
            let tx_json = serde_json::to_string(&tx)?;
            writeln!(output, "{}", tx_json)?;
        }
        Command::MultiSigGen {
            num_sigs,
            threshold,
        } => {
            multisig_gen(num_sigs, threshold)?;
        }
        Command::MultiSigPrepare {
            tx_json_filename,
            my_index,
            nonces_filename,
        } => {
            let tx_json = read_from_file(tx_json_filename.clone());
            let tx: Tx = serde_json::from_str(&tx_json).or(Err(WalletError::TxParse))?;
            let (tx, nonces) = make_commitments(my_index, tx, &mut rng);
            let mut output = create_file(nonces_filename)?;
            let nonces_json = serde_json::to_string(&nonces)?;
            writeln!(output, "{}", nonces_json)?;
            let mut output = create_file(tx_json_filename)?;
            let tx_json = serde_json::to_string(&tx)?;
            writeln!(output, "{}", tx_json)?;
        }
        Command::MultiSigPreSign {
            spending_key,
            tx_json_filename,
        } => {
            let tx_json = read_from_file(tx_json_filename.clone());
            let tx: Tx = serde_json::from_str(&tx_json).or(Err(WalletError::TxParse))?;
            let tx_bin = pre_multi_sign(spending_key, tx)?;
            let mut output = create_file(tx_json_filename)?;
            let tx_bin_json = serde_json::to_string(&tx_bin)?;
            writeln!(output, "{}", tx_bin_json)?;
        }
        Command::MultiSigSign {
            tx_json_filename,
            nonce_filename,
            secret_share_filename,
            signature_filename,
        } => {
            let tx_json = read_from_file(tx_json_filename.clone());
            let tx_bin: TxBin = serde_json::from_str(&tx_json)?;
            let nonce_json = read_from_file(Some(nonce_filename));
            let nonces: Vec<SigningNonces> = serde_json::from_str(&nonce_json).context("nonce")?;
            let share_json = read_from_file(Some(secret_share_filename));
            let share: SharePackage = serde_json::from_str(&share_json).context("share")?;
            let signature_share = multi_sign_one(tx_bin, &nonces, share)?;
            let signature_json = serde_json::to_string(&signature_share)?;
            let mut output = create_file(Some(signature_filename))?;
            writeln!(output, "{}", signature_json)?;
        }
        Command::MultiSigCombine {
            pubkeys_filename,
            tx_json_filename,
            raw_tx_filename,
            signature_filename,
        } => {
            let pubkeys_json = read_from_file(Some(pubkeys_filename));
            let pubkeys: PublicKeyPackage = serde_json::from_str(&pubkeys_json)?;
            let tx_json = read_from_file(Some(tx_json_filename));
            let tx_bin: TxBin = serde_json::from_str(&tx_json)?;
            let signatures = signature_filename
                .iter()
                .map(|filename| {
                    let sig_json = read_from_file(Some(filename.clone()));
                    let signature_share: Vec<SignatureShare> =
                        serde_json::from_str(&sig_json).unwrap();
                    signature_share
                })
                .collect::<Vec<_>>();
            let raw_tx = combine(tx_bin, &pubkeys, &signatures)?;
            let mut output = create_file(Some(raw_tx_filename))?;
            writeln!(output, "{}", hex::encode(&raw_tx.data))?;
        }
        Command::Sign {
            spending_key,
            tx_json_filename,
            output_filename,
        } => {
            let mut output = create_file(output_filename)?;
            let tx_json = read_from_file(tx_json_filename);
            let tx: Tx = serde_json::from_str(&tx_json).or(Err(WalletError::TxParse))?;
            let raw_tx = sign_tx(&spending_key, &tx, &prog_opt)?;
            writeln!(output, "{}", hex::encode(&raw_tx.data))?;
        }
        Command::Submit { raw_tx_file } => {
            let raw_tx = read_from_file(raw_tx_file);
            let raw_tx = RawTransaction {
                data: hex::decode(raw_tx)?,
                height: 0,
            };
            submit(raw_tx, &prog_opt.lightnode_url).await?;
        }
    }

    Ok(())
}
