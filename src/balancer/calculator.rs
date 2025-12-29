//! Rebalance calculation logic.
//!
//! This module contains the pure calculation logic for determining what
//! trades need to be made to rebalance a portfolio.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{DesiredAllocation, Money, Wallet};

/// Represents the difference between current and desired allocation for a symbol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationDiff {
    /// The trading symbol.
    pub symbol: String,
    /// Current percentage allocation (0-100).
    pub current_percent: Decimal,
    /// Desired percentage allocation (0-100).
    pub desired_percent: Decimal,
    /// Difference (desired - current).
    pub diff_percent: Decimal,
    /// Current value in base currency.
    pub current_value: Decimal,
    /// Desired value in base currency.
    pub desired_value: Decimal,
    /// Value difference (desired - current).
    pub diff_value: Decimal,
}

impl AllocationDiff {
    /// Returns true if we need to buy this asset.
    #[must_use]
    pub fn needs_buy(&self) -> bool {
        self.diff_value > Decimal::ZERO
    }

    /// Returns true if we need to sell this asset.
    #[must_use]
    pub fn needs_sell(&self) -> bool {
        self.diff_value < Decimal::ZERO
    }

    /// Returns the absolute value difference.
    #[must_use]
    pub fn abs_diff_value(&self) -> Decimal {
        self.diff_value.abs()
    }

    /// Returns the absolute percentage difference.
    #[must_use]
    pub fn abs_diff_percent(&self) -> Decimal {
        self.diff_percent.abs()
    }
}

/// Calculator for portfolio rebalancing.
#[derive(Debug, Clone)]
pub struct RebalanceCalculator {
    /// Minimum trade size as a percentage of portfolio value.
    min_trade_percent: Decimal,
    /// Minimum trade size in absolute value.
    min_trade_value: Decimal,
    /// Tolerance for considering allocation "close enough" (percentage).
    tolerance_percent: Decimal,
}

impl Default for RebalanceCalculator {
    fn default() -> Self {
        Self {
            min_trade_percent: Decimal::new(1, 1), // 0.1%
            min_trade_value: Decimal::new(1, 0),   // 1 unit of currency
            tolerance_percent: Decimal::new(5, 1), // 0.5%
        }
    }
}

impl RebalanceCalculator {
    /// Creates a new calculator with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum trade percentage.
    #[must_use]
    pub const fn with_min_trade_percent(mut self, percent: Decimal) -> Self {
        self.min_trade_percent = percent;
        self
    }

    /// Sets the minimum trade value.
    #[must_use]
    pub const fn with_min_trade_value(mut self, value: Decimal) -> Self {
        self.min_trade_value = value;
        self
    }

    /// Sets the tolerance percentage.
    #[must_use]
    pub const fn with_tolerance_percent(mut self, percent: Decimal) -> Self {
        self.tolerance_percent = percent;
        self
    }

    /// Calculates the allocation differences between current and desired allocations.
    #[must_use]
    pub fn calculate_diffs(
        &self,
        wallet: &Wallet,
        desired: &DesiredAllocation,
    ) -> Vec<AllocationDiff> {
        let total_value = wallet.total_value().amount();
        if total_value.is_zero() {
            return Vec::new();
        }

        let current_alloc = wallet.current_allocation();
        let mut all_symbols: Vec<String> = current_alloc
            .symbols()
            .chain(desired.symbols())
            .cloned()
            .collect();
        all_symbols.sort();
        all_symbols.dedup();

        all_symbols
            .into_iter()
            .map(|symbol| {
                let current_pct = current_alloc.get(&symbol).unwrap_or(Decimal::ZERO);
                let desired_pct = desired.get(&symbol).unwrap_or(Decimal::ZERO);
                let diff_pct = desired_pct - current_pct;

                let current_val = total_value * current_pct / Decimal::from(100);
                let desired_val = total_value * desired_pct / Decimal::from(100);
                let diff_val = desired_val - current_val;

                AllocationDiff {
                    symbol,
                    current_percent: current_pct,
                    desired_percent: desired_pct,
                    diff_percent: diff_pct,
                    current_value: current_val,
                    desired_value: desired_val,
                    diff_value: diff_val,
                }
            })
            .collect()
    }

