//! Exchange provider abstraction layer.
//!
//! This module defines the traits and types for interacting with various
//! exchanges and brokers. The abstraction allows supporting multiple
//! exchanges (T-Bank, Binance, etc.) with a unified interface.

mod error;
mod provider;
mod types;

pub use error::{ExchangeError, ExchangeResult};
pub use provider::ExchangeProvider;
pub use types::{
    ExchangeInfo, Instrument, InstrumentType, MarketData, MarketStatus, OrderBook, OrderBookEntry,
    Ticker,
};
