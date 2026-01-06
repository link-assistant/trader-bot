//! User and account domain models.
//!
//! This module provides the core domain models for multi-user, multi-account
//! trading configurations following the Links Notation structure.
//!
//! # Terminology
//!
//! - **User**: A connection to an exchange/broker (has API credentials)
//! - **Account**: A trading account within that broker (can have multiple per user)
//! - **Allocation**: The target portfolio allocation for an account
//!
//! # Links Notation Configuration
//!
//! ```text
//! users
//!   (
//!     broker tbank
//!     token TBANK_API_TOKEN
//!     accounts
//!       main
//!         allocation
//!           SBER 30
//!           LKOH 30
//!           GAZP 40
//!       trading
//!         allocation
//!           AAPL 50
//!           GOOGL 50
//!   )
//!   (
//!     exchange binance
//!     "api key" BINANCE_API_KEY
//!     accounts
//!       spot_main
//!         allocation
//!           BTC 50
//!           ETH 50
//!   )
//! ```

use super::Allocation;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of broker/exchange connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BrokerType {
    /// T-Bank (formerly Tinkoff Investments).
    #[default]
    TBank,
    /// Binance cryptocurrency exchange.
    Binance,
    /// Interactive Brokers.
    InteractiveBrokers,
    /// Simulated exchange (for testing).
    Simulator,
    /// Custom broker with a name.
    Custom(String),
}

impl BrokerType {
    /// Creates a broker type from a string name.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "tbank" | "t-bank" | "tinkoff" => Self::TBank,
            "binance" => Self::Binance,
            "ib" | "ibkr" | "interactive_brokers" | "interactivebrokers" => {
                Self::InteractiveBrokers
            }
            "sim" | "simulator" | "simulated" | "demo" => Self::Simulator,
            _ => Self::Custom(name.to_string()),
        }
    }

    /// Returns the broker name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::TBank => "tbank",
            Self::Binance => "binance",
            Self::InteractiveBrokers => "interactive_brokers",
            Self::Simulator => "simulator",
            Self::Custom(name) => name,
        }
    }
}

/// Configuration for a trading account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique account identifier within the user.
    pub id: String,

    /// Human-readable account name.
    #[serde(default)]
    pub name: String,

    /// Exchange account ID (from broker's system).
    #[serde(default)]
    pub exchange_account_id: String,

    /// Target portfolio allocation for this account.
    pub allocation: Allocation,

    /// Rebalancing interval in seconds.
    #[serde(default = "default_balance_interval")]
    pub balance_interval_secs: u64,

    /// Delay between orders in milliseconds.
    #[serde(default = "default_order_delay")]
    pub order_delay_ms: u64,

    /// Minimum trade percentage threshold.
    #[serde(default = "default_min_trade_percent")]
    pub min_trade_percent: Decimal,

    /// Tolerance for considering allocation balanced.
    #[serde(default = "default_tolerance")]
    pub tolerance_percent: Decimal,
}

fn default_balance_interval() -> u64 {
    3600 // 1 hour
}

fn default_order_delay() -> u64 {
    1000 // 1 second
}

fn default_min_trade_percent() -> Decimal {
    Decimal::new(1, 1) // 0.1%
}

fn default_tolerance() -> Decimal {
    Decimal::new(5, 1) // 0.5%
}

impl Account {
    /// Creates a new account with the given ID and allocation.
    #[must_use]
    pub fn new(id: impl Into<String>, allocation: Allocation) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            exchange_account_id: String::new(),
            allocation,
            balance_interval_secs: default_balance_interval(),
            order_delay_ms: default_order_delay(),
            min_trade_percent: default_min_trade_percent(),
            tolerance_percent: default_tolerance(),
        }
    }

    /// Sets the account name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the exchange account ID.
    #[must_use]
    pub fn with_exchange_id(mut self, id: impl Into<String>) -> Self {
        self.exchange_account_id = id.into();
        self
    }

    /// Sets the balance interval.
    #[must_use]
    pub fn with_balance_interval(mut self, secs: u64) -> Self {
        self.balance_interval_secs = secs;
        self
    }
}

/// A user representing a connection to an exchange/broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier (auto-generated if not provided).
    #[serde(default)]
    pub id: String,

    /// Human-readable user name.
    #[serde(default)]
    pub name: String,

    /// The broker/exchange type.
    pub broker: BrokerType,

    /// Environment variable name for the API token.
    pub token_env_var: String,

    /// Additional API credentials (e.g., API secret for Binance).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub credentials: HashMap<String, String>,

    /// Trading accounts for this user.
    #[serde(default)]
    pub accounts: Vec<Account>,

    /// Whether this user is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl User {
    /// Creates a new user for a broker.
    #[must_use]
    pub fn new(broker: BrokerType, token_env_var: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            broker,
            token_env_var: token_env_var.into(),
            credentials: HashMap::new(),
            accounts: Vec::new(),
            enabled: true,
        }
    }

    /// Sets the user ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the user name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Adds a credential (e.g., API secret).
    #[must_use]
    pub fn with_credential(mut self, key: impl Into<String>, env_var: impl Into<String>) -> Self {
        self.credentials.insert(key.into(), env_var.into());
        self
    }

    /// Adds an account to this user.
    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }

    /// Returns an account by ID.
    #[must_use]
    pub fn get_account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Returns a mutable account by ID.
    pub fn get_account_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// Returns all enabled accounts.
    pub fn active_accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.iter()
    }

    /// Returns all tickers across all accounts.
    #[must_use]
    pub fn all_tickers(&self) -> Vec<String> {
        let mut tickers = Vec::new();
        for account in &self.accounts {
            for ticker in account.allocation.all_tickers() {
                if !tickers.contains(&ticker) {
                    tickers.push(ticker);
                }
            }
        }
        tickers
    }
}

