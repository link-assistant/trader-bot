//! Trader Bot - A configurable trading bot with multi-strategy support.
//!
//! This crate provides a comprehensive trading bot framework with:
//! - Multi-exchange support through a unified abstraction layer
//! - Multiple trading strategies (balancing, scalping, holding)
//! - Recursive strategy composition (balancer can balance between sub-strategies)
//! - Multi-user, multi-account support
//! - Market simulator for backtesting and testing
//!
//! # Architecture
//!
//! The crate follows Clean Architecture principles with clear separation:
//!
//! - **domain**: Core business types (Position, Wallet, Order, Money, Trade)
//! - **exchange**: Exchange API abstraction layer (ExchangeProvider trait)
//! - **strategy**: Trading strategies (balancing, scalping, holding)
//! - **adapters**: Exchange-specific implementations (T-Bank, Binance, etc.)
//! - **simulator**: Market simulation for testing
//! - **config**: Configuration management for multi-user/multi-account setups
//!
//! # Features
//!
//! - **Portfolio Balancing**: Rebalance portfolio to target allocations
//! - **Scalping Strategy**: High-frequency buy-low sell-high trading
//! - **Holding Strategy**: Configurable asset holding with position limits
//! - **Strategy Composition**: Balance between multiple sub-strategies
//! - **Multi-Exchange**: T-Bank, Binance, Interactive Brokers support
//! - **Multi-Account**: Manage multiple accounts per user
//! - **Multi-User**: Support multiple users with isolated configurations
//! - **Market Simulator**: Full-featured simulator for backtesting
//!
//! # Example
//!
//! ```rust,no_run
//! use trader_bot::{
//!     strategy::{BalancerEngine, BalancerConfig},
//!     domain::DesiredAllocation,
//!     simulator::SimulatedExchange,
//! };
//! use rust_decimal_macros::dec;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a simulated exchange
//!     let exchange = SimulatedExchange::new("USD");
//!     exchange.add_instrument("AAPL", "Apple Inc.", dec!(150), 1);
//!     exchange.add_instrument("GOOGL", "Alphabet Inc.", dec!(2800), 1);
//!     exchange.create_account("my_account", dec!(100000));
//!
//!     // Define target allocation
//!     let mut target = DesiredAllocation::new();
//!     target.set("AAPL", dec!(50));
//!     target.set("GOOGL", dec!(50));
//!
//!     // Create and run balancer
//!     let exchange = Arc::new(exchange);
//!     let config = BalancerConfig::default();
//!     let mut engine = BalancerEngine::new(exchange, config);
//!
//!     let result = engine.rebalance("my_account", &target).await;
//!     println!("Rebalance result: {:?}", result);
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod adapters;
pub mod cli;
pub mod config;
pub mod domain;
pub mod exchange;
pub mod simulator;
pub mod strategy;

// Include lino_args from lib/rust directory
#[path = "../lib/rust/lino_args/mod.rs"]
pub mod lino_args;

// Re-export balancer module at the old path for backwards compatibility
#[doc(hidden)]
pub use strategy::balancer;

/// Package version (matches Cargo.toml version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export commonly used types at the crate root.
pub mod prelude {
    pub use crate::cli::{Cli, RuntimeConfig};
    pub use crate::config::{AccountConfig, AppConfig, UserConfig};
    pub use crate::domain::{
        DesiredAllocation, Money, Order, OrderDirection, OrderStatus, PlannedOrder, PlannedOrders,
        Position, Trade, Wallet,
    };
    pub use crate::exchange::{ExchangeError, ExchangeProvider, ExchangeResult};
    pub use crate::lino_args::{getenv, getenv_bool, getenv_decimal, getenv_int, getenv_u64};
    pub use crate::simulator::SimulatedExchange;
    pub use crate::strategy::{
        BalancerConfig, BalancerEngine, HoldingStrategy, RebalanceCalculator, RebalancePlan,
        ScalpingStrategy, Strategy, StrategyAction, StrategyDecision, TradingSettings,
    };
}
