use crate::combined_book::{CombinedBook, CombinedBookSnapshot};
use crate::exchange::{ExchangeError, ExchangeOrder, ExchangeWebSocket, Orderbook};
use crate::orderbook::{orderbook_aggregator_server::OrderbookAggregator, Empty, Level, Summary};
use futures_util::stream;
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::StreamMap;
use tonic::{Request, Response, Status};

pub trait ExchangeStream:
    Stream<Item = Result<Orderbook, ExchangeError>> + Unpin + ExchangeWebSocket + Send + Sync
{
}
impl<T> ExchangeStream for T where
    T: Stream<Item = Result<Orderbook, ExchangeError>> + Unpin + ExchangeWebSocket + Send + Sync
{
}

pub struct OrderbookProcessor {
    exchanges: Vec<Box<dyn ExchangeStream>>,
    combined_book: crate::combined_book::CombinedBook,
    snapshot_sender: broadcast::Sender<CombinedBookSnapshot>,
}

impl OrderbookProcessor {
    pub fn new(exchanges: Vec<Box<dyn ExchangeStream>>, max_size: usize) -> Self {
        let (snapshot_sender, _) = broadcast::channel(100);
        OrderbookProcessor {
            exchanges,
            combined_book: CombinedBook::new(max_size),
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
                    eprintln!("Error processing snapshot: {:?}", e);
                }
            }
        }
    }

    fn send_snapshot_update(&mut self, snapshot: CombinedBookSnapshot) {
        let _ = self.snapshot_sender.send(snapshot);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CombinedBookSnapshot> {
        self.snapshot_sender.subscribe()
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

impl From<ExchangeOrder> for Level {
    fn from(order: ExchangeOrder) -> Self {
        Level {
            exchange: order.exchange.to_string(),
            price: order.price,
            amount: order.amount,
        }
    }
}

impl From<CombinedBookSnapshot> for Summary {
    fn from(snapshot: CombinedBookSnapshot) -> Self {
        Summary {
            spread: snapshot.spread,
            asks: snapshot.asks.into_iter().map(Level::from).collect(),
            bids: snapshot.bids.into_iter().map(Level::from).collect(),
        }
    }
}

pub struct OrderbookAggregatorImpl {
    sender: broadcast::Sender<CombinedBookSnapshot>,
}

impl OrderbookAggregatorImpl {
    pub fn new(sender: broadcast::Sender<CombinedBookSnapshot>) -> Self {
        Self { sender }
    }

    fn create_receiver(&self) -> broadcast::Receiver<CombinedBookSnapshot> {
        self.sender.subscribe()
    }
}

fn broadcast_to_stream<T: Clone>(
    receiver: broadcast::Receiver<T>,
) -> impl futures_util::Stream<Item = Result<T, broadcast::error::RecvError>> {
    stream::unfold(receiver, |mut rx| async {
        match rx.recv().await {
            Ok(msg) => Some((Ok(msg), rx)),
            Err(e) => match e {
                broadcast::error::RecvError::Closed => None,
                other => Some((Err(other), rx)),
            },
        }
    })
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorImpl {
    type BookSummaryStream =
        Pin<Box<dyn futures_util::Stream<Item = Result<Summary, Status>> + Send>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let receiver = self.create_receiver();
        let stream = broadcast_to_stream(receiver).filter_map(|result| async move {
            match result {
                Ok(snapshot) => Some(Ok(snapshot.into())),
                Err(_) => None,
            }
        });

        Ok(Response::new(Box::pin(stream) as Self::BookSummaryStream))
    }
}
