//! lino-arguments - A unified configuration library for Rust.
//!
//! This module provides a unified configuration system combining environment variables
//! and CLI arguments with a clear priority chain.
//!
//! # Priority (highest to lowest)
//!
//! 1. CLI arguments (manually entered options)
//! 2. Environment variables (case-insensitive lookup)
//! 3. Configuration file (lenv file specified via CLI)
//! 4. Default values
//!
//! # Example
//!
//! ```rust,no_run
//! use trader_bot::lino_args::{getenv, getenv_int, getenv_bool};
//!
//! // Get string value with default
//! let api_key = getenv("API_KEY", "default_key");
//!
//! // Get integer with default
//! let port = getenv_int("PORT", 3000);
//!
//! // Get boolean with default
//! let debug = getenv_bool("DEBUG", false);
//! ```

mod case;
mod env;
mod lenv;

pub use case::*;
pub use env::*;
pub use lenv::*;

#[cfg(test)]
mod tests;
