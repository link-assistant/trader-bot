//! Exchange provider trait definition.

use async_trait::async_trait;

use super::{ExchangeInfo, ExchangeResult, Instrument, MarketData, OrderBook};
use crate::domain::{Order, OrderDirection, Wallet};

/// Trait for exchange/broker API providers.
///
/// This abstraction allows the balancer to work with different exchanges
/// (T-Bank, Binance, Interactive Brokers, etc.) through a unified interface.
#[async_trait]
pub trait ExchangeProvider: Send + Sync {
    /// Returns information about this exchange.
    fn info(&self) -> &ExchangeInfo;

    /// Returns the exchange identifier.
    fn id(&self) -> &str {
        &self.info().id
    }

    /// Returns the exchange name.
    fn name(&self) -> &str {
        &self.info().name
    }

    /// Tests the connection and authentication.
    async fn ping(&self) -> ExchangeResult<()>;

    /// Gets account information and current positions (wallet).
    async fn get_wallet(&self, account_id: &str) -> ExchangeResult<Wallet>;

    /// Gets information about a specific instrument.
    async fn get_instrument(&self, symbol: &str) -> ExchangeResult<Instrument>;

    /// Gets a list of all available instruments.
    async fn list_instruments(&self) -> ExchangeResult<Vec<Instrument>>;

    /// Gets current market data for an instrument.
    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketData>;

    /// Gets market data for multiple instruments.
    async fn get_market_data_batch(&self, symbols: &[String]) -> ExchangeResult<Vec<MarketData>> {
        let mut results = Vec::with_capacity(symbols.len());
        for symbol in symbols {
            results.push(self.get_market_data(symbol).await?);
        }
        Ok(results)
    }

    /// Gets the order book for an instrument.
    async fn get_order_book(&self, symbol: &str, depth: u32) -> ExchangeResult<OrderBook>;

    /// Places an order.
    async fn place_order(&self, account_id: &str, order: Order) -> ExchangeResult<Order>;

    /// Cancels an order.
    async fn cancel_order(&self, account_id: &str, order_id: &str) -> ExchangeResult<Order>;

    /// Gets the current status of an order.
    async fn get_order(&self, account_id: &str, order_id: &str) -> ExchangeResult<Order>;

    /// Gets all active orders.
    async fn get_active_orders(&self, account_id: &str) -> ExchangeResult<Vec<Order>>;

    /// Checks if the market is currently open for a symbol.
    async fn is_market_open(&self, symbol: &str) -> ExchangeResult<bool> {
        let data = self.get_market_data(symbol).await?;
        Ok(data.market_status.is_tradeable())
    }

    /// Creates a market buy order.
    async fn market_buy(&self, account_id: &str, symbol: &str, lots: u32) -> ExchangeResult<Order> {
        let order = Order::market(
            generate_order_id(),
            symbol,
            self.id(),
            OrderDirection::Buy,
            lots,
        );
        self.place_order(account_id, order).await
    }

    /// Creates a market sell order.
    async fn market_sell(
        &self,
        account_id: &str,
        symbol: &str,
        lots: u32,
    ) -> ExchangeResult<Order> {
        let order = Order::market(
            generate_order_id(),
            symbol,
            self.id(),
            OrderDirection::Sell,
            lots,
        );
        self.place_order(account_id, order).await
    }
}

/// Generates a unique order ID.
fn generate_order_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Atomic counter to ensure uniqueness even if called in same nanosecond
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("ORD-{timestamp}-{counter}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_id_generation() {
        let id1 = generate_order_id();
        let id2 = generate_order_id();

        assert!(id1.starts_with("ORD-"));
        assert!(id2.starts_with("ORD-"));
        // IDs should be unique (though not guaranteed in tests due to timing)
        assert_ne!(id1, id2);
    }
}
