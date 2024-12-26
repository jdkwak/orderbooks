use async_trait::async_trait;

pub enum Exchange {
    Bitstamp,
    Binance,
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
pub trait ExchangeWebSocket {
    async fn initialise(&mut self);
    async fn process_book_update(&mut self) -> Option<Orderbook>;
}
