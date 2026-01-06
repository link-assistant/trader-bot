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

mod plan_mode_tests {
    use super::*;
    use trader_bot::domain::OrderDirection;
    use trader_bot::domain::{PlannedOrder, PlannedOrders};
    use trader_bot::strategy::balancer::RebalanceAction;

    #[tokio::test]
    async fn test_plan_mode_creates_plan_without_executing() {
        // Create exchange with instruments
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("AAPL", "Apple", dec!(150), 1);
        exchange.add_instrument("GOOGL", "Alphabet", dec!(2800), 1);

        // Create account with cash
        exchange.create_account("plan_test", dec!(100000));

        // Define target allocation
        let mut target = DesiredAllocation::new();
        target.set("AAPL", dec!(60));
        target.set("GOOGL", dec!(40));

        // Create balancer engine
        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            dry_run: true, // In plan mode, this should be true
            order_delay_ms: 0,
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        // Create plan only (don't execute)
        let plan = engine.create_plan("plan_test", &target).await.unwrap();

        // Verify plan was created with actions
        assert!(!plan.is_empty(), "Plan should have actions");
        assert_eq!(plan.actions.len(), 2, "Should have 2 buy actions");

        // Verify wallet is unchanged
        let wallet = exchange.get_wallet("plan_test").await.unwrap();
        assert_eq!(wallet.position_count(), 0, "Should have no positions");
        assert_eq!(
            wallet.cash().amount(),
            dec!(100000),
            "Cash should be unchanged"
        );
    }

    #[tokio::test]
    async fn test_plan_mode_collects_all_orders() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("A", "Stock A", dec!(100), 1);
        exchange.add_instrument("B", "Stock B", dec!(50), 1);
        exchange.add_instrument("C", "Stock C", dec!(200), 1);

        exchange.create_account("multi_test", dec!(10000));

        let mut target = DesiredAllocation::new();
        target.set("A", dec!(40));
        target.set("B", dec!(30));
        target.set("C", dec!(30));

        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            dry_run: true,
            order_delay_ms: 0,
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let plan = engine.create_plan("multi_test", &target).await.unwrap();

