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

use crate::exchange::{Exchange, ExchangeWebSocket, Order, Orderbook};

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
    async fn initialise(&mut self) {
        let (ws_stream, _) = connect_async(self.url.clone() + &self.channel)
            .await
            .expect("Failed to connect to Binance Websocket");

        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
    }

    async fn process_book_update(&mut self) -> Option<Orderbook> {
        let reader = self.read.as_mut()?;
        // TODO: Only parse the last (most recent) orderbook msg.
        if let Some(response) = reader.next().await {
            match response {
                Ok(msg) => match serde_json::from_str::<BinanceOrderbook>(msg.to_text().unwrap()) {
                    Ok(parsed) => {
                        if let Ok(order_book) = Orderbook::try_from(parsed) {
                            return Some(order_book);
                        }
                    }
                    Err(_) => {
                        println!("ignoring non-BinanceWsMessage: {}", msg);
                    }
                },
                Err(_) => {}
            }
        }
        None
    }
}
