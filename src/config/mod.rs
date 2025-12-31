//! Configuration management for the trading bot.
//!
//! This module provides configuration types for multi-user, multi-account,
//! and multi-strategy setups.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::domain::DesiredAllocation;
use crate::strategy::BalancerConfig;

/// Mode for determining desired allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AllocationMode {
    /// Manual allocation with fixed percentages.
    #[default]
    Manual,
    /// Allocation based on market capitalization.
    MarketCap,
    /// Allocation based on assets under management.
    Aum,
    /// Decorrelation strategy.
    Decorrelation,
}

/// Behavior when market is closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarketClosureBehavior {
    /// Skip the rebalancing iteration.
    #[default]
    Skip,
    /// Force orders (may be queued or rejected).
    Force,
    /// Perform calculations but don't place orders.
    DryRun,
}

/// Margin trading configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginConfig {
    /// Whether margin trading is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Portfolio multiplier (1.0 = no margin).
    #[serde(default = "default_multiplier")]
    pub multiplier: Decimal,
    /// Maximum margin size in base currency.
    #[serde(default)]
    pub max_margin_size: Option<Decimal>,
}

fn default_multiplier() -> Decimal {
    Decimal::ONE
}

impl Default for MarginConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            multiplier: Decimal::ONE,
            max_margin_size: None,
        }
    }
}

/// Configuration for a single trading account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Unique account identifier.
    pub id: String,
    /// Human-readable account name.
    pub name: String,
    /// Exchange/broker identifier.
    pub exchange: String,
    /// Exchange account ID (from broker).
    pub exchange_account_id: String,
    /// API token environment variable name (not the actual token).
    #[serde(default = "default_token_env")]
    pub token_env_var: String,
    /// Desired wallet allocation.
    #[serde(default)]
    pub desired_allocation: HashMap<String, Decimal>,
    /// Allocation mode.
    #[serde(default)]
    pub allocation_mode: AllocationMode,
    /// Rebalancing interval in seconds.
    #[serde(default = "default_balance_interval")]
    pub balance_interval_secs: u64,
    /// Delay between orders in milliseconds.
    #[serde(default = "default_order_delay")]
    pub order_delay_ms: u64,
    /// Margin trading configuration.
    #[serde(default)]
    pub margin: MarginConfig,
    /// Behavior when market is closed.
    #[serde(default)]
    pub market_closure_behavior: MarketClosureBehavior,
    /// Minimum profit percentage for closing positions.
    #[serde(default)]
    pub min_profit_percent_for_close: Option<Decimal>,
    /// Minimum trade percentage threshold.
    #[serde(default = "default_min_trade_percent")]
    pub min_trade_percent: Decimal,
    /// Tolerance for considering allocation balanced.
    #[serde(default = "default_tolerance")]
    pub tolerance_percent: Decimal,
}

fn default_token_env() -> String {
    "EXCHANGE_API_TOKEN".to_string()
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

impl AccountConfig {
    /// Converts the desired allocation to a DesiredAllocation instance.
    #[must_use]
    pub fn to_desired_allocation(&self) -> DesiredAllocation {
        DesiredAllocation::from_map(self.desired_allocation.clone())
    }

    /// Converts to a BalancerConfig.
    #[must_use]
    pub fn to_balancer_config(&self) -> BalancerConfig {
        BalancerConfig {
            min_trade_percent: self.min_trade_percent,
            min_trade_value: Decimal::new(100, 0), // 100 currency units
            tolerance_percent: self.tolerance_percent,
            order_delay_ms: self.order_delay_ms,
            dry_run: matches!(self.market_closure_behavior, MarketClosureBehavior::DryRun),
            max_orders_per_cycle: 20,
            min_profit_percent_for_close: self.min_profit_percent_for_close,
        }
    }
}

/// Configuration for a single user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Unique user identifier.
    pub id: String,
    /// Human-readable user name.
    pub name: String,
    /// Email address (optional).
    #[serde(default)]
    pub email: Option<String>,
    /// User's accounts.
    #[serde(default)]
    pub accounts: Vec<AccountConfig>,
    /// Whether the user is active.
    #[serde(default = "default_true")]
    pub active: bool,
}

fn default_true() -> bool {
    true
}

