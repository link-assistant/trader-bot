//! Scalping trading strategy implementation.
//!
//! The scalping strategy aims to profit from small price movements through
//! rapid buy-sell cycles. It places buy orders at the best bid price and
//! sell orders at a price that ensures minimum profit.

use super::settings::TradingSettings;
use super::traits::{MarketState, Strategy, StrategyAction, StrategyDecision};
use crate::domain::Order;
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{debug, info, trace};

/// Tracks lots owned and their purchase prices.
#[derive(Debug, Clone, Default)]
struct OwnedLots {
    /// Maps purchase price to quantity at that price.
    lots_by_price: HashMap<String, (Decimal, u32)>,
}

impl OwnedLots {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, price: Decimal, quantity: u32) {
        let key = price.to_string();
        let entry = self.lots_by_price.entry(key).or_insert((price, 0));
        entry.1 += quantity;
    }

    fn remove(&mut self, quantity: u32) -> Option<Decimal> {
        // Remove from the lowest priced lots first (FIFO by price)
        let mut to_remove = quantity;
        let mut removed_price = None;

        let mut keys_to_update: Vec<(String, u32)> = Vec::new();

        // Sort keys by price to ensure FIFO
        let mut sorted_entries: Vec<_> = self.lots_by_price.iter().collect();
        sorted_entries.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));

        for (key, (price, qty)) in sorted_entries {
            if to_remove == 0 {
                break;
            }

            if *qty <= to_remove {
                to_remove -= qty;
                keys_to_update.push((key.clone(), 0));
            } else {
                keys_to_update.push((key.clone(), qty - to_remove));
                to_remove = 0;
            }

            removed_price = Some(*price);
        }

        for (key, new_qty) in keys_to_update {
            if new_qty == 0 {
                self.lots_by_price.remove(&key);
            } else if let Some(entry) = self.lots_by_price.get_mut(&key) {
                entry.1 = new_qty;
            }
        }

        removed_price
    }

    fn total_quantity(&self) -> u32 {
        self.lots_by_price.values().map(|(_, qty)| qty).sum()
    }

    fn lowest_price(&self) -> Option<Decimal> {
        self.lots_by_price
            .values()
            .min_by(|a, b| a.0.cmp(&b.0))
            .map(|(price, _)| *price)
    }

    fn clear(&mut self) {
        self.lots_by_price.clear();
    }
}

/// Scalping trading strategy.
///
/// This strategy implements a simple scalping approach:
/// 1. Place buy orders at the best bid price when conditions are favorable
/// 2. Track purchased lots and their prices
/// 3. Place sell orders at a price that ensures minimum profit
/// 4. Adjust orders as market conditions change
#[derive(Debug)]
pub struct ScalpingStrategy {
    settings: TradingSettings,
    owned_lots: OwnedLots,
    active_buy_order: Option<String>,
    active_sell_orders: HashMap<String, Decimal>,
}

impl ScalpingStrategy {
    /// Creates a new scalping strategy with the given settings.
    #[must_use]
    pub fn new(settings: TradingSettings) -> Self {
        Self {
            settings,
            owned_lots: OwnedLots::new(),
            active_buy_order: None,
            active_sell_orders: HashMap::new(),
        }
    }

    /// Returns the settings.
    #[must_use]
    pub fn settings(&self) -> &TradingSettings {
        &self.settings
    }

    /// Checks if we should place a buy order.
    fn should_buy(&self, state: &MarketState) -> bool {
        // Check if trading is enabled
        if !self.settings.enabled {
            trace!("Trading disabled");
            return false;
        }

        // Check if we're within trading hours
        let current_time = state.timestamp.time();
        if !self.settings.is_trading_time(current_time) {
            trace!("Outside trading hours");
            return false;
        }

        // Check if we already have a buy order
        if self.active_buy_order.is_some() {
            trace!("Already have active buy order");
            return false;
        }

        // Check if we're at max position
        let current_position = u64::from(self.owned_lots.total_quantity());
        if current_position >= self.settings.max_position_lots {
            trace!("At max position: {}", current_position);
            return false;
        }

        // Check market liquidity from order book
        if let Some(ref order_book) = state.order_book {
            let bid_depth = order_book.bid_depth(self.settings.market_order_book_depth);
            if bid_depth < self.settings.minimum_market_order_size_to_buy {
                trace!("Insufficient bid liquidity: {}", bid_depth);
                return false;
            }
        }

        // Check if we have enough cash
        if let Some(best_ask) = state.best_ask() {
            let lot_cost = best_ask * Decimal::from(self.settings.lot_size);
            if state.available_cash < lot_cost {
                trace!("Insufficient cash: {} < {}", state.available_cash, lot_cost);
                return false;
            }
        }

        true
    }

