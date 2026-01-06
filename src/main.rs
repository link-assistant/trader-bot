//! Trader Bot - CLI application.
//!
//! This is the command-line interface for the configurable trading bot.
//! Uses Links Notation configuration with the following priority:
//!
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. Configuration file (lenv or json)
//! 4. Default values

use rust_decimal_macros::dec;
use std::sync::Arc;
use tracing::{debug, error, info};
use tracing_subscriber::FmtSubscriber;
use trader_bot::cli::Cli;
use trader_bot::domain::{DesiredAllocation, PlannedOrder, PlannedOrders};
use trader_bot::prelude::*;
use trader_bot::simulator::SimulatedExchange;

#[tokio::main]
async fn main() {
    // Parse CLI with lenv support (loads .lenv file automatically)
    let cli = Cli::parse_with_lenv();

    // Initialize logging with configured level
    let subscriber = FmtSubscriber::builder()
        .with_max_level(cli.get_log_level())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    info!("Trader Bot v{}", trader_bot::VERSION);
    debug!("CLI options: {:?}", cli);

    // Load configuration file if specified
    match cli.load_config() {
        Ok(Some(config)) => {
            let config = cli.merge_with_config(config);
            info!(
                "Loaded configuration with {} account(s)",
                config.all_accounts().count()
            );

            if cli.verbose {
                debug!("Config: {:?}", config);
            }

            // Run with file-based configuration
            run_with_config(&cli, config).await;
        }
        Ok(None) => {
            if cli.demo {
                info!("Running demo mode...");
                if cli.plan {
                    info!("PLAN MODE enabled - orders will be calculated but not executed");
                }
                demo_simulation(&cli).await;
            } else {
                info!("No configuration file found. Use --config to specify one, or --demo for demo mode.");
                info!("Looking for: config.json, trader-bot.json, or .trader-bot.json");
            }
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    }
}

/// Runs the trading bot with file-based configuration.
#[allow(clippy::unused_async)]
async fn run_with_config(cli: &Cli, config: trader_bot::config::AppConfig) {
    info!("Starting trading bot...");

    if cli.plan {
        info!("Running in PLAN MODE - orders will be calculated but not executed");
    } else if cli.dry_run || config.settings.dry_run {
        info!("Running in DRY-RUN mode - no actual trades will be executed");
    }

    // Filter accounts if specific account/user requested
    let accounts: Vec<_> = match (&cli.account, &cli.user) {
        (Some(account_id), _) => config
            .all_accounts()
            .filter(|a| a.id == *account_id)
            .collect(),
        (None, Some(user_id)) => config
            .get_user(user_id)
            .map(|u| u.accounts.iter().collect())
            .unwrap_or_default(),
        (None, None) => config.all_accounts().collect(),
    };

    if accounts.is_empty() {
        error!("No accounts found matching criteria");
        return;
    }

    info!("Processing {} account(s)", accounts.len());

    for account in accounts {
        info!("Account: {} ({})", account.name, account.id);
        info!("  Exchange: {}", account.exchange);
        info!("  Allocation mode: {:?}", account.allocation_mode);

        // Show desired allocation
        if !account.desired_allocation.is_empty() {
            info!("  Target allocation:");
            for (symbol, pct) in &account.desired_allocation {
                info!("    {}: {}%", symbol, pct);
            }
        }

        // In real implementation, we would:
        // 1. Create exchange adapter based on account.exchange
        // 2. Create balancer engine
        // 3. Run rebalancing (or just plan if --plan mode)

        if cli.plan {
            info!("  Would calculate rebalancing plan (plan mode)...");
        } else if cli.run_once {
            info!("  Would execute single rebalance...");
        } else {
            info!(
                "  Would start continuous rebalancing (interval: {}s)",
                account.balance_interval_secs
            );
        }
    }

    info!("Configuration loaded successfully. Exchange adapters are placeholders.");
    info!("Use --demo for a working demonstration with simulated exchange.");
    if cli.plan {
        info!("Use --demo --plan to see plan mode in action with simulated data.");
    }
}

/// Demonstrates the balancer with a simulated exchange.
async fn demo_simulation(cli: &Cli) {
    info!("Running simulation demo...");

    // Create simulated exchange
    let exchange = SimulatedExchange::new("USD");

    // Add some instruments
    exchange.add_instrument("AAPL", "Apple Inc.", dec!(150), 1);
    exchange.add_instrument("GOOGL", "Alphabet Inc.", dec!(2800), 1);
    exchange.add_instrument("MSFT", "Microsoft Corp.", dec!(400), 1);
    exchange.add_instrument("AMZN", "Amazon.com Inc.", dec!(180), 1);

    // Create account with initial cash
    let account_id = "demo_account";
    exchange.create_account(account_id, dec!(100000));

    info!("Created demo account with $100,000");

    // Define target allocation
    let mut target = DesiredAllocation::new();
    target.set("AAPL", dec!(30));
    target.set("GOOGL", dec!(25));
    target.set("MSFT", dec!(25));
    target.set("AMZN", dec!(20));

    info!("Target allocation:");
    info!("  AAPL: 30%");
    info!("  GOOGL: 25%");
    info!("  MSFT: 25%");
    info!("  AMZN: 20%");

    // Create balancer with CLI options
    let exchange = Arc::new(exchange);
    let order_delay = cli.order_delay.unwrap_or(100);

    // In plan mode, we always set dry_run to true to prevent execution
    let effective_dry_run = cli.dry_run || cli.plan;
    let config = BalancerConfig {
        dry_run: effective_dry_run,
        order_delay_ms: order_delay,
        ..BalancerConfig::default()
    };
    let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

    if cli.plan {
        info!("PLAN MODE enabled - calculating orders without executing");
    } else if cli.dry_run {
        info!("DRY-RUN mode enabled - simulating without actual orders");
    }

    // In plan mode, only create the plan and display it
    if cli.plan {
        info!("Creating rebalancing plan...");
        match engine.create_plan(account_id, &target).await {
            Ok(plan) => {
                let mut planned_orders = PlannedOrders::new();

                if plan.is_empty() {
                    info!("No rebalancing needed - portfolio is already balanced.");
                    if let Some(reason) = &plan.empty_reason {
                        info!("Reason: {}", reason);
                    }
                } else {
                    // Convert plan actions to PlannedOrders for display
                    for action in &plan.actions {
                        let order = PlannedOrder::from_rebalance_action(action, account_id);
                        planned_orders.add_order(order);
                    }
                    planned_orders.set_total_buy_value(plan.total_buy_value.clone());
                    planned_orders.set_total_sell_value(plan.total_sell_value.clone());

                    // Display the planned orders
                    planned_orders.display();

                    info!("Plan summary: {}", plan.summary());
                }
            }
            Err(e) => {
                error!("Failed to create plan: {}", e);
            }
        }
    } else {
        // Normal execution mode (or dry-run mode)
        info!("Running initial rebalance...");
        match engine.rebalance(account_id, &target).await {
            Ok(result) => {
                info!("Rebalance complete!");
                info!("  Trades executed: {}", result.executed.len());
                if !result.executed.is_empty() {
                    for action in &result.executed {
                        info!(
                            "    {} {} lots of {} @ {}",
                            if action.is_buy() { "BUY" } else { "SELL" },
                            action.lots,
                            action.symbol,
                            action.estimated_price
                        );
                    }
                }
            }
            Err(e) => {
                error!("Rebalance failed: {}", e);
            }
        }
    }

    // Show final portfolio (state after planning or execution)
    match exchange.get_wallet(account_id).await {
        Ok(wallet) => {
            if cli.plan {
                info!("Current portfolio (unchanged - plan mode):");
            } else {
                info!("Final portfolio:");
            }
            info!("  Cash: {}", wallet.cash());
            if wallet.positions().is_empty() {
                info!("  Positions: none");
            } else {
                info!("  Positions:");
                for pos in wallet.positions() {
                    info!(
                        "    {}: {} units @ {} = {}",
                        pos.symbol(),
                        pos.quantity(),
                        pos.current_price(),
                        pos.market_value()
                    );
                }
            }
            info!("  Total value: {}", wallet.total_value());

            let alloc = wallet.current_allocation();
            if !alloc.is_empty() {
                info!("Current allocation:");
                for (symbol, pct) in alloc.iter() {
                    info!("    {}: {:.2}%", symbol, pct);
                }
            }
        }
        Err(e) => {
            error!("Failed to get wallet: {}", e);
        }
    }

    info!("Demo complete!");
}
