//! Rebalance plan and action types.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::{Money, OrderDirection};

/// A single rebalancing action to be executed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RebalanceAction {
    /// The trading symbol.
    pub symbol: String,
    /// The exchange.
    pub exchange: String,
    /// Buy or sell.
    pub direction: OrderDirection,
    /// Number of lots to trade.
    pub lots: u32,
    /// Estimated price per lot.
    pub estimated_price: Money,
    /// Estimated total value of the trade.
    pub estimated_value: Money,
    /// Current allocation percentage.
    pub current_percent: Decimal,
    /// Target allocation percentage after this trade.
    pub target_percent: Decimal,
    /// Priority order (lower = higher priority).
    pub priority: u32,
}

impl RebalanceAction {
    /// Returns true if this is a buy action.
    #[must_use]
    pub const fn is_buy(&self) -> bool {
        matches!(self.direction, OrderDirection::Buy)
    }

    /// Returns true if this is a sell action.
    #[must_use]
    pub const fn is_sell(&self) -> bool {
        matches!(self.direction, OrderDirection::Sell)
    }
}

/// A complete rebalancing plan containing all actions to execute.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RebalancePlan {
    /// Actions to execute, in priority order.
    pub actions: Vec<RebalanceAction>,
    /// Total value of sell orders.
    pub total_sell_value: Money,
    /// Total value of buy orders.
    pub total_buy_value: Money,
    /// Number of trades required.
    pub trade_count: usize,
    /// Whether this is a dry run (no actual trades).
    pub dry_run: bool,
    /// Reason if the plan is empty.
    pub empty_reason: Option<String>,
}

impl RebalancePlan {
    /// Creates a new empty plan.
    #[must_use]
    pub fn new(currency: &str) -> Self {
        Self {
            actions: Vec::new(),
            total_sell_value: Money::zero(currency),
            total_buy_value: Money::zero(currency),
            trade_count: 0,
            dry_run: false,
            empty_reason: None,
        }
    }

    /// Creates a plan with a reason for being empty.
    #[must_use]
    pub fn empty_with_reason(currency: &str, reason: impl Into<String>) -> Self {
        Self {
            actions: Vec::new(),
            total_sell_value: Money::zero(currency),
            total_buy_value: Money::zero(currency),
            trade_count: 0,
            dry_run: false,
            empty_reason: Some(reason.into()),
        }
    }

    /// Returns true if the plan has no actions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Returns all sell actions.
    pub fn sells(&self) -> impl Iterator<Item = &RebalanceAction> {
        self.actions.iter().filter(|a| a.is_sell())
    }

    /// Returns all buy actions.
    pub fn buys(&self) -> impl Iterator<Item = &RebalanceAction> {
        self.actions.iter().filter(|a| a.is_buy())
    }

    /// Returns the number of sell actions.
    #[must_use]
    pub fn sell_count(&self) -> usize {
        self.actions.iter().filter(|a| a.is_sell()).count()
    }

    /// Returns the number of buy actions.
    #[must_use]
    pub fn buy_count(&self) -> usize {
        self.actions.iter().filter(|a| a.is_buy()).count()
    }

    /// Adds an action to the plan.
    pub fn add_action(&mut self, action: RebalanceAction) {
        if action.is_sell() {
            if let Some(new_total) = self.total_sell_value.checked_add(&action.estimated_value) {
                self.total_sell_value = new_total;
            }
        } else if let Some(new_total) = self.total_buy_value.checked_add(&action.estimated_value) {
            self.total_buy_value = new_total;
        }
        self.actions.push(action);
        self.trade_count = self.actions.len();
    }

    /// Marks this plan as a dry run.
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Sorts actions by priority.
    pub fn sort_by_priority(&mut self) {
        self.actions.sort_by_key(|a| a.priority);
    }

    /// Returns a summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_empty() {
            return self
                .empty_reason
                .clone()
                .unwrap_or_else(|| "No rebalancing needed".into());
        }

        format!(
            "{} trades ({} sells for {}, {} buys for {})",
            self.trade_count,
            self.sell_count(),
            self.total_sell_value,
            self.buy_count(),
            self.total_buy_value
        )
    }
}

