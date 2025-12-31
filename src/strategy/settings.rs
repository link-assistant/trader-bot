//! Trading settings for configuring strategy behavior.

use chrono::NaiveTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Configuration settings for trading strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSettings {
    /// The instrument to trade.
    pub instrument: String,

    /// Minimum profit in price steps before considering a sell.
    pub minimum_profit_steps: u32,

    /// Minimum market order size (in lots) to consider buying.
    pub minimum_market_order_size_to_buy: u64,

    /// Minimum market order size (in lots) to consider selling.
    pub minimum_market_order_size_to_sell: u64,

    /// Minimum market order size to change buy price.
    pub minimum_market_order_size_to_change_buy_price: u64,

    /// Minimum market order size to change sell price.
    pub minimum_market_order_size_to_change_sell_price: u64,

    /// Order book depth to analyze.
    pub market_order_book_depth: usize,

    /// Maximum number of lots to hold.
    pub max_position_lots: u64,

    /// Lot size for orders.
    pub lot_size: u64,

    /// Earliest time to start buying.
    pub minimum_time_to_buy: Option<NaiveTime>,

    /// Latest time to buy.
    pub maximum_time_to_buy: Option<NaiveTime>,

    /// Delta for early sell (sell if price drops by this many steps).
    pub early_sell_owned_lots_delta: Option<u32>,

    /// Multiplier for early sell.
    pub early_sell_owned_lots_multiplier: Option<Decimal>,

    /// Price step for the instrument (tick size).
    pub price_step: Decimal,

    /// Whether trading is enabled.
    pub enabled: bool,
}

impl Default for TradingSettings {
    fn default() -> Self {
        Self {
            instrument: String::new(),
            minimum_profit_steps: 1,
            minimum_market_order_size_to_buy: 10,
            minimum_market_order_size_to_sell: 10,
            minimum_market_order_size_to_change_buy_price: 50,
            minimum_market_order_size_to_change_sell_price: 50,
            market_order_book_depth: 5,
            max_position_lots: 100,
            lot_size: 1,
            minimum_time_to_buy: None,
            maximum_time_to_buy: None,
            early_sell_owned_lots_delta: None,
            early_sell_owned_lots_multiplier: None,
            price_step: Decimal::new(1, 2), // 0.01
            enabled: true,
        }
    }
}

impl TradingSettings {
    /// Creates new trading settings for an instrument.
    #[must_use]
    pub fn new(instrument: impl Into<String>) -> Self {
        Self {
            instrument: instrument.into(),
            ..Default::default()
        }
    }

    /// Builder method to set minimum profit steps.
    #[must_use]
    pub fn with_minimum_profit_steps(mut self, steps: u32) -> Self {
        self.minimum_profit_steps = steps;
        self
    }

    /// Builder method to set price step.
    #[must_use]
    pub fn with_price_step(mut self, step: Decimal) -> Self {
        self.price_step = step;
        self
    }

    /// Builder method to set lot size.
    #[must_use]
    pub fn with_lot_size(mut self, size: u64) -> Self {
        self.lot_size = size;
        self
    }

    /// Builder method to set max position.
    #[must_use]
    pub fn with_max_position(mut self, lots: u64) -> Self {
        self.max_position_lots = lots;
        self
    }

    /// Builder method to set order book depth.
    #[must_use]
    pub fn with_order_book_depth(mut self, depth: usize) -> Self {
        self.market_order_book_depth = depth;
        self
    }

    /// Builder method to set minimum market order size to buy.
    #[must_use]
    pub fn with_min_order_size_to_buy(mut self, size: u64) -> Self {
        self.minimum_market_order_size_to_buy = size;
        self
    }

    /// Builder method to set minimum market order size to sell.
    #[must_use]
    pub fn with_min_order_size_to_sell(mut self, size: u64) -> Self {
        self.minimum_market_order_size_to_sell = size;
        self
    }

    /// Builder method to set trading time window.
    #[must_use]
    pub fn with_trading_hours(mut self, start: NaiveTime, end: NaiveTime) -> Self {
        self.minimum_time_to_buy = Some(start);
        self.maximum_time_to_buy = Some(end);
        self
    }

    /// Builder method to enable/disable trading.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns the minimum profit amount.
    #[must_use]
    pub fn minimum_profit(&self) -> Decimal {
        self.price_step * Decimal::from(self.minimum_profit_steps)
    }

    /// Checks if trading is allowed at the given time.
    #[must_use]
    pub fn is_trading_time(&self, time: NaiveTime) -> bool {
        match (self.minimum_time_to_buy, self.maximum_time_to_buy) {
            (Some(start), Some(end)) => time >= start && time <= end,
            (Some(start), None) => time >= start,
            (None, Some(end)) => time <= end,
            (None, None) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = TradingSettings::default();
        assert_eq!(settings.minimum_profit_steps, 1);
        assert!(settings.enabled);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = TradingSettings::new("TEST")
            .with_minimum_profit_steps(2)
            .with_price_step(Decimal::new(1, 1))
            .with_max_position(50);

        assert_eq!(settings.instrument, "TEST");
        assert_eq!(settings.minimum_profit_steps, 2);
        assert_eq!(settings.price_step, Decimal::new(1, 1));
        assert_eq!(settings.max_position_lots, 50);
    }

    #[test]
    fn test_minimum_profit() {
        let settings = TradingSettings::new("TEST")
            .with_minimum_profit_steps(3)
            .with_price_step(Decimal::new(1, 2)); // 0.01

        assert_eq!(settings.minimum_profit(), Decimal::new(3, 2)); // 0.03
    }

    #[test]
    fn test_trading_time() {
        let settings = TradingSettings::new("TEST").with_trading_hours(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        );

        assert!(settings.is_trading_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()));
        assert!(!settings.is_trading_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap()));
        assert!(!settings.is_trading_time(NaiveTime::from_hms_opt(18, 0, 0).unwrap()));
    }

    #[test]
    fn test_no_trading_time_restrictions() {
        let settings = TradingSettings::new("TEST");
        assert!(settings.is_trading_time(NaiveTime::from_hms_opt(3, 0, 0).unwrap()));
        assert!(settings.is_trading_time(NaiveTime::from_hms_opt(23, 0, 0).unwrap()));
    }
}
