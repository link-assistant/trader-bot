//! Order types for trading operations.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Money;

/// Direction of the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderDirection {
    /// Buy order.
    Buy,
    /// Sell order.
    Sell,
}

impl OrderDirection {
    /// Returns the opposite direction.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// Type of the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Market order - executes at best available price.
    Market,
    /// Limit order - executes at specified price or better.
    Limit,
    /// Stop order - triggers market order when price reaches stop price.
    Stop,
    /// Stop limit order - triggers limit order when price reaches stop price.
    StopLimit,
}

/// Current status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Order has been created but not yet submitted.
    Pending,
    /// Order has been submitted to the exchange.
    Submitted,
    /// Order is partially filled.
    PartiallyFilled,
    /// Order has been completely filled.
    Filled,
    /// Order has been cancelled.
    Cancelled,
    /// Order has been rejected by the exchange.
    Rejected,
    /// Order has expired.
    Expired,
}

impl OrderStatus {
    /// Returns true if the order is in a terminal state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Filled | Self::Cancelled | Self::Rejected | Self::Expired
        )
    }

    /// Returns true if the order is still active.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Submitted | Self::PartiallyFilled
        )
    }
}

/// Represents a trading order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier (client-side).
    id: String,
    /// Exchange-assigned order identifier (set after submission).
    exchange_order_id: Option<String>,
    /// The trading symbol.
    symbol: String,
    /// The exchange where the order is placed.
    exchange: String,
    /// Order direction (buy/sell).
    direction: OrderDirection,
    /// Order type.
    order_type: OrderType,
    /// Quantity in lots.
    quantity_lots: u32,
    /// Limit price (for limit orders).
    limit_price: Option<Money>,
    /// Stop price (for stop orders).
    stop_price: Option<Money>,
    /// Current order status.
    status: OrderStatus,
    /// Quantity filled so far (in lots).
    filled_lots: u32,
    /// Average fill price.
    average_fill_price: Option<Money>,
    /// Timestamp when order was created.
    created_at: DateTime<Utc>,
    /// Timestamp when order was last updated.
    updated_at: DateTime<Utc>,
    /// Optional message (e.g., rejection reason).
    message: Option<String>,
}

