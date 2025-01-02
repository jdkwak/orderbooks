use crate::exchange::{Exchange, ExchangeError, ExchangeOrder, ExchangeWebSocket, Orderbook};
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::stream::Stream;
use futures_util::{stream::SplitStream, StreamExt};
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
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

impl TryFrom<BinanceOrder> for ExchangeOrder {
    type Error = Box<dyn std::error::Error>;

    fn try_from(order: BinanceOrder) -> Result<Self, Self::Error> {
        let exchange = Exchange::Binance;
        let price = order.price.parse::<f64>()?;
        let amount = order.quantity.parse::<f64>()?;
        Ok(ExchangeOrder {
            exchange,
            price,
            amount,
        })
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
            exchange_ts,
            bids,
            asks,
        })
    }
}
pub struct BinanceWebSocket {
    venue: Exchange,
    url: String,
    channel: String,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl BinanceWebSocket {
    pub fn new(trading_pair: &str) -> Self {
        Self {
            venue: Exchange::Binance,
            url: "wss://stream.binance.com:9443/ws/".to_string(),
            channel: trading_pair.to_string() + "@depth20@1000ms",
            write: None,
            read: None,
        }
    }
}

#[async_trait]
impl ExchangeWebSocket for BinanceWebSocket {
    fn get_exchange(&self) -> Exchange {
        self.venue.clone()
    }

    async fn initialise(&mut self) -> Result<(), ExchangeError> {
        let (ws_stream, _) = connect_async(format!("{}{}", self.url, self.channel)).await?;
        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
        Ok(())
    }
}

impl Stream for BinanceWebSocket {
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
                        serde_json::from_str::<BinanceOrderbook>(text)
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
