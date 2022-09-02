use clap::Parser;
use ethers::{
    prelude::StreamExt,
    providers::{Middleware, Provider, TransactionStream, Ws},
};
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

mod timed_tx;
pub use timed_tx::TimedTransaction;

use crate::timed_tx::TimedTransactionStream;

#[derive(Debug, Clone, Parser)]
#[clap(
    name = "poolhistory",
    about = "This gathers mempool transactions from a provider and records when transactions are received."
)]
pub struct RunArgs {
    /// The **websocket** provider URL (e.g. wss://localhost:8546)
    #[clap(short, long, env = "ETH_RPC_URL", value_name = "URL")]
    rpc_url: String,

    /// Whether or not to print transactions that have already been included in a block
    #[clap(short, long)]
    show_old_txs: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse CLI arguments
    let opts: RunArgs = RunArgs::parse();

    // critically important message
    println!("Thanks for using rjected's cool mempool tool!");

    // set up tracing_subscriber
    let filter = if std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_default()
        .is_empty()
    {
        EnvFilter::new("mempool_history=info")
    } else {
        EnvFilter::from_default_env()
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    // initialize provider
    let ws = Ws::connect(opts.rpc_url).await?;
    let provider = Provider::new(ws);

    // subscribe on newPendingTransactions
    let tx_hash_stream = provider.subscribe_pending_txs().await?;
    let tx_stream = TransactionStream::new(&provider, tx_hash_stream, 12);
    let mut timed_tx_stream = TimedTransactionStream::new(tx_stream);

    while let Some(timed_tx) = timed_tx_stream.next().await {
        // just panic if error? TODO: make this better or remove todo
        let timed_tx = timed_tx.unwrap();
        if timed_tx.transaction.block_number.is_none() || opts.show_old_txs {
            info!("Timestamped and collected a transaction: {:#?}", timed_tx);
        }
    }

    // async is _so FUN_
    Ok(())
}
