# Trader Bot

A unified trading bot framework written in Rust with multi-exchange support, multiple trading strategies, and comprehensive testing.

[![CI/CD Pipeline](https://github.com/link-assistant/trader-bot/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-assistant/trader-bot/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Quick Start

### Installation

```bash
cargo install --git https://github.com/link-assistant/trader-bot.git
```

Or build from source:

```bash
git clone https://github.com/link-assistant/trader-bot.git
cd trader-bot
cargo build --release
```

### CLI Usage

Run demo mode to see how it works:

```bash
trader-bot --demo
```

Plan mode shows what orders would be placed without executing them:

```bash
trader-bot --demo --plan
```

Run with a configuration file:

```bash
trader-bot --config config.lenv
```

### Configuration

Configuration uses [Links Notation](https://github.com/link-foundation/links-notation) via [lino-arguments](https://github.com/link-foundation/lino-arguments) for a unified configuration system.

Configuration is loaded with the following priority:

```
priority:
  CLI arguments
  environment variables
  configuration files
  default values
```

#### CLI Options

```
trader-bot:
  --config <path>
    Path to configuration file
  --lenv <path>
    Path to lenv file for environment variables
  --log-level <level>
    trace | debug | info | warn | error
    default: info
  --verbose
    Enable verbose output
  --dry-run
    Run without executing actual trades
  --plan
    Show planned orders without executing
  --account <id>
    Specific account to use
  --user <id>
    Specific user for multi-user setups
  --balance-interval <seconds>
    Rebalancing interval override
  --order-delay <milliseconds>
    Delay between orders
  --run-once
    Run once and exit
  --demo
    Run with simulated exchange
```

#### Environment Variables

All options can be set via environment variables:

```
TRADER_BOT_CONFIG: path/to/config.lenv
TRADER_BOT_LOG_LEVEL: info
TRADER_BOT_VERBOSE: true
TRADER_BOT_DRY_RUN: false
TRADER_BOT_PLAN: false
TRADER_BOT_ACCOUNT: account1
TRADER_BOT_USER: user1
TRADER_BOT_BALANCE_INTERVAL: 3600
TRADER_BOT_ORDER_DELAY: 100
TRADER_BOT_RUN_ONCE: false
LENV_FILE: .lenv
```

#### Lenv Configuration File

Create a `.lenv` file using [lino-env](https://github.com/link-foundation/lino-env) format:

```
# Trading bot configuration

# API tokens (keep secret)
TBANK_API_TOKEN: your_api_token_here
BINANCE_API_KEY: your_binance_key
BINANCE_API_SECRET: your_binance_secret

# Bot settings
TRADER_BOT_LOG_LEVEL: info
TRADER_BOT_VERBOSE: false
TRADER_BOT_DRY_RUN: false
TRADER_BOT_BALANCE_INTERVAL: 3600
```

#### JSON Configuration File

For complex multi-account setups, use JSON format:

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
      "accounts": [
        {
          "id": "account1",
          "exchange": "tbank",
          "exchange_account_id": "12345",
          "token_env_var": "TBANK_API_TOKEN",
          "desired_allocation": {
            "SBER": 30,
            "LKOH": 30,
            "GAZP": 40
          }
        }
      ]
    }
  ]
}
```

### Examples

```bash
# Demo mode - see the bot in action
trader-bot --demo

# Plan mode - preview orders without execution
trader-bot --demo --plan

# Verbose demo with planning
trader-bot --demo --plan --verbose

# Use specific config file
trader-bot --config trading.lenv

# Override balance interval
trader-bot --config config.lenv --balance-interval 1800

# Run once and exit
trader-bot --config config.lenv --run-once
```

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

## Library Usage

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

- [Links Notation](https://github.com/link-foundation/links-notation) - Configuration format
- [lino-arguments](https://github.com/link-foundation/lino-arguments) - Unified configuration system
- [lino-env](https://github.com/link-foundation/lino-env) - Environment file format
- [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles) - Architecture guidelines
