//! Exchange-related types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::Money;

/// Type of financial instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    /// Stock/equity.
    Stock,
    /// Exchange-traded fund.
    Etf,
    /// Bond.
    Bond,
    /// Currency pair.
    Currency,
    /// Futures contract.
    Futures,
    /// Options contract.
    Options,
    /// Cryptocurrency.
    Crypto,
    /// Other/unknown type.
    Other,
}

/// Current market status for an instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    /// Market is open for trading.
    Open,
    /// Market is closed.
    Closed,
    /// Pre-market session.
    PreMarket,
    /// After-hours session.
    AfterHours,
    /// Trading is halted.
    Halted,
    /// Status is unknown.
    Unknown,
}

impl MarketStatus {
    /// Returns true if trading is possible.
    #[must_use]
    pub const fn is_tradeable(&self) -> bool {
        matches!(self, Self::Open | Self::PreMarket | Self::AfterHours)
    }
}

/// Information about a tradeable instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instrument {
    /// Trading symbol (e.g., "AAPL", "BTC-USD").
    pub symbol: String,
    /// Full name of the instrument.
    pub name: String,
    /// Unique identifier used by the exchange (e.g., FIGI).
    pub exchange_id: String,
    /// Exchange where the instrument is traded.
    pub exchange: String,
    /// Type of instrument.
    pub instrument_type: InstrumentType,
    /// Base currency for the instrument.
    pub currency: String,
    /// Minimum lot size.
    pub lot_size: u32,
    /// Minimum price increment.
    pub min_price_increment: Decimal,
    /// Minimum order quantity in lots.
    pub min_quantity: u32,
    /// Whether the instrument is currently tradeable.
    pub tradeable: bool,
    /// Whether margin trading is allowed.
    pub margin_available: bool,
    /// Whether short selling is allowed.
    pub short_available: bool,
}

impl Instrument {
    /// Returns the quantity in units for the given number of lots.
    #[must_use]
    pub fn lots_to_units(&self, lots: u32) -> Decimal {
        Decimal::from(lots) * Decimal::from(self.lot_size)
    }

    /// Returns the number of complete lots for the given quantity.
    #[must_use]
    pub fn units_to_lots(&self, units: Decimal) -> u32 {
        (units / Decimal::from(self.lot_size))
            .floor()
            .to_string()
            .parse()
            .unwrap_or(0)
    }

    /// Rounds a price to the minimum price increment.
    #[must_use]
    pub fn round_price(&self, price: Decimal) -> Decimal {
        if self.min_price_increment.is_zero() {
            return price;
        }
        let steps = (price / self.min_price_increment).round();
        steps * self.min_price_increment
    }
}

/// Real-time market data for an instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketData {
    /// Trading symbol.
    pub symbol: String,
    /// Last traded price.
    pub last_price: Money,
    /// Best bid price.
    pub bid: Option<Money>,
    /// Best ask price.
    pub ask: Option<Money>,
    /// 24-hour volume.
    pub volume_24h: Option<Decimal>,
    /// 24-hour high.
    pub high_24h: Option<Money>,
    /// 24-hour low.
    pub low_24h: Option<Money>,
    /// Price change in the last 24 hours.
    pub change_24h: Option<Money>,
    /// Percentage change in the last 24 hours.
    pub change_percent_24h: Option<Decimal>,
    /// Current market status.
    pub market_status: MarketStatus,
    /// Timestamp of the data.
    pub timestamp: DateTime<Utc>,
}

impl MarketData {
    /// Returns the mid price (average of bid and ask).
    #[must_use]
    pub fn mid_price(&self) -> Option<Money> {
        let bid = self.bid.as_ref()?;
        let ask = self.ask.as_ref()?;
        let mid = (bid.amount() + ask.amount()) / Decimal::from(2);
        Some(Money::new(mid, bid.currency()))
    }

    /// Returns the spread (ask - bid).
    #[must_use]
    pub fn spread(&self) -> Option<Money> {
        let bid = self.bid.as_ref()?;
        let ask = self.ask.as_ref()?;
        ask.checked_sub(bid)
    }

    /// Returns the spread as a percentage of the mid price.
    #[must_use]
    pub fn spread_percent(&self) -> Option<Decimal> {
        let mid = self.mid_price()?;
        let spread = self.spread()?;
        if mid.is_zero() {
            return None;
        }
        Some(spread.amount() / mid.amount() * Decimal::from(100))
    }
}

/// Ticker data (simplified market data).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticker {
    /// Trading symbol.
    pub symbol: String,
    /// Last traded price.
    pub price: Money,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// Single entry in an order book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBookEntry {
    /// Price level.
    pub price: Money,
    /// Quantity available at this price.
    pub quantity: Decimal,
}

