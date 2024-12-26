pub mod aggregator;
pub mod binance;
pub mod bitstamp;
pub mod exchange;

use crate::binance::BinanceWebSocket;
use crate::bitstamp::BitstampWebSocket;
use crate::exchange::ExchangeWebSocket;

#[tokio::main]
async fn main() {
    let mut bitstamp_ws = BitstampWebSocket::new();
    bitstamp_ws.initialise().await;
    let bitstamp_orderbooks = tokio::spawn(async move {
        loop {
            if let Some(order_book) = bitstamp_ws.process_book_update().await {
                println!(
                    "Bitstamp Top bid, ask: {}@{}, {}@{}",
                    order_book.bids.first().unwrap().quantity,
                    order_book.bids.first().unwrap().price,
                    order_book.asks.first().unwrap().quantity,
                    order_book.asks.first().unwrap().price,
                )
            }
        }
    });

    let mut binance_ws = BinanceWebSocket::new();
    binance_ws.initialise().await;
    let binance_orderbooks = tokio::spawn(async move {
        loop {
            if let Some(order_book) = binance_ws.process_book_update().await {
                println!(
                    "Binance  Top bid, ask: {}@{}, {}@{}",
                    order_book.bids.first().unwrap().quantity,
                    order_book.bids.first().unwrap().price,
                    order_book.asks.first().unwrap().quantity,
                    order_book.asks.first().unwrap().price,
                )
            }
        }
    });

    let _ = tokio::try_join!(bitstamp_orderbooks);
    let _ = tokio::try_join!(binance_orderbooks);
}
