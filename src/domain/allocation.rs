//! Recursive allocation system for portfolio management.
//!
//! This module provides a flexible allocation system that supports:
//! - Simple ticker-based allocations (e.g., SBER 30%, LKOH 30%, GAZP 40%)
//! - Strategy-based allocations (e.g., 50% balancer, 30% scalper, 20% hold)
//! - Nested/recursive allocations (balancers containing other balancers)
//!
//! # Links Notation Configuration
//!
//! ```text
//! allocation
//!   strategy balancer
//!     portion 30
//!       allocation
//!         strategy hold
//!         ticker SBER
//!     portion 30
//!       allocation
//!         strategy hold
//!         ticker LKOH
//!     portion 40
//!       allocation
//!         strategy hold
//!         ticker GAZP
//! ```
//!
//! This is equivalent to the short notation:
//!
//! ```text
//! allocation
//!   SBER 30
//!   LKOH 30
//!   GAZP 40
//! ```

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of allocation strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StrategyType {
    /// Balancer strategy - maintains target allocations across sub-allocations.
    #[default]
    Balancer,
    /// Hold strategy - maintains a position in a single ticker.
    Hold,
    /// Scalper strategy - buy-low/sell-high with FIFO tracking.
    Scalper,
    /// DCA (Dollar Cost Average) strategy - regular purchases.
    Dca,
    /// Custom strategy with a name.
    Custom(String),
}

impl StrategyType {
    /// Creates a strategy type from a string name.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "balancer" | "balance" | "rebalance" => Self::Balancer,
            "hold" | "holding" | "hodl" => Self::Hold,
            "scalp" | "scalper" | "scalping" => Self::Scalper,
            "dca" | "dollar_cost_average" => Self::Dca,
            _ => Self::Custom(name.to_string()),
        }
    }

    /// Returns the strategy name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Balancer => "balancer",
            Self::Hold => "hold",
            Self::Scalper => "scalper",
            Self::Dca => "dca",
            Self::Custom(name) => name,
        }
    }
}

/// A portion of an allocation with its own strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationPortion {
    /// The percentage of the parent allocation (0-100).
    pub percentage: Decimal,
    /// The allocation details for this portion.
    pub allocation: Allocation,
}

impl AllocationPortion {
    /// Creates a new allocation portion.
    #[must_use]
    pub fn new(percentage: Decimal, allocation: Allocation) -> Self {
        Self {
            percentage,
            allocation,
        }
    }

    /// Creates a simple hold portion for a ticker.
    #[must_use]
    pub fn hold(ticker: impl Into<String>, percentage: Decimal) -> Self {
        Self {
            percentage,
            allocation: Allocation::hold(ticker),
        }
    }
}

/// A recursive allocation structure supporting nested strategies.
///
/// An allocation can be:
/// 1. A simple ticker hold (ticker + optional strategy)
/// 2. A strategy with portions (balancer managing sub-allocations)
/// 3. A mix of both
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Allocation {
    /// The strategy to use for this allocation.
    #[serde(default)]
    pub strategy: StrategyType,

    /// The ticker symbol (for hold/scalper strategies).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// Child portions (for balancer strategy).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portions: Vec<AllocationPortion>,

    /// Strategy-specific parameters.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub params: HashMap<String, String>,
}

impl Allocation {
    /// Creates a new empty allocation with default balancer strategy.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new balancer allocation with portions.
    #[must_use]
    pub fn balancer(portions: Vec<AllocationPortion>) -> Self {
        Self {
            strategy: StrategyType::Balancer,
            ticker: None,
            portions,
            params: HashMap::new(),
        }
    }

