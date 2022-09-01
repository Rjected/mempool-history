use std::time::{Duration, SystemTime};

use ethers::types::{Transaction, TxHash};

/// This represents a transaction, its hash, and a timestamp
#[derive(Debug, Clone)]
pub struct TimedTransaction {
    /// The transaction hash we used to request the transaction from the provider
    pub hash: TxHash,

    /// The time the transaction hash was received
    pub timestamp: SystemTime,

    /// The full transaction body
    pub transaction: Transaction,
}

impl TimedTransaction {
    /// Constructs a [`TimedTransaction`] from a [`TimedHash`], computing the timestamp by adding
    /// the timed hash `duration` to the [`SystemTime`] provided.
    ///
    /// This populates the [`TimedTransaction`] transaction with the `full_tx` provided.
    #[inline(always)]
    pub fn from_timed_hash(timed: TimedHash, since: SystemTime, full_tx: Transaction) -> Self {
        TimedTransaction {
            hash: timed.hash,
            // could panic, let's keep the return type Self for now, we can use checked_add if we
            // really need to.
            timestamp: since + timed.duration,
            transaction: full_tx,
        }
    }
}

#[derive(Debug, Clone)]
/// This represents a hash with the duration elapsed since the program started
pub struct TimedHash {
    /// A transaction hash
    pub hash: TxHash,

    /// The duration since the program started
    pub duration: Duration,
}