    /// Determines the buy price based on order book.
    fn get_buy_price(&self, state: &MarketState) -> Option<Decimal> {
        state.best_bid()
    }

    /// Calculates the sell price for owned lots.
    fn get_sell_price(&self, buy_price: Decimal) -> Decimal {
        buy_price + self.settings.minimum_profit()
    }

    /// Checks if we should place a sell order for owned lots.
    fn should_sell(&self, state: &MarketState) -> bool {
        // Check if we have lots to sell
        if self.owned_lots.total_quantity() == 0 {
            return false;
        }

        // Check market liquidity from order book
        if let Some(ref order_book) = state.order_book {
            let ask_depth = order_book.ask_depth(self.settings.market_order_book_depth);
            if ask_depth < self.settings.minimum_market_order_size_to_sell {
                trace!("Insufficient ask liquidity: {}", ask_depth);
                return false;
            }
        }

        true
    }

    /// Checks if we should adjust existing buy order price.
    fn should_adjust_buy_order(&self, state: &MarketState) -> Option<(String, Decimal)> {
        let order_id = self.active_buy_order.as_ref()?;

        // Find the current order
        let current_order = state.active_orders.iter().find(|o| o.id() == order_id)?;

        let current_price = current_order.limit_price()?.amount();
        let best_bid = state.best_bid()?;

        // Check if best bid has changed and has enough liquidity
        if best_bid != current_price {
            if let Some(ref order_book) = state.order_book {
                let bid_depth = order_book.bid_depth(self.settings.market_order_book_depth);
                if bid_depth >= self.settings.minimum_market_order_size_to_change_buy_price {
                    return Some((order_id.clone(), best_bid));
                }
            }
        }

        None
    }

