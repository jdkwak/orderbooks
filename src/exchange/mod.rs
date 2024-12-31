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

#[derive(Debug, Clone)]
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
    pub exchange_ts: u64,
    pub bids: Vec<ExchangeOrder>,
    pub asks: Vec<ExchangeOrder>,
}

#[derive(Debug, Clone)]
pub struct ExchangeOrder {
    pub exchange: Exchange,
    pub price: f64,
    pub amount: f64,
}

#[async_trait]
pub trait ExchangeWebSocket: Send {
    fn get_exchange(&self) -> Exchange;
    async fn initialise(&mut self) -> Result<(), ExchangeError>;
}