    /// Filters diffs to only include those that need rebalancing.
    #[must_use]
    pub fn filter_significant_diffs(&self, diffs: Vec<AllocationDiff>) -> Vec<AllocationDiff> {
        diffs
            .into_iter()
            .filter(|d| {
                d.abs_diff_percent() > self.tolerance_percent
                    && d.abs_diff_value() >= self.min_trade_value
            })
            .collect()
    }

    /// Sorts diffs by priority (sells first, then buys, ordered by magnitude).
    #[must_use]
    pub fn sort_by_priority(&self, mut diffs: Vec<AllocationDiff>) -> Vec<AllocationDiff> {
        // Sells come first (to free up cash), then buys
        // Within each category, larger absolute differences first
        diffs.sort_by(|a, b| {
            let a_is_sell = a.needs_sell();
            let b_is_sell = b.needs_sell();

            match (a_is_sell, b_is_sell) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.abs_diff_value().cmp(&a.abs_diff_value()),
            }
        });
        diffs
    }

    /// Calculates the number of lots to trade for a given value difference.
    #[must_use]
    pub fn calculate_lots(
        &self,
        diff_value: Decimal,
        price_per_lot: Decimal,
        _lot_size: u32,
    ) -> u32 {
        if price_per_lot.is_zero() {
            return 0;
        }

        let lots = (diff_value.abs() / price_per_lot).floor();
        lots.to_string().parse().unwrap_or(0)
    }

    /// Estimates the total cost of executing all trades.
    #[must_use]
    pub fn estimate_total_trade_value(&self, diffs: &[AllocationDiff]) -> Money {
        let total: Decimal = diffs.iter().map(|d| d.abs_diff_value()).sum();
        // We use an empty string for currency since this is abstract
        Money::new(total, "")
    }

    /// Checks if the portfolio is balanced within tolerance.
    #[must_use]
    pub fn is_balanced(&self, wallet: &Wallet, desired: &DesiredAllocation) -> bool {
        let diffs = self.calculate_diffs(wallet, desired);
        let significant = self.filter_significant_diffs(diffs);
        significant.is_empty()
    }

    /// Calculates the maximum deviation from desired allocation.
    #[must_use]
    pub fn max_deviation(&self, wallet: &Wallet, desired: &DesiredAllocation) -> Decimal {
        let diffs = self.calculate_diffs(wallet, desired);
        diffs
            .iter()
            .map(|d| d.abs_diff_percent())
            .max()
            .unwrap_or(Decimal::ZERO)
    }
}

/// Builds desired allocation based on different modes.
///
/// This enum is prepared for future use to support multiple allocation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AllocationMode {
    /// Manual allocation with fixed percentages.
    Manual,
    /// Allocation based on market capitalization.
    MarketCap,
    /// Allocation based on assets under management.
    Aum,
    /// Decorrelation strategy (diff between market cap and AUM).
    Decorrelation,
}

/// Metric data for automatic allocation calculation.
///
/// This struct is prepared for future use to support automatic allocation strategies.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AssetMetrics {
    /// Market data by symbol.
    pub data: HashMap<String, MetricValue>,
}

/// Metrics for a single asset.
///
/// This struct is prepared for future use with automatic allocation strategies.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetricValue {
    /// Market capitalization.
    pub market_cap: Option<Decimal>,
    /// Assets under management.
    pub aum: Option<Decimal>,
}

#[allow(dead_code)]
impl AssetMetrics {
    /// Creates a new empty metrics collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds metrics for a symbol.
    pub fn add(
        &mut self,
        symbol: impl Into<String>,
        market_cap: Option<Decimal>,
        aum: Option<Decimal>,
    ) {
        self.data
            .insert(symbol.into(), MetricValue { market_cap, aum });
    }

    /// Calculates allocation based on market cap.
    #[must_use]
    pub fn calculate_market_cap_allocation(&self) -> DesiredAllocation {
        let total: Decimal = self.data.values().filter_map(|m| m.market_cap).sum();

        if total.is_zero() {
            return DesiredAllocation::new();
        }

        let mut alloc = DesiredAllocation::new();
        for (symbol, metrics) in &self.data {
            if let Some(cap) = metrics.market_cap {
                let pct = cap / total * Decimal::from(100);
                alloc.set(symbol, pct);
            }
        }
        alloc
    }

