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

use crate::exchange::{Exchange, ExchangeWebSocket, Order, Orderbook};

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
struct Subcription {
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
    async fn initialise(&mut self) {
        let subscription = Subcription {
            event: "bts:subscribe".to_string(),
            data: Channel {
                channel: self.channel.clone(),
            },
        };

        let json_subscription =
            serde_json::to_string(&subscription).expect("Failed to serialize JSON");

        let (ws_stream, _) = connect_async(self.url.clone())
            .await
            .expect("Failed to connect to Bitstamp Websocket");

        let (mut write, read) = ws_stream.split();
        write
            .send(Message::Text(json_subscription.into()))
            .await
            .expect("Failed to subscribe to Bitsamp Orderbook data");

        self.write = Some(write);
        self.read = Some(read);
    }
    async fn process_book_update(&mut self) -> Option<Orderbook> {
        let reader = self.read.as_mut()?;
        // TODO: Only parse the last (most recent) orderbook msg.
        if let Some(response) = reader.next().await {
            match response {
                Ok(msg) => {
                    match serde_json::from_str::<BitstampOrderbook>(msg.to_text().unwrap()) {
                        Ok(parsed) => {
                            if let Ok(order_book) = Orderbook::try_from(parsed) {
                                return Some(order_book);
                            }
                        }
                        Err(_) => {
                            println!("ignoring non-BitstampWsMessage: {}", msg);
                        }
                    }
                }
                Err(_) => {}
            }
        }
        None
    }
}
