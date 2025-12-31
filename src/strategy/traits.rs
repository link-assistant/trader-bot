//! Strategy traits and types.
//!
//! This module defines the core abstractions for trading strategies.

use crate::domain::{Order, Position};
use crate::exchange::ExchangeResult;
use crate::exchange::OrderBook;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// An action the strategy wants to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyAction {
    /// Place a new buy order.
    Buy {
        /// The symbol to buy.
        symbol: String,
        /// Number of lots to buy.
        quantity_lots: u32,
        /// Optional limit price (None for market order).
        price: Option<Decimal>,
    },

    /// Place a new sell order.
    Sell {
        /// The symbol to sell.
        symbol: String,
        /// Number of lots to sell.
        quantity_lots: u32,
        /// Optional limit price (None for market order).
        price: Option<Decimal>,
    },

    /// Cancel an existing order.
    Cancel {
        /// The order ID to cancel.
        order_id: String,
    },

    /// Modify an existing order's price.
    ModifyPrice {
        /// The order ID to modify.
        order_id: String,
        /// The new price.
        new_price: Decimal,
    },

    /// Do nothing.
    Hold,
}

impl StrategyAction {
    /// Creates a market buy action.
    #[must_use]
    pub fn market_buy(symbol: impl Into<String>, quantity_lots: u32) -> Self {
        Self::Buy {
            symbol: symbol.into(),
            quantity_lots,
            price: None,
        }
    }

    /// Creates a limit buy action.
    #[must_use]
    pub fn limit_buy(symbol: impl Into<String>, quantity_lots: u32, price: Decimal) -> Self {
        Self::Buy {
            symbol: symbol.into(),
            quantity_lots,
            price: Some(price),
        }
    }

    /// Creates a market sell action.
    #[must_use]
    pub fn market_sell(symbol: impl Into<String>, quantity_lots: u32) -> Self {
        Self::Sell {
            symbol: symbol.into(),
            quantity_lots,
            price: None,
        }
    }

    /// Creates a limit sell action.
    #[must_use]
    pub fn limit_sell(symbol: impl Into<String>, quantity_lots: u32, price: Decimal) -> Self {
        Self::Sell {
            symbol: symbol.into(),
            quantity_lots,
            price: Some(price),
        }
    }

    /// Creates a cancel action.
    #[must_use]
    pub fn cancel(order_id: impl Into<String>) -> Self {
        Self::Cancel {
            order_id: order_id.into(),
        }
    }

    /// Creates a hold action.
    #[must_use]
    pub fn hold() -> Self {
        Self::Hold
    }

    /// Checks if this is a hold action.
    #[must_use]
    pub fn is_hold(&self) -> bool {
        matches!(self, Self::Hold)
    }

    /// Returns the symbol for buy/sell actions.
    #[must_use]
    pub fn symbol(&self) -> Option<&str> {
        match self {
            Self::Buy { symbol, .. } | Self::Sell { symbol, .. } => Some(symbol),
            _ => None,
        }
    }
}

/// The decision made by a strategy, containing multiple possible actions.
#[derive(Debug, Clone, Default)]
pub struct StrategyDecision {
    actions: Vec<StrategyAction>,
    reason: Option<String>,
}

impl StrategyDecision {
    /// Creates a new empty decision.
    #[must_use]
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            reason: None,
        }
    }

    /// Creates a decision with a single action.
    #[must_use]
    pub fn single(action: StrategyAction) -> Self {
        Self {
            actions: vec![action],
            reason: None,
        }
    }

    /// Creates a hold decision.
    #[must_use]
    pub fn hold() -> Self {
        Self::single(StrategyAction::Hold)
    }

    /// Creates a hold decision with a reason.
    #[must_use]
    pub fn hold_with_reason(reason: impl Into<String>) -> Self {
        Self {
            actions: vec![StrategyAction::Hold],
            reason: Some(reason.into()),
        }
    }

    /// Adds an action to the decision.
    pub fn add_action(&mut self, action: StrategyAction) {
        self.actions.push(action);
    }

    /// Sets the reason for the decision.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Returns the actions.
    #[must_use]
    pub fn actions(&self) -> &[StrategyAction] {
        &self.actions
    }

    /// Returns the reason for the decision.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// Checks if this is a hold decision (no actions or only hold actions).
    #[must_use]
    pub fn is_hold(&self) -> bool {
        self.actions.is_empty() || self.actions.iter().all(StrategyAction::is_hold)
    }

    /// Returns the number of actions.
    #[must_use]
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
}

