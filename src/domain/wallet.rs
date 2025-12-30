//! Wallet and portfolio types.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Money, Position};

/// Represents a desired portfolio allocation.
///
/// The keys are asset symbols and values are target percentages (0-100).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DesiredAllocation {
    allocations: HashMap<String, Decimal>,
}

impl DesiredAllocation {
    /// Creates a new empty allocation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an allocation from a HashMap.
    #[must_use]
    pub fn from_map(allocations: HashMap<String, Decimal>) -> Self {
        Self { allocations }
    }

    /// Sets the target percentage for a symbol.
    pub fn set(&mut self, symbol: impl Into<String>, percentage: Decimal) {
        self.allocations.insert(symbol.into(), percentage);
    }

    /// Gets the target percentage for a symbol.
    #[must_use]
    pub fn get(&self, symbol: &str) -> Option<Decimal> {
        self.allocations.get(symbol).copied()
    }

    /// Returns all symbols in the allocation.
    pub fn symbols(&self) -> impl Iterator<Item = &String> {
        self.allocations.keys()
    }

    /// Returns an iterator over (symbol, percentage) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Decimal)> {
        self.allocations.iter()
    }

    /// Returns the number of assets in the allocation.
    #[must_use]
    pub fn len(&self) -> usize {
        self.allocations.len()
    }

    /// Returns true if the allocation is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.allocations.is_empty()
    }

    /// Calculates the total percentage (should sum to 100 for a valid allocation).
    #[must_use]
    pub fn total_percentage(&self) -> Decimal {
        self.allocations.values().sum()
    }

    /// Validates that the allocation sums to approximately 100%.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let total = self.total_percentage();
        let diff = (total - Decimal::from(100)).abs();
        diff < Decimal::new(1, 2) // Allow 0.01% tolerance
    }

    /// Normalizes the allocation to sum to exactly 100%.
    #[must_use]
    pub fn normalized(&self) -> Self {
        let total = self.total_percentage();
        if total.is_zero() {
            return self.clone();
        }

        let normalized: HashMap<String, Decimal> = self
            .allocations
            .iter()
            .map(|(k, v)| (k.clone(), *v * Decimal::from(100) / total))
            .collect();

        Self {
            allocations: normalized,
        }
    }
}

/// Represents a wallet (portfolio) containing multiple positions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    /// The base currency for value calculations.
    base_currency: String,
    /// Cash balance.
    cash: Money,
    /// Positions in the wallet.
    positions: Vec<Position>,
}

impl Wallet {
    /// Creates a new empty wallet.
    #[must_use]
    pub fn new(base_currency: impl Into<String>) -> Self {
        let currency: String = base_currency.into();
        Self {
            cash: Money::zero(&currency),
            base_currency: currency,
            positions: Vec::new(),
        }
    }

    /// Creates a wallet with initial cash balance.
    #[must_use]
    pub fn with_cash(base_currency: impl Into<String>, cash: Money) -> Self {
        let currency: String = base_currency.into();
        Self {
            cash,
            base_currency: currency,
            positions: Vec::new(),
        }
    }

    /// Returns the base currency.
    #[must_use]
    pub fn base_currency(&self) -> &str {
        &self.base_currency
    }

    /// Returns the cash balance.
    #[must_use]
    pub const fn cash(&self) -> &Money {
        &self.cash
    }

    /// Returns all positions.
    #[must_use]
    pub fn positions(&self) -> &[Position] {
        &self.positions
    }

    /// Returns a mutable reference to positions.
    pub fn positions_mut(&mut self) -> &mut Vec<Position> {
        &mut self.positions
    }

    /// Adds a position to the wallet.
    pub fn add_position(&mut self, position: Position) {
        // Check if position with same symbol already exists
        if let Some(existing) = self
            .positions
            .iter_mut()
            .find(|p| p.symbol() == position.symbol())
        {
            // Update existing position
            let new_qty = existing.quantity() + position.quantity();
            existing.update_quantity(new_qty);
            existing.update_price(position.current_price().clone());
        } else {
            self.positions.push(position);
        }
    }

    /// Removes a position by symbol.
    pub fn remove_position(&mut self, symbol: &str) -> Option<Position> {
        if let Some(idx) = self.positions.iter().position(|p| p.symbol() == symbol) {
            Some(self.positions.remove(idx))
        } else {
            None
        }
    }

