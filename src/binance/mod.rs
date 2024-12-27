use crate::exchange::{Exchange, ExchangeError, ExchangeWebSocket, Order, Orderbook};
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::{stream::SplitStream, StreamExt};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

#[derive(Deserialize, Debug)]
struct BinanceOrderbook {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    // TODO: potential optimization:
    // only deserialize first 10 orders.
    bids: Vec<BinanceOrder>,
    asks: Vec<BinanceOrder>,
}

#[derive(Deserialize, Debug)]
pub struct BinanceOrder {
    price: String,
    quantity: String,
}

impl TryFrom<BinanceOrder> for Order {
    type Error = Box<dyn std::error::Error>;

    fn try_from(order: BinanceOrder) -> Result<Self, Self::Error> {
        let price = order.price.parse::<f64>()?;
        let quantity = order.quantity.parse::<f64>()?;
        Ok(Order { price, quantity })
    }
}

impl TryFrom<BinanceOrderbook> for Orderbook {
    type Error = Box<dyn std::error::Error>;

    fn try_from(msg: BinanceOrderbook) -> Result<Self, Self::Error> {
        let exchange_ts = msg.last_update_id;

        let bids = msg
            .bids
            .into_iter()
            .take(10)
            .map(|order| order.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let asks = msg
            .asks
            .into_iter()
            .take(10)
            .map(|order| order.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Orderbook {
            exchange: Exchange::Binance,
            exchange_ts,
            bids,
            asks,
        })
    }
}
pub struct BinanceWebSocket {
    url: String,
    channel: String,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl BinanceWebSocket {
    pub fn new() -> Self {
        BinanceWebSocket {
            url: "wss://stream.binance.com:9443/ws/".to_string(),
            channel: "ethbtc@depth20@1000ms".to_string(),
            write: None,
            read: None,
        }
    }
}

#[async_trait]
impl ExchangeWebSocket for BinanceWebSocket {
    async fn initialise(&mut self) -> Result<(), ExchangeError> {
        let (ws_stream, _) = connect_async(format!("{}{}", self.url, self.channel)).await?;
        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
        Ok(())
    }

    async fn process_book_update(&mut self) -> Result<Orderbook, ExchangeError> {
        let reader = self.read.as_mut().ok_or(ExchangeError::WebSocketError(
            tokio_tungstenite::tungstenite::Error::AlreadyClosed,
        ))?;

        let response = reader.next().await.ok_or_else(|| {
            ExchangeError::WebSocketError(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
        })??;

        let parsed: BinanceOrderbook = serde_json::from_str(response.to_text()?)?;
        Orderbook::try_from(parsed).map_err(|_| ExchangeError::ConversionError)
    }
}
