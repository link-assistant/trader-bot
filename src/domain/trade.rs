//! Trade type for recording executed trades.

use crate::domain::{Money, OrderDirection};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A unique trade identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId(String);

impl TradeId {
    /// Creates a new trade ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Creates a new random trade ID.
    #[must_use]
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Returns the ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TradeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A completed trade execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Unique trade identifier.
    id: TradeId,
    /// Related order ID (if any).
    order_id: Option<String>,
    /// Trading symbol.
    symbol: String,
    /// Exchange where trade occurred.
    exchange: String,
    /// Trade direction (buy/sell).
    direction: OrderDirection,
    /// Number of lots traded.
    quantity_lots: u32,
    /// Price per lot.
    price: Money,
    /// Total value of trade (quantity * price).
    value: Money,
    /// Commission/fee paid.
    commission: Option<Money>,
    /// Time of trade execution.
    executed_at: DateTime<Utc>,
}

impl Trade {
    /// Creates a new trade.
    #[must_use]
    pub fn new(
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        direction: OrderDirection,
        quantity_lots: u32,
        price: Money,
    ) -> Self {
        let value = price.multiply(Decimal::from(quantity_lots));
        Self {
            id: TradeId::random(),
            order_id: None,
            symbol: symbol.into(),
            exchange: exchange.into(),
            direction,
            quantity_lots,
            price,
            value,
            commission: None,
            executed_at: Utc::now(),
        }
    }

    /// Creates a buy trade.
    #[must_use]
    pub fn buy(
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        quantity_lots: u32,
        price: Money,
    ) -> Self {
        Self::new(symbol, exchange, OrderDirection::Buy, quantity_lots, price)
    }

    /// Creates a sell trade.
    #[must_use]
    pub fn sell(
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        quantity_lots: u32,
        price: Money,
    ) -> Self {
        Self::new(symbol, exchange, OrderDirection::Sell, quantity_lots, price)
    }

    /// Sets the trade ID.
    #[must_use]
    pub fn with_id(mut self, id: TradeId) -> Self {
        self.id = id;
        self
    }

    /// Sets the order ID.
    #[must_use]
    pub fn with_order_id(mut self, order_id: impl Into<String>) -> Self {
        self.order_id = Some(order_id.into());
        self
    }

    /// Sets the commission.
    #[must_use]
    pub fn with_commission(mut self, commission: Money) -> Self {
        self.commission = Some(commission);
        self
    }

    /// Sets the execution time.
    #[must_use]
    pub fn with_executed_at(mut self, executed_at: DateTime<Utc>) -> Self {
        self.executed_at = executed_at;
        self
    }

    /// Returns the trade ID.
    #[must_use]
    pub fn id(&self) -> &TradeId {
        &self.id
    }

    /// Returns the order ID if any.
    #[must_use]
    pub fn order_id(&self) -> Option<&str> {
        self.order_id.as_deref()
    }

    /// Returns the symbol.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the exchange.
    #[must_use]
    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    /// Returns the trade direction.
    #[must_use]
    pub fn direction(&self) -> OrderDirection {
        self.direction
    }

    /// Returns the quantity in lots.
    #[must_use]
    pub fn quantity_lots(&self) -> u32 {
        self.quantity_lots
    }

    /// Returns the price per lot.
    #[must_use]
    pub fn price(&self) -> &Money {
        &self.price
    }

    /// Returns the total trade value.
    #[must_use]
    pub fn value(&self) -> &Money {
        &self.value
    }

    /// Returns the commission if any.
    #[must_use]
    pub fn commission(&self) -> Option<&Money> {
        self.commission.as_ref()
    }

    /// Returns the execution time.
    #[must_use]
    pub fn executed_at(&self) -> DateTime<Utc> {
        self.executed_at
    }

    /// Returns true if this is a buy trade.
    #[must_use]
    pub fn is_buy(&self) -> bool {
        matches!(self.direction, OrderDirection::Buy)
    }

    /// Returns true if this is a sell trade.
    #[must_use]
    pub fn is_sell(&self) -> bool {
        matches!(self.direction, OrderDirection::Sell)
    }

