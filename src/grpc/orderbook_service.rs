use crate::combined_book::CombinedBookSnapshot;
use crate::exchange::ExchangeOrder;
use crate::orderbook::{orderbook_aggregator_server::OrderbookAggregator, Empty, Level, Summary};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::watch;
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
    receiver: watch::Receiver<CombinedBookSnapshot>,
}

impl OrderbookService {
    pub fn new(receiver: watch::Receiver<CombinedBookSnapshot>) -> Self {
        Self { receiver }
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

        let receiver = self.receiver.clone();
        let stream =
            tokio_stream::wrappers::WatchStream::new(receiver).filter_map(move |snapshot| {
                let request_id = request_id;
                async move {
                    debug!(
                        request_id = %request_id,
                        snapshot = ?snapshot,
                        "Sending data to subscriber"
                    );
                    Some(Ok(snapshot.into()))
                }
            });

        Ok(Response::new(Box::pin(stream) as Self::BookSummaryStream))
    }
}
