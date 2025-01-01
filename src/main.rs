pub mod binance;
pub mod bitstamp;
pub mod combined_book;
pub mod config;
pub mod exchange;
pub mod orderbook_processor;
use crate::binance::BinanceWebSocket;
use crate::bitstamp::BitstampWebSocket;
use crate::config::load_config;
use exchange::ExchangeError;
use futures_util::StreamExt;
use orderbook_processor::{ExchangeStream, OrderbookProcessor};
pub mod orderbook {
    tonic::include_proto!("orderbook");
}

#[tokio::main]
async fn main() -> Result<(), ExchangeError> {
    let config = load_config("config/config.json5").expect("Failed to load config file");
    let mut websockets: Vec<Box<dyn ExchangeStream>> = vec![];

    for exchange_name in config.exchanges {
        match exchange_name.as_str() {
            "Binance" => websockets.push(Box::new(BinanceWebSocket::new())),
            "Bitstamp" => websockets.push(Box::new(BitstampWebSocket::new())),
            _ => eprintln!("Warning: Unsupported exchange '{}'", exchange_name),
        }
    }
    let mut orderbook_processor = OrderbookProcessor::new(websockets, config.max_size);

    orderbook_processor.initialise_exchanges().await?;

    while let Some(update) = orderbook_processor.next().await {
        match update {
            Ok(orderbook) => {
                println!(
                    "AGG: {} Top bid, ask: {}@{}, {}@{}",
                    orderbook.bids.first().unwrap().exchange,
                    orderbook.bids.first().unwrap().amount,
                    orderbook.bids.first().unwrap().price,
                    orderbook.asks.first().unwrap().amount,
                    orderbook.asks.first().unwrap().price,
                );
            }
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }

    Ok(())
}