    /// Checks if we should adjust existing sell order price.
    fn should_adjust_sell_order(&self, state: &MarketState) -> Option<(String, Decimal)> {
        // Find a sell order that could be improved
        for (order_id, _target_price) in &self.active_sell_orders {
            let order = state.active_orders.iter().find(|o| o.id() == order_id)?;

            let current_price = order.limit_price()?.amount();
            let best_ask = state.best_ask()?;

            // If best ask is lower than our sell price and still profitable
            if best_ask < current_price {
                if let Some(lowest_buy) = self.owned_lots.lowest_price() {
                    let min_sell = self.get_sell_price(lowest_buy);
                    if best_ask >= min_sell {
                        if let Some(ref order_book) = state.order_book {
                            let ask_depth =
                                order_book.ask_depth(self.settings.market_order_book_depth);
                            if ask_depth
                                >= self.settings.minimum_market_order_size_to_change_sell_price
                            {
                                return Some((order_id.clone(), best_ask));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Creates actions for the current market state.
    fn create_actions(&self, state: &MarketState) -> StrategyDecision {
        let mut decision = StrategyDecision::new();

        // Check if we should adjust buy order
        if let Some((order_id, new_price)) = self.should_adjust_buy_order(state) {
            debug!("Adjusting buy order {} to price {}", order_id, new_price);
            decision.add_action(StrategyAction::cancel(order_id));
            decision.add_action(StrategyAction::limit_buy(
                &self.settings.instrument,
                self.settings.lot_size as u32,
                new_price,
            ));
            return decision.with_reason("Adjusting buy order to new best bid");
        }

        // Check if we should adjust sell order
        if let Some((order_id, new_price)) = self.should_adjust_sell_order(state) {
            debug!("Adjusting sell order {} to price {}", order_id, new_price);
            decision.add_action(StrategyAction::cancel(order_id));
            decision.add_action(StrategyAction::limit_sell(
                &self.settings.instrument,
                self.owned_lots.total_quantity(),
                new_price,
            ));
            return decision.with_reason("Adjusting sell order to better price");
        }

        // Check if we should place a new buy order
        if self.should_buy(state) {
            if let Some(buy_price) = self.get_buy_price(state) {
                debug!("Placing buy order at price {}", buy_price);
                decision.add_action(StrategyAction::limit_buy(
                    &self.settings.instrument,
                    self.settings.lot_size as u32,
                    buy_price,
                ));
                return decision.with_reason("Placing new buy order at best bid");
            }
        }

        // Check if we should place a sell order
        if self.should_sell(state) && self.active_sell_orders.is_empty() {
            if let Some(lowest_buy) = self.owned_lots.lowest_price() {
                let sell_price = self.get_sell_price(lowest_buy);
                debug!("Placing sell order at price {}", sell_price);
                decision.add_action(StrategyAction::limit_sell(
                    &self.settings.instrument,
                    self.owned_lots.total_quantity(),
                    sell_price,
                ));
                return decision.with_reason("Placing new sell order with minimum profit");
            }
        }

        StrategyDecision::hold_with_reason("No action needed")
    }
}

#[async_trait]
impl Strategy for ScalpingStrategy {
    fn name(&self) -> &str {
        "Scalping Strategy"
    }

    fn id(&self) -> &str {
        &self.settings.instrument
    }

    async fn decide(&self, state: &MarketState) -> StrategyDecision {
        trace!(
            "Deciding for {} - position: {} lots, cash: {}",
            state.symbol,
            state.position_lots(),
            state.available_cash
        );

        self.create_actions(state)
    }

    async fn on_order_filled(&mut self, order: &Order) {
        match order.direction() {
            crate::domain::OrderDirection::Buy => {
                // Add to owned lots
                if let Some(price) = order.limit_price() {
                    self.owned_lots.add(price.amount(), order.filled_lots());
                    info!(
                        "Bought {} lots at {} - total owned: {}",
                        order.filled_lots(),
                        price.amount(),
                        self.owned_lots.total_quantity()
                    );
                }
                // Clear active buy order
                self.active_buy_order = None;
            }
            crate::domain::OrderDirection::Sell => {
                // Remove from owned lots
                self.owned_lots.remove(order.filled_lots());
                info!(
                    "Sold {} lots - remaining owned: {}",
                    order.filled_lots(),
                    self.owned_lots.total_quantity()
                );
                // Remove from active sell orders
                self.active_sell_orders.remove(order.id());
            }
        }
    }

    async fn on_order_cancelled(&mut self, order: &Order) {
        match order.direction() {
            crate::domain::OrderDirection::Buy => {
                if self
                    .active_buy_order
                    .as_ref()
                    .is_some_and(|id| id == order.id())
                {
                    self.active_buy_order = None;
                }
            }
            crate::domain::OrderDirection::Sell => {
                self.active_sell_orders.remove(order.id());
            }
        }
    }

    async fn reset(&mut self) {
        self.owned_lots.clear();
        self.active_buy_order = None;
        self.active_sell_orders.clear();
    }

    fn symbols(&self) -> Vec<String> {
        vec![self.settings.instrument.clone()]
    }

    fn is_enabled(&self) -> bool {
        self.settings.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_settings() -> TradingSettings {
        TradingSettings::new("TEST")
            .with_minimum_profit_steps(2)
            .with_price_step(Decimal::new(1, 2)) // 0.01
            .with_lot_size(1)
            .with_max_position(10)
            .with_min_order_size_to_buy(5)
            .with_min_order_size_to_sell(5)
            .with_order_book_depth(3)
    }

    #[tokio::test]
    async fn test_strategy_creation() {
        let settings = create_test_settings();
        let strategy = ScalpingStrategy::new(settings);
        assert_eq!(strategy.name(), "Scalping Strategy");
    }

    #[tokio::test]
    async fn test_sell_price_calculation() {
        let settings = create_test_settings();
        let strategy = ScalpingStrategy::new(settings);

        let buy_price = dec!(100.00);
        let sell_price = strategy.get_sell_price(buy_price);

        // With 2 profit steps and 0.01 step size, sell should be 0.02 higher
        assert_eq!(sell_price, dec!(100.02));
    }

    #[tokio::test]
    async fn test_strategy_reset() {
        let settings = create_test_settings();
        let mut strategy = ScalpingStrategy::new(settings);

        // Add some state
        strategy.owned_lots.add(dec!(100.0), 10);
        strategy.active_buy_order = Some("test-order".to_string());

        // Reset
        strategy.reset().await;

        assert_eq!(strategy.owned_lots.total_quantity(), 0);
        assert!(strategy.active_buy_order.is_none());
        assert!(strategy.active_sell_orders.is_empty());
    }

    #[test]
    fn test_owned_lots_fifo() {
        let mut lots = OwnedLots::new();

        // Add lots at different prices
        lots.add(dec!(100.0), 5);
        lots.add(dec!(101.0), 5);

        assert_eq!(lots.total_quantity(), 10);

        // Remove should take from lowest price first
        lots.remove(3);
        assert_eq!(lots.total_quantity(), 7);

        // Lowest price should still be 100 (but only 2 lots)
        let lowest = lots.lowest_price().unwrap();
        assert_eq!(lowest, dec!(100.0));
    }

    #[test]
    fn test_symbols() {
        let settings = TradingSettings::new("AAPL");
        let strategy = ScalpingStrategy::new(settings);
        assert_eq!(strategy.symbols(), vec!["AAPL".to_string()]);
    }
}
