//! Holding strategy implementation.
//!
//! The holding strategy maintains a position in an asset up to a configured
//! maximum percentage or absolute value of the portfolio. This is a passive
//! strategy that buys when under-allocated and can optionally sell when
//! over-allocated.

use super::traits::{MarketState, Strategy, StrategyAction, StrategyDecision};
use crate::domain::Order;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

/// Configuration for a holding strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingConfig {
    /// The symbol to hold.
    pub symbol: String,

    /// Target allocation as percentage of portfolio (0-100).
    /// If set, this takes precedence over absolute_value.
    pub target_percent: Option<Decimal>,

    /// Target allocation as absolute value in base currency.
    pub absolute_value: Option<Decimal>,

    /// Maximum allowed deviation from target before rebalancing (percentage points).
    pub tolerance_percent: Decimal,

    /// Whether to sell when over-allocated.
    pub allow_sell: bool,

    /// Minimum trade size in lots.
    pub min_trade_lots: u32,

    /// Whether the strategy is enabled.
    pub enabled: bool,
}

impl Default for HoldingConfig {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            target_percent: None,
            absolute_value: None,
            tolerance_percent: Decimal::new(5, 1), // 0.5%
            allow_sell: false,
            min_trade_lots: 1,
            enabled: true,
        }
    }
}

impl HoldingConfig {
    /// Creates a new holding config for a symbol with a target percentage.
    #[must_use]
    pub fn with_percent(symbol: impl Into<String>, target_percent: Decimal) -> Self {
        Self {
            symbol: symbol.into(),
            target_percent: Some(target_percent),
            ..Default::default()
        }
    }

    /// Creates a new holding config for a symbol with an absolute value target.
    #[must_use]
    pub fn with_absolute(symbol: impl Into<String>, absolute_value: Decimal) -> Self {
        Self {
            symbol: symbol.into(),
            absolute_value: Some(absolute_value),
            ..Default::default()
        }
    }

    /// Sets the tolerance percentage.
    #[must_use]
    pub fn with_tolerance(mut self, tolerance: Decimal) -> Self {
        self.tolerance_percent = tolerance;
        self
    }

    /// Enables or disables selling when over-allocated.
    #[must_use]
    pub fn with_allow_sell(mut self, allow: bool) -> Self {
        self.allow_sell = allow;
        self
    }

    /// Sets the minimum trade size in lots.
    #[must_use]
    pub fn with_min_trade_lots(mut self, lots: u32) -> Self {
        self.min_trade_lots = lots;
        self
    }

    /// Enables or disables the strategy.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Holding strategy that maintains a target allocation.
#[derive(Debug)]
pub struct HoldingStrategy {
    config: HoldingConfig,
}

impl HoldingStrategy {
    /// Creates a new holding strategy with the given configuration.
    #[must_use]
    pub fn new(config: HoldingConfig) -> Self {
        Self { config }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &HoldingConfig {
        &self.config
    }

    /// Calculates the target value for this holding.
    fn calculate_target_value(&self, total_portfolio_value: Decimal) -> Decimal {
        self.config.target_percent.map_or_else(
            || self.config.absolute_value.unwrap_or(Decimal::ZERO),
            |target_percent| total_portfolio_value * target_percent / Decimal::from(100),
        )
    }

    /// Determines if we need to buy or sell based on current vs target allocation.
    fn calculate_action(
        &self,
        state: &MarketState,
        total_portfolio_value: Decimal,
    ) -> StrategyDecision {
        if !self.config.enabled {
            return StrategyDecision::hold_with_reason("Strategy disabled");
        }

        let target_value = self.calculate_target_value(total_portfolio_value);
        if target_value.is_zero() {
            return StrategyDecision::hold_with_reason("No target value configured");
        }

        // Calculate current value
        let current_value = state
            .position
            .as_ref()
            .map_or(Decimal::ZERO, |p| p.market_value().amount());

        let diff_value = target_value - current_value;
        let diff_percent = if total_portfolio_value.is_zero() {
            Decimal::ZERO
        } else {
            (diff_value.abs() / total_portfolio_value) * Decimal::from(100)
        };

        trace!(
            "{}: current={}, target={}, diff={} ({}%)",
            self.config.symbol,
            current_value,
            target_value,
            diff_value,
            diff_percent
        );

        // Check if within tolerance
        if diff_percent <= self.config.tolerance_percent {
            return StrategyDecision::hold_with_reason("Within tolerance");
        }

        // Get current price for lot calculation
        let price = match state.last_price {
            Some(p) if !p.is_zero() => p,
            _ => return StrategyDecision::hold_with_reason("No price available"),
        };

        // Calculate lots needed
        let lots_needed = (diff_value.abs() / price).floor();
        let lots_u32 = lots_needed.to_string().parse::<u32>().unwrap_or(0);

        if lots_u32 < self.config.min_trade_lots {
            return StrategyDecision::hold_with_reason("Trade size below minimum");
        }

        let mut decision = StrategyDecision::new();

        if diff_value > Decimal::ZERO {
            // Need to buy
            if state.available_cash >= diff_value {
                debug!(
                    "Holding strategy: buying {} lots of {} at ~{}",
                    lots_u32, self.config.symbol, price
                );
                decision.add_action(StrategyAction::market_buy(&self.config.symbol, lots_u32));
                decision = decision.with_reason("Under-allocated, buying to target");
            } else {
                return StrategyDecision::hold_with_reason("Insufficient cash");
            }
        } else if self.config.allow_sell {
            // Need to sell
            let position_lots = state.position_lots();
            let lots_to_sell = lots_u32.min(position_lots);
            if lots_to_sell > 0 {
                debug!(
                    "Holding strategy: selling {} lots of {}",
                    lots_to_sell, self.config.symbol
                );
                decision.add_action(StrategyAction::market_sell(
                    &self.config.symbol,
                    lots_to_sell,
                ));
                decision = decision.with_reason("Over-allocated, selling to target");
            } else {
                return StrategyDecision::hold_with_reason("No position to sell");
            }
        } else {
            return StrategyDecision::hold_with_reason("Over-allocated but sell not allowed");
        }

        decision
    }
}

#[async_trait]
impl Strategy for HoldingStrategy {
    fn name(&self) -> &str {
        "Holding Strategy"
    }

