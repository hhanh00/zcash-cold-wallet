use clap::Clap;
use zcash_coldwallet::sign::sign_tx;
use zcash_coldwallet::transact::submit;
use zcash_coldwallet::{
    account::get_balance,
    chain::{init_account, init_db, sync},
    grpc::RawTransaction,
    keys::generate_key,
    transact::prepare_tx,
    Result, Tx, LIGHTNODE_URL,
};

#[derive(Clap)]
struct ZCashColdWallet {
    lightnode_url: Option<String>,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clap)]
enum Command {
    Generate,
    InitDb,
    InitAccount {
        viewing_key: String,
    },
    GetBalance,
    Sync,
    PrepareTx {
        recipient_addr: String,
        amount: u64,
    },
    Sign {
        spending_key: String,
        tx_json: String,
    },
    Submit {
        raw_tx: String,
    },
}
/*

generate -> seed, derivation_path, secret_key, viewing_key, address
init_db
init_account viewing_key
sync
getbalance -> balance
prepare_tx recipient_addr amount -> tx_json
sign tx_json secret_key-> raw_tx_bytes
submit raw_tx_bytes
 */

#[tokio::main]
async fn main() -> Result<()> {
    let opts = ZCashColdWallet::parse();
    let cmd = opts.cmd;
    let lightnode_url = opts.lightnode_url.unwrap_or(LIGHTNODE_URL.to_string());
    match cmd {
        Command::Generate => generate_key()?,
        Command::InitDb => init_db()?,
        Command::InitAccount { viewing_key } => init_account(viewing_key)?,
        Command::Sync => sync(&lightnode_url).await?,
        Command::GetBalance => get_balance()?,
        Command::PrepareTx {
            amount,
            recipient_addr,
        } => {
            let tx = prepare_tx(&recipient_addr, amount)?;
            let tx_json = serde_json::to_string(&tx)?;
            println!("{}", tx_json);
        }
        Command::Sign {
            spending_key,
            tx_json,
        } => {
            let tx: Tx = serde_json::from_str(&tx_json).expect("Could not parse transaction json");
            let raw_tx = sign_tx(&spending_key, &tx)?;
            println!("{}", hex::encode(&raw_tx.data));
        }
        Command::Submit { raw_tx } => {
            let raw_tx = RawTransaction {
                data: hex::decode(raw_tx)?,
                height: 0,
            };
            submit(&lightnode_url, raw_tx).await?;
        }
    }

    Ok(())
}
