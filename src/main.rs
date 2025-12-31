//! Trader Bot - CLI application.
//!
//! This is the command-line interface for the configurable trading bot.

use rust_decimal_macros::dec;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use trader_bot::domain::DesiredAllocation;
use trader_bot::prelude::*;
use trader_bot::simulator::SimulatedExchange;

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    info!("Trader Bot v{}", trader_bot::VERSION);
    info!("Starting portfolio balancer demo...");

    // Demo with simulated exchange
    demo_simulation().await;
}

/// Demonstrates the balancer with a simulated exchange.
async fn demo_simulation() {
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

    // Create balancer
    let exchange = Arc::new(exchange);
    let config = BalancerConfig {
        dry_run: false,
        order_delay_ms: 100,
        ..BalancerConfig::default()
    };
    let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

    // Run initial rebalance
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
            info!("Rebalance failed: {}", e);
        }
    }

    // Show final portfolio
    match exchange.get_wallet(account_id).await {
        Ok(wallet) => {
            info!("Final portfolio:");
            info!("  Cash: {}", wallet.cash());
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
            info!("  Total value: {}", wallet.total_value());

            let alloc = wallet.current_allocation();
            info!("Current allocation:");
            for (symbol, pct) in alloc.iter() {
                info!("    {}: {:.2}%", symbol, pct);
            }
        }
        Err(e) => {
            info!("Failed to get wallet: {}", e);
        }
    }

    info!("Demo complete!");
}
