//! Simulated exchange implementation.

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::market::MarketManager;
use crate::domain::{Money, Order, OrderDirection, OrderStatus, Position, Wallet};
use crate::exchange::{
    ExchangeError, ExchangeInfo, ExchangeProvider, ExchangeResult, Instrument, InstrumentType,
    MarketData, MarketStatus, OrderBook, OrderBookEntry,
};

/// Simulated account state.
#[derive(Debug, Clone)]
struct AccountState {
    cash: Money,
    positions: HashMap<String, Position>,
    orders: HashMap<String, Order>,
}

impl AccountState {
    fn new(currency: &str, initial_cash: Decimal) -> Self {
        Self {
            cash: Money::new(initial_cash, currency),
            positions: HashMap::new(),
            orders: HashMap::new(),
        }
    }

    fn to_wallet(&self, currency: &str) -> Wallet {
        let mut wallet = Wallet::with_cash(currency, self.cash.clone());
        for pos in self.positions.values() {
            wallet.add_position(pos.clone());
        }
        wallet
    }
}

/// A simulated exchange for testing.
#[derive(Debug)]
pub struct SimulatedExchange {
    info: ExchangeInfo,
    market: Arc<RwLock<MarketManager>>,
    accounts: Arc<RwLock<HashMap<String, AccountState>>>,
    base_currency: String,
    /// Simulated latency in milliseconds.
    latency_ms: u64,
    /// Whether to fail the next operation (for testing error handling).
    fail_next: Arc<RwLock<Option<ExchangeError>>>,
}

impl SimulatedExchange {
    /// Creates a new simulated exchange.
    #[must_use]
    pub fn new(base_currency: impl Into<String>) -> Self {
        let currency = base_currency.into();
        Self {
            info: ExchangeInfo {
                id: "SIMULATOR".to_string(),
                name: "Simulated Exchange".to_string(),
                url: None,
                available: true,
                supported_types: vec![
                    InstrumentType::Stock,
                    InstrumentType::Etf,
                    InstrumentType::Crypto,
                ],
            },
            market: Arc::new(RwLock::new(MarketManager::new(&currency))),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            base_currency: currency,
            latency_ms: 0,
            fail_next: Arc::new(RwLock::new(None)),
        }
    }

    /// Sets the simulated latency.
    pub fn set_latency(&mut self, ms: u64) {
        self.latency_ms = ms;
    }

    /// Makes the next operation fail with the given error.
    pub fn fail_next_with(&self, error: ExchangeError) {
        *self.fail_next.write().unwrap() = Some(error);
    }

    /// Adds an instrument to the market.
    pub fn add_instrument(
        &self,
        symbol: impl Into<String>,
        name: impl Into<String>,
        price: Decimal,
        lot_size: u32,
    ) {
        self.market
            .write()
            .unwrap()
            .add_instrument(symbol, name, price, lot_size);
    }

    /// Creates an account with initial cash.
    pub fn create_account(&self, account_id: impl Into<String>, initial_cash: Decimal) {
        let mut accounts = self.accounts.write().unwrap();
        accounts.insert(
            account_id.into(),
            AccountState::new(&self.base_currency, initial_cash),
        );
    }

    /// Adds a position to an account.
    pub fn add_position(&self, account_id: &str, position: Position) {
        let mut accounts = self.accounts.write().unwrap();
        if let Some(account) = accounts.get_mut(account_id) {
            account
                .positions
                .insert(position.symbol().to_string(), position);
        }
    }

    /// Sets the price for an instrument.
    pub fn set_price(&self, symbol: &str, price: Decimal) {
        self.market.write().unwrap().set_price(symbol, price);
    }

    /// Sets the market status for all instruments.
    pub fn set_market_status(&self, status: MarketStatus) {
        self.market.write().unwrap().set_market_status(status);
    }

    /// Advances the market by one tick.
    pub fn tick(&self) {
        self.market.write().unwrap().tick();
    }

