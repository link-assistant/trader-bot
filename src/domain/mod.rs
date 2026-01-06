//! Domain types for the trader bot.
//!
//! This module contains core domain models that represent trading concepts
//! independent of any specific exchange or broker API.
//!
//! # Allocation System
//!
//! The allocation module provides a recursive allocation system that supports:
//! - Simple ticker-based allocations (e.g., SBER 30%, LKOH 30%, GAZP 40%)
//! - Strategy-based allocations (e.g., 50% balancer, 30% scalper, 20% hold)
//! - Nested/recursive allocations (balancers containing other balancers)
//!
//! # User/Account Model
//!
//! Users represent connections to exchanges/brokers and can have multiple accounts.
//! Each account has its own allocation configuration.

mod allocation;
mod decimal;
mod money;
mod order;
mod plan;
mod position;
mod trade;
mod user;
mod wallet;

pub use allocation::{Allocation, AllocationPortion, StrategyType};
pub use decimal::Decimal;
pub use money::Money;
pub use order::{Order, OrderDirection, OrderStatus, OrderType};
pub use plan::{PlannedOrder, PlannedOrders};
pub use position::Position;
pub use trade::{Trade, TradeHistory, TradeId};
pub use user::{Account, BrokerType, ConfigValidationError, Settings, TradingConfig, User};
pub use wallet::{DesiredAllocation, Wallet};
