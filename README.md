# Trader Bot

A unified trading bot framework written in Rust with multi-exchange support, multiple trading strategies, and comprehensive testing.

[![CI/CD Pipeline](https://github.com/link-assistant/trader-bot/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-assistant/trader-bot/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Features

- **Multiple Trading Strategies**:
  - **Balancer Strategy**: Automatically rebalance portfolios to target allocations
  - **Scalping Strategy**: Buy-low/sell-high with FIFO lot tracking
  - **Holding Strategy**: Maintain target allocations per asset
  - Extensible strategy trait for custom implementations
- **Portfolio Allocation Modes**:
  - Manual (fixed percentages)
  - Market Cap weighted
  - AUM (Assets Under Management) weighted
  - Decorrelation strategy
- **Multi-Exchange Support**: Abstracted exchange layer with adapters for:
  - T-Bank (formerly Tinkoff Investments)
  - Binance (spot trading)
  - Interactive Brokers (TWS/IB Gateway)
  - Extensible for other brokers
- **Multi-User/Multi-Account**: Manage multiple users and accounts from a single configuration
- **Market Simulator**: Built-in simulator for backtesting and testing strategies
- **Comprehensive Testing**: Unit tests, integration tests, and scenario-based tests
- **Clean Architecture**: Follows [code architecture principles](https://github.com/link-foundation/code-architecture-principles)

## Architecture

The crate follows Clean Architecture principles with clear separation of concerns:

```
src/
├── adapters/        # Exchange-specific implementations
│   ├── binance.rs   # Binance adapter
│   ├── interactive_brokers.rs  # IB adapter
│   └── tbank.rs     # T-Bank adapter
├── config/          # Configuration management
├── domain/          # Core business types (Position, Wallet, Order, Money, Trade)
├── exchange/        # Exchange API abstraction layer (ExchangeProvider trait)
├── simulator/       # Market simulation for testing
└── strategy/        # Trading strategies
    ├── balancer/    # Portfolio rebalancing (engine, calculator, actions)
    ├── scalper.rs   # Scalping strategy with FIFO tracking
    ├── holding.rs   # Position holding strategy
    └── traits.rs    # Strategy trait definition
```

### Key Design Principles

- **Modularity**: Split into independently understandable modules
- **Separation of Concerns**: Domain logic separate from exchange APIs
- **Abstraction**: Exchange-agnostic design via `ExchangeProvider` trait
- **Strategy Pattern**: Composable strategies via the `Strategy` trait
- **Testability**: Pure calculation logic with comprehensive tests
- **Immutability**: Value types for domain concepts (Money, Position)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/link-assistant/trader-bot.git
cd trader-bot

# Build the project
cargo build

# Run tests
cargo test

# Run the demo
cargo run

# Run an example
cargo run --example basic_usage
```

### Basic Usage - Portfolio Balancing

```rust
use trader_bot::{
    strategy::balancer::{BalancerConfig, BalancerEngine},
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

### Using Trading Strategies

```rust
use trader_bot::strategy::{
    Strategy, StrategyDecision, MarketState,
    ScalpingStrategy, TradingSettings,
    HoldingStrategy, HoldingConfig,
};
use rust_decimal_macros::dec;

// Create a scalping strategy
let settings = TradingSettings::new("AAPL")
    .with_minimum_profit_steps(2)
    .with_max_position(100);
let scalper = ScalpingStrategy::new(settings);

// Create a holding strategy
let config = HoldingConfig::new("GOOGL")
    .with_percent(dec!(25));  // Target 25% allocation
let holder = HoldingStrategy::new(config);

// Get strategy decisions
let state = MarketState::new("AAPL", "USD")
    .with_cash(dec!(10000))
    .with_last_price(dec!(150));

let decision = scalper.decide(&state).await;
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
  "users": [
    {
      "id": "user1",
      "name": "John Doe",
      "email": "john@example.com",
      "accounts": [
        {
          "id": "account1",
          "name": "Main Trading Account",
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
      ],
      "active": true
    }
  ],
  "accounts": [
    {
      "id": "shared",
      "name": "Shared Account",
      "exchange": "binance",
      "exchange_account_id": "67890",
      "token_env_var": "BINANCE_API_KEY",
      "desired_allocation": {
        "BTC": 50,
        "ETH": 50
      },
      "allocation_mode": "market_cap"
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
use trader_bot::simulator::{ScenarioBuilder, PriceModel};
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

### Available Adapters

| Exchange | Adapter | Status |
|----------|---------|--------|
| T-Bank | `TBankAdapter` | Placeholder |
| Binance | `BinanceAdapter` | Placeholder |
| Interactive Brokers | `InteractiveBrokersAdapter` | Placeholder |
| Simulator | `SimulatedExchange` | Full implementation |

### Implementing a New Exchange

To add support for a new exchange, implement the `ExchangeProvider` trait:

```rust
use async_trait::async_trait;
use trader_bot::exchange::{ExchangeProvider, ExchangeResult, /* ... */};

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

## Strategies

### Implementing Custom Strategies

To create a custom trading strategy, implement the `Strategy` trait:

```rust
use async_trait::async_trait;
use trader_bot::strategy::{Strategy, StrategyDecision, MarketState};
use trader_bot::domain::Order;

struct MyStrategy {
    symbol: String,
}

#[async_trait]
impl Strategy for MyStrategy {
    fn name(&self) -> &str { "my_strategy" }

    async fn decide(&self, state: &MarketState) -> StrategyDecision {
        // Your trading logic here
        StrategyDecision::hold()
    }

    async fn on_order_filled(&mut self, order: &Order) {
        // Handle order fills
    }

    async fn on_order_cancelled(&mut self, order: &Order) {
        // Handle cancellations
    }

    async fn reset(&mut self) {
        // Reset strategy state
    }

    fn symbols(&self) -> Vec<String> {
        vec![self.symbol.clone()]
    }
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
cargo clippy --all-targets --all-features -- -D warnings

# Run all checks
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
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
│   ├── adapters/          # Exchange adapters
│   ├── config/            # Configuration
│   ├── domain/            # Core domain types
│   ├── exchange/          # Exchange abstraction
│   ├── simulator/         # Market simulator
│   ├── strategy/          # Trading strategies
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

- Originally based on [balancer-trader-bot](https://github.com/link-assistant/trader-bot)
- Incorporates ideas from [scalper-trader-bot](https://github.com/link-assistant/scalper-trader-bot)
- Follows [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles)
