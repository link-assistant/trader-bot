//! lino-arguments - A unified configuration library for Rust.
//!
//! This module provides a unified configuration system combining environment variables
//! and CLI arguments with a clear priority chain, using Links Notation format.
//!
//! # Priority (highest to lowest)
//!
//! 1. CLI arguments (manually entered options)
//! 2. Environment variables (case-insensitive lookup)
//! 3. Configuration file (lenv file specified via CLI)
//! 4. Default values
//!
//! # Links Notation
//!
//! This module supports [Links Notation](https://github.com/link-foundation/links-notation)
//! for hierarchical configuration files. The indented syntax allows for complex nested
//! structures with intuitive readability.
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
//!
//! ## Hierarchical Configuration
//!
//! ```rust,no_run
//! use trader_bot::lino_args::lino::parse_lino;
//!
//! let content = r#"
//! settings
//!   log_level info
//!   verbose false
//!
//! users
//!   (
//!     broker tbank
//!     accounts
//!       main
//!         allocation
//!           SBER 30
//!           LKOH 70
//!   )
//! "#;
//!
//! let config = parse_lino(content).unwrap();
//! assert_eq!(config.get_str("settings/log_level"), Some("info"));
//! ```

mod case;
mod env;
mod lenv;
pub mod lino;

pub use case::*;
pub use env::*;
pub use lenv::*;

#[cfg(test)]
mod tests;
