//! Portfolio balancer implementation.
//!
//! This module contains the core rebalancing logic that calculates what trades
//! need to be made to bring a portfolio in line with target allocations.
//!
//! # Strategy Composition
//!
//! The balancer can now balance between different sub-strategies, not just
//! individual assets. For example:
//! - Balance 50% to a HoldingStrategy for BTC
//! - Balance 30% to a ScalpingStrategy for ETH
//! - Balance 20% to another BalancerEngine managing stocks

mod calculator;
mod engine;
mod rebalance;

pub use calculator::{AllocationDiff, RebalanceCalculator};
pub use engine::{BalancerConfig, BalancerEngine, RebalanceResult};
pub use rebalance::{RebalanceAction, RebalancePlan, RebalancePlanBuilder};
