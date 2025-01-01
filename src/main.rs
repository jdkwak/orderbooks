pub mod aggregator;
pub mod binance;
pub mod bitstamp;
pub mod combined_orderbook;
pub mod config;
pub mod exchange;
use crate::binance::BinanceWebSocket;
use crate::bitstamp::BitstampWebSocket;
use crate::config::load_config;
use aggregator::{Aggregator, ExchangeStream};
use exchange::ExchangeError;
use futures_util::StreamExt;

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
    let mut aggregator = Aggregator::new(websockets, config.max_size);

    aggregator.initialise_exchanges().await?;

    while let Some(update) = aggregator.next().await {
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
