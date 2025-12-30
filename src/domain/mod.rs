//! Domain types for the portfolio balancer trading bot.
//!
//! This module contains core domain models that represent trading concepts
//! independent of any specific exchange or broker API.

mod decimal;
mod money;
mod order;
mod position;
mod wallet;

pub use decimal::Decimal;
pub use money::Money;
pub use order::{Order, OrderDirection, OrderStatus, OrderType};
pub use position::Position;
pub use wallet::{DesiredAllocation, Wallet};