    fn id(&self) -> &str {
        &self.config.symbol
    }

    async fn decide(&self, state: &MarketState) -> StrategyDecision {
        // For holding strategy, we need the total portfolio value to calculate target
        // This is typically passed via the state or we estimate from available info
        let total_value = state.available_cash
            + state
                .position
                .as_ref()
                .map_or(Decimal::ZERO, |p| p.market_value().amount());

        self.calculate_action(state, total_value)
    }

    async fn on_order_filled(&mut self, _order: &Order) {
        // Holding strategy doesn't track individual fills
    }

    async fn on_order_cancelled(&mut self, _order: &Order) {
        // Holding strategy doesn't track individual cancellations
    }

    async fn reset(&mut self) {
        // Holding strategy is stateless, nothing to reset
    }

    fn symbols(&self) -> Vec<String> {
        vec![self.config.symbol.clone()]
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_holding_config_with_percent() {
        let config = HoldingConfig::with_percent("AAPL", dec!(20))
            .with_tolerance(dec!(1))
            .with_allow_sell(true);

        assert_eq!(config.symbol, "AAPL");
        assert_eq!(config.target_percent, Some(dec!(20)));
        assert_eq!(config.tolerance_percent, dec!(1));
        assert!(config.allow_sell);
    }

    #[test]
    fn test_holding_config_with_absolute() {
        let config = HoldingConfig::with_absolute("GOOGL", dec!(10000));

        assert_eq!(config.symbol, "GOOGL");
        assert_eq!(config.absolute_value, Some(dec!(10000)));
        assert!(config.target_percent.is_none());
    }

    #[test]
    fn test_calculate_target_value_percent() {
        let config = HoldingConfig::with_percent("AAPL", dec!(25));
        let strategy = HoldingStrategy::new(config);

        let target = strategy.calculate_target_value(dec!(100000));
        assert_eq!(target, dec!(25000));
    }

    #[test]
    fn test_calculate_target_value_absolute() {
        let config = HoldingConfig::with_absolute("AAPL", dec!(15000));
        let strategy = HoldingStrategy::new(config);

        let target = strategy.calculate_target_value(dec!(100000));
        assert_eq!(target, dec!(15000));
    }

    #[tokio::test]
    async fn test_strategy_symbols() {
        let config = HoldingConfig::with_percent("MSFT", dec!(10));
        let strategy = HoldingStrategy::new(config);

        assert_eq!(strategy.symbols(), vec!["MSFT".to_string()]);
    }

    #[tokio::test]
    async fn test_strategy_disabled() {
        let config = HoldingConfig::with_percent("AAPL", dec!(10)).with_enabled(false);
        let strategy = HoldingStrategy::new(config);

        let state = MarketState::new("AAPL", "USD").with_cash(dec!(10000));
        let decision = strategy.decide(&state).await;

        assert!(decision.is_hold());
        assert_eq!(decision.reason(), Some("Strategy disabled"));
    }
}
