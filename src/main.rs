pub mod aggregator;
pub mod binance;
pub mod bitstamp;
pub mod exchange;

use aggregator::Aggregator;

use crate::binance::BinanceWebSocket;
use crate::bitstamp::BitstampWebSocket;

#[tokio::main]
async fn main() {
    let mut aggregator = Aggregator {
        exchanges: vec![
            Box::new(BitstampWebSocket::new()),
            Box::new(BinanceWebSocket::new()),
        ],
    };
    aggregator
        .initialise()
        .await
        .expect("Failed to initialise Bitstamp Websocket");
    let aggregator_orderbooks = tokio::spawn(async move {
        loop {
            if let Ok(order_book) = aggregator.process_book_update().await {
                println!(
                    "AGG: {} Top bid, ask: {}@{}, {}@{}",
                    order_book.exchange,
                    order_book.bids.first().unwrap().quantity,
                    order_book.bids.first().unwrap().price,
                    order_book.asks.first().unwrap().quantity,
                    order_book.asks.first().unwrap().price,
                )
            }
        }
    });

    let _ = tokio::try_join!(aggregator_orderbooks);
}
