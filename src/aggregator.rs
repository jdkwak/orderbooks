use crate::combined_book::CombinedBook;
use crate::exchange::{ExchangeError, ExchangeWebSocket, Orderbook};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::StreamMap;

pub trait ExchangeStream:
    Stream<Item = Result<Orderbook, ExchangeError>> + Unpin + ExchangeWebSocket + Send
{
}
impl<T> ExchangeStream for T where
    T: Stream<Item = Result<Orderbook, ExchangeError>> + Unpin + ExchangeWebSocket + Send
{
}

pub struct Aggregator {
    exchanges: Vec<Box<dyn ExchangeStream>>,
    combined_book: CombinedBook,
}

impl Aggregator {
    pub fn new(exchanges: Vec<Box<dyn ExchangeStream>>, max_size: usize) -> Self {
        Aggregator {
            exchanges,
            combined_book: CombinedBook::new(max_size),
        }
    }

    pub async fn initialise_exchanges(&mut self) -> Result<(), ExchangeError> {
        for exchange in &mut self.exchanges {
            if let Err(err) = exchange.initialise().await {
                return Err(ExchangeError::Unknown(format!(
                    "Error initializing {}: {}",
                    exchange.get_exchange(),
                    err
                )));
            }
        }
        Ok(())
    }
}

impl Stream for Aggregator {
    type Item = Result<Orderbook, ExchangeError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        let mut stream_map = StreamMap::new();

        for (index, exchange) in this.exchanges.iter_mut().enumerate() {
            stream_map.insert(index, Pin::new(exchange));
        }

        match stream_map.poll_next_unpin(cx) {
            Poll::Ready(Some((_, Ok(orderbook)))) => Poll::Ready(Some(Ok(orderbook))),
            Poll::Ready(Some((_, Err(e)))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
