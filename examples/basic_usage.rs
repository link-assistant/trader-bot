//! Basic usage example for balancer-trader-bot.
//!
//! This example demonstrates portfolio rebalancing with a simulated exchange.
//!
//! Run with: `cargo run --example basic_usage`

use balancer_trader_bot::balancer::{BalancerConfig, BalancerEngine};
use balancer_trader_bot::domain::DesiredAllocation;
use balancer_trader_bot::exchange::ExchangeProvider;
use balancer_trader_bot::simulator::SimulatedExchange;
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Balancer Trader Bot - Basic Usage Example");
    println!("==========================================\n");

    // Create a simulated exchange
    let exchange = SimulatedExchange::new("USD");

    // Add some instruments (stocks)
    exchange.add_instrument("AAPL", "Apple Inc.", dec!(150), 1);
    exchange.add_instrument("GOOGL", "Alphabet Inc.", dec!(2800), 1);
    exchange.add_instrument("MSFT", "Microsoft Corp.", dec!(400), 1);
    exchange.add_instrument("AMZN", "Amazon.com Inc.", dec!(180), 1);

    println!("Available instruments:");
    println!("  AAPL  (Apple)     @ $150");
    println!("  GOOGL (Alphabet)  @ $2800");
    println!("  MSFT  (Microsoft) @ $400");
    println!("  AMZN  (Amazon)    @ $180");
    println!();

    // Create an account with initial cash
    let account_id = "demo_account";
    exchange.create_account(account_id, dec!(100000));
    println!("Created account with $100,000 initial cash\n");

    // Define target allocation
    let mut target = DesiredAllocation::new();
    target.set("AAPL", dec!(30)); // 30% Apple
    target.set("GOOGL", dec!(25)); // 25% Alphabet
    target.set("MSFT", dec!(25)); // 25% Microsoft
    target.set("AMZN", dec!(20)); // 20% Amazon

    println!("Target allocation:");
    println!("  AAPL:  30%");
    println!("  GOOGL: 25%");
    println!("  MSFT:  25%");
    println!("  AMZN:  20%");
    println!();

    // Create the balancer engine
    let exchange = Arc::new(exchange);
    let config = BalancerConfig {
        dry_run: false,
        order_delay_ms: 100, // Small delay between orders
        ..BalancerConfig::default()
    };
    let mut engine = BalancerEngine::new(Arc::clone(&exchange), config);

    // Run the rebalancing
    println!("Running initial rebalance...\n");
    match engine.rebalance(account_id, &target).await {
        Ok(result) => {
            println!("Rebalance completed successfully!");
            println!("  Trades executed: {}", result.executed.len());

            if !result.executed.is_empty() {
                println!("\nExecuted trades:");
                for action in &result.executed {
                    let direction = if action.is_buy() { "BUY" } else { "SELL" };
                    println!(
                        "  {} {} lots of {} @ {}",
                        direction, action.lots, action.symbol, action.estimated_price
                    );
                }
            }
        }
        Err(e) => {
            println!("Rebalance failed: {}", e);
        }
    }

    // Show final portfolio state
    println!("\nFinal Portfolio:");
    println!("================");
    match exchange.get_wallet(account_id).await {
        Ok(wallet) => {
            println!("Cash: {}", wallet.cash());
            println!("\nPositions:");
            for pos in wallet.positions() {
                println!(
                    "  {}: {} units @ {} = {}",
                    pos.symbol(),
                    pos.quantity(),
                    pos.current_price(),
                    pos.market_value()
                );
            }
            println!("\nTotal Value: {}", wallet.total_value());

            println!("\nActual Allocation:");
            let alloc = wallet.current_allocation();
            for (symbol, pct) in alloc.iter() {
                println!("  {}: {:.2}%", symbol, pct);
            }
            println!("  Cash: {:.2}%", wallet.cash_percentage());
        }
        Err(e) => {
            println!("Failed to get wallet: {}", e);
        }
    }

    println!("\nExample complete!");
}
