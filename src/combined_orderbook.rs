use crate::exchange::{Exchange, Orderbook};

#[derive(Debug, Clone)]
pub struct ExchangeOrder {
    pub exchange: Exchange,
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Default)]
pub struct CombinedBookSnapshot {
    pub spread: f64,
    pub asks: Vec<ExchangeOrder>,
    pub bids: Vec<ExchangeOrder>,
}

pub struct CombinedBook {
    snapshot: CombinedBookSnapshot,
}

impl CombinedBook {
    pub fn new() -> Self {
        Self {
            snapshot: CombinedBookSnapshot {
                spread: 0.0,
                asks: Vec::new(),
                bids: Vec::new(),
            },
        }
    }

    pub fn update(&mut self, order_book: &Orderbook) {
        // Implement logic to update the combined book based on new order data
    }

    pub fn get_snapshot(&self) -> CombinedBookSnapshot {
        self.snapshot.clone()
    }
}