    /// Calculates allocation based on AUM.
    #[must_use]
    pub fn calculate_aum_allocation(&self) -> DesiredAllocation {
        let total: Decimal = self.data.values().filter_map(|m| m.aum).sum();

        if total.is_zero() {
            return DesiredAllocation::new();
        }

        let mut alloc = DesiredAllocation::new();
        for (symbol, metrics) in &self.data {
            if let Some(aum) = metrics.aum {
                let pct = aum / total * Decimal::from(100);
                alloc.set(symbol, pct);
            }
        }
        alloc
    }

    /// Calculates decorrelation-based allocation.
    /// This favors assets where market cap > AUM (potentially undervalued).
    #[must_use]
    pub fn calculate_decorrelation_allocation(&self) -> DesiredAllocation {
        // Calculate decorrelation scores (market_cap / aum ratio)
        let mut scores: HashMap<String, Decimal> = HashMap::new();

        for (symbol, metrics) in &self.data {
            if let (Some(cap), Some(aum)) = (metrics.market_cap, metrics.aum) {
                if !aum.is_zero() {
                    // Higher score = market cap is larger relative to AUM
                    let score = cap / aum;
                    scores.insert(symbol.clone(), score);
                }
            }
        }

        if scores.is_empty() {
            return DesiredAllocation::new();
        }

        let total: Decimal = scores.values().copied().sum();
        let mut alloc = DesiredAllocation::new();

        for (symbol, score) in scores {
            let pct = score / total * Decimal::from(100);
            alloc.set(&symbol, pct);
        }

        alloc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Position;
    use rust_decimal_macros::dec;

    fn make_usd(amount: Decimal) -> Money {
        Money::new(amount, "USD")
    }

    fn create_test_wallet() -> Wallet {
        let mut wallet = Wallet::with_cash("USD", make_usd(dec!(1000)));
        wallet.add_position(Position::new(
            "AAPL",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(100)),
        ));
        wallet.add_position(Position::new(
            "GOOGL",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(200)),
        ));
        wallet.add_position(Position::new(
            "MSFT",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(100)),
        ));
        wallet
    }

    #[test]
    fn test_calculate_diffs() {
        let wallet = create_test_wallet();
        // Total = 1000 cash + 1000 AAPL + 2000 GOOGL + 1000 MSFT = 5000

        let mut desired = DesiredAllocation::new();
        desired.set("AAPL", dec!(30)); // Want 30%, have 20%
        desired.set("GOOGL", dec!(30)); // Want 30%, have 40%
        desired.set("MSFT", dec!(20)); // Want 20%, have 20%
                                       // Cash stays at 20%

        let calc = RebalanceCalculator::new();
        let diffs = calc.calculate_diffs(&wallet, &desired);

        let aapl_diff = diffs.iter().find(|d| d.symbol == "AAPL").unwrap();
        assert_eq!(aapl_diff.current_percent, dec!(20)); // 1000/5000 = 20%
        assert_eq!(aapl_diff.desired_percent, dec!(30));
        assert_eq!(aapl_diff.diff_percent, dec!(10)); // Need to increase by 10%
        assert!(aapl_diff.needs_buy());

        let googl_diff = diffs.iter().find(|d| d.symbol == "GOOGL").unwrap();
        assert_eq!(googl_diff.current_percent, dec!(40)); // 2000/5000 = 40%
        assert_eq!(googl_diff.desired_percent, dec!(30));
        assert!(googl_diff.needs_sell());
    }

    #[test]
    fn test_filter_significant_diffs() {
        let diffs = vec![
            AllocationDiff {
                symbol: "A".into(),
                current_percent: dec!(10),
                desired_percent: dec!(10.3),
                diff_percent: dec!(0.3),
                current_value: dec!(1000),
                desired_value: dec!(1030),
                diff_value: dec!(30),
            },
            AllocationDiff {
                symbol: "B".into(),
                current_percent: dec!(20),
                desired_percent: dec!(25),
                diff_percent: dec!(5),
                current_value: dec!(2000),
                desired_value: dec!(2500),
                diff_value: dec!(500),
            },
        ];

        let calc = RebalanceCalculator::new();
        let significant = calc.filter_significant_diffs(diffs);

        assert_eq!(significant.len(), 1);
        assert_eq!(significant[0].symbol, "B");
    }

    #[test]
    fn test_sort_by_priority() {
        let diffs = vec![
            AllocationDiff {
                symbol: "BUY_SMALL".into(),
                current_percent: dec!(10),
                desired_percent: dec!(15),
                diff_percent: dec!(5),
                current_value: dec!(1000),
                desired_value: dec!(1500),
                diff_value: dec!(500), // Buy
            },
            AllocationDiff {
                symbol: "SELL_BIG".into(),
                current_percent: dec!(30),
                desired_percent: dec!(20),
                diff_percent: dec!(-10),
                current_value: dec!(3000),
                desired_value: dec!(2000),
                diff_value: dec!(-1000), // Sell
            },
            AllocationDiff {
                symbol: "BUY_BIG".into(),
                current_percent: dec!(5),
                desired_percent: dec!(20),
                diff_percent: dec!(15),
                current_value: dec!(500),
                desired_value: dec!(2000),
                diff_value: dec!(1500), // Buy
            },
        ];

        let calc = RebalanceCalculator::new();
        let sorted = calc.sort_by_priority(diffs);

        // Sells first, then buys ordered by size
        assert_eq!(sorted[0].symbol, "SELL_BIG"); // Only sell
        assert_eq!(sorted[1].symbol, "BUY_BIG"); // Larger buy
        assert_eq!(sorted[2].symbol, "BUY_SMALL"); // Smaller buy
    }

    #[test]
    fn test_is_balanced() {
        let mut wallet = Wallet::with_cash("USD", make_usd(dec!(0)));
        wallet.add_position(Position::new("A", "X", dec!(50), make_usd(dec!(100))));
        wallet.add_position(Position::new("B", "X", dec!(50), make_usd(dec!(100))));

        let mut desired = DesiredAllocation::new();
        desired.set("A", dec!(50));
        desired.set("B", dec!(50));

        let calc = RebalanceCalculator::new();
        assert!(calc.is_balanced(&wallet, &desired));
    }

    #[test]
    fn test_calculate_lots() {
        let calc = RebalanceCalculator::new();

        // If we need to buy 1000 USD worth, and price per lot is 100, we need 10 lots
        assert_eq!(calc.calculate_lots(dec!(1000), dec!(100), 1), 10);

        // Partial lots are floored
        assert_eq!(calc.calculate_lots(dec!(550), dec!(100), 1), 5);
    }

    #[test]
    fn test_asset_metrics_market_cap_allocation() {
        let mut metrics = AssetMetrics::new();
        metrics.add("A", Some(dec!(1000)), None);
        metrics.add("B", Some(dec!(3000)), None);
        metrics.add("C", Some(dec!(6000)), None);
        // Total = 10000

        let alloc = metrics.calculate_market_cap_allocation();

        assert_eq!(alloc.get("A"), Some(dec!(10))); // 1000/10000 = 10%
        assert_eq!(alloc.get("B"), Some(dec!(30))); // 3000/10000 = 30%
        assert_eq!(alloc.get("C"), Some(dec!(60))); // 6000/10000 = 60%
    }

    #[test]
    fn test_decorrelation_allocation() {
        let mut metrics = AssetMetrics::new();
        // A: market cap / AUM = 2.0 (undervalued by AUM)
        metrics.add("A", Some(dec!(2000)), Some(dec!(1000)));
        // B: market cap / AUM = 0.5 (overvalued by AUM)
        metrics.add("B", Some(dec!(1000)), Some(dec!(2000)));
        // Total ratio = 2.5

        let alloc = metrics.calculate_decorrelation_allocation();

        // A should get 2.0/2.5 = 80%
        assert_eq!(alloc.get("A"), Some(dec!(80)));
        // B should get 0.5/2.5 = 20%
        assert_eq!(alloc.get("B"), Some(dec!(20)));
    }
}
