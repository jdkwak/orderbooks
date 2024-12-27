use crate::exchange::{Exchange, ExchangeError, ExchangeWebSocket, Order, Orderbook};
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

#[derive(Deserialize, Debug)]
pub struct BitstampOrderbook {
    pub data: OrderbookData,
    pub channel: String,
    pub event: String,
}

#[derive(Deserialize, Debug)]
pub struct OrderbookData {
    #[serde(skip)]
    pub timestamp: String,
    pub microtimestamp: String,
    // TODO: potential optimization:
    // only deserialize first 10 orders.
    pub bids: Vec<BitstampOrder>,
    pub asks: Vec<BitstampOrder>,
}

#[derive(Deserialize, Debug)]
pub struct BitstampOrder {
    pub price: String,
    pub quantity: String,
}

impl TryFrom<BitstampOrder> for Order {
    type Error = Box<dyn std::error::Error>;

    fn try_from(order: BitstampOrder) -> Result<Self, Self::Error> {
        let price = order.price.parse::<f64>()?;
        let quantity = order.quantity.parse::<f64>()?;
        Ok(Order { price, quantity })
    }
}

impl TryFrom<BitstampOrderbook> for Orderbook {
    type Error = Box<dyn std::error::Error>;

    fn try_from(msg: BitstampOrderbook) -> Result<Self, Self::Error> {
        // Parse the microtimestamp as u64
        let exchange_ts = msg.data.microtimestamp.parse::<u64>()?;

        // Convert and limit bids and asks to the first 10
        let bids = msg
            .data
            .bids
            .into_iter()
            .take(10)
            .map(|order| order.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let asks = msg
            .data
            .asks
            .into_iter()
            .take(10)
            .map(|order| order.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Orderbook {
            exchange: Exchange::Bitstamp,
            exchange_ts,
            bids,
            asks,
        })
    }
}

#[derive(Serialize)]
struct Subscription {
    event: String,
    data: Channel,
}

#[derive(Serialize)]
struct Channel {
    channel: String,
}

pub struct BitstampWebSocket {
    url: String,
    channel: String,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl BitstampWebSocket {
    pub fn new() -> Self {
        BitstampWebSocket {
            url: "wss://ws.bitstamp.net/".to_string(),
            channel: "order_book_ethbtc".to_string(),
            write: None,
            read: None,
        }
    }
}

#[async_trait]
impl ExchangeWebSocket for BitstampWebSocket {
    async fn initialise(&mut self) -> Result<(), ExchangeError> {
        let subscription = Subscription {
            event: "bts:subscribe".to_string(),
            data: Channel {
                channel: self.channel.clone(),
            },
        };
        let json_subscription =
            serde_json::to_string(&subscription).map_err(|e| ExchangeError::ParsingError(e))?;
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, read) = ws_stream.split();

        write
            .send(Message::Text(json_subscription.into()))
            .await
            .map_err(|e| ExchangeError::WebSocketError(e))?;

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
        let parsed: BitstampOrderbook = serde_json::from_str(response.to_text()?)?;
        Orderbook::try_from(parsed).map_err(|_| ExchangeError::ConversionError)
    }
}