/// Order book (market depth).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBook {
    /// Trading symbol.
    pub symbol: String,
    /// Bid levels (buyers), sorted from highest to lowest price.
    pub bids: Vec<OrderBookEntry>,
    /// Ask levels (sellers), sorted from lowest to highest price.
    pub asks: Vec<OrderBookEntry>,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    /// Creates an empty order book.
    #[must_use]
    pub fn empty(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Creates an order book with the given levels.
    #[must_use]
    pub fn with_levels(
        symbol: impl Into<String>,
        bids: Vec<OrderBookEntry>,
        asks: Vec<OrderBookEntry>,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            bids,
            asks,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Returns the best bid (highest buy price).
    #[must_use]
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bids.first()
    }

    /// Returns the best ask (lowest sell price).
    #[must_use]
    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.asks.first()
    }

    /// Returns the best bid price.
    #[must_use]
    pub fn best_bid_price(&self) -> Option<Decimal> {
        self.bids.first().map(|e| e.price.amount())
    }

    /// Returns the best ask price.
    #[must_use]
    pub fn best_ask_price(&self) -> Option<Decimal> {
        self.asks.first().map(|e| e.price.amount())
    }

    /// Returns the spread.
    #[must_use]
    pub fn spread(&self) -> Option<Money> {
        let bid = self.best_bid()?;
        let ask = self.best_ask()?;
        ask.price.checked_sub(&bid.price)
    }

    /// Returns the total bid depth up to the given level count.
    #[must_use]
    pub fn bid_depth(&self, levels: usize) -> u64 {
        self.bids
            .iter()
            .take(levels)
            .map(|e| e.quantity.floor().to_string().parse::<u64>().unwrap_or(0))
            .sum()
    }

    /// Returns the total ask depth up to the given level count.
    #[must_use]
    pub fn ask_depth(&self, levels: usize) -> u64 {
        self.asks
            .iter()
            .take(levels)
            .map(|e| e.quantity.floor().to_string().parse::<u64>().unwrap_or(0))
            .sum()
    }

    /// Returns true if the order book is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    /// Returns the number of bid levels.
    #[must_use]
    pub fn bid_levels(&self) -> usize {
        self.bids.len()
    }

    /// Returns the number of ask levels.
    #[must_use]
    pub fn ask_levels(&self) -> usize {
        self.asks.len()
    }
}

/// General information about an exchange.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeInfo {
    /// Exchange identifier (e.g., "MOEX", "BINANCE").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Exchange website URL.
    pub url: String,
    /// Supported instrument types.
    pub supported_types: Vec<InstrumentType>,
}

impl ExchangeInfo {
    /// Creates a new ExchangeInfo.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        url: impl Into<String>,
        supported_types: Vec<InstrumentType>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url: url.into(),
            supported_types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_usd(amount: Decimal) -> Money {
        Money::new(amount, "USD")
    }

    #[test]
    fn test_instrument_lot_conversion() {
        let instrument = Instrument {
            symbol: "SBER".into(),
            name: "Sberbank".into(),
            exchange_id: "BBG004730N88".into(),
            exchange: "MOEX".into(),
            instrument_type: InstrumentType::Stock,
            currency: "RUB".into(),
            lot_size: 10,
            min_price_increment: dec!(0.01),
            min_quantity: 1,
            tradeable: true,
            margin_available: true,
            short_available: false,
        };

        assert_eq!(instrument.lots_to_units(5), dec!(50));
        assert_eq!(instrument.units_to_lots(dec!(55)), 5); // 55/10 = 5 complete lots
    }

    #[test]
    fn test_instrument_price_rounding() {
        let instrument = Instrument {
            symbol: "AAPL".into(),
            name: "Apple Inc.".into(),
            exchange_id: "AAPL".into(),
            exchange: "NASDAQ".into(),
            instrument_type: InstrumentType::Stock,
            currency: "USD".into(),
            lot_size: 1,
            min_price_increment: dec!(0.01),
            min_quantity: 1,
            tradeable: true,
            margin_available: true,
            short_available: true,
        };

        assert_eq!(instrument.round_price(dec!(150.123)), dec!(150.12));
        // Note: Uses banker's rounding (round half to even) - 15012.5 rounds to 15012
        assert_eq!(instrument.round_price(dec!(150.125)), dec!(150.12));
        // A value that clearly rounds up
        assert_eq!(instrument.round_price(dec!(150.126)), dec!(150.13));
    }

    #[test]
    fn test_market_data_spread() {
        let data = MarketData {
            symbol: "AAPL".into(),
            last_price: make_usd(dec!(150)),
            bid: Some(make_usd(dec!(149.95))),
            ask: Some(make_usd(dec!(150.05))),
            volume_24h: None,
            high_24h: None,
            low_24h: None,
            change_24h: None,
            change_percent_24h: None,
            market_status: MarketStatus::Open,
            timestamp: Utc::now(),
        };

        assert_eq!(data.mid_price().unwrap().amount(), dec!(150));
        assert_eq!(data.spread().unwrap().amount(), dec!(0.10));
    }

    #[test]
    fn test_market_status_tradeable() {
        assert!(MarketStatus::Open.is_tradeable());
        assert!(MarketStatus::PreMarket.is_tradeable());
        assert!(MarketStatus::AfterHours.is_tradeable());
        assert!(!MarketStatus::Closed.is_tradeable());
        assert!(!MarketStatus::Halted.is_tradeable());
    }
}
