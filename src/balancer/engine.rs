//! Balancer engine that orchestrates the rebalancing process.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rust_decimal::Decimal;
use tracing::{debug, info, warn};

use super::{AllocationDiff, RebalanceAction, RebalanceCalculator, RebalancePlan};
use crate::domain::{DesiredAllocation, OrderDirection, OrderStatus, Wallet};
use crate::exchange::{ExchangeError, ExchangeProvider, ExchangeResult, Instrument};

/// Configuration for the balancer engine.
#[derive(Debug, Clone)]
pub struct BalancerConfig {
    /// Minimum trade size as percentage of portfolio.
    pub min_trade_percent: Decimal,
    /// Minimum trade value in base currency.
    pub min_trade_value: Decimal,
    /// Tolerance for considering allocation "close enough".
    pub tolerance_percent: Decimal,
    /// Delay between consecutive orders (milliseconds).
    pub order_delay_ms: u64,
    /// Whether to actually execute trades or just simulate.
    pub dry_run: bool,
    /// Maximum number of orders per rebalancing cycle.
    pub max_orders_per_cycle: usize,
    /// Minimum profit percentage required for selling a position.
    pub min_profit_percent_for_close: Option<Decimal>,
}

impl Default for BalancerConfig {
    fn default() -> Self {
        Self {
            min_trade_percent: Decimal::new(1, 1), // 0.1%
            min_trade_value: Decimal::new(100, 0), // 100 units
            tolerance_percent: Decimal::new(5, 1), // 0.5%
            order_delay_ms: 1000,                  // 1 second
            dry_run: false,
            max_orders_per_cycle: 20,
            min_profit_percent_for_close: None,
        }
    }
}

/// Result of a single rebalancing cycle.
#[derive(Debug, Clone)]
pub struct RebalanceResult {
    /// The plan that was created.
    pub plan: RebalancePlan,
    /// Actions that were successfully executed.
    pub executed: Vec<RebalanceAction>,
    /// Actions that failed.
    pub failed: Vec<(RebalanceAction, String)>,
    /// Whether the cycle completed successfully.
    pub success: bool,
    /// Error message if the cycle failed.
    pub error: Option<String>,
}

impl RebalanceResult {
    /// Creates a successful result.
    #[must_use]
    pub fn success(plan: RebalancePlan, executed: Vec<RebalanceAction>) -> Self {
        Self {
            plan,
            executed,
            failed: Vec::new(),
            success: true,
            error: None,
        }
    }

    /// Creates a failed result.
    #[must_use]
    pub fn failure(plan: RebalancePlan, error: impl Into<String>) -> Self {
        Self {
            plan,
            executed: Vec::new(),
            failed: Vec::new(),
            success: false,
            error: Some(error.into()),
        }
    }
}

/// The main balancer engine.
pub struct BalancerEngine<E: ExchangeProvider> {
    exchange: Arc<E>,
    config: BalancerConfig,
    calculator: RebalanceCalculator,
    instrument_cache: HashMap<String, Instrument>,
}

impl<E: ExchangeProvider> BalancerEngine<E> {
    /// Creates a new balancer engine.
    #[must_use]
    pub fn new(exchange: Arc<E>, config: BalancerConfig) -> Self {
        let calculator = RebalanceCalculator::new()
            .with_min_trade_percent(config.min_trade_percent)
            .with_min_trade_value(config.min_trade_value)
            .with_tolerance_percent(config.tolerance_percent);

        Self {
            exchange,
            config,
            calculator,
            instrument_cache: HashMap::new(),
        }
    }

    /// Returns the current configuration.
    #[must_use]
    pub const fn config(&self) -> &BalancerConfig {
        &self.config
    }

    /// Updates the configuration.
    pub fn set_config(&mut self, config: BalancerConfig) {
        self.calculator = RebalanceCalculator::new()
            .with_min_trade_percent(config.min_trade_percent)
            .with_min_trade_value(config.min_trade_value)
            .with_tolerance_percent(config.tolerance_percent);
        self.config = config;
    }

    /// Clears the instrument cache.
    pub fn clear_cache(&mut self) {
        self.instrument_cache.clear();
    }

