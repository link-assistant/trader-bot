//! Market simulator for testing trading strategies.
//!
//! This module provides a simulated exchange that can be used for:
//! - Unit testing the balancer logic
//! - Backtesting trading strategies
//! - Integration testing without real API calls

mod exchange;
mod market;
mod scenario;

pub use exchange::SimulatedExchange;
pub use market::{MarketState, PriceGenerator, PriceModel};
pub use scenario::{Scenario, ScenarioBuilder, ScenarioResult};