    /// Gets a reference to the market manager.
    #[must_use]
    pub fn market(&self) -> Arc<RwLock<MarketManager>> {
        Arc::clone(&self.market)
    }

    /// Simulates network latency.
    async fn simulate_latency(&self) {
        if self.latency_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.latency_ms)).await;
        }
    }

    /// Checks and returns any pending failure.
    fn check_failure(&self) -> ExchangeResult<()> {
        let mut fail_next = self.fail_next.write().unwrap();
        if let Some(error) = fail_next.take() {
            return Err(error);
        }
        Ok(())
    }
}

#[async_trait]
impl ExchangeProvider for SimulatedExchange {
    fn info(&self) -> &ExchangeInfo {
        &self.info
    }

    async fn ping(&self) -> ExchangeResult<()> {
        self.simulate_latency().await;
        self.check_failure()?;
        Ok(())
    }

    async fn get_wallet(&self, account_id: &str) -> ExchangeResult<Wallet> {
        self.simulate_latency().await;
        self.check_failure()?;

        let accounts = self.accounts.read().unwrap();
        let account = accounts.get(account_id).ok_or_else(|| {
            ExchangeError::AuthenticationFailed(format!("Account {account_id} not found"))
        })?;

        // Update position prices from market
        let market = self.market.read().unwrap();
        let mut wallet = account.to_wallet(&self.base_currency);

        for pos in wallet.positions_mut() {
            if let Some(data) = market.get_market_data(pos.symbol()) {
                pos.update_price(data.last_price);
            }
        }

        Ok(wallet)
    }

    async fn get_instrument(&self, symbol: &str) -> ExchangeResult<Instrument> {
        self.simulate_latency().await;
        self.check_failure()?;

        let market = self.market.read().unwrap();
        market
            .get_instrument(symbol)
            .cloned()
            .ok_or_else(|| ExchangeError::InstrumentNotFound(symbol.to_string()))
    }

    async fn list_instruments(&self) -> ExchangeResult<Vec<Instrument>> {
        self.simulate_latency().await;
        self.check_failure()?;

        let market = self.market.read().unwrap();
        Ok(market.list_instruments())
    }

    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketData> {
        self.simulate_latency().await;
        self.check_failure()?;

        let market = self.market.read().unwrap();
        market
            .get_market_data(symbol)
            .ok_or_else(|| ExchangeError::InstrumentNotFound(symbol.to_string()))
    }

