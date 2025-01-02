use crate::exchange::{ExchangeOrder, Orderbook};

#[derive(Debug, Clone, Default)]
pub struct CombinedBookSnapshot {
    pub spread: f64,
    pub asks: Vec<ExchangeOrder>,
    pub bids: Vec<ExchangeOrder>,
}

pub struct CombinedBook {
    snapshot: CombinedBookSnapshot,
    max_orders: usize,
}

impl CombinedBook {
    pub fn new(max_orders: usize) -> Self {
        Self {
            snapshot: CombinedBookSnapshot {
                spread: 0.0,
                asks: Vec::new(),
                bids: Vec::new(),
            },
            max_orders,
        }
    }

    pub fn update(&mut self, order_book: Orderbook) {
        // Clear stale orders for the same exchange
        let incoming_exchange = &order_book.bids.first().map(|o| o.exchange.clone());
        if let Some(exchange) = incoming_exchange {
            self.snapshot
                .bids
                .retain(|order| order.exchange != *exchange);
            self.snapshot
                .asks
                .retain(|order| order.exchange != *exchange);
        }

        // Helper function to merge orders efficiently
        fn merge_orders(
            combined_orders: &mut Vec<ExchangeOrder>,
            new_orders: &[ExchangeOrder],
            max_orders: usize,
            is_better: fn(&ExchangeOrder, &ExchangeOrder) -> bool,
        ) {
            let mut result = Vec::with_capacity(max_orders);
            let mut combined_iter = combined_orders.iter();
            let mut new_iter = new_orders.iter();
            let mut combined_next = combined_iter.next();
            let mut new_next = new_iter.next();

            while result.len() < max_orders {
                match (combined_next, new_next) {
                    (Some(c), Some(n)) => {
                        if is_better(n, c) {
                            result.push(n.clone());
                            new_next = new_iter.next();
                        } else {
                            result.push(c.clone());
                            combined_next = combined_iter.next();
                        }
                    }
                    (Some(c), None) => {
                        result.push(c.clone());
                        combined_next = combined_iter.next();
                    }
                    (None, Some(n)) => {
                        result.push(n.clone());
                        new_next = new_iter.next();
                    }
                    (None, None) => break,
                }
            }

            *combined_orders = result;
        }

        // Merge bids (higher price, then higher amount is better)
        merge_orders(
            &mut self.snapshot.bids,
            &order_book.bids,
            self.max_orders,
            |a, b| a.price > b.price || (a.price == b.price && a.amount > b.amount),
        );

        // Merge asks (lower price, then higher amount is better)
        merge_orders(
            &mut self.snapshot.asks,
            &order_book.asks,
            self.max_orders,
            |a, b| a.price < b.price || (a.price == b.price && a.amount > b.amount),
        );

        // Recalculate the spread
        if let (Some(best_bid), Some(best_ask)) =
            (self.snapshot.bids.first(), self.snapshot.asks.first())
        {
            self.snapshot.spread = best_ask.price - best_bid.price;
        } else {
            self.snapshot.spread = 0.0;
        }
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
    fn test_update_exceeding_max_orders() {
        let mut combined_book = CombinedBook::new(3);
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
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 97.0,
                    amount: 0.5,
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
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 104.0,
                    amount: 0.5,
                },
            ],
        };

        combined_book.update(order_book);

        assert_eq!(combined_book.snapshot.bids.len(), 3);
        assert_eq!(combined_book.snapshot.asks.len(), 3);
        assert_eq!(combined_book.snapshot.bids[0].price, 100.0);
        assert_eq!(combined_book.snapshot.asks[0].price, 101.0);
        assert_eq!(combined_book.snapshot.bids[2].price, 98.0);
        assert_eq!(combined_book.snapshot.asks[2].price, 103.0);
    }

    #[test]
    fn test_update_replacement_of_stale_orders() {
        let mut combined_book = CombinedBook::new(3);
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
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 98.0,
                amount: 1.5,
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
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 103.0,
                amount: 1.5,
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

    #[test]
    fn test_update_empty_combined_book() {
        let mut combined_book = CombinedBook::new(10);
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
        let mut combined_book = CombinedBook::new(10);
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

    #[test]
    fn test_update_existing_mixed_book() {
        let mut combined_book = CombinedBook::new(10);
        combined_book.snapshot.bids = vec![
            ExchangeOrder {
                exchange: Exchange::Binance,
                price: 100.0,
                amount: 1.0,
            },
            ExchangeOrder {
                exchange: Exchange::Bitstamp,
                price: 100.0,
                amount: 0.5,
            },
            ExchangeOrder {
                exchange: Exchange::Binance,
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
                amount: 4.0,
            },
            ExchangeOrder {
                exchange: Exchange::Binance,
                price: 102.0,
                amount: 2.0,
            },
        ];

        let order_book = Orderbook {
            exchange_ts: 1234567890,
            bids: vec![
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 100.0,
                    amount: 0.3,
                },
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 99.5,
                    amount: 2.5,
                },
            ],
            asks: vec![
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 100.5,
                    amount: 1.5,
                },
                ExchangeOrder {
                    exchange: Exchange::Binance,
                    price: 102.0,
                    amount: 5.0,
                },
            ],
        };

        combined_book.update(order_book);

        assert_eq!(combined_book.snapshot.bids.len(), 3);
        assert_eq!(combined_book.snapshot.asks.len(), 4);

        assert_eq!(combined_book.snapshot.bids[0].price, 100.0);
        assert_eq!(combined_book.snapshot.bids[0].amount, 0.5);
        assert_eq!(combined_book.snapshot.bids[0].exchange, Exchange::Bitstamp);

        assert_eq!(combined_book.snapshot.asks[0].price, 100.5);
        assert_eq!(combined_book.snapshot.asks[0].amount, 1.5);
        assert_eq!(combined_book.snapshot.asks[0].exchange, Exchange::Binance);

        assert_eq!(combined_book.snapshot.bids[1].price, 100.0);
        assert_eq!(combined_book.snapshot.bids[1].amount, 0.3);
        assert_eq!(combined_book.snapshot.bids[1].exchange, Exchange::Binance);

        assert_eq!(combined_book.snapshot.asks[2].price, 101.0);
        assert_eq!(combined_book.snapshot.asks[2].amount, 1.0);
        assert_eq!(combined_book.snapshot.asks[2].exchange, Exchange::Bitstamp);

        assert_eq!(combined_book.snapshot.bids[2].price, 99.5);
        assert_eq!(combined_book.snapshot.bids[2].amount, 2.5);
        assert_eq!(combined_book.snapshot.bids[2].exchange, Exchange::Binance);

        assert_eq!(combined_book.snapshot.asks[2].price, 102.0);
        assert_eq!(combined_book.snapshot.asks[2].amount, 5.0);
        assert_eq!(combined_book.snapshot.asks[2].exchange, Exchange::Binance);

        assert_eq!(combined_book.snapshot.asks[3].price, 102.0);
        assert_eq!(combined_book.snapshot.asks[3].amount, 4.0);
        assert_eq!(combined_book.snapshot.asks[3].exchange, Exchange::Bitstamp);

        assert_eq!(combined_book.snapshot.asks[2].price, 102.0);
        assert_eq!(combined_book.snapshot.asks[2].amount, 4.0);
        assert_eq!(combined_book.snapshot.asks[2].exchange, Exchange::Bitstamp);

        assert_eq!(combined_book.snapshot.spread, 0.5);
    }
}