    /// Gets an instrument, using cache if available.
    async fn get_instrument(&mut self, symbol: &str) -> ExchangeResult<Instrument> {
        if let Some(instrument) = self.instrument_cache.get(symbol) {
            return Ok(instrument.clone());
        }

        let instrument = self.exchange.get_instrument(symbol).await?;
        self.instrument_cache
            .insert(symbol.to_string(), instrument.clone());
        Ok(instrument)
    }

    /// Creates a rebalancing plan without executing it.
    pub async fn create_plan(
        &mut self,
        account_id: &str,
        desired: &DesiredAllocation,
    ) -> ExchangeResult<RebalancePlan> {
        info!("Creating rebalance plan for account {}", account_id);

        // Get current wallet state
        let wallet = self.exchange.get_wallet(account_id).await?;
        let currency = wallet.base_currency().to_string();

        // Check if already balanced
        if self.calculator.is_balanced(&wallet, desired) {
            debug!("Portfolio already balanced within tolerance");
            return Ok(RebalancePlan::empty_with_reason(
                &currency,
                "Portfolio already balanced",
            ));
        }

        // Calculate differences
        let diffs = self.calculator.calculate_diffs(&wallet, desired);
        let significant = self.calculator.filter_significant_diffs(diffs);
        let sorted = self.calculator.sort_by_priority(significant);

        if sorted.is_empty() {
            return Ok(RebalancePlan::empty_with_reason(
                &currency,
                "No significant rebalancing needed",
            ));
        }

        // Build the plan
        let mut plan = RebalancePlan::new(&currency);
        plan.set_dry_run(self.config.dry_run);

        for (i, diff) in sorted.iter().enumerate() {
            if i >= self.config.max_orders_per_cycle {
                warn!(
                    "Reached max orders per cycle ({}), truncating plan",
                    self.config.max_orders_per_cycle
                );
                break;
            }

            if let Some(action) = self.diff_to_action(&wallet, diff).await? {
                plan.add_action(action);
            }
        }

        info!(
            "Created plan with {} actions: {}",
            plan.trade_count,
            plan.summary()
        );

        Ok(plan)
    }

    /// Converts an allocation diff to a rebalance action.
    async fn diff_to_action(
        &mut self,
        wallet: &Wallet,
        diff: &AllocationDiff,
    ) -> ExchangeResult<Option<RebalanceAction>> {
        let position = wallet.get_position(&diff.symbol);

        // Get instrument info
        let instrument = match self.get_instrument(&diff.symbol).await {
            Ok(i) => i,
            Err(ExchangeError::InstrumentNotFound(_)) => {
                warn!("Instrument {} not found, skipping", diff.symbol);
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        // Get current market data
        let market_data = self.exchange.get_market_data(&diff.symbol).await?;

        // Check if market is open
        if !market_data.market_status.is_tradeable() {
            debug!("Market closed for {}, skipping", diff.symbol);
            return Ok(None);
        }

        let price_per_unit = market_data.last_price.clone();
        let lot_size = instrument.lot_size;
        let price_per_lot = price_per_unit.multiply(Decimal::from(lot_size));

        let direction = if diff.needs_buy() {
            OrderDirection::Buy
        } else {
            OrderDirection::Sell
        };

        // Calculate lots
        let lots =
            self.calculator
                .calculate_lots(diff.diff_value, price_per_lot.amount(), lot_size);

        if lots == 0 {
            debug!("Trade for {} would be 0 lots, skipping", diff.symbol);
            return Ok(None);
        }

        // Check min profit requirement for sells
        if direction == OrderDirection::Sell {
            if let Some(min_profit) = self.config.min_profit_percent_for_close {
                if let Some(pos) = position {
                    if let Some(pnl_pct) = pos.unrealized_pnl_percent() {
                        if pnl_pct < min_profit {
                            debug!(
                                "Skipping sell of {} due to min profit requirement ({:.2}% < {:.2}%)",
                                diff.symbol, pnl_pct, min_profit
                            );
                            return Ok(None);
                        }
                    }
                }
            }

            // Check if we have enough tradeable lots
            if let Some(pos) = position {
                let available = pos.tradeable_lots();
                if lots > available {
                    debug!(
                        "Reducing sell lots for {} from {} to {} (available)",
                        diff.symbol, lots, available
                    );
                    if available == 0 {
                        return Ok(None);
                    }
                }
            }
        }

        let estimated_value = price_per_lot.multiply(Decimal::from(lots));

        Ok(Some(RebalanceAction {
            symbol: diff.symbol.clone(),
            exchange: instrument.exchange.clone(),
            direction,
            lots,
            estimated_price: price_per_lot,
            estimated_value,
            current_percent: diff.current_percent,
            target_percent: diff.desired_percent,
            priority: 0, // Will be set by plan builder
        }))
    }

    /// Executes a rebalancing plan.
    pub async fn execute_plan(
        &self,
        account_id: &str,
        plan: &RebalancePlan,
    ) -> ExchangeResult<RebalanceResult> {
        if plan.is_empty() {
            return Ok(RebalanceResult::success(plan.clone(), Vec::new()));
        }

        if plan.dry_run {
            info!("Dry run mode - not executing {} trades", plan.trade_count);
            return Ok(RebalanceResult::success(plan.clone(), Vec::new()));
        }

        info!("Executing {} trades", plan.trade_count);

        let mut executed = Vec::new();
        let mut failed = Vec::new();

        for (i, action) in plan.actions.iter().enumerate() {
            if i > 0 && self.config.order_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.order_delay_ms)).await;
            }

            match self.execute_action(account_id, action).await {
                Ok(_) => {
                    info!(
                        "Executed {} {} lots of {}",
                        if action.is_buy() { "buy" } else { "sell" },
                        action.lots,
                        action.symbol
                    );
                    executed.push(action.clone());
                }
                Err(e) => {
                    warn!(
                        "Failed to execute {} for {}: {}",
                        if action.is_buy() { "buy" } else { "sell" },
                        action.symbol,
                        e
                    );
                    failed.push((action.clone(), e.to_string()));
                }
            }
        }

