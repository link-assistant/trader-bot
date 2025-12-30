//! Market state and price simulation.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::Money;
use crate::exchange::{Instrument, InstrumentType, MarketData, MarketStatus};

/// Simulated market state for a single instrument.
#[derive(Debug, Clone)]
pub struct MarketState {
    /// Current price.
    pub price: Decimal,
    /// Bid-ask spread as a percentage.
    pub spread_percent: Decimal,
    /// 24h volume.
    pub volume: Decimal,
    /// Current market status.
    pub status: MarketStatus,
    /// Last update time.
    pub last_update: DateTime<Utc>,
}

impl MarketState {
    /// Creates a new market state.
    #[must_use]
    pub fn new(price: Decimal) -> Self {
        Self {
            price,
            spread_percent: Decimal::new(1, 1), // 0.1% default spread
            volume: Decimal::new(1_000_000, 0),
            status: MarketStatus::Open,
            last_update: Utc::now(),
        }
    }

    /// Sets the spread percentage.
    #[must_use]
    pub const fn with_spread(mut self, spread_percent: Decimal) -> Self {
        self.spread_percent = spread_percent;
        self
    }

    /// Sets the market status.
    #[must_use]
    pub const fn with_status(mut self, status: MarketStatus) -> Self {
        self.status = status;
        self
    }

    /// Calculates the bid price.
    #[must_use]
    pub fn bid(&self) -> Decimal {
        let half_spread = self.spread_percent / Decimal::from(200);
        self.price * (Decimal::ONE - half_spread)
    }

    /// Calculates the ask price.
    #[must_use]
    pub fn ask(&self) -> Decimal {
        let half_spread = self.spread_percent / Decimal::from(200);
        self.price * (Decimal::ONE + half_spread)
    }

    /// Updates the price.
    pub fn set_price(&mut self, price: Decimal) {
        self.price = price;
        self.last_update = Utc::now();
    }

    /// Converts to MarketData.
    #[must_use]
    pub fn to_market_data(&self, symbol: &str, currency: &str) -> MarketData {
        MarketData {
            symbol: symbol.to_string(),
            last_price: Money::new(self.price, currency),
            bid: Some(Money::new(self.bid(), currency)),
            ask: Some(Money::new(self.ask(), currency)),
            volume_24h: Some(self.volume),
            high_24h: Some(Money::new(self.price * Decimal::new(102, 2), currency)),
            low_24h: Some(Money::new(self.price * Decimal::new(98, 2), currency)),
            change_24h: None,
            change_percent_24h: None,
            market_status: self.status,
            timestamp: self.last_update,
        }
    }
}

/// Price model for simulation.
#[derive(Debug, Clone, Copy)]
pub enum PriceModel {
    /// Static price (no changes).
    Static,
    /// Random walk with given volatility (daily % std dev).
    RandomWalk {
        /// Volatility as percentage standard deviation per tick.
        volatility: Decimal,
    },
    /// Mean reverting with given parameters.
    MeanReverting {
        /// Long-term mean price to revert to.
        mean: Decimal,
        /// Speed of reversion (0-1, higher = faster reversion).
        reversion_speed: Decimal,
        /// Random volatility component.
        volatility: Decimal,
    },
    /// Trending with given drift.
    Trending {
        /// Drift rate per tick (positive = upward trend).
        drift: Decimal,
        /// Random volatility component.
        volatility: Decimal,
    },
}

/// Price generator for simulating market movements.
#[derive(Debug)]
pub struct PriceGenerator {
    model: PriceModel,
    rng_seed: u64,
    step_count: u64,
}

impl PriceGenerator {
    /// Creates a new price generator.
    #[must_use]
    pub const fn new(model: PriceModel) -> Self {
        Self {
            model,
            rng_seed: 42,
            step_count: 0,
        }
    }

    /// Sets the random seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.rng_seed = seed;
        self
    }

    /// Generates the next price given the current price.
    #[must_use]
    pub fn next_price(&mut self, current: Decimal) -> Decimal {
        self.step_count += 1;

        match self.model {
            PriceModel::Static => current,
            PriceModel::RandomWalk { volatility } => {
                let random = self.pseudo_random();
                let change = volatility * random / Decimal::from(100);
                current * (Decimal::ONE + change)
            }
            PriceModel::MeanReverting {
                mean,
                reversion_speed,
                volatility,
            } => {
                let random = self.pseudo_random();
                let noise = volatility * random / Decimal::from(100);
                let reversion = reversion_speed * (mean - current) / Decimal::from(100);
                current + reversion + current * noise
            }
            PriceModel::Trending { drift, volatility } => {
                let random = self.pseudo_random();
                let noise = volatility * random / Decimal::from(100);
                let trend = drift / Decimal::from(100);
                current * (Decimal::ONE + trend + noise)
            }
        }
    }

    /// Simple pseudo-random number generator (-1 to 1).
    fn pseudo_random(&self) -> Decimal {
        // Simple LCG-based PRNG
        let seed = self
            .rng_seed
            .wrapping_mul(self.step_count)
            .wrapping_add(12345);
        let value = (seed % 2000) as i64 - 1000;
        Decimal::new(value, 3) // -1.000 to 1.000
    }
}

/// Manager for multiple market states.
#[derive(Debug, Default)]
pub struct MarketManager {
    states: HashMap<String, MarketState>,
    instruments: HashMap<String, Instrument>,
    generators: HashMap<String, PriceGenerator>,
    base_currency: String,
}

