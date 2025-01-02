use async_trait::async_trait;
use futures_util::stream::Stream;
use serde_json::Error as SerdeError;
use std::fmt;
use thiserror::Error;

pub mod binance;
pub mod bitstamp;
use crate::binance::BinanceWebSocket;
use crate::bitstamp::BitstampWebSocket;

#[derive(Debug, Clone, PartialEq)]
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

pub trait ExchangeStream:
    Stream<Item = Result<Orderbook, ExchangeError>> + Unpin + ExchangeWebSocket + Send + Sync
{
}
impl<T> ExchangeStream for T where
    T: Stream<Item = Result<Orderbook, ExchangeError>> + Unpin + ExchangeWebSocket + Send + Sync
{
}

pub fn instantiate_exchange_websocket(
    exchange: &str,
) -> Result<Box<dyn ExchangeStream>, ExchangeError> {
    match exchange {
        "Binance" => Ok(Box::new(BinanceWebSocket::new())),
        "Bitstamp" => Ok(Box::new(BitstampWebSocket::new())),
        _ => Err(ExchangeError::Unsupported(exchange.to_string())),
    }
}

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("WebSocket read error")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Invalid message format")]
    ParsingError(#[from] SerdeError),
    #[error("Conversion to orderbook failed")]
    ConversionError,
    #[error("Unsupported Exchange: {0}")]
    Unsupported(String),
    #[error("Uknown error: {0}")]
    Unknown(String),
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