    async fn get_order_book(&self, symbol: &str, depth: u32) -> ExchangeResult<OrderBook> {
        self.simulate_latency().await;
        self.check_failure()?;

        let market = self.market.read().unwrap();
        let state = market
            .get_state(symbol)
            .ok_or_else(|| ExchangeError::InstrumentNotFound(symbol.to_string()))?;

        // Generate synthetic order book
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        let base_price = state.price;

        for i in 0..depth {
            let offset = Decimal::from(i + 1) * Decimal::new(1, 2); // 0.01 per level

            bids.push(OrderBookEntry {
                price: Money::new(base_price - offset, &self.base_currency),
                quantity: Decimal::from(100 * (depth - i)),
            });

            asks.push(OrderBookEntry {
                price: Money::new(base_price + offset, &self.base_currency),
                quantity: Decimal::from(100 * (depth - i)),
            });
        }

        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn place_order(&self, account_id: &str, mut order: Order) -> ExchangeResult<Order> {
        self.simulate_latency().await;
        self.check_failure()?;

        let mut accounts = self.accounts.write().unwrap();
        let account = accounts.get_mut(account_id).ok_or_else(|| {
            ExchangeError::AuthenticationFailed(format!("Account {account_id} not found"))
        })?;

        let market = self.market.read().unwrap();
        let instrument = market
            .get_instrument(order.symbol())
            .ok_or_else(|| ExchangeError::InstrumentNotFound(order.symbol().to_string()))?;
        let market_data = market
            .get_market_data(order.symbol())
            .ok_or_else(|| ExchangeError::InstrumentNotFound(order.symbol().to_string()))?;

        // Check market status
        if !market_data.market_status.is_tradeable() {
            return Err(ExchangeError::MarketClosed {
                instrument: order.symbol().to_string(),
            });
        }

        // Calculate trade value
        let price = market_data.last_price.amount();
        let lot_value = price * Decimal::from(instrument.lot_size);
        let total_value = lot_value * Decimal::from(order.quantity_lots());

        match order.direction() {
            OrderDirection::Buy => {
                // Check sufficient funds
                if account.cash.amount() < total_value {
                    return Err(ExchangeError::InsufficientFunds {
                        required: total_value.to_string(),
                        available: account.cash.amount().to_string(),
                    });
                }

                // Execute buy
                let new_cash = account.cash.amount() - total_value;
                account.cash = Money::new(new_cash, &self.base_currency);

                let quantity = Decimal::from(order.quantity_lots() * instrument.lot_size);
                if let Some(pos) = account.positions.get_mut(order.symbol()) {
                    pos.update_quantity(pos.quantity() + quantity);
                    pos.update_price(market_data.last_price.clone());
                } else {
                    let new_pos = Position::builder(order.symbol(), &instrument.exchange)
                        .quantity(quantity)
                        .lot_size(instrument.lot_size)
                        .current_price(market_data.last_price.clone())
                        .build();
                    account
                        .positions
                        .insert(order.symbol().to_string(), new_pos);
                }
            }
            OrderDirection::Sell => {
                // Check sufficient position
                let pos = account.positions.get_mut(order.symbol()).ok_or_else(|| {
                    ExchangeError::InsufficientFunds {
                        required: order.quantity_lots().to_string(),
                        available: "0".to_string(),
                    }
                })?;

                let sell_quantity = Decimal::from(order.quantity_lots() * instrument.lot_size);
                if pos.quantity() < sell_quantity {
                    return Err(ExchangeError::InsufficientFunds {
                        required: sell_quantity.to_string(),
                        available: pos.quantity().to_string(),
                    });
                }

                // Execute sell
                let new_quantity = pos.quantity() - sell_quantity;
                if new_quantity.is_zero() {
                    account.positions.remove(order.symbol());
                } else {
                    pos.update_quantity(new_quantity);
                }

                let new_cash = account.cash.amount() + total_value;
                account.cash = Money::new(new_cash, &self.base_currency);
            }
        }

        // Mark order as filled
        order.set_exchange_order_id(format!(
            "SIM-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        order.record_fill(order.quantity_lots(), market_data.last_price);
        order.set_status(OrderStatus::Filled);

        // Store order
        account.orders.insert(order.id().to_string(), order.clone());

        Ok(order)
    }

    async fn cancel_order(&self, account_id: &str, order_id: &str) -> ExchangeResult<Order> {
        self.simulate_latency().await;
        self.check_failure()?;

        let mut accounts = self.accounts.write().unwrap();
        let account = accounts.get_mut(account_id).ok_or_else(|| {
            ExchangeError::AuthenticationFailed(format!("Account {account_id} not found"))
        })?;

        let mut order = account
            .orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| ExchangeError::OrderNotFound(order_id.to_string()))?;

        if order.status().is_terminal() {
            return Err(ExchangeError::InvalidOrder(format!(
                "Order {order_id} is already in terminal state"
            )));
        }

        order.set_status(OrderStatus::Cancelled);
        account.orders.insert(order_id.to_string(), order.clone());

        Ok(order)
    }

    async fn get_order(&self, account_id: &str, order_id: &str) -> ExchangeResult<Order> {
        self.simulate_latency().await;
        self.check_failure()?;

        let accounts = self.accounts.read().unwrap();
        let account = accounts.get(account_id).ok_or_else(|| {
            ExchangeError::AuthenticationFailed(format!("Account {account_id} not found"))
        })?;

        account
            .orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| ExchangeError::OrderNotFound(order_id.to_string()))
    }

    async fn get_active_orders(&self, account_id: &str) -> ExchangeResult<Vec<Order>> {
        self.simulate_latency().await;
        self.check_failure()?;

        let accounts = self.accounts.read().unwrap();
        let account = accounts.get(account_id).ok_or_else(|| {
            ExchangeError::AuthenticationFailed(format!("Account {account_id} not found"))
        })?;

        Ok(account
            .orders
            .values()
            .filter(|o| o.status().is_active())
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_exchange() -> SimulatedExchange {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("AAPL", "Apple Inc.", dec!(150), 1);
        exchange.add_instrument("GOOGL", "Alphabet Inc.", dec!(2800), 1);
        exchange.create_account("test_account", dec!(100000));
        exchange
    }

    #[tokio::test]
    async fn test_get_wallet() {
        let exchange = create_test_exchange();

        let wallet = exchange.get_wallet("test_account").await.unwrap();
        assert_eq!(wallet.cash().amount(), dec!(100000));
        assert_eq!(wallet.position_count(), 0);
    }

    #[tokio::test]
    async fn test_place_buy_order() {
        let exchange = create_test_exchange();

        let order = Order::market("ORD1", "AAPL", "SIMULATOR", OrderDirection::Buy, 10);
        let result = exchange.place_order("test_account", order).await.unwrap();

        assert_eq!(result.status(), OrderStatus::Filled);
        assert_eq!(result.filled_lots(), 10);

        // Check wallet updated
        let wallet = exchange.get_wallet("test_account").await.unwrap();
        assert_eq!(wallet.cash().amount(), dec!(100000) - dec!(1500)); // 10 * 150
        assert_eq!(wallet.get_position("AAPL").unwrap().quantity(), dec!(10));
    }

    #[tokio::test]
    async fn test_place_sell_order() {
        let exchange = create_test_exchange();

        // First buy
        let buy_order = Order::market("ORD1", "AAPL", "SIMULATOR", OrderDirection::Buy, 10);
        exchange
            .place_order("test_account", buy_order)
            .await
            .unwrap();

        // Then sell
        let sell_order = Order::market("ORD2", "AAPL", "SIMULATOR", OrderDirection::Sell, 5);
        let result = exchange
            .place_order("test_account", sell_order)
            .await
            .unwrap();

        assert_eq!(result.status(), OrderStatus::Filled);

        let wallet = exchange.get_wallet("test_account").await.unwrap();
        assert_eq!(wallet.get_position("AAPL").unwrap().quantity(), dec!(5));
    }

    #[tokio::test]
    async fn test_insufficient_funds() {
        let exchange = SimulatedExchange::new("USD");
        exchange.add_instrument("EXPENSIVE", "Expensive Stock", dec!(1000000), 1);
        exchange.create_account("poor_account", dec!(100));

        let order = Order::market("ORD1", "EXPENSIVE", "SIMULATOR", OrderDirection::Buy, 1);
        let result = exchange.place_order("poor_account", order).await;

        assert!(matches!(
            result,
            Err(ExchangeError::InsufficientFunds { .. })
        ));
    }

    #[tokio::test]
    async fn test_market_closed() {
        let exchange = create_test_exchange();
        exchange.set_market_status(MarketStatus::Closed);

        let order = Order::market("ORD1", "AAPL", "SIMULATOR", OrderDirection::Buy, 10);
        let result = exchange.place_order("test_account", order).await;

        assert!(matches!(result, Err(ExchangeError::MarketClosed { .. })));
    }

    #[tokio::test]
    async fn test_fail_next() {
        let exchange = create_test_exchange();
        exchange.fail_next_with(ExchangeError::NetworkError("Simulated failure".into()));

        let result = exchange.ping().await;
        assert!(matches!(result, Err(ExchangeError::NetworkError(_))));

        // Next call should succeed
        let result = exchange.ping().await;
        assert!(result.is_ok());
    }
}