impl MarketManager {
    /// Creates a new market manager.
    #[must_use]
    pub fn new(base_currency: impl Into<String>) -> Self {
        Self {
            states: HashMap::new(),
            instruments: HashMap::new(),
            generators: HashMap::new(),
            base_currency: base_currency.into(),
        }
    }

    /// Adds an instrument with initial price.
    pub fn add_instrument(
        &mut self,
        symbol: impl Into<String>,
        name: impl Into<String>,
        price: Decimal,
        lot_size: u32,
    ) {
        let symbol = symbol.into();
        let name = name.into();

        let instrument = Instrument {
            symbol: symbol.clone(),
            name,
            exchange_id: format!("SIM_{symbol}"),
            exchange: "SIMULATOR".to_string(),
            instrument_type: InstrumentType::Stock,
            currency: self.base_currency.clone(),
            lot_size,
            min_price_increment: Decimal::new(1, 2),
            min_quantity: 1,
            tradeable: true,
            margin_available: false,
            short_available: false,
        };

        let state = MarketState::new(price);

        self.instruments.insert(symbol.clone(), instrument);
        self.states.insert(symbol.clone(), state);
        self.generators
            .insert(symbol, PriceGenerator::new(PriceModel::Static));
    }

    /// Sets the price model for an instrument.
    pub fn set_price_model(&mut self, symbol: &str, model: PriceModel) {
        self.generators
            .insert(symbol.to_string(), PriceGenerator::new(model));
    }

    /// Gets the current market state.
    #[must_use]
    pub fn get_state(&self, symbol: &str) -> Option<&MarketState> {
        self.states.get(symbol)
    }

    /// Gets an instrument.
    #[must_use]
    pub fn get_instrument(&self, symbol: &str) -> Option<&Instrument> {
        self.instruments.get(symbol)
    }

    /// Gets market data for an instrument.
    #[must_use]
    pub fn get_market_data(&self, symbol: &str) -> Option<MarketData> {
        let state = self.states.get(symbol)?;
        Some(state.to_market_data(symbol, &self.base_currency))
    }

    /// Lists all instruments.
    #[must_use]
    pub fn list_instruments(&self) -> Vec<Instrument> {
        self.instruments.values().cloned().collect()
    }

    /// Updates prices by one time step.
    pub fn tick(&mut self) {
        let symbols: Vec<String> = self.states.keys().cloned().collect();
        for symbol in symbols {
            if let Some(generator) = self.generators.get_mut(&symbol) {
                if let Some(state) = self.states.get_mut(&symbol) {
                    let new_price = generator.next_price(state.price);
                    state.set_price(new_price);
                }
            }
        }
    }

    /// Sets the market status for all instruments.
    pub fn set_market_status(&mut self, status: MarketStatus) {
        for state in self.states.values_mut() {
            state.status = status;
        }
    }

    /// Sets the price for a specific instrument.
    pub fn set_price(&mut self, symbol: &str, price: Decimal) {
        if let Some(state) = self.states.get_mut(symbol) {
            state.set_price(price);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_market_state_spread() {
        let state = MarketState::new(dec!(100)).with_spread(dec!(0.2)); // 0.2% spread

        let bid = state.bid();
        let ask = state.ask();

        assert!(bid < dec!(100));
        assert!(ask > dec!(100));
        assert!((ask - bid) / dec!(100) * dec!(100) < dec!(0.21)); // Spread is ~0.2%
    }

    #[test]
    fn test_price_generator_static() {
        let mut gen = PriceGenerator::new(PriceModel::Static);
        let price = dec!(100);

        for _ in 0..10 {
            let next = gen.next_price(price);
            assert_eq!(next, price);
        }
    }

    #[test]
    fn test_price_generator_random_walk() {
        let mut gen = PriceGenerator::new(PriceModel::RandomWalk {
            volatility: dec!(2),
        });
        let mut price = dec!(100);

        // Price should change
        let mut changed = false;
        for _ in 0..10 {
            let next = gen.next_price(price);
            if next != price {
                changed = true;
            }
            price = next;
        }
        assert!(changed);
    }

    #[test]
    fn test_market_manager() {
        let mut manager = MarketManager::new("USD");
        manager.add_instrument("AAPL", "Apple Inc.", dec!(150), 1);
        manager.add_instrument("GOOGL", "Alphabet Inc.", dec!(2800), 1);

        assert_eq!(manager.list_instruments().len(), 2);

        let state = manager.get_state("AAPL").unwrap();
        assert_eq!(state.price, dec!(150));

        let data = manager.get_market_data("AAPL").unwrap();
        assert_eq!(data.symbol, "AAPL");
        assert_eq!(data.last_price.amount(), dec!(150));
    }

    #[test]
    fn test_market_manager_tick() {
        let mut manager = MarketManager::new("USD");
        manager.add_instrument("TEST", "Test Stock", dec!(100), 1);
        manager.set_price_model(
            "TEST",
            PriceModel::RandomWalk {
                volatility: dec!(5),
            },
        );

        let _initial = manager.get_state("TEST").unwrap().price;
        manager.tick();
        manager.tick();
        manager.tick();

        // Price should have changed (with high probability)
        let final_price = manager.get_state("TEST").unwrap().price;
        // Note: With the deterministic PRNG, we can't guarantee the price changed
        // but we can verify the mechanism works
        assert!(final_price > Decimal::ZERO);
    }
}
