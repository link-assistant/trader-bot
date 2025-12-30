//! Portfolio balancer implementation.
//!
//! This module contains the core rebalancing logic that calculates what trades
//! need to be made to bring a portfolio in line with target allocations.

mod calculator;
mod engine;
mod rebalance;

pub use calculator::{AllocationDiff, RebalanceCalculator};
pub use engine::{BalancerConfig, BalancerEngine, RebalanceResult};
pub use rebalance::{RebalanceAction, RebalancePlan};