impl Order {
    /// Creates a new market order.
    #[must_use]
    pub fn market(
        id: impl Into<String>,
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        direction: OrderDirection,
        quantity_lots: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            exchange_order_id: None,
            symbol: symbol.into(),
            exchange: exchange.into(),
            direction,
            order_type: OrderType::Market,
            quantity_lots,
            limit_price: None,
            stop_price: None,
            status: OrderStatus::Pending,
            filled_lots: 0,
            average_fill_price: None,
            created_at: now,
            updated_at: now,
            message: None,
        }
    }

    /// Creates a new limit order.
    #[must_use]
    pub fn limit(
        id: impl Into<String>,
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        direction: OrderDirection,
        quantity_lots: u32,
        limit_price: Money,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            exchange_order_id: None,
            symbol: symbol.into(),
            exchange: exchange.into(),
            direction,
            order_type: OrderType::Limit,
            quantity_lots,
            limit_price: Some(limit_price),
            stop_price: None,
            status: OrderStatus::Pending,
            filled_lots: 0,
            average_fill_price: None,
            created_at: now,
            updated_at: now,
            message: None,
        }
    }

    /// Returns the order ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the exchange order ID if set.
    #[must_use]
    pub fn exchange_order_id(&self) -> Option<&str> {
        self.exchange_order_id.as_deref()
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

    /// Returns the order direction.
    #[must_use]
    pub const fn direction(&self) -> OrderDirection {
        self.direction
    }

    /// Returns the order type.
    #[must_use]
    pub const fn order_type(&self) -> OrderType {
        self.order_type
    }

    /// Returns the quantity in lots.
    #[must_use]
    pub const fn quantity_lots(&self) -> u32 {
        self.quantity_lots
    }

    /// Returns the limit price if set.
    #[must_use]
    pub const fn limit_price(&self) -> Option<&Money> {
        self.limit_price.as_ref()
    }

    /// Returns the current status.
    #[must_use]
    pub const fn status(&self) -> OrderStatus {
        self.status
    }

    /// Returns the number of lots filled.
    #[must_use]
    pub const fn filled_lots(&self) -> u32 {
        self.filled_lots
    }

    /// Returns the number of lots remaining.
    #[must_use]
    pub const fn remaining_lots(&self) -> u32 {
        self.quantity_lots - self.filled_lots
    }

    /// Returns the average fill price if any fills occurred.
    #[must_use]
    pub const fn average_fill_price(&self) -> Option<&Money> {
        self.average_fill_price.as_ref()
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the last update timestamp.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns the message if set.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Calculates the fill percentage.
    #[must_use]
    pub fn fill_percentage(&self) -> Decimal {
        if self.quantity_lots == 0 {
            return Decimal::ZERO;
        }
        Decimal::from(self.filled_lots) / Decimal::from(self.quantity_lots) * Decimal::from(100)
    }

    /// Updates the order status.
    pub fn set_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Sets the exchange order ID.
    pub fn set_exchange_order_id(&mut self, id: impl Into<String>) {
        self.exchange_order_id = Some(id.into());
        self.updated_at = Utc::now();
    }

    /// Records a fill.
    pub fn record_fill(&mut self, lots: u32, price: Money) {
        let prev_filled = self.filled_lots;
        self.filled_lots = (self.filled_lots + lots).min(self.quantity_lots);

        // Update average fill price
        if let Some(avg) = &self.average_fill_price {
            if prev_filled > 0 {
                let prev_value = avg.amount() * Decimal::from(prev_filled);
                let new_value = price.amount() * Decimal::from(lots);
                let total_lots = Decimal::from(self.filled_lots);
                let new_avg = (prev_value + new_value) / total_lots;
                self.average_fill_price = Some(Money::new(new_avg, price.currency()));
            }
        } else {
            self.average_fill_price = Some(price);
        }

        // Update status
        if self.filled_lots >= self.quantity_lots {
            self.status = OrderStatus::Filled;
        } else if self.filled_lots > 0 {
            self.status = OrderStatus::PartiallyFilled;
        }

        self.updated_at = Utc::now();
    }

    /// Sets a message (e.g., rejection reason).
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
        self.updated_at = Utc::now();
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
    fn test_market_order_creation() {
        let order = Order::market("ORD001", "AAPL", "NASDAQ", OrderDirection::Buy, 10);
        assert_eq!(order.id(), "ORD001");
        assert_eq!(order.symbol(), "AAPL");
        assert_eq!(order.direction(), OrderDirection::Buy);
        assert_eq!(order.order_type(), OrderType::Market);
        assert_eq!(order.quantity_lots(), 10);
        assert_eq!(order.status(), OrderStatus::Pending);
    }

    #[test]
    fn test_limit_order_creation() {
        let order = Order::limit(
            "ORD002",
            "SBER",
            "MOEX",
            OrderDirection::Sell,
            5,
            make_usd(dec!(250)),
        );
        assert_eq!(order.order_type(), OrderType::Limit);
        assert_eq!(order.limit_price().unwrap().amount(), dec!(250));
    }

    #[test]
    fn test_order_fill() {
        let mut order = Order::market("ORD001", "AAPL", "NASDAQ", OrderDirection::Buy, 10);

        order.record_fill(3, make_usd(dec!(150)));
        assert_eq!(order.filled_lots(), 3);
        assert_eq!(order.status(), OrderStatus::PartiallyFilled);
        assert_eq!(order.average_fill_price().unwrap().amount(), dec!(150));

        order.record_fill(7, make_usd(dec!(151)));
        assert_eq!(order.filled_lots(), 10);
        assert_eq!(order.status(), OrderStatus::Filled);
    }

    #[test]
    fn test_order_status_terminal() {
        assert!(!OrderStatus::Pending.is_terminal());
        assert!(!OrderStatus::Submitted.is_terminal());
        assert!(!OrderStatus::PartiallyFilled.is_terminal());
        assert!(OrderStatus::Filled.is_terminal());
        assert!(OrderStatus::Cancelled.is_terminal());
        assert!(OrderStatus::Rejected.is_terminal());
        assert!(OrderStatus::Expired.is_terminal());
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(OrderDirection::Buy.opposite(), OrderDirection::Sell);
        assert_eq!(OrderDirection::Sell.opposite(), OrderDirection::Buy);
    }
}
