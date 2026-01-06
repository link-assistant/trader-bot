# Trader Bot

A unified trading bot framework written in Rust with multi-exchange support, multiple trading strategies, and comprehensive testing.

[![CI/CD Pipeline](https://github.com/link-assistant/trader-bot/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-assistant/trader-bot/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Quick Start

```bash
# Install
cargo install --git https://github.com/link-assistant/trader-bot.git

# Run demo mode
trader-bot --demo

# Plan mode (see orders without executing)
trader-bot --demo --plan

# With configuration file
trader-bot --config config.lino
```

## Configuration

Trader Bot uses [Links Notation](https://github.com/link-foundation/links-notation) for configuration. The indented syntax provides clear, readable configuration files.

### Simple Configuration

Create a `config.lino` file:

```
# Simple trading configuration

settings
  log_level info
  verbose false

users
  (
    broker tbank
    token TBANK_API_TOKEN
    accounts
      main
        allocation
          SBER 30
          LKOH 30
          GAZP 40
  )
```

### Multi-User Configuration

```
# Multi-user trading configuration

settings
  log_level info
  verbose false
  dry_run false

users
  (
    broker tbank
    token TBANK_API_TOKEN
    accounts
      main
        allocation
          strategy balancer
            portion 30
              allocation
                strategy hold
                ticker SBER
            portion 30
              allocation
                strategy hold
                ticker LKOH
            portion 40
              allocation
                strategy hold
                ticker GAZP
      trading
        allocation
          AAPL 50
          GOOGL 50
  )
  (
    exchange binance
    "api key" BINANCE_API_KEY
    accounts
      spot_main
        allocation
          BTC 50
          ETH 50
  )
```

### Allocation Syntax

The short syntax and full recursive syntax are equivalent:

```
# Short syntax (simple allocations)
allocation
  SBER 30
  LKOH 30
  GAZP 40

# Full recursive syntax (advanced features)
allocation
  strategy balancer
    portion 30
      allocation
        strategy hold
        ticker SBER
    portion 30
      allocation
        strategy hold
        ticker LKOH
    portion 40
      allocation
        strategy hold
        ticker GAZP
```

The full syntax enables nested balancers and mixed strategies:

```
# 60% stocks, 40% crypto
allocation
  strategy balancer
    portion 60
      allocation
        strategy balancer
          portion 50
            allocation
              strategy hold
              ticker SBER
          portion 50
            allocation
              strategy hold
              ticker LKOH
    portion 40
      allocation
        strategy balancer
          portion 70
            allocation
              strategy hold
              ticker BTC
          portion 30
            allocation
              strategy hold
              ticker ETH
```

## CLI Options

```
trader-bot
  --config <path>
    Path to configuration file
    env TRADER_BOT_CONFIG

  --lenv <path>
    Path to lenv file for environment variables
    env LENV_FILE

  --log-level <level>
    values
      trace
      debug
      info
      warn
      error
    default info
    env TRADER_BOT_LOG_LEVEL

  --verbose
    Enable verbose output
    default false
    env TRADER_BOT_VERBOSE

  --dry-run
    Run without executing actual trades
    default false
    env TRADER_BOT_DRY_RUN

  --plan
    Show planned orders without executing
    default false
    env TRADER_BOT_PLAN

  --account <id>
    Specific account to use
    env TRADER_BOT_ACCOUNT

  --user <id>
    Specific user for multi-user setups
    env TRADER_BOT_USER

  --balance-interval <seconds>
    Rebalancing interval override
    env TRADER_BOT_BALANCE_INTERVAL

  --order-delay <milliseconds>
    Delay between orders
    env TRADER_BOT_ORDER_DELAY

  --run-once
    Run once and exit
    default false
    env TRADER_BOT_RUN_ONCE

  --demo
    Run with simulated exchange
    default false
```

## Environment Variables

Create a `.lenv` file for sensitive credentials:

```
# API tokens (keep secret!)
TBANK_API_TOKEN: your_token_here
BINANCE_API_KEY: your_binance_key
BINANCE_API_SECRET: your_binance_secret

# Bot settings
TRADER_BOT_LOG_LEVEL: info
TRADER_BOT_VERBOSE: false
TRADER_BOT_BALANCE_INTERVAL: 3600
```

Configuration priority (highest to lowest):

```
priority
  CLI arguments
  environment variables
  configuration files
  default values
```

## Examples

```bash
# Demo mode - see the bot in action
trader-bot --demo

# Plan mode - preview orders without execution
trader-bot --demo --plan

# Verbose demo with planning
trader-bot --demo --plan --verbose

# Use specific config file
trader-bot --config trading.lino

# Override balance interval
trader-bot --config config.lino --balance-interval 1800

# Run once and exit
trader-bot --config config.lino --run-once

# Specific user and account
trader-bot --config config.lino --user user1 --account main
```

## Features

- **Trading Strategies**
  - Balancer: Automatically rebalance portfolios to target allocations
  - Scalper: Buy-low/sell-high with FIFO lot tracking
  - Hold: Maintain target allocations per asset
  - Extensible strategy trait for custom implementations

- **Recursive Allocation System**
  - Simple ticker-based allocations
  - Nested balancer strategies
  - Mix strategies (e.g., 50% balancer, 30% scalper, 20% hold)

- **Multi-Exchange Support**
  - T-Bank (formerly Tinkoff Investments)
  - Binance (spot trading)
  - Interactive Brokers (TWS/IB Gateway)
  - Extensible for other brokers

- **Multi-User/Multi-Account**
  - Multiple broker connections per configuration
  - Multiple accounts per broker
  - Individual allocation per account

- **Market Simulator**
  - Built-in simulator for testing
  - Scenario-based backtesting

## Architecture

```
src/
├── adapters/          # Exchange-specific implementations
│   ├── binance.rs
│   ├── interactive_brokers.rs
│   └── tbank.rs
├── config/            # Configuration management
├── domain/            # Core business types
│   ├── allocation.rs  # Recursive allocation system
│   ├── user.rs        # User/Account models
│   ├── wallet.rs      # Portfolio types
│   └── ...
├── exchange/          # Exchange abstraction layer
├── simulator/         # Market simulation
└── strategy/          # Trading strategies
    ├── balancer/      # Portfolio rebalancing
    ├── scalper.rs     # Scalping strategy
    └── holding.rs     # Position holding
```

Key design principles:

```
principles
  modularity
    Independent composable modules
  separation_of_concerns
    Domain logic separate from APIs
  strategy_pattern
    Composable strategies via Strategy trait
  testability
    Pure calculation logic with comprehensive tests
  links_notation
    Human readable configuration
```

## Development

```bash
# Format code
cargo fmt

# Run lints
cargo clippy --all-targets --all-features

# Run tests
cargo test

# Build
cargo build --release
```

## Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_test

# Specific test
cargo test allocation
```

## License

[Unlicense](LICENSE) - Public Domain

## References

- [Links Notation](https://github.com/link-foundation/links-notation) - Configuration format
- [lino-arguments](https://github.com/link-foundation/lino-arguments) - Unified configuration system
- [lino-env](https://github.com/link-foundation/lino-env) - Environment file format
- [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles) - Architecture guidelines
