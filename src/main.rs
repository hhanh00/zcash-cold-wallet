use clap::Clap;
use std::fs::File;
use zcash_coldwallet::sign::sign_tx;
use zcash_coldwallet::transact::submit;
use zcash_coldwallet::{
    account::{init_account, get_balance},
    chain::{init_db, sync, scan},
    checkpoint::find_height,
    grpc::RawTransaction,
    keys::generate_key,
    transact::prepare_tx,
    Opt, Result, Tx, WalletError, ZECUnit, constants::LIGHTNODE_URL,
};
use chrono::NaiveDate;

#[derive(Clap)]
struct ZCashColdWallet {
    #[clap(short, long)]
    lightwalletd_url: Option<String>,
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
    ReIndex,
    PrepareTx {
        recipient_addr: String,
        amount: String,
        output_filename: Option<String>,
    },
    Sign {
        spending_key: String,
        tx_json_file: Option<String>,
        output_filename: Option<String>,
    },
    Submit {
        raw_tx_file: Option<String>,
    },
}

fn read_from_file(file_name: Option<String>) -> String {
    let mut input: Box<dyn std::io::Read> = match file_name {
        Some(file_name) => Box::new(File::open(file_name).unwrap()),
        None => Box::new(std::io::stdin()),
    };
    let mut s = String::new();
    input.read_to_string(&mut s).unwrap();
    s.trim_end().to_string()
}

fn create_file(filename: Option<String>) -> Result<Box<dyn std::io::Write>> {
    let output: Box<dyn std::io::Write> = match filename {
        Some(file_name) => Box::new(File::create(file_name)?),
        None => Box::new(std::io::stdout()),
    };
    Ok(output)
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
    let mut prog_opt = Opt {
        lightnode_url: LIGHTNODE_URL.to_string(),
        unit: ZECUnit::Zec,
    };
    let opts = ZCashColdWallet::parse();
    let cmd = opts.cmd;
    if let Some(lightnode_url) = opts.lightwalletd_url {
        prog_opt.lightnode_url = lightnode_url;
    }
    prog_opt.unit = opts.unit;

    match cmd {
        Command::Generate { output_filename } => {
            let keys = generate_key()?;
            let mut output = create_file(output_filename)?;
            writeln!(output, "seed phrase: {}", keys.phrase)?;
            writeln!(output, "spending key: {}", keys.spending_key)?;
            writeln!(output, "derivation path: {}", keys.derivation_path)?;
            writeln!(output, "viewing key: {}", keys.viewing_key)?;
            writeln!(output, "payment address: {}", keys.address)?;
        },
        Command::InitDb => init_db("")?,
        Command::InitAccount {
            viewing_key,
            birth_day,
        } => {
            let birth_height = if let Some(birth_day) = birth_day {
                find_height(&prog_opt.lightnode_url, &birth_day).await?
            } else {
                u64::MAX
            };
            init_account("", &prog_opt.lightnode_url, &viewing_key, birth_height).await?
        },
        Command::Sync => { sync("", &prog_opt.lightnode_url, u32::MAX).await?; }
        Command::ReIndex => scan("")?,
        Command::GetBalance => { get_balance("", prog_opt.unit)?; }
        Command::PrepareTx {
            amount,
            recipient_addr,
            output_filename,
        } => {
            let mut output = create_file(output_filename)?;
            let tx = prepare_tx("", &recipient_addr, amount, &prog_opt.unit)?;
            let tx_json = serde_json::to_string(&tx)?;
            writeln!(output, "{}", tx_json)?;
        }
        Command::Sign {
            spending_key,
            tx_json_file,
            output_filename,
        } => {
            let mut output = create_file(output_filename)?;
            let tx_json = read_from_file(tx_json_file);
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