/// Market state provided to the strategy for decision making.
#[derive(Debug, Clone)]
pub struct MarketState {
    /// The instrument being traded.
    pub symbol: String,
    /// Current order book (if available).
    pub order_book: Option<OrderBook>,
    /// Current position (if any).
    pub position: Option<Position>,
    /// Active orders.
    pub active_orders: Vec<Order>,
    /// Available cash for trading.
    pub available_cash: Decimal,
    /// Current timestamp.
    pub timestamp: DateTime<Utc>,
    /// Last price of the instrument.
    pub last_price: Option<Decimal>,
    /// Currency of the instrument.
    pub currency: String,
}

impl MarketState {
    /// Creates a new market state.
    #[must_use]
    pub fn new(symbol: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            order_book: None,
            position: None,
            active_orders: Vec::new(),
            available_cash: Decimal::ZERO,
            timestamp: Utc::now(),
            last_price: None,
            currency: currency.into(),
        }
    }

    /// Sets the order book.
    pub fn with_order_book(mut self, order_book: OrderBook) -> Self {
        self.order_book = Some(order_book);
        self
    }

    /// Sets the position.
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets the available cash.
    pub fn with_cash(mut self, cash: Decimal) -> Self {
        self.available_cash = cash;
        self
    }

    /// Sets the last price.
    pub fn with_last_price(mut self, price: Decimal) -> Self {
        self.last_price = Some(price);
        self
    }

    /// Returns the best bid price from the order book.
    #[must_use]
    pub fn best_bid(&self) -> Option<Decimal> {
        self.order_book.as_ref().and_then(|ob| ob.best_bid_price())
    }

    /// Returns the best ask price from the order book.
    #[must_use]
    pub fn best_ask(&self) -> Option<Decimal> {
        self.order_book.as_ref().and_then(|ob| ob.best_ask_price())
    }

    /// Returns the current position quantity in lots.
    #[must_use]
    pub fn position_lots(&self) -> u32 {
        self.position.as_ref().map_or(0, |p| p.lots_held())
    }
}

/// A trading strategy that makes decisions based on market state.
///
/// Strategies are composable - a balancer strategy can contain sub-strategies.
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Returns the name of the strategy.
    fn name(&self) -> &str;

    /// Returns a unique identifier for this strategy instance.
    fn id(&self) -> &str {
        self.name()
    }

    /// Makes a decision based on the current market state.
    async fn decide(&self, state: &MarketState) -> StrategyDecision;

    /// Called when an order is filled.
    async fn on_order_filled(&mut self, order: &Order);

    /// Called when an order is cancelled.
    async fn on_order_cancelled(&mut self, order: &Order);

    /// Resets the strategy state.
    async fn reset(&mut self);

    /// Returns the symbols this strategy trades.
    fn symbols(&self) -> Vec<String>;

    /// Returns whether the strategy is enabled.
    fn is_enabled(&self) -> bool {
        true
    }
}

/// A strategy executor that runs strategies against an exchange.
#[async_trait]
pub trait StrategyExecutor: Send + Sync {
    /// Executes the strategy decision, placing orders on the exchange.
    async fn execute(
        &self,
        account_id: &str,
        decision: &StrategyDecision,
    ) -> ExchangeResult<Vec<Order>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_action_creation() {
        let buy = StrategyAction::market_buy("AAPL", 10);
        assert!(matches!(buy, StrategyAction::Buy { price: None, .. }));

        let sell = StrategyAction::market_sell("AAPL", 5);
        assert!(matches!(sell, StrategyAction::Sell { price: None, .. }));
    }

    #[test]
    fn test_strategy_decision() {
        let mut decision = StrategyDecision::new();
        decision.add_action(StrategyAction::market_buy("AAPL", 10));
        decision.add_action(StrategyAction::market_sell("GOOGL", 5));

        assert_eq!(decision.actions().len(), 2);
        assert!(!decision.is_hold());
    }

    #[test]
    fn test_hold_decision() {
        let decision = StrategyDecision::hold_with_reason("Market closed");
        assert!(decision.is_hold());
        assert_eq!(decision.reason(), Some("Market closed"));
    }

    #[test]
    fn test_market_state() {
        let state = MarketState::new("AAPL", "USD")
            .with_cash(Decimal::from(10000))
            .with_last_price(Decimal::from(150));

        assert_eq!(state.symbol, "AAPL");
        assert_eq!(state.available_cash, Decimal::from(10000));
        assert_eq!(state.last_price, Some(Decimal::from(150)));
    }
}
