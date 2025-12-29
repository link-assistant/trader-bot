//! Position type representing a holding in a portfolio.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Money;

/// Represents a position (holding) in the portfolio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// The trading symbol/ticker (e.g., "AAPL", "BTC-USD").
    symbol: String,
    /// The exchange/market identifier (e.g., "MOEX", "NASDAQ", "BINANCE").
    exchange: String,
    /// Unique instrument identifier used by the exchange (e.g., FIGI for T-Bank).
    instrument_id: Option<String>,
    /// Quantity of the asset held.
    quantity: Decimal,
    /// The lot size for this instrument (how many units per lot).
    lot_size: u32,
    /// Current price per unit.
    current_price: Money,
    /// Average cost basis per unit (for P&L calculations).
    average_cost: Option<Money>,
    /// Whether the position is blocked/frozen.
    blocked: bool,
    /// Number of lots that are blocked.
    blocked_lots: u32,
}

impl Position {
    /// Creates a new Position.
    #[must_use]
    pub fn new(
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        quantity: Decimal,
        current_price: Money,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            exchange: exchange.into(),
            instrument_id: None,
            quantity,
            lot_size: 1,
            current_price,
            average_cost: None,
            blocked: false,
            blocked_lots: 0,
        }
    }

    /// Creates a Position builder for more complex construction.
    #[must_use]
    pub fn builder(symbol: impl Into<String>, exchange: impl Into<String>) -> PositionBuilder {
        PositionBuilder::new(symbol, exchange)
    }

    /// Returns the symbol.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the exchange.
    #[must_use]
    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    /// Returns the instrument ID if set.
    #[must_use]
    pub fn instrument_id(&self) -> Option<&str> {
        self.instrument_id.as_deref()
    }

    /// Returns the quantity held.
    #[must_use]
    pub const fn quantity(&self) -> Decimal {
        self.quantity
    }

    /// Returns the lot size.
    #[must_use]
    pub const fn lot_size(&self) -> u32 {
        self.lot_size
    }

    /// Returns the current price.
    #[must_use]
    pub const fn current_price(&self) -> &Money {
        &self.current_price
    }

    /// Returns the average cost if set.
    #[must_use]
    pub const fn average_cost(&self) -> Option<&Money> {
        self.average_cost.as_ref()
    }

    /// Returns whether the position is blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        self.blocked
    }

    /// Returns the number of blocked lots.
    #[must_use]
    pub const fn blocked_lots(&self) -> u32 {
        self.blocked_lots
    }

    /// Calculates the total market value of the position.
    #[must_use]
    pub fn market_value(&self) -> Money {
        self.current_price.multiply(self.quantity)
    }

    /// Calculates the number of complete lots held.
    #[must_use]
    pub fn lots_held(&self) -> u32 {
        let lot_size = Decimal::from(self.lot_size);
        (self.quantity / lot_size)
            .floor()
            .to_string()
            .parse::<u32>()
            .unwrap_or(0)
    }

    /// Calculates the number of tradeable lots (total - blocked).
    #[must_use]
    pub fn tradeable_lots(&self) -> u32 {
        self.lots_held().saturating_sub(self.blocked_lots)
    }

    /// Calculates the unrealized P&L if average cost is known.
    #[must_use]
    pub fn unrealized_pnl(&self) -> Option<Money> {
        let avg_cost = self.average_cost.as_ref()?;
        let cost_basis = avg_cost.multiply(self.quantity);
        let market_value = self.market_value();
        market_value.checked_sub(&cost_basis)
    }

    /// Calculates the unrealized P&L percentage if average cost is known.
    #[must_use]
    pub fn unrealized_pnl_percent(&self) -> Option<Decimal> {
        let avg_cost = self.average_cost.as_ref()?;
        if avg_cost.is_zero() {
            return None;
        }
        let pnl = self.unrealized_pnl()?;
        let cost_basis = avg_cost.multiply(self.quantity);
        Some((pnl.amount() / cost_basis.amount()) * Decimal::from(100))
    }

    /// Updates the current price.
    pub fn update_price(&mut self, price: Money) {
        self.current_price = price;
    }

    /// Updates the quantity.
    pub fn update_quantity(&mut self, quantity: Decimal) {
        self.quantity = quantity;
    }
}

