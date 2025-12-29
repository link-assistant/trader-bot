# Balancer Trader Bot

A portfolio balancer and trading bot written in Rust with multi-exchange support.

[![CI/CD Pipeline](https://github.com/link-assistant/balancer-trader-bot/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-assistant/balancer-trader-bot/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Features

- **Portfolio Rebalancing**: Automatically rebalance your portfolio to maintain target allocations
- **Multiple Allocation Strategies**:
  - Manual (fixed percentages)
  - Market Cap weighted
  - AUM (Assets Under Management) weighted
  - Decorrelation strategy
- **Multi-Exchange Support**: Abstracted exchange layer supporting multiple brokers
  - T-Bank (formerly Tinkoff)
  - Crypto exchanges (e.g., Binance)
  - Extensible for other brokers
- **Market Simulator**: Built-in simulator for backtesting and testing
- **Comprehensive Testing**: Unit tests, integration tests, and scenario-based tests
- **Clean Architecture**: Follows [code architecture principles](https://github.com/link-foundation/code-architecture-principles)

## Architecture

The crate follows Clean Architecture principles with clear separation of concerns:

```
src/
├── domain/          # Core business types (Position, Wallet, Order, Money)
├── exchange/        # Exchange API abstraction layer (ExchangeProvider trait)
├── balancer/        # Portfolio rebalancing logic and calculations
├── simulator/       # Market simulation for testing
└── config/          # Configuration management
```

### Key Design Principles

- **Modularity**: Split into independently understandable modules
- **Separation of Concerns**: Domain logic separate from exchange APIs
- **Abstraction**: Exchange-agnostic design via `ExchangeProvider` trait
- **Testability**: Pure calculation logic with comprehensive tests
- **Immutability**: Value types for domain concepts (Money, Position)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/link-assistant/balancer-trader-bot.git
cd balancer-trader-bot

# Build the project
cargo build

# Run tests
cargo test

# Run the demo
cargo run

# Run an example
cargo run --example basic_usage
```

### Basic Usage

```rust
use balancer_trader_bot::{
    balancer::{BalancerConfig, BalancerEngine},
    domain::DesiredAllocation,
    simulator::SimulatedExchange,
};
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create a simulated exchange
    let exchange = SimulatedExchange::new("USD");
    exchange.add_instrument("AAPL", "Apple Inc.", dec!(150), 1);
    exchange.add_instrument("GOOGL", "Alphabet Inc.", dec!(2800), 1);
    exchange.create_account("my_account", dec!(100000));

    // Define target allocation
    let mut target = DesiredAllocation::new();
    target.set("AAPL", dec!(50));
    target.set("GOOGL", dec!(50));

    // Create and run balancer
    let exchange = Arc::new(exchange);
    let config = BalancerConfig::default();
    let mut engine = BalancerEngine::new(exchange, config);

    let result = engine.rebalance("my_account", &target).await;
    println!("Rebalance result: {:?}", result);
}
```

## Configuration

Create a `config.json` file:

```json
{
  "version": "1.0.0",
  "settings": {
    "log_level": "info",
    "verbose": false
  },
  "accounts": [
    {
      "id": "main",
      "name": "Main Account",
      "exchange": "tbank",
      "exchange_account_id": "12345",
      "token_env_var": "TBANK_API_TOKEN",
      "desired_allocation": {
        "SBER": 30,
        "LKOH": 30,
        "GAZP": 40
      },
      "allocation_mode": "manual",
      "balance_interval_secs": 3600
    }
  ]
}
```

## Testing

### Unit Tests

Unit tests are included in each module:

```bash
cargo test --lib
```

### Integration Tests

Integration tests verify the complete workflow:

```bash
cargo test --test integration_test
```

### Scenario-based Tests

The simulator supports scenario-based testing:

```rust
use balancer_trader_bot::simulator::{ScenarioBuilder, PriceModel};
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_volatile_market() {
    let scenario = ScenarioBuilder::new("Volatile market")
        .with_cash(dec!(100000))
        .with_instrument("STOCK", "Test Stock", dec!(100), 1)
        .with_target("STOCK", dec!(80))
        .with_price_model("STOCK", PriceModel::RandomWalk { volatility: dec!(5) })
        .with_ticks(100)
        .with_rebalance_interval(10)
        .assert_min_value(dec!(80000))
        .build();

    let result = scenario.run().await;
    assert!(result.passed);
}
```

## Exchange Support

### Implementing a New Exchange

To add support for a new exchange, implement the `ExchangeProvider` trait:

```rust
use async_trait::async_trait;
use balancer_trader_bot::exchange::{ExchangeProvider, ExchangeResult, /* ... */};

struct MyExchange {
    // Exchange-specific fields
}

#[async_trait]
impl ExchangeProvider for MyExchange {
    fn info(&self) -> &ExchangeInfo { /* ... */ }
    async fn ping(&self) -> ExchangeResult<()> { /* ... */ }
    async fn get_wallet(&self, account_id: &str) -> ExchangeResult<Wallet> { /* ... */ }
    // ... implement other methods
}
```

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting (CI style)
cargo fmt --check

# Run Clippy lints
cargo clippy --all-targets --all-features

# Run all checks
cargo fmt --check && cargo clippy --all-targets --all-features && cargo test
```

### Pre-commit Hooks

Install pre-commit hooks for automatic checks:

```bash
pip install pre-commit
pre-commit install
```

## Project Structure

```
.
├── .github/workflows/     # CI/CD pipeline
├── changelog.d/           # Changelog fragments
├── examples/
│   └── basic_usage.rs     # Usage examples
├── src/
│   ├── balancer/          # Rebalancing logic
│   ├── config/            # Configuration
│   ├── domain/            # Core domain types
│   ├── exchange/          # Exchange abstraction
│   ├── simulator/         # Market simulator
│   ├── lib.rs             # Library entry
│   └── main.rs            # CLI entry
├── tests/
│   └── integration_test.rs
├── Cargo.toml
└── README.md
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes and add tests
4. Run quality checks: `cargo fmt && cargo clippy && cargo test`
5. Add a changelog fragment
6. Commit and create a Pull Request

## License

[Unlicense](LICENSE) - Public Domain

## References

- Reimplementation of [tinkoff-invest-etf-balancer-bot](https://github.com/suenot/tinkoff-invest-etf-balancer-bot)
- Follows [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles)
