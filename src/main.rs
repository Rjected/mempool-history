use clap::Parser;
use ethers::{
    prelude::StreamExt,
    providers::{Middleware, Provider, Ws},
};
use std::time::{Instant, SystemTime};
use tokio::sync::mpsc;
use tracing::{debug, info};
use tracing_subscriber::{prelude::*, EnvFilter};

mod timed_tx;
pub use timed_tx::{TimedHash, TimedTransaction};

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

// just chose 4096 because it's a relatively large constant
pub const HASH_BUFFER_SIZE: usize = 4096 * 1024;

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

    // let's just initialize the SystemTime and Instant right after one another.
    // we'll use the `Duration` returned using `Instant` and add it to `SystemTime` when
    // persisting.
    let time_started = SystemTime::now();
    let instant_started = Instant::now();

    // subscribe on newPendingTransactions
    let mut tx_hash_stream = provider.subscribe_pending_txs().await?;

    // let's wrap the SubscriptionStream with a `tokio::sync::mpsc` channel.
    // ugh need better names - we have a tx_hash_stream and hash_tx, one is a stream the other is a
    // channel but it's still confusing
    let (hash_tx, mut hash_rx): (mpsc::Sender<TimedHash>, mpsc::Receiver<TimedHash>) =
        mpsc::channel(HASH_BUFFER_SIZE);

    let tx_provider = provider.clone();
    // waiting for full tx responses shouldn't block the pending tx subscription.
    tokio::spawn(async move {
        // for some reason infura sometimes returns null for a transaction hash it actually has.
        // so, let's create a list of tx hashes that fail and re-request them when we're woken up
        // by a new tx hash.
        let mut missing_transactions: Vec<TimedHash> = vec![];

        while let Some(timed_hash) = hash_rx.recv().await {
            let mut new_missing_txs = vec![];
            // invariant: after this loop missing_transactions is empty
            while let Some(timed_hash) = missing_transactions.pop() {
                // this code is dupicated for clarity, another way to solve this is pushing the
                // timed_hash to missing_transactions and just running this while loop
                let full_tx = tx_provider.get_transaction(timed_hash.hash).await.unwrap();
                let full_tx = match full_tx {
                    Some(tx) => tx,
                    None => {
                        new_missing_txs.push(timed_hash);
                        continue;
                    }
                };
                let timed_tx = TimedTransaction::from_timed_hash(timed_hash, time_started, full_tx);
                if timed_tx.transaction.block_number.is_none() || opts.show_old_txs {
                    info!("Timestamped and collected a transaction: {:#?}", timed_tx);
                }
            }

            // TODO: remove unwraps by putting the main loop in a tokio task, then using a channel
            // for errors?
            let full_tx = tx_provider.get_transaction(timed_hash.hash).await.unwrap();
            debug!("requesting tx for hash {:?}", timed_hash.hash);
            let full_tx = match full_tx {
                Some(tx) => tx,
                None => {
                    new_missing_txs.push(timed_hash);
                    continue;
                }
            };

            // we know missing_transactions is empty (see invariant above) so we can overwrite here
            missing_transactions = new_missing_txs;

            let timed_tx = TimedTransaction::from_timed_hash(timed_hash, time_started, full_tx);
            if timed_tx.transaction.block_number.is_none() || opts.show_old_txs {
                info!("Timestamped and collected a transaction: {:#?}", timed_tx);
            }
        }
    });

    // main loop for sending txs
    loop {
        if let Some(hash) = tx_hash_stream.next().await {
            // first, timestamp
            // TODO: does tracing do a better job at timestamping?
            let duration = instant_started.elapsed();
            let timed_hash = TimedHash { hash, duration };

            // now, let's send the hash
            // NOTE: a failure mode of this program is when this line blocks due to a full hash
            // channel. This would cause timestamps to be slightly off, since there could be more
            // hashes in the tx_hash_stream, but hash_tx could be full.
            hash_tx
                .send(timed_hash)
                .await
                .expect("Sending to a hash stream should not fail unless the program is crashing");
        }
    }

    // async is _so FUN_
}
