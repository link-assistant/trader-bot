# lino-arguments (Rust)

A unified configuration library for Rust applications that automatically loads configuration from multiple sources with a clear priority chain.

This is a temporary copy of the [lino-arguments](https://github.com/link-foundation/lino-arguments) library, specifically the Rust implementation, integrated into the trader-bot project.

## Features

- **Multi-source configuration** with clear priority hierarchy
- **Case-insensitive environment variable lookup** supporting multiple naming conventions
- **Lenv file support** for `.lenv` configuration files (similar to `.env`)
- **Case conversion utilities** for flexible variable naming
- **Type-safe environment variable access** with built-in parsing

## Priority Chain

Configuration values are resolved in the following order (highest to lowest):

1. **CLI arguments** - Explicitly provided command-line options
2. **Environment variables** - System environment variables (case-insensitive)
3. **Configuration files** - `.lenv` files or JSON config files
4. **Default values** - Fallback values defined in code

## Installation

This module is part of the `trader-bot` project. To use it in your Rust code:

```rust
use trader_bot::lino_args::{getenv, getenv_int, getenv_bool};
```

## Usage

### Environment Variable Helpers

The library provides case-insensitive environment variable access:

```rust
use trader_bot::lino_args::{getenv, getenv_int, getenv_bool, getenv_decimal};
use rust_decimal_macros::dec;

// Get string value with default
let api_key = getenv("API_KEY", "default_key");

// Get integer with default
let port = getenv_int("PORT", 3000);

// Get boolean with default (accepts: true/false, 1/0, yes/no, on/off)
let debug = getenv_bool("DEBUG", false);

// Get decimal with default
let threshold = getenv_decimal("THRESHOLD", dec!(0.5));

// Get unsigned 64-bit integer
let max_orders = getenv_u64("MAX_ORDERS", 100);
```

#### Case-Insensitive Lookup

Environment variables are looked up using multiple case conventions:

```rust
// All of these will find the same variable:
// API_KEY, apiKey, api-key, api_key, ApiKey
let key = getenv("api_key", "default");
```

The lookup tries these variations in order:
1. Original key (as provided)
2. UPPER_CASE (screaming snake case)
3. camelCase
4. kebab-case
5. snake_case
6. PascalCase

### Case Conversion Utilities

Convert between different naming conventions:

```rust
use trader_bot::lino_args::{
    to_upper_case, to_camel_case, to_kebab_case,
    to_snake_case, to_pascal_case
};

// Convert to UPPER_CASE
assert_eq!(to_upper_case("apiKey"), "API_KEY");

// Convert to camelCase
assert_eq!(to_camel_case("API_KEY"), "apiKey");

// Convert to kebab-case
assert_eq!(to_kebab_case("apiKey"), "api-key");

// Convert to snake_case
assert_eq!(to_snake_case("apiKey"), "api_key");

// Convert to PascalCase
assert_eq!(to_pascal_case("api_key"), "ApiKey");
```

### Lenv File Support

Load configuration from `.lenv` files:

```rust
use trader_bot::lino_args::{load_lenv, load_lenv_into_env, auto_load_lenv};
use std::collections::HashMap;

// Parse lenv file into HashMap
let vars: HashMap<String, String> = load_lenv(".lenv").unwrap();

// Load lenv file into environment variables
// (doesn't overwrite existing env vars)
let loaded_count = load_lenv_into_env(".lenv").unwrap();

// Auto-load from common locations
// Searches: --lenv arg, LENV_FILE env var, ./.lenv, ~/.lenv
if let Ok(Some(count)) = auto_load_lenv() {
    println!("Loaded {} variables from lenv file", count);
}
```

#### Lenv File Format

The `.lenv` file format supports:

```sh
# Comments start with #

# Simple key-value pairs
API_KEY=my_secret_key
PORT=8080

# Quoted values (for values with spaces)
APP_NAME="My Trading Bot"
DESCRIPTION='A bot for automated trading'

# Boolean values
DEBUG=true
VERBOSE=false

# Numeric values
MAX_ORDERS=100
BALANCE_INTERVAL=3600
```

### Integration with Clap

Example using with Clap's derive macro:

```rust
use clap::Parser;
use trader_bot::lino_args::getenv_int;

#[derive(Parser)]
struct Args {
    /// Port to listen on
    #[arg(
        long,
        env = "PORT",
        default_value_t = getenv_int("PORT", 8080)
    )]
    port: i64,

    /// API key
    #[arg(long, env = "API_KEY")]
    api_key: Option<String>,
}
```

## API Reference

### Environment Variable Functions

- **`getenv(key, default)`** - Get string value with case-insensitive lookup
- **`getenv_int(key, default)`** - Get integer value (i64)
- **`getenv_bool(key, default)`** - Get boolean value
- **`getenv_decimal(key, default)`** - Get decimal value (rust_decimal::Decimal)
- **`getenv_u64(key, default)`** - Get unsigned 64-bit integer value

### Case Conversion Functions

- **`to_upper_case(s)`** - Convert to UPPER_CASE (SCREAMING_SNAKE_CASE)
- **`to_camel_case(s)`** - Convert to camelCase
- **`to_kebab_case(s)`** - Convert to kebab-case
- **`to_snake_case(s)`** - Convert to snake_case
- **`to_pascal_case(s)`** - Convert to PascalCase

### Lenv File Functions

- **`parse_lenv(content)`** - Parse lenv file content into HashMap
- **`load_lenv(path)`** - Load lenv file and return HashMap
- **`load_lenv_into_env(path)`** - Load lenv file into environment variables
- **`auto_load_lenv()`** - Auto-load from common locations

## Examples

### Complete Configuration Example

```rust
use trader_bot::lino_args::{auto_load_lenv, getenv, getenv_int, getenv_bool};

fn main() {
    // Auto-load lenv file if present
    let _ = auto_load_lenv();

    // Get configuration values with case-insensitive lookup
    let api_key = getenv("API_KEY", "");
    let port = getenv_int("PORT", 8080);
    let debug = getenv_bool("DEBUG", false);

    println!("API Key: {}", if api_key.is_empty() { "Not set" } else { "***" });
    println!("Port: {}", port);
    println!("Debug: {}", debug);
}
```

### Testing with Different Case Conventions

```rust
use trader_bot::lino_args::getenv;

// These all access the same variable:
std::env::set_var("MY_API_KEY", "secret123");

assert_eq!(getenv("my_api_key", ""), "secret123");
assert_eq!(getenv("myApiKey", ""), "secret123");
assert_eq!(getenv("my-api-key", ""), "secret123");
assert_eq!(getenv("MY_API_KEY", ""), "secret123");
```

## Error Handling

Lenv file operations return `Result<T, LenvError>`:

```rust
use trader_bot::lino_args::load_lenv;

match load_lenv(".lenv") {
    Ok(vars) => println!("Loaded {} variables", vars.len()),
    Err(e) => eprintln!("Error loading lenv file: {}", e),
}
```

## License

This is a temporary copy of code from [lino-arguments](https://github.com/link-foundation/lino-arguments), which is released under the Unlicense (public domain).

## See Also

- [lino-arguments repository](https://github.com/link-foundation/lino-arguments) - Original source repository
- [Links Notation](https://github.com/linksplatform) - Links Platform organization
