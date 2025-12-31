//! Trading strategies module.
//!
//! This module provides trading strategy implementations and abstractions
//! for building configurable trading systems.
//!
//! # Available Strategies
//!
//! - **Balancer**: Portfolio rebalancing strategy that maintains target allocations
//! - **Scalping**: High-frequency buy-low sell-high trading strategy
//! - **Holding**: Passive strategy that maintains a target allocation for a single asset
//!
//! # Strategy Composition
//!
//! The balancer strategy can be configured to balance between multiple sub-strategies,
//! enabling complex trading setups like:
//! - 50% in a holding strategy for BTC
//! - 30% in a scalping strategy for ETH
//! - 20% in another balancer that manages a stock portfolio

pub mod balancer;
mod holding;
mod scalper;
mod settings;
mod traits;

// Re-export strategy types
pub use balancer::{
    AllocationDiff, BalancerConfig, BalancerEngine, RebalanceAction, RebalanceCalculator,
    RebalancePlan, RebalancePlanBuilder, RebalanceResult,
};
pub use holding::{HoldingConfig, HoldingStrategy};
pub use scalper::ScalpingStrategy;
pub use settings::TradingSettings;
pub use traits::{MarketState, Strategy, StrategyAction, StrategyDecision, StrategyExecutor};
