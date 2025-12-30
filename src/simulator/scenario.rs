//! Scenario-based testing for trading strategies.

use rust_decimal::Decimal;
use std::sync::Arc;

use super::{PriceModel, SimulatedExchange};
use crate::balancer::{BalancerConfig, BalancerEngine, RebalanceResult};
use crate::domain::{DesiredAllocation, Money, Position};
use crate::exchange::ExchangeProvider;

/// Result of running a scenario.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Name of the scenario.
    pub name: String,
    /// Number of rebalancing cycles executed.
    pub cycles: usize,
    /// Total number of trades executed.
    pub total_trades: usize,
    /// Initial portfolio value.
    pub initial_value: Decimal,
    /// Final portfolio value.
    pub final_value: Decimal,
    /// Return percentage.
    pub return_percent: Decimal,
    /// Results from each rebalancing cycle.
    pub cycle_results: Vec<RebalanceResult>,
    /// Whether the scenario passed.
    pub passed: bool,
    /// Failure reason if not passed.
    pub failure_reason: Option<String>,
}

impl ScenarioResult {
    /// Creates a new scenario result.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cycles: 0,
            total_trades: 0,
            initial_value: Decimal::ZERO,
            final_value: Decimal::ZERO,
            return_percent: Decimal::ZERO,
            cycle_results: Vec::new(),
            passed: true,
            failure_reason: None,
        }
    }

    /// Marks the scenario as failed.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.passed = false;
        self.failure_reason = Some(reason.into());
    }

    /// Calculates the return percentage.
    pub fn calculate_return(&mut self) {
        if !self.initial_value.is_zero() {
            self.return_percent =
                (self.final_value - self.initial_value) / self.initial_value * Decimal::from(100);
        }
    }
}

/// A test scenario definition.
#[derive(Debug)]
pub struct Scenario {
    /// Scenario name.
    pub name: String,
    /// Initial cash balance.
    pub initial_cash: Decimal,
    /// Initial positions.
    pub initial_positions: Vec<(String, Decimal, Decimal)>, // (symbol, quantity, price)
    /// Instruments to add.
    pub instruments: Vec<(String, String, Decimal, u32)>, // (symbol, name, price, lot_size)
    /// Target allocation.
    pub target_allocation: DesiredAllocation,
    /// Balancer configuration.
    pub balancer_config: BalancerConfig,
    /// Price models for instruments.
    pub price_models: Vec<(String, PriceModel)>,
    /// Number of ticks to simulate.
    pub ticks: usize,
    /// Rebalance every N ticks.
    pub rebalance_interval: usize,
    /// Expected conditions to verify.
    pub assertions: Vec<ScenarioAssertion>,
}

/// Assertion to verify after running a scenario.
pub enum ScenarioAssertion {
    /// Portfolio value should be at least this amount.
    MinValue(Decimal),
    /// Portfolio value should be at most this amount.
    MaxValue(Decimal),
    /// Return should be at least this percentage.
    MinReturn(Decimal),
    /// Total trades should be at most this number.
    MaxTrades(usize),
    /// Final allocation should be within tolerance of target.
    AllocationWithinTolerance(Decimal),
    /// Custom assertion with description and check function.
    Custom(String, Arc<dyn Fn(&ScenarioResult) -> bool + Send + Sync>),
}

impl std::fmt::Debug for ScenarioAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinValue(v) => f.debug_tuple("MinValue").field(v).finish(),
            Self::MaxValue(v) => f.debug_tuple("MaxValue").field(v).finish(),
            Self::MinReturn(v) => f.debug_tuple("MinReturn").field(v).finish(),
            Self::MaxTrades(v) => f.debug_tuple("MaxTrades").field(v).finish(),
            Self::AllocationWithinTolerance(v) => {
                f.debug_tuple("AllocationWithinTolerance").field(v).finish()
            }
            Self::Custom(desc, _) => f.debug_tuple("Custom").field(desc).finish(),
        }
    }
}

impl Clone for ScenarioAssertion {
    fn clone(&self) -> Self {
        match self {
            Self::MinValue(v) => Self::MinValue(*v),
            Self::MaxValue(v) => Self::MaxValue(*v),
            Self::MinReturn(v) => Self::MinReturn(*v),
            Self::MaxTrades(v) => Self::MaxTrades(*v),
            Self::AllocationWithinTolerance(v) => Self::AllocationWithinTolerance(*v),
            Self::Custom(desc, f) => Self::Custom(desc.clone(), Arc::clone(f)),
        }
    }
}

