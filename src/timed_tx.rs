use ethers::{providers::ProviderError, types::Transaction};
use futures_core::stream::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant, SystemTime},
};

/// This represents a transaction, its hash, and a timestamp
#[derive(Debug, Clone)]
pub struct TimedTransaction {
    /// The time the transaction hash was received
    pub timestamp: SystemTime,

    /// The full transaction body
    pub transaction: Transaction,
}

impl TimedTransaction {
    /// Constructs a [`TimedTransaction`] from a [`Transaction`], computing the timestamp by adding
    /// the `Duration` to the [`SystemTime`] provided.
    #[inline(always)]
    pub fn timestamp_tx(full_tx: Transaction, since: SystemTime, elapsed: Duration) -> Self {
        TimedTransaction {
            // could panic, let's keep the return type Self for now, we can use checked_add if we
            // really need to.
            timestamp: since + elapsed,
            transaction: full_tx,
        }
    }
}

/// This represents a stream similar to [`TransactionStream`](ethers_providers::TransactionStream),
/// but with timestamped transactions.
pub struct TimedTransactionStream<St> {
    /// The transaction stream being timestamped
    pub stream: St,

    /// The time when the stream started to be timestamped
    pub start: SystemTime,

    /// The instant when the stream started to be timestamped
    pub instant: Instant,
}

impl<St> TimedTransactionStream<St> {
    /// Create a new `TransactionStream` instance, setting the internal `time` and `instant` to
    /// `now`.
    pub fn new(stream: St) -> Self {
        Self {
            stream,
            start: SystemTime::now(),
            instant: Instant::now(),
        }
    }
}

impl<'a, St, T> Stream for TimedTransactionStream<St>
where
    St: Stream<Item = Result<Transaction, T>> + Unpin + 'a,
    ProviderError: From<T>,
{
    type Item = Result<TimedTransaction, ProviderError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match Stream::poll_next(Pin::new(&mut this.stream), cx) {
            Poll::Ready(tx) => Poll::Ready(tx.map(|tx| {
                tx.map(|non_stamped| {
                    TimedTransaction::timestamp_tx(non_stamped, this.start, this.instant.elapsed())
                })
                .map_err(|err| ProviderError::from(err))
            })),
            _ => Poll::Pending,
        }
    }
}
