use crate::combined_book::CombinedBookSnapshot;
use crate::exchange::ExchangeOrder;
use crate::orderbook::{orderbook_aggregator_server::OrderbookAggregator, Empty, Level, Summary};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};
use tracing::{debug, info, instrument};

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
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;

    #[instrument(skip(self, _request))]
    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let client_addr = _request.remote_addr();
        let request_id = uuid::Uuid::new_v4();

        info!(
            request_id = %request_id,
            client_addr = ?client_addr,
            "New subscribe request received"
        );

        let receiver = self.create_receiver();
        let stream = BroadcastStream::new(receiver).filter_map(move |result| {
            let request_id = request_id;
            async move {
                match result {
                    Ok(snapshot) => {
                        debug!(
                            request_id = %request_id,
                            snapshot = ?snapshot,
                            "Sending data to subscriber"
                        );
                        Some(Ok(snapshot.into()))
                    }
                    Err(e) => {
                        debug!(
                            request_id = %request_id,
                            error = ?e,
                            "Error in data stream for subscriber"
                        );
                        None
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(stream) as Self::BookSummaryStream))
    }
}
