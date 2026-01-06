//! Command-line interface for the trading bot.
//!
//! Uses clap for argument parsing with Links Notation configuration support.
//! Configuration is loaded with the following priority:
//!
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. Configuration file (--config or TRADER_BOT_CONFIG)
//! 4. Default values

use crate::config::{AppConfig, ConfigError};
use crate::lino_args::{auto_load_lenv, getenv, getenv_bool, getenv_u64};
use clap::Parser;
use std::path::PathBuf;
use tracing::Level;

/// Trader Bot - A configurable trading bot with multi-strategy support.
#[derive(Parser, Debug)]
#[command(
    name = "trader-bot",
    version,
    about = "A configurable trading bot with multi-exchange support, portfolio balancing, scalping strategies, and multi-account management"
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Path to configuration file (JSON format).
    #[arg(short, long, env = "TRADER_BOT_CONFIG")]
    pub config: Option<PathBuf>,

    /// Path to lenv file for environment variables.
    #[arg(short, long, env = "LENV_FILE")]
    pub lenv: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info", env = "TRADER_BOT_LOG_LEVEL")]
    pub log_level: String,

    /// Enable verbose output.
    #[arg(short, long, default_value = "false", env = "TRADER_BOT_VERBOSE")]
    pub verbose: bool,

    /// Run in dry-run mode (no actual trades).
    #[arg(long, default_value = "false", env = "TRADER_BOT_DRY_RUN")]
    pub dry_run: bool,

    /// Run in plan mode - connect to real market, calculate orders but don't execute them.
    /// Displays detailed information about what orders would be placed.
    #[arg(long, default_value = "false", env = "TRADER_BOT_PLAN")]
    pub plan: bool,

    /// Account ID to use (if not specified, uses all configured accounts).
    #[arg(short, long, env = "TRADER_BOT_ACCOUNT")]
    pub account: Option<String>,

    /// User ID to use (for multi-user setups).
    #[arg(short, long, env = "TRADER_BOT_USER")]
    pub user: Option<String>,

    /// Rebalancing interval in seconds (overrides config).
    #[arg(long, env = "TRADER_BOT_BALANCE_INTERVAL")]
    pub balance_interval: Option<u64>,

    /// Delay between orders in milliseconds.
    #[arg(long, env = "TRADER_BOT_ORDER_DELAY")]
    pub order_delay: Option<u64>,

    /// Run once and exit (instead of continuous operation).
    #[arg(long, default_value = "false", env = "TRADER_BOT_RUN_ONCE")]
    pub run_once: bool,

    /// Run demo mode with simulated exchange.
    #[arg(long, default_value = "false")]
    pub demo: bool,
}

impl Cli {
    /// Parses CLI arguments with Links Notation configuration support.
    ///
    /// This function:
    /// 1. Attempts to auto-load lenv file
    /// 2. Parses CLI arguments (which also reads env vars via clap)
    /// 3. Applies priority chain for configuration
    pub fn parse_with_lenv() -> Self {
        // Auto-load lenv file first (before clap parses env vars)
        // This ensures lenv values are available as env vars for clap
        let _ = auto_load_lenv();

        // Parse CLI with clap (will also use env vars)
        Self::parse()
    }

