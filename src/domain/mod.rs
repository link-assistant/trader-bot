//! Domain types for the trader bot.
//!
//! This module contains core domain models that represent trading concepts
//! independent of any specific exchange or broker API.

mod decimal;
mod money;
mod order;
mod plan;
mod position;
mod trade;
mod wallet;

pub use decimal::Decimal;
pub use money::Money;
pub use order::{Order, OrderDirection, OrderStatus, OrderType};
pub use plan::{PlannedOrder, PlannedOrders};
pub use position::Position;
pub use trade::{Trade, TradeHistory, TradeId};
pub use wallet::{DesiredAllocation, Wallet};