/// Builder for creating rebalance plans.
pub struct RebalancePlanBuilder {
    currency: String,
    actions: Vec<RebalanceAction>,
    dry_run: bool,
}

impl RebalancePlanBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new(currency: impl Into<String>) -> Self {
        Self {
            currency: currency.into(),
            actions: Vec::new(),
            dry_run: false,
        }
    }

    /// Adds a sell action.
    #[must_use]
    pub fn sell(
        mut self,
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        lots: u32,
        price: Money,
        current_percent: Decimal,
        target_percent: Decimal,
    ) -> Self {
        let estimated_value = price.multiply(Decimal::from(lots));
        self.actions.push(RebalanceAction {
            symbol: symbol.into(),
            exchange: exchange.into(),
            direction: OrderDirection::Sell,
            lots,
            estimated_price: price,
            estimated_value,
            current_percent,
            target_percent,
            priority: self.actions.len() as u32,
        });
        self
    }

    /// Adds a buy action.
    #[must_use]
    pub fn buy(
        mut self,
        symbol: impl Into<String>,
        exchange: impl Into<String>,
        lots: u32,
        price: Money,
        current_percent: Decimal,
        target_percent: Decimal,
    ) -> Self {
        let estimated_value = price.multiply(Decimal::from(lots));
        self.actions.push(RebalanceAction {
            symbol: symbol.into(),
            exchange: exchange.into(),
            direction: OrderDirection::Buy,
            lots,
            estimated_price: price,
            estimated_value,
            current_percent,
            target_percent,
            priority: self.actions.len() as u32,
        });
        self
    }

    /// Marks the plan as a dry run.
    #[must_use]
    pub const fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Builds the rebalance plan.
    #[must_use]
    pub fn build(self) -> RebalancePlan {
        let mut plan = RebalancePlan::new(&self.currency);
        plan.dry_run = self.dry_run;
        for action in self.actions {
            plan.add_action(action);
        }
        plan.sort_by_priority();
        plan
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
    fn test_rebalance_plan_builder() {
        let plan = RebalancePlanBuilder::new("USD")
            .sell(
                "GOOGL",
                "NASDAQ",
                5,
                make_usd(dec!(100)),
                dec!(40),
                dec!(30),
            )
            .buy("AAPL", "NASDAQ", 3, make_usd(dec!(150)), dec!(20), dec!(30))
            .build();

        assert_eq!(plan.trade_count, 2);
        assert_eq!(plan.sell_count(), 1);
        assert_eq!(plan.buy_count(), 1);
        assert_eq!(plan.total_sell_value.amount(), dec!(500)); // 5 * 100
        assert_eq!(plan.total_buy_value.amount(), dec!(450)); // 3 * 150
    }

    #[test]
    fn test_empty_plan() {
        let plan = RebalancePlan::empty_with_reason("USD", "Portfolio already balanced");
        assert!(plan.is_empty());
        assert_eq!(plan.empty_reason, Some("Portfolio already balanced".into()));
    }

    #[test]
    fn test_plan_summary() {
        let plan = RebalancePlanBuilder::new("USD")
            .sell("A", "X", 10, make_usd(dec!(100)), dec!(30), dec!(20))
            .buy("B", "X", 5, make_usd(dec!(200)), dec!(10), dec!(20))
            .build();

        let summary = plan.summary();
        assert!(summary.contains("2 trades"));
        assert!(summary.contains("1 sells"));
        assert!(summary.contains("1 buys"));
    }

    #[test]
    fn test_action_direction() {
        let buy_action = RebalanceAction {
            symbol: "A".into(),
            exchange: "X".into(),
            direction: OrderDirection::Buy,
            lots: 1,
            estimated_price: make_usd(dec!(100)),
            estimated_value: make_usd(dec!(100)),
            current_percent: dec!(10),
            target_percent: dec!(20),
            priority: 0,
        };

        assert!(buy_action.is_buy());
        assert!(!buy_action.is_sell());
    }
}
