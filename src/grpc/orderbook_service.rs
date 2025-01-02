use crate::combined_book::CombinedBookSnapshot;
use crate::exchange::ExchangeOrder;
use crate::orderbook::{orderbook_aggregator_server::OrderbookAggregator, Empty, Level, Summary};
use futures_util::StreamExt;
use std::pin::Pin;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

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

pub struct OrderbookService {
    sender: broadcast::Sender<CombinedBookSnapshot>,
}

impl OrderbookService {
    pub fn new(mut receiver: broadcast::Receiver<CombinedBookSnapshot>) -> Self {
        let (sender, _internal_receiver) = broadcast::channel(100);

        let sender_clone = sender.clone();
        tokio::spawn(async move {
            while let Ok(snapshot) = receiver.recv().await {
                let _ = sender_clone.send(snapshot);
            }
        });

        Self { sender }
    }

    fn create_receiver(&self) -> broadcast::Receiver<CombinedBookSnapshot> {
        self.sender.subscribe()
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookService {
    type BookSummaryStream =
        Pin<Box<dyn futures_util::Stream<Item = Result<Summary, Status>> + Send>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let receiver = self.create_receiver();
        let stream = BroadcastStream::new(receiver).filter_map(|result| async move {
            match result {
                Ok(snapshot) => Some(Ok(snapshot.into())),
                Err(_) => None,
            }
        });

        Ok(Response::new(Box::pin(stream) as Self::BookSummaryStream))
    }
}