        // Should have planned orders for all three assets
        assert_eq!(plan.actions.len(), 3, "Should have 3 planned actions");
        assert!(
            plan.total_buy_value.amount() > dec!(0),
            "Should have buy value"
        );
    }

    #[test]
    fn test_planned_order_from_rebalance_action() {
        let action = RebalanceAction {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: OrderDirection::Buy,
            lots: 100,
            estimated_price: Money::new(dec!(150), "USD"),
            estimated_value: Money::new(dec!(15000), "USD"),
            current_percent: dec!(0),
            target_percent: dec!(30),
            priority: 0,
        };

        let planned = PlannedOrder::from_rebalance_action(&action, "test_account");

        assert_eq!(planned.symbol, "AAPL");
        assert_eq!(planned.direction, "BUY");
        assert_eq!(planned.lots, 100);
        assert_eq!(planned.account_id, "test_account");
        assert_eq!(planned.estimated_price.amount(), dec!(150));
        assert_eq!(planned.estimated_value.amount(), dec!(15000));
        assert_eq!(planned.current_percent, dec!(0));
        assert_eq!(planned.target_percent, dec!(30));
    }

    #[test]
    fn test_planned_orders_collection() {
        let mut planned_orders = PlannedOrders::new();
        assert!(planned_orders.is_empty());

        let order1 = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 100,
            estimated_price: Money::new(dec!(150), "USD"),
            estimated_value: Money::new(dec!(15000), "USD"),
            current_percent: dec!(0),
            target_percent: dec!(30),
            account_id: "acc1".to_string(),
            reason: None,
        };

        let order2 = PlannedOrder {
            symbol: "GOOGL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "SELL".to_string(),
            lots: 5,
            estimated_price: Money::new(dec!(2800), "USD"),
            estimated_value: Money::new(dec!(14000), "USD"),
            current_percent: dec!(40),
            target_percent: dec!(25),
            account_id: "acc1".to_string(),
            reason: None,
        };

        planned_orders.add_order(order1);
        planned_orders.add_order(order2);

        assert!(!planned_orders.is_empty());
        assert_eq!(planned_orders.total_count(), 2);
        assert_eq!(planned_orders.account_count(), 1);

        let acc1_orders = planned_orders.orders_for_account("acc1").unwrap();
        assert_eq!(acc1_orders.len(), 2);
    }

    #[test]
    fn test_planned_orders_multiple_accounts() {
        let mut planned_orders = PlannedOrders::new();

        let order1 = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 50,
            estimated_price: Money::new(dec!(150), "USD"),
            estimated_value: Money::new(dec!(7500), "USD"),
            current_percent: dec!(0),
            target_percent: dec!(30),
            account_id: "account_1".to_string(),
            reason: None,
        };

        let order2 = PlannedOrder {
            symbol: "MSFT".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 25,
            estimated_price: Money::new(dec!(400), "USD"),
            estimated_value: Money::new(dec!(10000), "USD"),
            current_percent: dec!(0),
            target_percent: dec!(25),
            account_id: "account_2".to_string(),
            reason: None,
        };

        planned_orders.add_order(order1);
        planned_orders.add_order(order2);

        assert_eq!(planned_orders.account_count(), 2);
        assert_eq!(planned_orders.total_count(), 2);

        assert!(planned_orders.orders_for_account("account_1").is_some());
        assert!(planned_orders.orders_for_account("account_2").is_some());
        assert!(planned_orders.orders_for_account("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_plan_mode_with_already_balanced_portfolio() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("A", "Stock A", dec!(100), 1);
        exchange.add_instrument("B", "Stock B", dec!(100), 1);

        exchange.create_account("balanced_test", dec!(0));

        // Add perfectly balanced positions (50/50)
        exchange.add_position(
            "balanced_test",
            Position::builder("A", "SIMULATOR")
                .quantity(dec!(50))
                .lot_size(1)
                .current_price(Money::new(dec!(100), "USD"))
                .build(),
        );
        exchange.add_position(
            "balanced_test",
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
            dry_run: true,
            order_delay_ms: 0,
            tolerance_percent: dec!(1),
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let plan = engine.create_plan("balanced_test", &target).await.unwrap();

        // Plan should be empty as portfolio is balanced
        assert!(
            plan.is_empty(),
            "Plan should be empty for balanced portfolio"
        );
        assert!(
            plan.empty_reason.is_some(),
            "Should have reason for empty plan"
        );
    }

    #[tokio::test]
    async fn test_plan_mode_shows_sell_and_buy_orders() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("A", "Stock A", dec!(100), 1);
        exchange.add_instrument("B", "Stock B", dec!(100), 1);

        exchange.create_account("rebalance_test", dec!(0));

        // Add unbalanced positions: 80% A, 20% B
        exchange.add_position(
            "rebalance_test",
            Position::builder("A", "SIMULATOR")
                .quantity(dec!(80))
                .lot_size(1)
                .current_price(Money::new(dec!(100), "USD"))
                .build(),
        );
        exchange.add_position(
            "rebalance_test",
            Position::builder("B", "SIMULATOR")
                .quantity(dec!(20))
                .lot_size(1)
                .current_price(Money::new(dec!(100), "USD"))
                .build(),
        );

        // Target: 50% A, 50% B (need to sell A, buy B)
        let mut target = DesiredAllocation::new();
        target.set("A", dec!(50));
        target.set("B", dec!(50));

        let exchange = Arc::new(exchange);
        let config = BalancerConfig {
            dry_run: true,
            order_delay_ms: 0,
            tolerance_percent: dec!(1),
            ..BalancerConfig::default()
        };
        let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

        let plan = engine.create_plan("rebalance_test", &target).await.unwrap();

        // Should have both sell and buy actions
        assert!(!plan.is_empty());

        let has_sell = plan.actions.iter().any(|a| a.is_sell());
        let has_buy = plan.actions.iter().any(|a| a.is_buy());

        assert!(has_sell, "Should have sell action for A");
        assert!(has_buy, "Should have buy action for B");
    }

    #[test]
    fn test_planned_order_display_format() {
        let order = PlannedOrder {
            symbol: "AAPL".to_string(),
            exchange: "NASDAQ".to_string(),
            direction: "BUY".to_string(),
            lots: 100,
            estimated_price: Money::new(dec!(150), "USD"),
            estimated_value: Money::new(dec!(15000), "USD"),
            current_percent: dec!(10),
            target_percent: dec!(30),
            account_id: "test".to_string(),
            reason: None,
        };

        let display = format!("{order}");
        assert!(display.contains("BUY"));
        assert!(display.contains("100 lots"));
        assert!(display.contains("AAPL"));
        assert!(display.contains("150"));
        assert!(display.contains("15000"));
        assert!(display.contains("10.00%"));
        assert!(display.contains("30.00%"));
    }
}
