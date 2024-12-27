use std::fmt;

use async_trait::async_trait;
use serde_json::Error as SerdeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("WebSocket read error")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Invalid message format")]
    ParsingError(#[from] SerdeError),
    #[error("Conversion to orderbook failed")]
    ConversionError,
    #[error("Uknown error: {0}")]
    Unknown(String),
}

pub enum Exchange {
    Bitstamp,
    Binance,
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exchange::Bitstamp => write!(f, "Bitstamp"),
            Exchange::Binance => write!(f, "Binance"),
        }
    }
}

pub struct Orderbook {
    pub exchange: Exchange,
    pub exchange_ts: u64,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

#[derive(Clone)]
pub struct Order {
    pub price: f64,
    pub quantity: f64,
}

#[async_trait]
pub trait ExchangeWebSocket: Send {
    async fn initialise(&mut self) -> Result<(), ExchangeError>;
    async fn process_book_update(&mut self) -> Result<Orderbook, ExchangeError>;
}