impl ScenarioAssertion {
    /// Checks if the assertion passes.
    pub fn check(&self, result: &ScenarioResult) -> bool {
        match self {
            Self::MinValue(min) => result.final_value >= *min,
            Self::MaxValue(max) => result.final_value <= *max,
            Self::MinReturn(min) => result.return_percent >= *min,
            Self::MaxTrades(max) => result.total_trades <= *max,
            Self::AllocationWithinTolerance(_) => true, // Checked separately
            Self::Custom(_, check) => check(result),
        }
    }

    /// Returns a description of the assertion.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::MinValue(v) => format!("Portfolio value >= {v}"),
            Self::MaxValue(v) => format!("Portfolio value <= {v}"),
            Self::MinReturn(r) => format!("Return >= {r}%"),
            Self::MaxTrades(n) => format!("Total trades <= {n}"),
            Self::AllocationWithinTolerance(t) => format!("Allocation within {t}% of target"),
            Self::Custom(desc, _) => desc.clone(),
        }
    }
}

/// Builder for creating scenarios.
pub struct ScenarioBuilder {
    scenario: Scenario,
}

impl ScenarioBuilder {
    /// Creates a new scenario builder.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            scenario: Scenario {
                name: name.into(),
                initial_cash: Decimal::new(10000, 0),
                initial_positions: Vec::new(),
                instruments: Vec::new(),
                target_allocation: DesiredAllocation::new(),
                balancer_config: BalancerConfig::default(),
                price_models: Vec::new(),
                ticks: 100,
                rebalance_interval: 10,
                assertions: Vec::new(),
            },
        }
    }

    /// Sets the initial cash balance.
    #[must_use]
    pub const fn with_cash(mut self, cash: Decimal) -> Self {
        self.scenario.initial_cash = cash;
        self
    }

    /// Adds an instrument.
    #[must_use]
    pub fn with_instrument(
        mut self,
        symbol: impl Into<String>,
        name: impl Into<String>,
        price: Decimal,
        lot_size: u32,
    ) -> Self {
        self.scenario
            .instruments
            .push((symbol.into(), name.into(), price, lot_size));
        self
    }

    /// Adds an initial position.
    #[must_use]
    pub fn with_position(
        mut self,
        symbol: impl Into<String>,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        self.scenario
            .initial_positions
            .push((symbol.into(), quantity, price));
        self
    }

    /// Sets a target allocation percentage.
    #[must_use]
    pub fn with_target(mut self, symbol: impl Into<String>, percent: Decimal) -> Self {
        self.scenario.target_allocation.set(symbol, percent);
        self
    }

    /// Sets the price model for an instrument.
    #[must_use]
    pub fn with_price_model(mut self, symbol: impl Into<String>, model: PriceModel) -> Self {
        self.scenario.price_models.push((symbol.into(), model));
        self
    }

    /// Sets the number of ticks to simulate.
    #[must_use]
    pub const fn with_ticks(mut self, ticks: usize) -> Self {
        self.scenario.ticks = ticks;
        self
    }

    /// Sets the rebalance interval.
    #[must_use]
    pub const fn with_rebalance_interval(mut self, interval: usize) -> Self {
        self.scenario.rebalance_interval = interval;
        self
    }

    /// Sets the balancer to dry run mode.
    #[must_use]
    pub fn dry_run(mut self) -> Self {
        self.scenario.balancer_config.dry_run = true;
        self
    }

    /// Adds an assertion.
    #[must_use]
    pub fn assert(mut self, assertion: ScenarioAssertion) -> Self {
        self.scenario.assertions.push(assertion);
        self
    }

    /// Asserts minimum portfolio value.
    #[must_use]
    pub fn assert_min_value(self, value: Decimal) -> Self {
        self.assert(ScenarioAssertion::MinValue(value))
    }

    /// Asserts minimum return percentage.
    #[must_use]
    pub fn assert_min_return(self, percent: Decimal) -> Self {
        self.assert(ScenarioAssertion::MinReturn(percent))
    }

    /// Asserts maximum number of trades.
    #[must_use]
    pub fn assert_max_trades(self, max: usize) -> Self {
        self.assert(ScenarioAssertion::MaxTrades(max))
    }

    /// Builds the scenario.
    #[must_use]
    pub fn build(self) -> Scenario {
        self.scenario
    }
}