    /// Returns the net value (value minus commission).
    #[must_use]
    pub fn net_value(&self) -> Money {
        match &self.commission {
            Some(comm) if self.is_buy() => {
                // For buys, we pay value + commission
                self.value.checked_add(comm).unwrap_or(self.value.clone())
            }
            Some(comm) if self.is_sell() => {
                // For sells, we receive value - commission
                self.value.checked_sub(comm).unwrap_or(self.value.clone())
            }
            _ => self.value.clone(),
        }
    }
}

/// A collection of trades for history tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
    /// Creates a new empty trade history.
    #[must_use]
    pub fn new() -> Self {
        Self { trades: Vec::new() }
    }

    /// Adds a trade to the history.
    pub fn add(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    /// Returns all trades.
    #[must_use]
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// Returns the number of trades.
    #[must_use]
    pub fn len(&self) -> usize {
        self.trades.len()
    }

    /// Returns true if there are no trades.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.trades.is_empty()
    }

    /// Returns trades for a specific symbol.
    pub fn for_symbol<'a>(&'a self, symbol: &'a str) -> impl Iterator<Item = &'a Trade> + 'a {
        self.trades.iter().filter(move |t| t.symbol() == symbol)
    }

    /// Returns trades in a time range.
    pub fn in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> impl Iterator<Item = &Trade> + '_ {
        self.trades
            .iter()
            .filter(move |t| t.executed_at >= start && t.executed_at <= end)
    }

    /// Calculates total buy value for a symbol.
    #[must_use]
    pub fn total_buy_value(&self, symbol: &str, currency: &str) -> Money {
        self.for_symbol(symbol)
            .filter(|t| t.is_buy())
            .fold(Money::zero(currency), |acc, t| {
                acc.checked_add(t.value()).unwrap_or(acc)
            })
    }

    /// Calculates total sell value for a symbol.
    #[must_use]
    pub fn total_sell_value(&self, symbol: &str, currency: &str) -> Money {
        self.for_symbol(symbol)
            .filter(|t| t.is_sell())
            .fold(Money::zero(currency), |acc, t| {
                acc.checked_add(t.value()).unwrap_or(acc)
            })
    }

    /// Clears all trades.
    pub fn clear(&mut self) {
        self.trades.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_usd(amount: Decimal) -> Money {
        Money::new(amount, "USD")
    }

    #[test]
    fn test_trade_creation() {
        let trade = Trade::buy("AAPL", "nasdaq", 10, make_usd(dec!(150)));

        assert_eq!(trade.symbol(), "AAPL");
        assert_eq!(trade.quantity_lots(), 10);
        assert_eq!(trade.price().amount(), dec!(150));
        assert_eq!(trade.value().amount(), dec!(1500));
        assert!(trade.is_buy());
    }

    #[test]
    fn test_trade_with_commission() {
        let trade = Trade::buy("AAPL", "nasdaq", 10, make_usd(dec!(100)))
            .with_commission(make_usd(dec!(5)));

        assert_eq!(trade.commission().unwrap().amount(), dec!(5));
        // Net value for buy = value + commission
        assert_eq!(trade.net_value().amount(), dec!(1005));
    }

    #[test]
    fn test_sell_net_value() {
        let trade = Trade::sell("AAPL", "nasdaq", 10, make_usd(dec!(100)))
            .with_commission(make_usd(dec!(5)));

        // Net value for sell = value - commission
        assert_eq!(trade.net_value().amount(), dec!(995));
    }

    #[test]
    fn test_trade_history() {
        let mut history = TradeHistory::new();

        history.add(Trade::buy("AAPL", "nasdaq", 10, make_usd(dec!(100))));
        history.add(Trade::sell("AAPL", "nasdaq", 5, make_usd(dec!(110))));
        history.add(Trade::buy("GOOGL", "nasdaq", 2, make_usd(dec!(2800))));

        assert_eq!(history.len(), 3);
        assert_eq!(history.for_symbol("AAPL").count(), 2);
        assert_eq!(history.for_symbol("GOOGL").count(), 1);
    }

    #[test]
    fn test_trade_id() {
        let id = TradeId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
        assert_eq!(id.to_string(), "test-123");
    }
}
