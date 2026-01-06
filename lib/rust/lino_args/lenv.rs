//! Links Notation Environment (lenv) file loading.
//!
//! Provides functionality to load configuration from `.lenv` files using
//! [Links Notation](https://github.com/link-foundation/links-notation) format.
//!
//! # Format
//!
//! The lenv format uses `: ` (colon-space) as separator following Links Notation:
//! - `KEY: value` - Simple key-value pairs
//! - `KEY: "value with spaces"` - Quoted values
//! - `KEY: 'value with spaces'` - Single-quoted values
//! - `# comment` - Comments (lines starting with #)
//! - Empty lines are ignored
//! - Indented nested structures are supported
//!
//! # Example
//!
//! ```text
//! # API tokens
//! TBANK_API_TOKEN: your_token_here
//! BINANCE_API_KEY: your_key_here
//!
//! # Settings
//! TRADER_BOT_LOG_LEVEL: info
//! TRADER_BOT_VERBOSE: false
//! ```

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Error type for lenv file operations.
#[derive(Debug)]
pub enum LenvError {
    /// IO error when reading the file.
    Io(io::Error),
    /// Parse error with line number and description.
    Parse {
        /// Line number where the error occurred.
        line: usize,
        /// Description of the parse error.
        message: String,
    },
}

impl std::fmt::Display for LenvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse { line, message } => write!(f, "Parse error on line {line}: {message}"),
        }
    }
}

impl std::error::Error for LenvError {}

impl From<io::Error> for LenvError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Parses a lenv file content into a key-value map.
///
/// # Format
///
/// The lenv format uses Links Notation with `: ` (colon-space) separator:
/// - `KEY: value` - Simple key-value pairs
/// - `KEY: "value with spaces"` - Quoted values
/// - `KEY: 'value with spaces'` - Single-quoted values
/// - `# comment` - Comments (lines starting with #)
/// - Empty lines are ignored
///
/// For backwards compatibility, `=` separator is also supported.
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::parse_lenv;
///
/// // Links Notation format (preferred)
/// let content = "# Configuration\nAPI_KEY: my_secret_key\nPORT: 8080\nDEBUG: true\n";
///
/// let vars = parse_lenv(content).unwrap();
/// assert_eq!(vars.get("API_KEY"), Some(&"my_secret_key".to_string()));
/// assert_eq!(vars.get("PORT"), Some(&"8080".to_string()));
/// ```
pub fn parse_lenv(content: &str) -> Result<HashMap<String, String>, LenvError> {
    let mut vars = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Try Links Notation format first (": " separator)
        // Then fall back to traditional format ("=" separator)
        let (key, value) = if let Some(colon_pos) = trimmed.find(": ") {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 2..].trim();
            (key, value)
        } else if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 1..].trim();
            (key, value)
        } else {
            return Err(LenvError::Parse {
                line: line_num + 1,
                message: "Missing ': ' or '=' separator".to_string(),
            });
        };

        if key.is_empty() {
            return Err(LenvError::Parse {
                line: line_num + 1,
                message: "Empty key".to_string(),
            });
        }

        // Handle quoted values
        let parsed_value = if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            // Remove quotes
            if value.len() >= 2 {
                value[1..value.len() - 1].to_string()
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        };

        vars.insert(key.to_string(), parsed_value);
    }

    Ok(vars)
}

/// Loads a lenv file and returns its key-value pairs.
///
/// # Arguments
///
/// * `path` - Path to the lenv file
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_lenv<P: AsRef<Path>>(path: P) -> Result<HashMap<String, String>, LenvError> {
    let content = fs::read_to_string(path)?;
    parse_lenv(&content)
}

/// Loads a lenv file and sets the values as environment variables.
///
/// Existing environment variables are NOT overwritten. This follows the
/// priority chain where existing env vars take precedence over file values.
///
/// # Arguments
///
/// * `path` - Path to the lenv file
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_lenv_into_env<P: AsRef<Path>>(path: P) -> Result<usize, LenvError> {
    let vars = load_lenv(path)?;
    let mut loaded = 0;

    for (key, value) in vars {
        // Only set if not already defined
        if std::env::var(&key).is_err() {
            std::env::set_var(&key, value);
            loaded += 1;
        }
    }

    Ok(loaded)
}

/// Tries to load a lenv file from common locations.
///
/// Searches in order:
/// 1. Path specified in `--lenv` or `-l` CLI argument
/// 2. `LENV_FILE` environment variable
/// 3. `.lenv` in current directory
/// 4. `.lenv` in home directory
///
/// # Returns
///
/// Returns `Ok(Some(count))` if a file was found and loaded (count = vars loaded),
/// `Ok(None)` if no file was found, or `Err` if a file was found but couldn't be parsed.
pub fn auto_load_lenv() -> Result<Option<usize>, LenvError> {
    // Check CLI argument first
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if (args[i] == "--lenv" || args[i] == "-l") && i + 1 < args.len() {
            let loaded = load_lenv_into_env(&args[i + 1])?;
            return Ok(Some(loaded));
        }
        if args[i].starts_with("--lenv=") {
            let path = &args[i][7..];
            let loaded = load_lenv_into_env(path)?;
            return Ok(Some(loaded));
        }
    }

    // Check LENV_FILE environment variable
    if let Ok(path) = std::env::var("LENV_FILE") {
        if Path::new(&path).exists() {
            let loaded = load_lenv_into_env(&path)?;
            return Ok(Some(loaded));
        }
    }

    // Check current directory
    if Path::new(".lenv").exists() {
        let loaded = load_lenv_into_env(".lenv")?;
        return Ok(Some(loaded));
    }

    // Check home directory
    if let Some(home) = dirs_home() {
        let home_lenv = home.join(".lenv");
        if home_lenv.exists() {
            let loaded = load_lenv_into_env(&home_lenv)?;
            return Ok(Some(loaded));
        }
    }

    Ok(None)
}

/// Gets the home directory path.
fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .map(std::path::PathBuf::from)
        })
}