        let mut result = RebalanceResult::success(plan.clone(), executed);
        result.failed = failed;

        if !result.failed.is_empty() {
            result.success = false;
            result.error = Some(format!(
                "{} of {} trades failed",
                result.failed.len(),
                plan.trade_count
            ));
        }

        Ok(result)
    }

    /// Executes a single rebalance action.
    async fn execute_action(
        &self,
        account_id: &str,
        action: &RebalanceAction,
    ) -> ExchangeResult<()> {
        let order = match action.direction {
            OrderDirection::Buy => {
                self.exchange
                    .market_buy(account_id, &action.symbol, action.lots)
                    .await?
            }
            OrderDirection::Sell => {
                self.exchange
                    .market_sell(account_id, &action.symbol, action.lots)
                    .await?
            }
        };

        // Check order status
        match order.status() {
            OrderStatus::Filled | OrderStatus::PartiallyFilled | OrderStatus::Submitted => Ok(()),
            OrderStatus::Rejected => Err(ExchangeError::OrderRejected(
                order.message().unwrap_or("Unknown reason").to_string(),
            )),
            status => Err(ExchangeError::UnexpectedResponse(format!(
                "Unexpected order status: {status:?}"
            ))),
        }
    }

    /// Runs a complete rebalancing cycle.
    pub async fn rebalance(
        &mut self,
        account_id: &str,
        desired: &DesiredAllocation,
    ) -> ExchangeResult<RebalanceResult> {
        let plan = self.create_plan(account_id, desired).await?;
        self.execute_plan(account_id, &plan).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balancer_config_default() {
        let config = BalancerConfig::default();
        assert!(!config.dry_run);
        assert_eq!(config.order_delay_ms, 1000);
        assert_eq!(config.max_orders_per_cycle, 20);
    }

    #[test]
    fn test_rebalance_result_success() {
        let plan = RebalancePlan::new("USD");
        let result = RebalanceResult::success(plan, Vec::new());
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_rebalance_result_failure() {
        let plan = RebalancePlan::new("USD");
        let result = RebalanceResult::failure(plan, "Test error");
        assert!(!result.success);
        assert_eq!(result.error, Some("Test error".into()));
    }
}