impl Scenario {
    /// Runs the scenario and returns the result.
    pub async fn run(&self) -> ScenarioResult {
        let mut result = ScenarioResult::new(&self.name);
        let account_id = "scenario_account";
        let currency = "USD";

        // Set up exchange
        let exchange = SimulatedExchange::new(currency);

        // Add instruments
        for (symbol, name, price, lot_size) in &self.instruments {
            exchange.add_instrument(symbol, name, *price, *lot_size);
        }

        // Set price models
        {
            let market = exchange.market();
            let mut market_guard = market.write().unwrap();
            for (symbol, model) in &self.price_models {
                market_guard.set_price_model(symbol, *model);
            }
        }

        // Create account
        exchange.create_account(account_id, self.initial_cash);

        // Add initial positions
        for (symbol, quantity, price) in &self.initial_positions {
            let instrument = exchange
                .market()
                .read()
                .unwrap()
                .get_instrument(symbol)
                .cloned();
            if let Some(inst) = instrument {
                let position = Position::builder(symbol, "SIMULATOR")
                    .quantity(*quantity)
                    .lot_size(inst.lot_size)
                    .current_price(Money::new(*price, currency))
                    .average_cost(Money::new(*price, currency))
                    .build();
                exchange.add_position(account_id, position);
            }
        }

        // Get initial value
        let initial_wallet = match exchange.get_wallet(account_id).await {
            Ok(w) => w,
            Err(e) => {
                result.fail(format!("Failed to get initial wallet: {e}"));
                return result;
            }
        };
        result.initial_value = initial_wallet.total_value().amount();

        // Create balancer
        let exchange = Arc::new(exchange);
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), self.balancer_config.clone());

        // Run simulation
        for tick in 0..self.ticks {
            // Update market prices
            exchange.tick();

            // Rebalance at intervals
            if tick > 0 && tick % self.rebalance_interval == 0 {
                match engine.rebalance(account_id, &self.target_allocation).await {
                    Ok(rebalance_result) => {
                        result.total_trades += rebalance_result.executed.len();
                        result.cycle_results.push(rebalance_result);
                        result.cycles += 1;
                    }
                    Err(e) => {
                        result.fail(format!("Rebalance failed at tick {tick}: {e}"));
                        return result;
                    }
                }
            }
        }

        // Get final value
        let final_wallet = match exchange.get_wallet(account_id).await {
            Ok(w) => w,
            Err(e) => {
                result.fail(format!("Failed to get final wallet: {e}"));
                return result;
            }
        };
        result.final_value = final_wallet.total_value().amount();
        result.calculate_return();

        // Check assertions
        for assertion in &self.assertions {
            if !assertion.check(&result) {
                result.fail(format!("Assertion failed: {}", assertion.description()));
                return result;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_simple_scenario() {
        let scenario = ScenarioBuilder::new("Simple rebalance test")
            .with_cash(dec!(10000))
            .with_instrument("A", "Stock A", dec!(100), 1)
            .with_instrument("B", "Stock B", dec!(100), 1)
            .with_target("A", dec!(50))
            .with_target("B", dec!(50))
            .with_ticks(20)
            .with_rebalance_interval(5)
            .assert_min_value(dec!(9000)) // Allow for some trading costs
            .build();

        let result = scenario.run().await;
        assert!(
            result.passed,
            "Scenario failed: {:?}",
            result.failure_reason
        );
        assert!(result.cycles > 0);
    }

    #[tokio::test]
    async fn test_scenario_with_initial_positions() {
        let scenario = ScenarioBuilder::new("Rebalance with existing positions")
            .with_cash(dec!(5000))
            .with_instrument("A", "Stock A", dec!(100), 1)
            .with_instrument("B", "Stock B", dec!(100), 1)
            .with_position("A", dec!(30), dec!(100)) // 30 shares @ 100 = 3000
            .with_target("A", dec!(40)) // Want 40%
            .with_target("B", dec!(40)) // Want 40%
            // Total = 5000 cash + 3000 A = 8000
            // 40% of 8000 = 3200 per asset
            .with_ticks(10)
            .with_rebalance_interval(5)
            .build();

        let result = scenario.run().await;
        assert!(
            result.passed,
            "Scenario failed: {:?}",
            result.failure_reason
        );
    }

    #[tokio::test]
    async fn test_dry_run_scenario() {
        let scenario = ScenarioBuilder::new("Dry run test")
            .with_cash(dec!(10000))
            .with_instrument("A", "Stock A", dec!(100), 1)
            .with_target("A", dec!(50))
            .with_ticks(20)
            .with_rebalance_interval(5)
            .dry_run()
            .build();

        let result = scenario.run().await;
        assert!(result.passed);
        assert_eq!(result.total_trades, 0); // No actual trades in dry run
    }
}
