//! Exchange adapters for connecting to real trading APIs.
//!
//! This module provides adapters for various exchanges and brokers:
//! - T-Bank (formerly Tinkoff) - Russian broker
//! - Binance - Cryptocurrency exchange
//! - Interactive Brokers - International broker
//!
//! Each adapter implements the `ExchangeProvider` trait for unified interaction.
//!
//! # Note
//!
//! These adapters are placeholder implementations. To use them with real APIs,
//! you'll need to:
//! 1. Add the appropriate SDK dependencies to `Cargo.toml`
//! 2. Implement the `connect()` method with proper authentication
//! 3. Implement market data and order execution methods

pub mod binance;
pub mod interactive_brokers;
pub mod tbank;

pub use binance::BinanceAdapter;
pub use interactive_brokers::InteractiveBrokersAdapter;
pub use tbank::TBankAdapter;
