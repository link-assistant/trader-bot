//! Environment variable helpers with case-insensitive lookup.
//!
//! These functions try multiple case variations when looking up environment variables,
//! making configuration more forgiving and consistent across platforms.

use super::case::{to_camel_case, to_kebab_case, to_pascal_case, to_snake_case, to_upper_case};
use std::env;

/// Gets an environment variable value with case-insensitive lookup.
///
/// Tries multiple case variations of the key:
/// - Original key
/// - UPPER_CASE
/// - camelCase
/// - kebab-case
/// - snake_case
/// - PascalCase
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
/// * `default` - Default value if the variable is not found
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::getenv;
///
/// // Will find MY_VAR regardless of how it's set:
/// // MY_VAR=value, myVar=value, my-var=value, my_var=value, MyVar=value
/// let value = getenv("my_var", "default");
/// ```
#[must_use]
pub fn getenv(key: &str, default: &str) -> String {
    let variants = [
        key.to_string(),
        to_upper_case(key),
        to_camel_case(key),
        to_kebab_case(key),
        to_snake_case(key),
        to_pascal_case(key),
    ];

    for variant in &variants {
        if let Ok(value) = env::var(variant) {
            return value;
        }
    }

    default.to_string()
}

/// Gets an environment variable as an integer with case-insensitive lookup.
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
/// * `default` - Default value if the variable is not found or cannot be parsed
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::getenv_int;
///
/// let port = getenv_int("PORT", 3000);
/// let timeout = getenv_int("timeout_ms", 5000);
/// ```
#[must_use]
pub fn getenv_int(key: &str, default: i64) -> i64 {
    let value = getenv(key, "");
    if value.is_empty() {
        return default;
    }
    value.parse().unwrap_or(default)
}

/// Gets an environment variable as a boolean with case-insensitive lookup.
///
/// Recognizes the following values (case-insensitive):
/// - True: "true", "1", "yes", "on"
/// - False: "false", "0", "no", "off"
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
/// * `default` - Default value if the variable is not found or cannot be parsed
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::getenv_bool;
///
/// let debug = getenv_bool("DEBUG", false);
/// let verbose = getenv_bool("VERBOSE", false);
/// ```
#[must_use]
pub fn getenv_bool(key: &str, default: bool) -> bool {
    let value = getenv(key, "");
    if value.is_empty() {
        return default;
    }
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => default,
    }
}

/// Gets an environment variable as a Decimal with case-insensitive lookup.
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
/// * `default` - Default value if the variable is not found or cannot be parsed
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::getenv_decimal;
/// use rust_decimal_macros::dec;
///
/// let threshold = getenv_decimal("THRESHOLD", dec!(0.5));
/// ```
#[must_use]
pub fn getenv_decimal(key: &str, default: rust_decimal::Decimal) -> rust_decimal::Decimal {
    let value = getenv(key, "");
    if value.is_empty() {
        return default;
    }
    value.parse().unwrap_or(default)
}

/// Gets an environment variable as an unsigned 64-bit integer.
///
/// # Arguments
///
/// * `key` - The environment variable name to look up
/// * `default` - Default value if the variable is not found or cannot be parsed
#[must_use]
pub fn getenv_u64(key: &str, default: u64) -> u64 {
    let value = getenv(key, "");
    if value.is_empty() {
        return default;
    }
    value.parse().unwrap_or(default)
}
