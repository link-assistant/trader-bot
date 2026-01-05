//! Plan mode types for displaying planned orders.
//!
//! This module provides types and utilities for the `--plan` option,
//! which allows users to see what orders would be placed without executing them.

use std::collections::HashMap;
use std::fmt;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Money;
use crate::strategy::RebalanceAction;

/// A planned order that would be executed if not in plan mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedOrder {
    /// The trading symbol.
    pub symbol: String,
    /// The exchange.
    pub exchange: String,
    /// Order direction ("BUY" or "SELL").
    pub direction: String,
    /// Number of lots to trade.
    pub lots: u32,
    /// Estimated price per lot.
    pub estimated_price: Money,
    /// Estimated total value of the trade.
    pub estimated_value: Money,
    /// Current allocation percentage.
    pub current_percent: Decimal,
    /// Target allocation percentage.
    pub target_percent: Decimal,
    /// The account this order is for.
    pub account_id: String,
    /// Optional reason or note for the order.
    pub reason: Option<String>,
}

impl PlannedOrder {
    /// Creates a new planned order from a rebalance action.
    #[must_use]
    pub fn from_rebalance_action(action: &RebalanceAction, account_id: &str) -> Self {
        Self {
            symbol: action.symbol.clone(),
            exchange: action.exchange.clone(),
            direction: if action.is_buy() {
                "BUY".to_string()
            } else {
                "SELL".to_string()
            },
            lots: action.lots,
            estimated_price: action.estimated_price.clone(),
            estimated_value: action.estimated_value.clone(),
            current_percent: action.current_percent,
            target_percent: action.target_percent,
            account_id: account_id.to_string(),
            reason: None,
        }
    }

    /// Adds a reason to this planned order.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

impl fmt::Display for PlannedOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} lots of {} @ {} (value: {}) [{:.2}% -> {:.2}%]",
            self.direction,
            self.lots,
            self.symbol,
            self.estimated_price,
            self.estimated_value,
            self.current_percent,
            self.target_percent
        )
    }
}

/// A collection of planned orders grouped by account.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlannedOrders {
    /// Planned orders grouped by account ID.
    orders: HashMap<String, Vec<PlannedOrder>>,
    /// Total value of all planned buy orders.
    total_buy_value: Option<Money>,
    /// Total value of all planned sell orders.
    total_sell_value: Option<Money>,
}

impl PlannedOrders {
    /// Creates a new empty collection of planned orders.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a planned order for an account.
    pub fn add_order(&mut self, order: PlannedOrder) {
        let account_id = order.account_id.clone();
        self.orders.entry(account_id).or_default().push(order);
    }

    /// Adds multiple planned orders for an account.
    pub fn add_orders(&mut self, account_id: &str, orders: impl IntoIterator<Item = PlannedOrder>) {
        for mut order in orders {
            order.account_id = account_id.to_string();
            self.add_order(order);
        }
    }