/// Global settings for the trading bot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable verbose output.
    #[serde(default)]
    pub verbose: bool,

    /// Run in dry-run mode (no actual trades).
    #[serde(default)]
    pub dry_run: bool,

    /// Run in plan mode (show what would happen).
    #[serde(default)]
    pub plan: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Top-level configuration containing users and global settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradingConfig {
    /// Global settings.
    #[serde(default)]
    pub settings: Settings,

    /// User configurations.
    #[serde(default)]
    pub users: Vec<User>,
}

impl TradingConfig {
    /// Creates a new empty configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a user to the configuration.
    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }

    /// Returns a user by ID.
    #[must_use]
    pub fn get_user(&self, id: &str) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    /// Returns all enabled users.
    pub fn active_users(&self) -> impl Iterator<Item = &User> {
        self.users.iter().filter(|u| u.enabled)
    }

    /// Returns all accounts across all users.
    pub fn all_accounts(&self) -> impl Iterator<Item = (&User, &Account)> {
        self.users
            .iter()
            .flat_map(|u| u.accounts.iter().map(move |a| (u, a)))
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.users.is_empty() {
            return Err(ConfigValidationError::NoUsers);
        }

        for user in &self.users {
            if user.token_env_var.is_empty() {
                return Err(ConfigValidationError::MissingToken(user.id.clone()));
            }

            if user.accounts.is_empty() {
                return Err(ConfigValidationError::NoAccounts(user.id.clone()));
            }

            for account in &user.accounts {
                if !account.allocation.is_valid() {
                    return Err(ConfigValidationError::InvalidAllocation(
                        user.id.clone(),
                        account.id.clone(),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Configuration validation errors.
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    /// No users configured.
    NoUsers,
    /// Missing API token for a user.
    MissingToken(String),
    /// No accounts configured for a user.
    NoAccounts(String),
    /// Invalid allocation for an account.
    InvalidAllocation(String, String),
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUsers => write!(f, "No users configured"),
            Self::MissingToken(user) => write!(f, "Missing API token for user '{user}'"),
            Self::NoAccounts(user) => write!(f, "No accounts configured for user '{user}'"),
            Self::InvalidAllocation(user, account) => {
                write!(
                    f,
                    "Invalid allocation for account '{account}' of user '{user}'"
                )
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_broker_type_from_name() {
        assert_eq!(BrokerType::from_name("tbank"), BrokerType::TBank);
        assert_eq!(BrokerType::from_name("BINANCE"), BrokerType::Binance);
        assert_eq!(BrokerType::from_name("ib"), BrokerType::InteractiveBrokers);
        assert_eq!(BrokerType::from_name("simulator"), BrokerType::Simulator);
        assert_eq!(
            BrokerType::from_name("custom"),
            BrokerType::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_create_user() {
        let user = User::new(BrokerType::TBank, "TBANK_TOKEN")
            .with_name("Test User")
            .with_credential("secret", "TBANK_SECRET");

        assert_eq!(user.broker, BrokerType::TBank);
        assert_eq!(user.token_env_var, "TBANK_TOKEN");
        assert_eq!(user.name, "Test User");
        assert_eq!(
            user.credentials.get("secret"),
            Some(&"TBANK_SECRET".to_string())
        );
    }

    #[test]
    fn test_create_account() {
        let alloc = Allocation::from_simple(&[
            ("SBER".to_string(), dec!(50)),
            ("LKOH".to_string(), dec!(50)),
        ]);

        let account = Account::new("main", alloc)
            .with_name("Main Account")
            .with_exchange_id("12345");

        assert_eq!(account.id, "main");
        assert_eq!(account.name, "Main Account");
        assert_eq!(account.exchange_account_id, "12345");
    }

    #[test]
    fn test_config_validation() {
        let mut config = TradingConfig::new();

        // Empty config should fail
        assert!(config.validate().is_err());

        // Add user without accounts
        let user = User::new(BrokerType::TBank, "TOKEN");
        config.add_user(user);
        assert!(config.validate().is_err());

        // Add account
        let alloc =
            Allocation::from_simple(&[("A".to_string(), dec!(50)), ("B".to_string(), dec!(50))]);
        config.users[0].add_account(Account::new("main", alloc));

        // Now it should pass
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_all_tickers() {
        let mut user = User::new(BrokerType::TBank, "TOKEN");

        let alloc1 = Allocation::from_simple(&[
            ("SBER".to_string(), dec!(50)),
            ("LKOH".to_string(), dec!(50)),
        ]);
        user.add_account(Account::new("acc1", alloc1));

        let alloc2 = Allocation::from_simple(&[("GAZP".to_string(), dec!(100))]);
        user.add_account(Account::new("acc2", alloc2));

        let tickers = user.all_tickers();
        assert!(tickers.contains(&"SBER".to_string()));
        assert!(tickers.contains(&"LKOH".to_string()));
        assert!(tickers.contains(&"GAZP".to_string()));
    }
}