    /// Gets a position by symbol.
    #[must_use]
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.iter().find(|p| p.symbol() == symbol)
    }

    /// Gets a mutable position by symbol.
    pub fn get_position_mut(&mut self, symbol: &str) -> Option<&mut Position> {
        self.positions.iter_mut().find(|p| p.symbol() == symbol)
    }

    /// Updates the cash balance.
    pub fn set_cash(&mut self, cash: Money) {
        self.cash = cash;
    }

    /// Adds to the cash balance.
    pub fn add_cash(&mut self, amount: Money) {
        if let Some(new_cash) = self.cash.checked_add(&amount) {
            self.cash = new_cash;
        }
    }

    /// Subtracts from the cash balance.
    pub fn subtract_cash(&mut self, amount: Money) {
        if let Some(new_cash) = self.cash.checked_sub(&amount) {
            self.cash = new_cash;
        }
    }

    /// Calculates the total market value of all positions.
    #[must_use]
    pub fn total_positions_value(&self) -> Money {
        let total: Decimal = self
            .positions
            .iter()
            .map(|p| p.market_value().amount())
            .sum();
        Money::new(total, &self.base_currency)
    }

    /// Calculates the total portfolio value (cash + positions).
    #[must_use]
    pub fn total_value(&self) -> Money {
        let positions_value = self.total_positions_value();
        self.cash
            .checked_add(&positions_value)
            .unwrap_or_else(|| positions_value)
    }

    /// Calculates the current allocation percentages.
    #[must_use]
    pub fn current_allocation(&self) -> DesiredAllocation {
        let total = self.total_value().amount();
        if total.is_zero() {
            return DesiredAllocation::new();
        }

        let allocations: HashMap<String, Decimal> = self
            .positions
            .iter()
            .map(|p| {
                let pct = p.market_value().amount() / total * Decimal::from(100);
                (p.symbol().to_string(), pct)
            })
            .collect();

        DesiredAllocation::from_map(allocations)
    }

    /// Calculates the cash percentage of total portfolio.
    #[must_use]
    pub fn cash_percentage(&self) -> Decimal {
        let total = self.total_value().amount();
        if total.is_zero() {
            return Decimal::ZERO;
        }
        self.cash.amount() / total * Decimal::from(100)
    }

    /// Returns the number of positions.
    #[must_use]
    pub fn position_count(&self) -> usize {
        self.positions.len()
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
    fn test_desired_allocation() {
        let mut alloc = DesiredAllocation::new();
        alloc.set("AAPL", dec!(30));
        alloc.set("GOOGL", dec!(30));
        alloc.set("MSFT", dec!(40));

        assert_eq!(alloc.get("AAPL"), Some(dec!(30)));
        assert_eq!(alloc.total_percentage(), dec!(100));
        assert!(alloc.is_valid());
    }

    #[test]
    fn test_allocation_normalization() {
        let mut alloc = DesiredAllocation::new();
        alloc.set("A", dec!(10));
        alloc.set("B", dec!(20));
        alloc.set("C", dec!(30)); // Total = 60

        let normalized = alloc.normalized();
        let total = normalized.total_percentage();
        assert!((total - dec!(100)).abs() < dec!(0.01));
    }

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::with_cash("USD", make_usd(dec!(10000)));
        assert_eq!(wallet.base_currency(), "USD");
        assert_eq!(wallet.cash().amount(), dec!(10000));
        assert!(wallet.positions().is_empty());
    }

    #[test]
    fn test_wallet_add_position() {
        let mut wallet = Wallet::with_cash("USD", make_usd(dec!(5000)));
        let pos = Position::new("AAPL", "NASDAQ", dec!(10), make_usd(dec!(150)));
        wallet.add_position(pos);

        assert_eq!(wallet.position_count(), 1);
        assert_eq!(wallet.get_position("AAPL").unwrap().quantity(), dec!(10));
    }

    #[test]
    fn test_wallet_total_value() {
        let mut wallet = Wallet::with_cash("USD", make_usd(dec!(5000)));
        wallet.add_position(Position::new(
            "AAPL",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(150)),
        ));
        wallet.add_position(Position::new(
            "GOOGL",
            "NASDAQ",
            dec!(5),
            make_usd(dec!(100)),
        ));

        // Cash: 5000, AAPL: 10*150=1500, GOOGL: 5*100=500
        // Total = 7000
        assert_eq!(wallet.total_value().amount(), dec!(7000));
    }

    #[test]
    fn test_wallet_current_allocation() {
        let mut wallet = Wallet::with_cash("USD", make_usd(dec!(0)));
        wallet.add_position(Position::new(
            "AAPL",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(100)),
        ));
        wallet.add_position(Position::new(
            "GOOGL",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(100)),
        ));

        let alloc = wallet.current_allocation();
        assert_eq!(alloc.get("AAPL"), Some(dec!(50)));
        assert_eq!(alloc.get("GOOGL"), Some(dec!(50)));
    }

    #[test]
    fn test_wallet_merge_positions() {
        let mut wallet = Wallet::new("USD");
        wallet.add_position(Position::new(
            "AAPL",
            "NASDAQ",
            dec!(10),
            make_usd(dec!(150)),
        ));
        wallet.add_position(Position::new(
            "AAPL",
            "NASDAQ",
            dec!(5),
            make_usd(dec!(155)),
        ));

        assert_eq!(wallet.position_count(), 1);
        assert_eq!(wallet.get_position("AAPL").unwrap().quantity(), dec!(15));
    }
}
