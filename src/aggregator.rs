use crate::exchange::Exchange;
use crate::exchange::ExchangeWebSocket;
use crate::exchange::Orderbook;

struct ExchangeOrder {
    exchange: Exchange,
    price: f64,
    amount: f64,
}

struct CombinedBook {
    spread: f64,
    asks: Vec<ExchangeOrder>,
    bids: Vec<ExchangeOrder>,
}

struct OrderbookAggregator {
    exchanges: Vec<Box<dyn ExchangeWebSocket>>,
    combined_book: CombinedBook,
}

// impl OrderbookAggregator {
//     async fn combine_orderbooks(&mut self) {
//         self.exchanges.into_iter().map(|ws| {
//             if let Some(order_book) = ws.process_book_update().await {
//                 self.update_combined_book(&order_book);
//             }
//         })
//     }

//     fn update_combined_book(&mut self, order_book: &Orderbook) {
//         if self.combined_book.bids.len() == 0 {
//             let _exchange_bids: Vec<ExchangeOrder> = order_book
//                 .bids
//                 .into_iter()
//                 .map(|b| ExchangeOrder {
//                     exchange: order_book.exchange,
//                     price: b.price,
//                     amount: b.quantity,
//                 })
//                 .collect();
//         }
//     }
// }