    /// Gets the effective log level.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn get_log_level(&self) -> Level {
        match self.log_level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }

    /// Loads configuration from file if specified.
    pub fn load_config(&self) -> Result<Option<AppConfig>, ConfigError> {
        if let Some(ref path) = self.config {
            let config = AppConfig::from_file(path)?;
            config.validate()?;
            return Ok(Some(config));
        }

        // Try to find config from environment or common locations
        let config_path = getenv("TRADER_BOT_CONFIG", "");
        if !config_path.is_empty() && std::path::Path::new(&config_path).exists() {
            let config = AppConfig::from_file(&config_path)?;
            config.validate()?;
            return Ok(Some(config));
        }

        // Check common config file locations
        for path in &["config.json", "trader-bot.json", ".trader-bot.json"] {
            if std::path::Path::new(path).exists() {
                let config = AppConfig::from_file(path)?;
                config.validate()?;
                return Ok(Some(config));
            }
        }

        Ok(None)
    }

    /// Creates a merged configuration from CLI args and file config.
    ///
    /// CLI arguments take priority over file config values.
    pub fn merge_with_config(&self, mut config: AppConfig) -> AppConfig {
        // Override global settings from CLI
        if self.verbose {
            config.settings.verbose = true;
        }
        if self.dry_run {
            config.settings.dry_run = true;
        }
        config.settings.log_level.clone_from(&self.log_level);

        // Override account settings if specified
        if let Some(interval) = self.balance_interval {
            for account in &mut config.accounts {
                account.balance_interval_secs = interval;
            }
            for user in &mut config.users {
                for account in &mut user.accounts {
                    account.balance_interval_secs = interval;
                }
            }
        }

        if let Some(delay) = self.order_delay {
            for account in &mut config.accounts {
                account.order_delay_ms = delay;
            }
            for user in &mut config.users {
                for account in &mut user.accounts {
                    account.order_delay_ms = delay;
                }
            }
        }

        config
    }
}

/// Runtime configuration combining CLI, environment, and file config.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RuntimeConfig {
    /// Path to configuration file.
    pub config_path: Option<PathBuf>,
    /// Path to lenv file.
    pub lenv_path: Option<PathBuf>,
    /// Log level.
    pub log_level: Level,
    /// Verbose output enabled.
    pub verbose: bool,
    /// Dry run mode enabled.
    pub dry_run: bool,
    /// Plan mode enabled - shows what orders would be placed without executing.
    pub plan: bool,
    /// Specific account to use.
    pub account: Option<String>,
    /// Specific user to use.
    pub user: Option<String>,
    /// Rebalancing interval override.
    pub balance_interval: Option<u64>,
    /// Order delay override.
    pub order_delay: Option<u64>,
    /// Run once mode.
    pub run_once: bool,
    /// Demo mode.
    pub demo: bool,
    /// Application config from file.
    pub app_config: Option<AppConfig>,
}

impl RuntimeConfig {
    /// Creates runtime config from CLI and loaded file config.
    pub fn from_cli(cli: Cli, app_config: Option<AppConfig>) -> Self {
        let log_level = cli.get_log_level();
        Self {
            config_path: cli.config,
            lenv_path: cli.lenv,
            log_level,
            verbose: cli.verbose,
            dry_run: cli.dry_run,
            plan: cli.plan,
            account: cli.account,
            user: cli.user,
            balance_interval: cli.balance_interval,
            order_delay: cli.order_delay,
            run_once: cli.run_once,
            demo: cli.demo,
            app_config,
        }
    }

    /// Creates runtime config from environment variables only (no CLI parsing).
    ///
    /// Useful for library usage or testing.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn from_env() -> Self {
        // Auto-load lenv
        let _ = auto_load_lenv();

        let log_level = match getenv("TRADER_BOT_LOG_LEVEL", "info")
            .to_lowercase()
            .as_str()
        {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };

        Self {
            config_path: std::env::var("TRADER_BOT_CONFIG").ok().map(PathBuf::from),
            lenv_path: std::env::var("LENV_FILE").ok().map(PathBuf::from),
            log_level,
            verbose: getenv_bool("TRADER_BOT_VERBOSE", false),
            dry_run: getenv_bool("TRADER_BOT_DRY_RUN", false),
            plan: getenv_bool("TRADER_BOT_PLAN", false),
            account: std::env::var("TRADER_BOT_ACCOUNT").ok(),
            user: std::env::var("TRADER_BOT_USER").ok(),
            balance_interval: {
                let v = getenv_u64("TRADER_BOT_BALANCE_INTERVAL", 0);
                if v > 0 {
                    Some(v)
                } else {
                    None
                }
            },
            order_delay: {
                let v = getenv_u64("TRADER_BOT_ORDER_DELAY", 0);
                if v > 0 {
                    Some(v)
                } else {
                    None
                }
            },
            run_once: getenv_bool("TRADER_BOT_RUN_ONCE", false),
            demo: false,
            app_config: None,
        }
    }
}
