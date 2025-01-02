use crate::combined_book::{CombinedBook, CombinedBookSnapshot};
use crate::config::Config;
use crate::exchange::{instantiate_exchange_websocket, ExchangeError, ExchangeStream};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::StreamMap;
use tracing::error;

pub struct OrderbookProcessor {
    exchanges: Vec<Box<dyn ExchangeStream>>,
    combined_book: crate::combined_book::CombinedBook,
    snapshot_sender: broadcast::Sender<CombinedBookSnapshot>,
}

impl OrderbookProcessor {
    pub fn new(config: Config) -> Self {
        let (snapshot_sender, _) = broadcast::channel(100);

        let mut exchanges = Vec::new();
        for exchange_name in config.exchanges {
            match instantiate_exchange_websocket(&exchange_name) {
                Ok(websocket) => exchanges.push(websocket),
                Err(e) => {
                    panic!(
                        "Error instantiating websocket for '{}': {:?}",
                        exchange_name, e
                    );
                }
            }
        }

        Self {
            exchanges,
            combined_book: CombinedBook::new(config.max_orders),
            snapshot_sender,
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

    pub async fn drive_and_broadcast(mut self) {
        while let Some(result) = self.next().await {
            match result {
                Ok(snapshot) => {
                    self.send_snapshot_update(snapshot);
                }
                Err(e) => {
                    error!("Error processing snapshot: {:?}", e);
                }
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CombinedBookSnapshot> {
        self.snapshot_sender.subscribe()
    }

    fn send_snapshot_update(&mut self, snapshot: CombinedBookSnapshot) {
        let _ = self.snapshot_sender.send(snapshot);
    }
}

impl Stream for OrderbookProcessor {
    type Item = Result<CombinedBookSnapshot, ExchangeError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut stream_map = StreamMap::new();
        for (index, exchange) in this.exchanges.iter_mut().enumerate() {
            stream_map.insert(index, Pin::new(exchange));
        }

        match stream_map.poll_next_unpin(cx) {
            Poll::Ready(Some((_, Ok(orderbook)))) => {
                this.combined_book.update(orderbook);
                let snapshot = this.combined_book.get_snapshot();
                Poll::Ready(Some(Ok(snapshot)))
            }
            Poll::Ready(Some((_, Err(e)))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
