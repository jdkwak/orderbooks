use async_trait::async_trait;
use futures_util::stream::Stream;
use serde_json::Error as SerdeError;
use std::fmt;
use thiserror::Error;

pub mod binance;
pub mod bitstamp;
use binance::BinanceWebSocket;
use bitstamp::BitstampWebSocket;

const BINANCE_STR: &str = "Binance";
const BITSTAMP_STR: &str = "Bitstamp";

#[derive(Debug, Clone, PartialEq)]
pub enum Exchange {
    Binance,
    Bitstamp,
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exchange::Binance => write!(f, "{}", BINANCE_STR),
            Exchange::Bitstamp => write!(f, "{}", BITSTAMP_STR),
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
    trading_pair: &str,
) -> Result<Box<dyn ExchangeStream>, ExchangeError> {
    match exchange {
        BINANCE_STR => Ok(Box::new(BinanceWebSocket::new(trading_pair))),
        BITSTAMP_STR => Ok(Box::new(BitstampWebSocket::new(trading_pair))),
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