    /// Returns true if there are no planned orders.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty() || self.orders.values().all(|v| v.is_empty())
    }

    /// Returns the total number of planned orders.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.orders.values().map(Vec::len).sum()
    }

    /// Returns the number of accounts with planned orders.
    #[must_use]
    pub fn account_count(&self) -> usize {
        self.orders.values().filter(|v| !v.is_empty()).count()
    }

    /// Returns an iterator over all account IDs with orders.
    pub fn accounts(&self) -> impl Iterator<Item = &str> {
        self.orders.keys().map(String::as_str)
    }

    /// Returns orders for a specific account.
    #[must_use]
    pub fn orders_for_account(&self, account_id: &str) -> Option<&[PlannedOrder]> {
        self.orders.get(account_id).map(Vec::as_slice)
    }

    /// Returns an iterator over all planned orders.
    pub fn all_orders(&self) -> impl Iterator<Item = &PlannedOrder> {
        self.orders.values().flat_map(|v| v.iter())
    }

    /// Sets the total buy value.
    pub fn set_total_buy_value(&mut self, value: Money) {
        self.total_buy_value = Some(value);
    }

    /// Sets the total sell value.
    pub fn set_total_sell_value(&mut self, value: Money) {
        self.total_sell_value = Some(value);
    }

    /// Returns the total buy value if set.
    #[must_use]
    pub fn total_buy_value(&self) -> Option<&Money> {
        self.total_buy_value.as_ref()
    }

    /// Returns the total sell value if set.
    #[must_use]
    pub fn total_sell_value(&self) -> Option<&Money> {
        self.total_sell_value.as_ref()
    }

    /// Displays the planned orders in a formatted way.
    pub fn display(&self) {
        if self.is_empty() {
            println!("\n📋 PLAN MODE: No orders would be placed");
            println!("   Portfolio is already balanced within tolerance.");
            return;
        }

        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║                     📋 PLANNED ORDERS                          ║");
        println!("║         (Plan mode - no actual orders will be placed)          ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        for account_id in self.accounts() {
            if let Some(orders) = self.orders_for_account(account_id) {
                if orders.is_empty() {
                    continue;
                }

                println!("Account: {}", account_id);
                println!("{}", "-".repeat(60));

                let buys: Vec<_> = orders.iter().filter(|o| o.direction == "BUY").collect();
                let sells: Vec<_> = orders.iter().filter(|o| o.direction == "SELL").collect();

                if !sells.is_empty() {
                    println!("  SELL orders:");
                    for order in &sells {
                        println!(
                            "    • {} {} lots @ {} = {}",
                            order.symbol, order.lots, order.estimated_price, order.estimated_value
                        );
                        println!(
                            "      Allocation: {:.2}% → {:.2}%",
                            order.current_percent, order.target_percent
                        );
                    }
                }

                if !buys.is_empty() {
                    println!("  BUY orders:");
                    for order in &buys {
                        println!(
                            "    • {} {} lots @ {} = {}",
                            order.symbol, order.lots, order.estimated_price, order.estimated_value
                        );
                        println!(
                            "      Allocation: {:.2}% → {:.2}%",
                            order.current_percent, order.target_percent
                        );
                    }
                }

                println!();
            }
        }

        // Summary
        println!("═══════════════════════════════════════════════════════════════");
        println!("SUMMARY:");
        println!("  Total accounts: {}", self.account_count());
        println!("  Total orders:   {}", self.total_count());
        if let Some(sell_value) = &self.total_sell_value {
            println!("  Total sells:    {}", sell_value);
        }
        if let Some(buy_value) = &self.total_buy_value {
            println!("  Total buys:     {}", buy_value);
        }
        println!("═══════════════════════════════════════════════════════════════\n");
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
    fn test_planned_order_display() {
        let order = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 10,
            estimated_price: make_usd(dec!(150)),
            estimated_value: make_usd(dec!(1500)),
            current_percent: dec!(20),
            target_percent: dec!(30),
            account_id: "acc1".to_string(),
            reason: None,
        };

        let display = format!("{order}");
        assert!(display.contains("BUY"));
        assert!(display.contains("10 lots"));
        assert!(display.contains("AAPL"));
        assert!(display.contains("20.00%"));
        assert!(display.contains("30.00%"));
    }

    #[test]
    fn test_planned_orders_collection() {
        let mut planned = PlannedOrders::new();
        assert!(planned.is_empty());
        assert_eq!(planned.total_count(), 0);

        let order1 = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 10,
            estimated_price: make_usd(dec!(150)),
            estimated_value: make_usd(dec!(1500)),
            current_percent: dec!(20),
            target_percent: dec!(30),
            account_id: "acc1".to_string(),
            reason: None,
        };

        let order2 = PlannedOrder {
            symbol: "GOOGL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "SELL".to_string(),
            lots: 5,
            estimated_price: make_usd(dec!(2800)),
            estimated_value: make_usd(dec!(14000)),
            current_percent: dec!(40),
            target_percent: dec!(25),
            account_id: "acc1".to_string(),
            reason: None,
        };

        planned.add_order(order1);
        planned.add_order(order2);

        assert!(!planned.is_empty());
        assert_eq!(planned.total_count(), 2);
        assert_eq!(planned.account_count(), 1);

        let acc_orders = planned.orders_for_account("acc1").unwrap();
        assert_eq!(acc_orders.len(), 2);
    }

    #[test]
    fn test_planned_order_with_reason() {
        let order = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 10,
            estimated_price: make_usd(dec!(150)),
            estimated_value: make_usd(dec!(1500)),
            current_percent: dec!(20),
            target_percent: dec!(30),
            account_id: "acc1".to_string(),
            reason: None,
        };

        let order = order.with_reason("Underweight in portfolio");
        assert_eq!(order.reason, Some("Underweight in portfolio".to_string()));
    }

    #[test]
    fn test_planned_orders_multiple_accounts() {
        let mut planned = PlannedOrders::new();

        let order1 = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 10,
            estimated_price: make_usd(dec!(150)),
            estimated_value: make_usd(dec!(1500)),
            current_percent: dec!(20),
            target_percent: dec!(30),
            account_id: "acc1".to_string(),
            reason: None,
        };

        let order2 = PlannedOrder {
            symbol: "MSFT".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 5,
            estimated_price: make_usd(dec!(400)),
            estimated_value: make_usd(dec!(2000)),
            current_percent: dec!(10),
            target_percent: dec!(25),
            account_id: "acc2".to_string(),
            reason: None,
        };

        planned.add_order(order1);
        planned.add_order(order2);

        assert_eq!(planned.account_count(), 2);
        assert_eq!(planned.total_count(), 2);

        assert!(planned.orders_for_account("acc1").is_some());
        assert!(planned.orders_for_account("acc2").is_some());
        assert!(planned.orders_for_account("acc3").is_none());
    }
}