    /// Creates a simple hold allocation for a ticker.
    #[must_use]
    pub fn hold(ticker: impl Into<String>) -> Self {
        Self {
            strategy: StrategyType::Hold,
            ticker: Some(ticker.into()),
            portions: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Creates a scalper allocation for a ticker.
    #[must_use]
    pub fn scalper(ticker: impl Into<String>) -> Self {
        Self {
            strategy: StrategyType::Scalper,
            ticker: Some(ticker.into()),
            portions: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Creates a simple allocation from ticker-percentage pairs.
    ///
    /// This is the "short notation" equivalent where:
    /// ```text
    /// allocation
    ///   SBER 30
    ///   LKOH 30
    ///   GAZP 40
    /// ```
    ///
    /// Becomes a balancer with hold portions.
    #[must_use]
    pub fn from_simple(allocations: &[(String, Decimal)]) -> Self {
        let portions = allocations
            .iter()
            .map(|(ticker, pct)| AllocationPortion::hold(ticker, *pct))
            .collect();

        Self::balancer(portions)
    }

    /// Creates an allocation from a HashMap of ticker -> percentage.
    #[must_use]
    pub fn from_map(allocations: HashMap<String, Decimal>) -> Self {
        let portions = allocations
            .into_iter()
            .map(|(ticker, pct)| AllocationPortion::hold(ticker, pct))
            .collect();

        Self::balancer(portions)
    }

    /// Sets a strategy parameter.
    pub fn set_param(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.params.insert(key.into(), value.into());
    }

    /// Gets a strategy parameter.
    #[must_use]
    pub fn get_param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Adds a portion to the allocation.
    pub fn add_portion(&mut self, portion: AllocationPortion) {
        self.portions.push(portion);
    }

    /// Returns true if this is a simple hold strategy.
    #[must_use]
    pub fn is_hold(&self) -> bool {
        matches!(self.strategy, StrategyType::Hold) && self.ticker.is_some()
    }

    /// Returns true if this is a balancer strategy.
    #[must_use]
    pub fn is_balancer(&self) -> bool {
        matches!(self.strategy, StrategyType::Balancer) && !self.portions.is_empty()
    }

    /// Returns all tickers involved in this allocation (recursively).
    #[must_use]
    pub fn all_tickers(&self) -> Vec<String> {
        let mut tickers = Vec::new();
        self.collect_tickers(&mut tickers);
        tickers
    }

    fn collect_tickers(&self, tickers: &mut Vec<String>) {
        if let Some(ref ticker) = self.ticker {
            if !tickers.contains(ticker) {
                tickers.push(ticker.clone());
            }
        }
        for portion in &self.portions {
            portion.allocation.collect_tickers(tickers);
        }
    }

    /// Calculates the total percentage of all portions.
    #[must_use]
    pub fn total_percentage(&self) -> Decimal {
        self.portions.iter().map(|p| p.percentage).sum()
    }

    /// Validates that portions sum to approximately 100%.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.portions.is_empty() {
            // Simple hold/scalper allocation is always valid if it has a ticker
            return self.ticker.is_some();
        }

        let total = self.total_percentage();
        let diff = (total - Decimal::from(100)).abs();
        diff < Decimal::ONE // Allow 1% tolerance
    }

    /// Normalizes portions to sum to exactly 100%.
    pub fn normalize(&mut self) {
        let total = self.total_percentage();
        if total.is_zero() {
            return;
        }

        let factor = Decimal::from(100) / total;
        for portion in &mut self.portions {
            portion.percentage *= factor;
        }
    }

    /// Flattens the allocation to a simple ticker -> percentage map.
    ///
    /// This recursively expands all nested allocations and calculates
    /// the effective percentage for each ticker.
    #[must_use]
    pub fn flatten(&self) -> HashMap<String, Decimal> {
        let mut result = HashMap::new();
        self.flatten_recursive(Decimal::from(100), &mut result);
        result
    }

    fn flatten_recursive(&self, parent_pct: Decimal, result: &mut HashMap<String, Decimal>) {
        // If this is a simple hold/scalper, add the ticker
        if let Some(ref ticker) = self.ticker {
            let entry = result.entry(ticker.clone()).or_insert(Decimal::ZERO);
            *entry += parent_pct;
            return;
        }

        // Otherwise, process portions
        for portion in &self.portions {
            let effective_pct = parent_pct * portion.percentage / Decimal::from(100);
            portion.allocation.flatten_recursive(effective_pct, result);
        }
    }
}

/// Converts a `DesiredAllocation` to the new `Allocation` format.
impl From<super::DesiredAllocation> for Allocation {
    fn from(desired: super::DesiredAllocation) -> Self {
        let map: HashMap<String, Decimal> = desired.iter().map(|(k, v)| (k.clone(), *v)).collect();
        Self::from_map(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simple_allocation() {
        let alloc = Allocation::from_simple(&[
            ("SBER".to_string(), dec!(30)),
            ("LKOH".to_string(), dec!(30)),
            ("GAZP".to_string(), dec!(40)),
        ]);

        assert!(alloc.is_balancer());
        assert!(alloc.is_valid());
        assert_eq!(alloc.portions.len(), 3);

        let flat = alloc.flatten();
        assert_eq!(flat.get("SBER"), Some(&dec!(30)));
        assert_eq!(flat.get("LKOH"), Some(&dec!(30)));
        assert_eq!(flat.get("GAZP"), Some(&dec!(40)));
    }

    #[test]
    fn test_hold_allocation() {
        let alloc = Allocation::hold("BTC");
        assert!(alloc.is_hold());
        assert_eq!(alloc.ticker, Some("BTC".to_string()));
        assert_eq!(alloc.all_tickers(), vec!["BTC".to_string()]);
    }

    #[test]
    fn test_nested_allocation() {
        // Create nested allocation:
        // 60% to stocks (SBER 50%, LKOH 50%)
        // 40% to crypto (BTC 70%, ETH 30%)
        let stocks = Allocation::balancer(vec![
            AllocationPortion::hold("SBER", dec!(50)),
            AllocationPortion::hold("LKOH", dec!(50)),
        ]);

        let crypto = Allocation::balancer(vec![
            AllocationPortion::hold("BTC", dec!(70)),
            AllocationPortion::hold("ETH", dec!(30)),
        ]);

        let root = Allocation::balancer(vec![
            AllocationPortion::new(dec!(60), stocks),
            AllocationPortion::new(dec!(40), crypto),
        ]);

        assert!(root.is_valid());

        let flat = root.flatten();
        // SBER: 60% * 50% = 30%
        // LKOH: 60% * 50% = 30%
        // BTC: 40% * 70% = 28%
        // ETH: 40% * 30% = 12%
        assert_eq!(flat.get("SBER"), Some(&dec!(30)));
        assert_eq!(flat.get("LKOH"), Some(&dec!(30)));
        assert_eq!(flat.get("BTC"), Some(&dec!(28)));
        assert_eq!(flat.get("ETH"), Some(&dec!(12)));

        let all_tickers = root.all_tickers();
        assert_eq!(all_tickers.len(), 4);
    }

    #[test]
    fn test_strategy_types() {
        assert_eq!(StrategyType::from_name("balancer"), StrategyType::Balancer);
        assert_eq!(StrategyType::from_name("HOLD"), StrategyType::Hold);
        assert_eq!(StrategyType::from_name("scalper"), StrategyType::Scalper);
        assert_eq!(
            StrategyType::from_name("custom_strat"),
            StrategyType::Custom("custom_strat".to_string())
        );
    }

    #[test]
    fn test_normalize_allocation() {
        let mut alloc = Allocation::balancer(vec![
            AllocationPortion::hold("A", dec!(30)),
            AllocationPortion::hold("B", dec!(20)),
        ]);

        // Total is 50%, normalize to 100%
        alloc.normalize();

        assert_eq!(alloc.portions[0].percentage, dec!(60)); // 30/50 * 100
        assert_eq!(alloc.portions[1].percentage, dec!(40)); // 20/50 * 100
    }

    #[test]
    fn test_deeply_nested_allocation() {
        // Test 3-level nesting
        let level3 = Allocation::balancer(vec![
            AllocationPortion::hold("X", dec!(50)),
            AllocationPortion::hold("Y", dec!(50)),
        ]);

        let level2 = Allocation::balancer(vec![AllocationPortion::new(dec!(100), level3)]);

        let root = Allocation::balancer(vec![
            AllocationPortion::new(dec!(50), level2),
            AllocationPortion::hold("Z", dec!(50)),
        ]);

        let flat = root.flatten();
        // X: 50% * 100% * 50% = 25%
        // Y: 50% * 100% * 50% = 25%
        // Z: 50%
        assert_eq!(flat.get("X"), Some(&dec!(25)));
        assert_eq!(flat.get("Y"), Some(&dec!(25)));
        assert_eq!(flat.get("Z"), Some(&dec!(50)));
    }
}
