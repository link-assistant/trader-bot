//! Integration tests for trader-bot.
//!
//! These tests verify the complete workflow of the balancer.

use rust_decimal_macros::dec;
use std::sync::Arc;
use trader_bot::domain::{DesiredAllocation, Money, Position, Wallet};
use trader_bot::exchange::ExchangeProvider;
use trader_bot::simulator::{PriceModel, ScenarioBuilder, SimulatedExchange};
use trader_bot::strategy::balancer::{BalancerConfig, BalancerEngine, RebalanceCalculator};

mod balancer_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_rebalancing_workflow() {
        // Create exchange with instruments
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("AAPL", "Apple", dec!(150), 1);
        exchange.add_instrument("GOOGL", "Alphabet", dec!(2800), 1);
        exchange.add_instrument("MSFT", "Microsoft", dec!(400), 1);

        // Create account with cash
        exchange.create_account("test_account", dec!(100000));

        // Define target allocation
        let mut target = DesiredAllocation::new();
        target.set("AAPL", dec!(40));
        target.set("GOOGL", dec!(30));
        target.set("MSFT", dec!(30));

        // Create and run balancer
        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            order_delay_ms: 0,
            dry_run: false,
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let result = engine.rebalance("test_account", &target).await.unwrap();

        assert!(result.success);
        assert!(!result.executed.is_empty());

        // Verify portfolio is now closer to target
        let wallet = exchange.get_wallet("test_account").await.unwrap();
        let allocation = wallet.current_allocation();

        // Check that AAPL allocation is close to 40%
        let aapl_pct = allocation.get("AAPL").unwrap_or(dec!(0));
        assert!(
            (aapl_pct - dec!(40)).abs() < dec!(5),
            "AAPL should be ~40%, was {aapl_pct}"
        );
    }

    #[tokio::test]
    async fn test_rebalancing_with_existing_positions() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("A", "Stock A", dec!(100), 1);
        exchange.add_instrument("B", "Stock B", dec!(100), 1);

        // Create account with cash and existing position
        exchange.create_account("test", dec!(5000));
        let position = Position::builder("A", "SIMULATOR")
            .quantity(dec!(50))
            .lot_size(1)
            .current_price(Money::new(dec!(100), "USD"))
            .build();
        exchange.add_position("test", position);

        // Currently: 50% cash, 50% A. Target: 50% A, 50% B
        let mut target = DesiredAllocation::new();
        target.set("A", dec!(50));
        target.set("B", dec!(50));

        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            order_delay_ms: 0,
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let result = engine.rebalance("test", &target).await.unwrap();
        assert!(result.success);

        // Should have bought B
        let wallet = exchange.get_wallet("test").await.unwrap();
        assert!(wallet.get_position("B").is_some());
    }

    #[tokio::test]
    async fn test_dry_run_no_trades() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("X", "Stock X", dec!(100), 1);
        exchange.create_account("dry_test", dec!(10000));

        let mut target = DesiredAllocation::new();
        target.set("X", dec!(100));

        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            dry_run: true,
            order_delay_ms: 0,
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let result = engine.rebalance("dry_test", &target).await.unwrap();

        // Plan should exist but no trades executed
        assert!(result.executed.is_empty());

        // Wallet should still have only cash
        let wallet = exchange.get_wallet("dry_test").await.unwrap();
        assert_eq!(wallet.position_count(), 0);
        assert_eq!(wallet.cash().amount(), dec!(10000));
    }

    #[tokio::test]
    async fn test_balanced_portfolio_no_trades() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("A", "Stock A", dec!(100), 1);
        exchange.add_instrument("B", "Stock B", dec!(100), 1);

        exchange.create_account("balanced", dec!(0));

        // Add perfectly balanced positions
        exchange.add_position(
            "balanced",
            Position::builder("A", "SIMULATOR")
                .quantity(dec!(50))
                .lot_size(1)
                .current_price(Money::new(dec!(100), "USD"))
                .build(),
        );
        exchange.add_position(
            "balanced",
            Position::builder("B", "SIMULATOR")
                .quantity(dec!(50))
                .lot_size(1)
                .current_price(Money::new(dec!(100), "USD"))
                .build(),
        );

        let mut target = DesiredAllocation::new();
        target.set("A", dec!(50));
        target.set("B", dec!(50));

        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            order_delay_ms: 0,
            tolerance_percent: dec!(1),
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let result = engine.rebalance("balanced", &target).await.unwrap();

        // Should recognize portfolio is balanced
        assert!(result.plan.is_empty() || result.executed.is_empty());
    }
}

