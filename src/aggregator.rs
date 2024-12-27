use crate::exchange::{Exchange, ExchangeError, ExchangeWebSocket, Orderbook};
use futures::future::{select_all, BoxFuture};
use futures::FutureExt;

pub struct ExchangeOrder {
    pub exchange: Exchange,
    pub price: f64,
    pub amount: f64,
}

pub struct CombinedBook {
    pub spread: f64,
    pub asks: Vec<ExchangeOrder>,
    pub bids: Vec<ExchangeOrder>,
}

pub struct Aggregator {
    pub exchanges: Vec<Box<dyn ExchangeWebSocket + Send>>,
}

impl Aggregator {
    /// Initialise all exchanges
    pub async fn initialise(&mut self) -> Result<(), ExchangeError> {
        for exchange in &mut self.exchanges {
            exchange.initialise().await?;
        }
        Ok(())
    }

    /// Concurrently await book updates from all exchanges and return the earliest one
    pub async fn process_book_update(&mut self) -> Result<Orderbook, ExchangeError> {
        // Collect futures from all exchanges
        let futures: Vec<BoxFuture<'_, Result<Orderbook, ExchangeError>>> = self
            .exchanges
            .iter_mut()
            .map(|exchange| exchange.process_book_update().boxed())
            .collect();

        if futures.is_empty() {
            return Err(ExchangeError::Unknown(
                "No exchanges to process.".to_string(),
            ));
        }

        // Wait for the first completed future and return its result
        let (result, _, _) = select_all(futures).await;
        result
    }
}
