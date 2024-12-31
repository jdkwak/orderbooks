use crate::exchange::{ExchangeOrder, Orderbook};

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

    pub fn update(&mut self, order_book: Orderbook) {
        // TODO
    }

    pub fn get_snapshot(&self) -> CombinedBookSnapshot {
        self.snapshot.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::exchange::Exchange;

    use super::*;

    #[test]
    fn test_update_empty_combined_book() {
        let mut combined_book = CombinedBook::new();
        let order_book = Orderbook {
            exchange_ts: 1234567890,
            bids: vec![
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 100.0,
                    amount: 1.0,
                },
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 99.0,
                    amount: 2.0,
                },
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 98.0,
                    amount: 1.5,
                },
            ],
            asks: vec![
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 101.0,
                    amount: 1.0,
                },
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 102.0,
                    amount: 2.0,
                },
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 103.0,
                    amount: 1.5,
                },
            ],
        };

        combined_book.update(order_book);

        assert_eq!(combined_book.snapshot.bids.len(), 3);
        assert_eq!(combined_book.snapshot.asks.len(), 3);
        assert_eq!(combined_book.snapshot.bids[0].price, 100.0);
        assert_eq!(combined_book.snapshot.asks[0].price, 101.0);
        assert_eq!(combined_book.snapshot.spread, 1.0);
    }

    #[test]
    fn test_update_existing_combined_book() {
        let mut combined_book = CombinedBook::new();
        combined_book.snapshot.bids = vec![
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 100.0,
                amount: 1.0,
            },
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 99.0,
                amount: 2.0,
            },
        ];
        combined_book.snapshot.asks = vec![
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 101.0,
                amount: 1.0,
            },
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 102.0,
                amount: 2.0,
            },
        ];

        let order_book = Orderbook {
            exchange_ts: 1234567890,
            bids: vec![
                ExchangeOrder {
                    exchange: Exchange::Bitstamp,
                    price: 101.0,
                    amount: 1.5,
                },
                ExchangeOrder {
                    exchange: Exchange::Bitstamp,
                    price: 99.5,
                    amount: 2.5,
                },
            ],
            asks: vec![
                ExchangeOrder {
                    exchange: Exchange::Bitstamp,
                    price: 100.5,
                    amount: 1.5,
                },
                ExchangeOrder {
                    exchange: Exchange::Bitstamp,
                    price: 103.0,
                    amount: 1.0,
                },
            ],
        };

        combined_book.update(order_book);

        assert_eq!(combined_book.snapshot.bids.len(), 2);
        assert_eq!(combined_book.snapshot.asks.len(), 2);
        assert_eq!(combined_book.snapshot.bids[0].price, 101.0);
        assert_eq!(combined_book.snapshot.asks[0].price, 100.5);
        assert_eq!(combined_book.snapshot.spread, -0.5);
    }
}
