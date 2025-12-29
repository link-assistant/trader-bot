//! Money value type representing an amount with currency.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a monetary amount with an associated currency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Money {
    /// The amount in the smallest currency unit (e.g., cents for USD, kopecks for RUB).
    amount: Decimal,
    /// The currency code (e.g., "USD", "RUB", "BTC").
    currency: String,
}

impl Money {
    /// Creates a new Money instance.
    #[must_use]
    pub fn new(amount: Decimal, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
        }
    }

    /// Creates a zero amount in the specified currency.
    #[must_use]
    pub fn zero(currency: impl Into<String>) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency: currency.into(),
        }
    }

    /// Returns the amount.
    #[must_use]
    pub const fn amount(&self) -> Decimal {
        self.amount
    }

    /// Returns the currency code.
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Returns true if the amount is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// Returns true if the amount is positive.
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && !self.amount.is_zero()
    }

    /// Returns true if the amount is negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }

    /// Adds two Money values if they have the same currency.
    /// Returns None if currencies don't match.
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        if self.currency != other.currency {
            return None;
        }
        Some(Self {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Subtracts two Money values if they have the same currency.
    /// Returns None if currencies don't match.
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        if self.currency != other.currency {
            return None;
        }
        Some(Self {
            amount: self.amount - other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Multiplies the money amount by a scalar value.
    #[must_use]
    pub fn multiply(&self, scalar: Decimal) -> Self {
        Self {
            amount: self.amount * scalar,
            currency: self.currency.clone(),
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_creation() {
        let money = Money::new(dec!(100.50), "USD");
        assert_eq!(money.amount(), dec!(100.50));
        assert_eq!(money.currency(), "USD");
    }

    #[test]
    fn test_money_zero() {
        let money = Money::zero("RUB");
        assert!(money.is_zero());
        assert_eq!(money.currency(), "RUB");
    }

    #[test]
    fn test_money_add_same_currency() {
        let a = Money::new(dec!(100), "USD");
        let b = Money::new(dec!(50), "USD");
        let result = a.checked_add(&b).unwrap();
        assert_eq!(result.amount(), dec!(150));
    }

    #[test]
    fn test_money_add_different_currency() {
        let a = Money::new(dec!(100), "USD");
        let b = Money::new(dec!(50), "EUR");
        assert!(a.checked_add(&b).is_none());
    }

    #[test]
    fn test_money_positive_negative() {
        let positive = Money::new(dec!(100), "USD");
        let negative = Money::new(dec!(-50), "USD");
        let zero = Money::zero("USD");

        assert!(positive.is_positive());
        assert!(!positive.is_negative());

        assert!(negative.is_negative());
        assert!(!negative.is_positive());

        assert!(zero.is_zero());
        assert!(!zero.is_positive());
        assert!(!zero.is_negative());
    }

    #[test]
    fn test_money_multiply() {
        let money = Money::new(dec!(100), "USD");
        let result = money.multiply(dec!(1.5));
        assert_eq!(result.amount(), dec!(150));
        assert_eq!(result.currency(), "USD");
    }
}