impl UserConfig {
    /// Creates a new user configuration.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            email: None,
            accounts: Vec::new(),
            active: true,
        }
    }

    /// Adds an account to the user.
    pub fn add_account(&mut self, account: AccountConfig) {
        self.accounts.push(account);
    }

    /// Gets an account by ID.
    #[must_use]
    pub fn get_account(&self, id: &str) -> Option<&AccountConfig> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Returns all active accounts.
    pub fn active_accounts(&self) -> impl Iterator<Item = &AccountConfig> {
        self.accounts.iter()
    }
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application version.
    #[serde(default = "default_version")]
    pub version: String,
    /// Global settings.
    #[serde(default)]
    pub settings: GlobalSettings,
    /// User configurations (for multi-user support).
    #[serde(default)]
    pub users: Vec<UserConfig>,
    /// Account configurations (for single-user mode, backwards compatible).
    #[serde(default)]
    pub accounts: Vec<AccountConfig>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Global application settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// Log level.
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Whether to enable verbose logging.
    #[serde(default)]
    pub verbose: bool,
    /// Whether to run in dry mode globally.
    #[serde(default)]
    pub dry_run: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl AppConfig {
    /// Loads configuration from a JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_json(&content)
    }

    /// Parses configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Serializes configuration to JSON.
    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))
    }

    /// Gets an account by ID (searches both direct accounts and user accounts).
    #[must_use]
    pub fn get_account(&self, id: &str) -> Option<&AccountConfig> {
        // First check direct accounts
        if let Some(account) = self.accounts.iter().find(|a| a.id == id) {
            return Some(account);
        }

        // Then check user accounts
        for user in &self.users {
            if let Some(account) = user.get_account(id) {
                return Some(account);
            }
        }

        None
    }

    /// Gets a user by ID.
    #[must_use]
    pub fn get_user(&self, id: &str) -> Option<&UserConfig> {
        self.users.iter().find(|u| u.id == id)
    }

    /// Returns all accounts from all users plus direct accounts.
    pub fn all_accounts(&self) -> impl Iterator<Item = &AccountConfig> {
        self.accounts
            .iter()
            .chain(self.users.iter().flat_map(|u| u.accounts.iter()))
    }

    /// Returns all active users.
    pub fn active_users(&self) -> impl Iterator<Item = &UserConfig> {
        self.users.iter().filter(|u| u.active)
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check we have at least some accounts (either direct or via users)
        let has_accounts =
            !self.accounts.is_empty() || self.users.iter().any(|u| !u.accounts.is_empty());

        if !has_accounts {
            return Err(ConfigError::ValidationError(
                "No accounts configured".to_string(),
            ));
        }

        // Validate direct accounts
        for account in &self.accounts {
            Self::validate_account(account)?;
        }

        // Validate user accounts
        for user in &self.users {
            if user.id.is_empty() {
                return Err(ConfigError::ValidationError(
                    "User ID cannot be empty".to_string(),
                ));
            }
            for account in &user.accounts {
                Self::validate_account(account)?;
            }
        }

        Ok(())
    }

    fn validate_account(account: &AccountConfig) -> Result<(), ConfigError> {
        if account.id.is_empty() {
            return Err(ConfigError::ValidationError(
                "Account ID cannot be empty".to_string(),
            ));
        }
        if account.exchange.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "Account {} has no exchange specified",
                account.id
            )));
        }

        // Validate allocation sums to ~100% for manual mode
        if account.allocation_mode == AllocationMode::Manual
            && !account.desired_allocation.is_empty()
        {
            let total: Decimal = account.desired_allocation.values().copied().sum();
            let diff = (total - Decimal::from(100)).abs();
            if diff > Decimal::from(1) {
                return Err(ConfigError::ValidationError(format!(
                    "Account {} allocation sums to {total}%, expected ~100%",
                    account.id
                )));
            }
        }

        Ok(())
    }
}

/// Configuration-related errors.
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// File I/O error.
    IoError(String),
    /// JSON parsing error.
    ParseError(String),
    /// JSON serialization error.
    SerializeError(String),
    /// Configuration validation error.
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
            Self::SerializeError(e) => write!(f, "Serialize error: {e}"),
            Self::ValidationError(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_config() {
        let json = r#"{
            "version": "1.0.0",
            "settings": {
                "log_level": "debug",
                "verbose": true
            },
            "accounts": [
                {
                    "id": "main",
                    "name": "Main Account",
                    "exchange": "tbank",
                    "exchange_account_id": "12345",
                    "desired_allocation": {
                        "SBER": 30,
                        "LKOH": 30,
                        "GAZP": 40
                    },
                    "allocation_mode": "manual",
                    "balance_interval_secs": 3600
                }
            ]
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.accounts.len(), 1);
        assert_eq!(config.accounts[0].id, "main");
        assert_eq!(
            config.accounts[0].desired_allocation.get("SBER"),
            Some(&dec!(30))
        );
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            version: "1.0.0".into(),
            settings: GlobalSettings::default(),
            users: vec![],
            accounts: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_allocation_validation() {
        let mut desired = HashMap::new();
        desired.insert("A".to_string(), dec!(30));
        desired.insert("B".to_string(), dec!(30));
        // Only 60% allocated

        let config = AppConfig {
            version: "1.0.0".into(),
            settings: GlobalSettings::default(),
            users: vec![],
            accounts: vec![AccountConfig {
                id: "test".into(),
                name: "Test".into(),
                exchange: "sim".into(),
                exchange_account_id: "123".into(),
                token_env_var: "TOKEN".into(),
                desired_allocation: desired,
                allocation_mode: AllocationMode::Manual,
                balance_interval_secs: 3600,
                order_delay_ms: 1000,
                margin: MarginConfig::default(),
                market_closure_behavior: MarketClosureBehavior::Skip,
                min_profit_percent_for_close: None,
                min_trade_percent: dec!(0.1),
                tolerance_percent: dec!(0.5),
            }],
        };

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_to_balancer_config() {
        let account = AccountConfig {
            id: "test".into(),
            name: "Test".into(),
            exchange: "sim".into(),
            exchange_account_id: "123".into(),
            token_env_var: "TOKEN".into(),
            desired_allocation: HashMap::new(),
            allocation_mode: AllocationMode::Manual,
            balance_interval_secs: 3600,
            order_delay_ms: 500,
            margin: MarginConfig::default(),
            market_closure_behavior: MarketClosureBehavior::Skip,
            min_profit_percent_for_close: Some(dec!(5)),
            min_trade_percent: dec!(0.2),
            tolerance_percent: dec!(1),
        };

        let balancer_config = account.to_balancer_config();
        assert_eq!(balancer_config.order_delay_ms, 500);
        assert_eq!(balancer_config.min_trade_percent, dec!(0.2));
        assert_eq!(balancer_config.min_profit_percent_for_close, Some(dec!(5)));
    }
}