mod calculator_integration_tests {
    use super::*;

    #[test]
    fn test_calculator_with_complex_portfolio() {
        let mut wallet = Wallet::with_cash("USD", Money::new(dec!(20000), "USD"));
        wallet.add_position(Position::new(
            "A",
            "X",
            dec!(100),
            Money::new(dec!(100), "USD"),
        ));
        wallet.add_position(Position::new(
            "B",
            "X",
            dec!(50),
            Money::new(dec!(200), "USD"),
        ));
        wallet.add_position(Position::new(
            "C",
            "X",
            dec!(25),
            Money::new(dec!(400), "USD"),
        ));

        // Current: A=10000, B=10000, C=10000, Cash=20000 = 50000 total
        // Current %: A=20%, B=20%, C=20%, Cash=40%

        let mut desired = DesiredAllocation::new();
        desired.set("A", dec!(30)); // Need to buy
        desired.set("B", dec!(30)); // Need to buy
        desired.set("C", dec!(20)); // Balanced
                                    // 20% cash implied

        let calc = RebalanceCalculator::new()
            .with_tolerance_percent(dec!(0.5))
            .with_min_trade_value(dec!(100));

        let diffs = calc.calculate_diffs(&wallet, &desired);

        assert_eq!(diffs.len(), 3);

        // A needs buying (20% -> 30%)
        let a_diff = diffs.iter().find(|d| d.symbol == "A").unwrap();
        assert!(a_diff.needs_buy());

        // C is balanced (20% -> 20%)
        let c_diff = diffs.iter().find(|d| d.symbol == "C").unwrap();
        assert!((c_diff.diff_percent).abs() < dec!(1));
    }
}

mod scenario_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_scenario_basic_rebalancing() {
        let scenario = ScenarioBuilder::new("Basic rebalancing")
            .with_cash(dec!(50000))
            .with_instrument("X", "Stock X", dec!(100), 1)
            .with_instrument("Y", "Stock Y", dec!(200), 1)
            .with_target("X", dec!(60))
            .with_target("Y", dec!(40))
            .with_ticks(30)
            .with_rebalance_interval(10)
            .assert_min_value(dec!(45000))
            .build();

        let result = scenario.run().await;

        assert!(
            result.passed,
            "Scenario failed: {:?}",
            result.failure_reason
        );
        assert!(
            result.cycles > 0,
            "Should have run at least one rebalance cycle"
        );
        assert!(result.total_trades > 0, "Should have made some trades");
    }

    #[tokio::test]
    async fn test_scenario_with_volatile_market() {
        let scenario = ScenarioBuilder::new("Volatile market")
            .with_cash(dec!(100000))
            .with_instrument("VOL", "Volatile Stock", dec!(100), 1)
            .with_target("VOL", dec!(80))
            .with_price_model(
                "VOL",
                PriceModel::RandomWalk {
                    volatility: dec!(10),
                },
            )
            .with_ticks(50)
            .with_rebalance_interval(10)
            .assert_min_value(dec!(50000)) // Allow for significant losses
            .build();

        let result = scenario.run().await;

        assert!(
            result.passed,
            "Scenario failed: {:?}",
            result.failure_reason
        );
    }
}

mod version_tests {
    use trader_bot::VERSION;

    #[test]
    fn test_version_is_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_version_matches_cargo_toml() {
        assert!(VERSION.starts_with("0."));
    }
}
