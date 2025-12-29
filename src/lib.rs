//! Balancer Trader Bot - A portfolio balancer and trading bot.
//!
//! This crate provides tools for automated portfolio rebalancing across
//! multiple exchanges and brokers.
//!
//! # Architecture
//!
//! The crate follows Clean Architecture principles with clear separation:
//!
//! - **domain**: Core business types (Position, Wallet, Order, Money)
//! - **exchange**: Exchange API abstraction layer (ExchangeProvider trait)
//! - **balancer**: Portfolio rebalancing logic and calculations
//! - **simulator**: Market simulation for testing
//! - **config**: Configuration management
//!
//! # Features
//!
//! - Multiple rebalancing strategies (manual, market cap, AUM, decorrelation)
//! - Exchange-agnostic design supporting multiple brokers
//! - Built-in market simulator for testing and backtesting
//! - Comprehensive test coverage with unit, integration, and e2e tests
//!
//! # Example
//!
//! ```rust,no_run
//! use balancer_trader_bot::{
//!     balancer::{BalancerConfig, BalancerEngine},
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

pub mod balancer;
pub mod config;
pub mod domain;
pub mod exchange;
pub mod simulator;

/// Package version (matches Cargo.toml version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export commonly used types at the crate root.
pub mod prelude {
    pub use crate::balancer::{BalancerConfig, BalancerEngine, RebalanceCalculator, RebalancePlan};
    pub use crate::config::{AccountConfig, AppConfig};
    pub use crate::domain::{
        DesiredAllocation, Money, Order, OrderDirection, OrderStatus, Position, Wallet,
    };
    pub use crate::exchange::{ExchangeError, ExchangeProvider, ExchangeResult};
    pub use crate::simulator::SimulatedExchange;
}