/// Builder for constructing Position instances.
pub struct PositionBuilder {
    symbol: String,
    exchange: String,
    instrument_id: Option<String>,
    quantity: Decimal,
    lot_size: u32,
    current_price: Option<Money>,
    average_cost: Option<Money>,
    blocked: bool,
    blocked_lots: u32,
}

impl PositionBuilder {
    fn new(symbol: impl Into<String>, exchange: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            exchange: exchange.into(),
            instrument_id: None,
            quantity: Decimal::ZERO,
            lot_size: 1,
            current_price: None,
            average_cost: None,
            blocked: false,
            blocked_lots: 0,
        }
    }

    /// Sets the instrument ID.
    #[must_use]
    pub fn instrument_id(mut self, id: impl Into<String>) -> Self {
        self.instrument_id = Some(id.into());
        self
    }

    /// Sets the quantity.
    #[must_use]
    pub const fn quantity(mut self, qty: Decimal) -> Self {
        self.quantity = qty;
        self
    }

    /// Sets the lot size.
    #[must_use]
    pub const fn lot_size(mut self, size: u32) -> Self {
        self.lot_size = size;
        self
    }

    /// Sets the current price.
    #[must_use]
    pub fn current_price(mut self, price: Money) -> Self {
        self.current_price = Some(price);
        self
    }

    /// Sets the average cost.
    #[must_use]
    pub fn average_cost(mut self, cost: Money) -> Self {
        self.average_cost = Some(cost);
        self
    }

    /// Marks the position as blocked.
    #[must_use]
    pub const fn blocked(mut self, is_blocked: bool) -> Self {
        self.blocked = is_blocked;
        self
    }

    /// Sets the number of blocked lots.
    #[must_use]
    pub const fn blocked_lots(mut self, lots: u32) -> Self {
        self.blocked_lots = lots;
        self
    }

    /// Builds the Position instance.
    ///
    /// # Panics
    ///
    /// Panics if current_price is not set.
    #[must_use]
    pub fn build(self) -> Position {
        Position {
            symbol: self.symbol,
            exchange: self.exchange,
            instrument_id: self.instrument_id,
            quantity: self.quantity,
            lot_size: self.lot_size,
            current_price: self.current_price.expect("current_price is required"),
            average_cost: self.average_cost,
            blocked: self.blocked,
            blocked_lots: self.blocked_lots,
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
    fn test_position_creation() {
        let pos = Position::new("AAPL", "NASDAQ", dec!(10), make_usd(dec!(150)));
        assert_eq!(pos.symbol(), "AAPL");
        assert_eq!(pos.exchange(), "NASDAQ");
        assert_eq!(pos.quantity(), dec!(10));
    }

    #[test]
    fn test_position_builder() {
        let pos = Position::builder("SBER", "MOEX")
            .instrument_id("BBG004730N88")
            .quantity(dec!(100))
            .lot_size(10)
            .current_price(make_usd(dec!(250)))
            .average_cost(make_usd(dec!(200)))
            .build();

        assert_eq!(pos.symbol(), "SBER");
        assert_eq!(pos.instrument_id(), Some("BBG004730N88"));
        assert_eq!(pos.lot_size(), 10);
        assert_eq!(pos.lots_held(), 10); // 100 / 10
    }

    #[test]
    fn test_market_value() {
        let pos = Position::new("BTC", "BINANCE", dec!(2.5), make_usd(dec!(40000)));
        let value = pos.market_value();
        assert_eq!(value.amount(), dec!(100000));
    }

    #[test]
    fn test_unrealized_pnl() {
        let pos = Position::builder("AAPL", "NASDAQ")
            .quantity(dec!(10))
            .current_price(make_usd(dec!(150)))
            .average_cost(make_usd(dec!(100)))
            .build();

        let pnl = pos.unrealized_pnl().unwrap();
        assert_eq!(pnl.amount(), dec!(500)); // (150 - 100) * 10

        let pnl_pct = pos.unrealized_pnl_percent().unwrap();
        assert_eq!(pnl_pct, dec!(50)); // 50% gain
    }

    #[test]
    fn test_lots_calculation() {
        let pos = Position::builder("SBER", "MOEX")
            .quantity(dec!(105))
            .lot_size(10)
            .current_price(make_usd(dec!(250)))
            .blocked_lots(3)
            .build();

        assert_eq!(pos.lots_held(), 10); // 105 / 10 = 10 complete lots
        assert_eq!(pos.tradeable_lots(), 7); // 10 - 3
    }
}
