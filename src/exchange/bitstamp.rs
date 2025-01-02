use crate::exchange::{Exchange, ExchangeError, ExchangeOrder, ExchangeWebSocket, Orderbook};
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::stream::Stream;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use serde::Deserialize;
use serde::Serialize;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
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

impl TryFrom<BitstampOrder> for ExchangeOrder {
    type Error = Box<dyn std::error::Error>;

    fn try_from(order: BitstampOrder) -> Result<Self, Self::Error> {
        let exchange = Exchange::Bitstamp;
        let price = order.price.parse::<f64>()?;
        let amount = order.quantity.parse::<f64>()?;
        Ok(ExchangeOrder {
            exchange,
            price,
            amount,
        })
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
    venue: Exchange,
    url: String,
    channel: String,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl BitstampWebSocket {
    pub fn new(trading_pair: &str) -> Self {
        Self {
            venue: Exchange::Bitstamp,
            url: "wss://ws.bitstamp.net/".to_string(),
            channel: "order_book_".to_string() + trading_pair,
            write: None,
            read: None,
        }
    }
}

#[async_trait]
impl ExchangeWebSocket for BitstampWebSocket {
    fn get_exchange(&self) -> Exchange {
        self.venue.clone()
    }

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
}

impl Stream for BitstampWebSocket {
    type Item = Result<Orderbook, ExchangeError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Ensure the reader is available
        let reader = match this.read.as_mut() {
            Some(reader) => reader,
            None => return Poll::Ready(None),
        };

        // Poll the next WebSocket message
        match Pin::new(reader).poll_next(cx) {
            Poll::Ready(Some(Ok(msg))) => {
                let orderbook_result = msg
                    .to_text()
                    .map_err(|_| ExchangeError::WebSocketError(WsError::Utf8))
                    .and_then(|text| {
                        serde_json::from_str::<BitstampOrderbook>(text)
                            .map_err(ExchangeError::ParsingError)
                    })
                    .and_then(|parsed| {
                        Orderbook::try_from(parsed).map_err(|_| ExchangeError::ConversionError)
                    });

                Poll::Ready(Some(orderbook_result))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(ExchangeError::WebSocketError(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
