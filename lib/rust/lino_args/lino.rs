//! Links Notation (Lino) parser for hierarchical configuration.
//!
//! This module provides parsing for indented Links Notation format, supporting
//! hierarchical nested structures for complex configuration.
//!
//! # Format
//!
//! Links Notation uses indentation to represent hierarchy:
//! - Lines are key-value pairs or just keys introducing a nested block
//! - Indentation (2 spaces per level recommended) defines nesting depth
//! - `key value` - Key with a simple value
//! - `key:` - Key introducing a nested block (children follow indented)
//! - `# comment` - Comments (lines starting with #)
//! - `(item)` - Tuple/list item
//!
//! # Example
//!
//! ```text
//! # Multi-user trading configuration
//! settings
//!   log_level info
//!   verbose false
//!
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
//!   )
//! ```

use std::collections::HashMap;

/// A parsed Links Notation value - can be a string, list, or nested structure.
#[derive(Debug, Clone, PartialEq)]
pub enum LinoValue {
    /// A simple string value.
    String(String),
    /// A list of values (from parentheses syntax).
    List(Vec<LinoNode>),
    /// A nested object/map structure.
    Object(HashMap<String, Self>),
    /// A numeric value.
    Number(f64),
    /// A boolean value.
    Bool(bool),
    /// Null/empty value.
    Null,
}

impl LinoValue {
    /// Returns the value as a string, if it is one.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the value as a string, converting numbers and bools.
    #[must_use]
    pub fn to_string_value(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Null => String::new(),
            Self::List(_) | Self::Object(_) => String::new(),
        }
    }

    /// Returns the value as a bool.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Some(true),
                "false" | "no" | "off" | "0" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns the value as a number.
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Returns the value as an object.
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Returns the value as a list.
    #[must_use]
    pub fn as_list(&self) -> Option<&[LinoNode]> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Gets a nested value by key path (dot-separated or slash-separated).
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&Self> {
        let parts: Vec<&str> = path.split(['.', '/']).collect();
        let mut current = self;

        for part in parts {
            match current {
                Self::Object(obj) => {
                    current = obj.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Gets a string value by path.
    #[must_use]
    pub fn get_str(&self, path: &str) -> Option<&str> {
        self.get(path).and_then(Self::as_str)
    }

    /// Gets a bool value by path.
    #[must_use]
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path).and_then(Self::as_bool)
    }

    /// Gets a number value by path.
    #[must_use]
    pub fn get_number(&self, path: &str) -> Option<f64> {
        self.get(path).and_then(Self::as_number)
    }
}

impl Default for LinoValue {
    fn default() -> Self {
        Self::Null
    }
}

/// A node in the Links Notation structure.
#[derive(Debug, Clone, PartialEq)]
pub struct LinoNode {
    /// The key/name of this node (empty for list items).
    pub key: String,
    /// The value of this node.
    pub value: LinoValue,
}

impl LinoNode {
    /// Creates a new node with a key and value.
    #[must_use]
    pub fn new(key: impl Into<String>, value: LinoValue) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }

    /// Creates a new string node.
    #[must_use]
    pub fn string(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: LinoValue::String(value.into()),
        }
    }
}

/// Error type for Links Notation parsing.
#[derive(Debug, Clone)]
pub struct LinoParseError {
    /// Line number where the error occurred.
    pub line: usize,
    /// Description of the error.
    pub message: String,
}

impl std::fmt::Display for LinoParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for LinoParseError {}

/// Represents a parsed line with its indentation level.
#[derive(Debug)]
struct ParsedLine {
    indent: usize,
    content: String,
    line_num: usize,
}

/// Parses Links Notation content into a structured value.
///
/// # Arguments
///
/// * `content` - The Links Notation text to parse
///
/// # Returns
///
/// Returns a `LinoValue::Object` containing the parsed structure.
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::lino::parse_lino;
///
/// let content = r#"
/// settings
///   log_level info
///   verbose false
///
/// users
///   user1
///     name John
/// "#;
///
/// let config = parse_lino(content).unwrap();
/// assert_eq!(config.get_str("settings/log_level"), Some("info"));
/// ```
pub fn parse_lino(content: &str) -> Result<LinoValue, LinoParseError> {
    let lines = parse_lines(content);
    if lines.is_empty() {
        return Ok(LinoValue::Object(HashMap::new()));
    }

    let (value, _) = parse_block(&lines, 0, 0)?;
    Ok(value)
}

/// Parses raw content into ParsedLine structures.
fn parse_lines(content: &str) -> Vec<ParsedLine> {
    let mut lines = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Calculate indentation (count leading spaces)
        let indent = line.len() - line.trim_start().len();

        lines.push(ParsedLine {
            indent,
            content: trimmed.to_string(),
            line_num: line_num + 1,
        });
    }

    lines
}

/// Parses a block of lines at a given indentation level.
fn parse_block(
    lines: &[ParsedLine],
    start: usize,
    base_indent: usize,
) -> Result<(LinoValue, usize), LinoParseError> {
    let mut result: HashMap<String, LinoValue> = HashMap::new();
    let mut list_items: Vec<LinoNode> = Vec::new();
    let mut is_list = false;
    let mut i = start;

    while i < lines.len() {
        let line = &lines[i];

        // If we've dedented past our block, we're done
        if line.indent < base_indent && i > start {
            break;
        }

        // Skip lines that are deeper than expected (they belong to a parent block)
        if line.indent > base_indent && i == start {
            i += 1;
            continue;
        }

        // Handle lines at our indentation level
        if line.indent == base_indent || (i == start && line.indent >= base_indent) {
            let current_indent = line.indent;

            // Check if this is a list item (starts with parenthesis)
            if line.content.starts_with('(') {
                is_list = true;

                // Check if it's a single-line tuple or multi-line block
                if line.content.ends_with(')') && line.content.len() > 2 {
                    // Single line tuple: (key value)
                    let inner = &line.content[1..line.content.len() - 1];
                    let node = parse_tuple_content(inner, line.line_num)?;
                    list_items.push(node);
                    i += 1;
                } else if line.content == "(" {
                    // Multi-line block - parse children
                    let child_indent = if i + 1 < lines.len() {
                        lines[i + 1].indent
                    } else {
                        current_indent + 2
                    };

                    let (child_value, next_i) = parse_block(lines, i + 1, child_indent)?;
                    list_items.push(LinoNode::new("", child_value));

                    // Find the closing parenthesis
                    i = next_i;
                    while i < lines.len() && lines[i].content != ")" {
                        i += 1;
                    }
                    if i < lines.len() && lines[i].content == ")" {
                        i += 1;
                    }
                } else {
                    // Inline content after opening paren
                    i += 1;
                }
            } else if line.content == ")" {
                // End of list block
                i += 1;
                break;
            } else {
                // Regular key-value or key with children
                let (key, value_str) = parse_key_value(&line.content);

                // Check if there are children at a deeper indentation
                let has_children = i + 1 < lines.len() && lines[i + 1].indent > current_indent;

                if has_children {
                    // Parse nested block
                    let child_indent = lines[i + 1].indent;
                    let (child_value, next_i) = parse_block(lines, i + 1, child_indent)?;

                    // If there was an inline value too, merge it
                    if let Some(v) = value_str {
                        if let LinoValue::Object(mut obj) = child_value {
                            // Add the inline value as a special "_value" key
                            obj.insert("_value".to_string(), parse_value_string(&v));
                            result.insert(key, LinoValue::Object(obj));
                        } else {
                            result.insert(key, child_value);
                        }
                    } else {
                        result.insert(key, child_value);
                    }

                    i = next_i;
                } else {
                    // Simple key-value
                    let value = value_str
                        .map(|v| parse_value_string(&v))
                        .unwrap_or(LinoValue::Null);
                    result.insert(key, value);
                    i += 1;
                }
            }
        } else {
            // Line is at different indentation - move on
            i += 1;
        }
    }

    if is_list {
        Ok((LinoValue::List(list_items), i))
    } else {
        Ok((LinoValue::Object(result), i))
    }
}

/// Parses a single line into key and optional value.
fn parse_key_value(line: &str) -> (String, Option<String>) {
    // Handle quoted keys like "log level" info
    if line.starts_with('"') {
        if let Some(end_quote) = line[1..].find('"') {
            let key = &line[1..=end_quote];
            let rest = line[end_quote + 2..].trim();

            if rest.is_empty() {
                return (normalize_key(key), None);
            }

            // Check for colon separator after the quoted key
            if rest.starts_with(": ") {
                return (normalize_key(key), Some(rest[2..].trim().to_string()));
            }

            return (normalize_key(key), Some(rest.to_string()));
        }
    }

    // Handle "key: value" format (Links Notation with colon)
    if let Some(colon_pos) = line.find(": ") {
        let key = line[..colon_pos].trim();
        let value = line[colon_pos + 2..].trim();
        return (normalize_key(key), Some(value.to_string()));
    }

    // Handle "key value" format (space separated)
    if let Some(space_pos) = line.find(' ') {
        let key = line[..space_pos].trim();
        let value = line[space_pos + 1..].trim();

        // Check if key ends with colon (block indicator)
        if let Some(key) = key.strip_suffix(':') {
            if value.is_empty() {
                return (normalize_key(key), None);
            }
            return (normalize_key(key), Some(value.to_string()));
        }

        return (normalize_key(key), Some(value.to_string()));
    }

    // Handle "key:" format (block indicator with no value)
    if let Some(key) = line.strip_suffix(':') {
        return (normalize_key(key), None);
    }

    // Just a key
    (normalize_key(line), None)
}

/// Parses tuple content like "key value" or "key: value".
fn parse_tuple_content(content: &str, _line_num: usize) -> Result<LinoNode, LinoParseError> {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return Ok(LinoNode::new("", LinoValue::Null));
    }

    let (key, value) = parse_key_value(trimmed);
    let value = value
        .map(|v| parse_value_string(&v))
        .unwrap_or(LinoValue::Null);

    Ok(LinoNode::new(key, value))
}

/// Normalizes a key by replacing quoted spaces with underscores.
fn normalize_key(key: &str) -> String {
    // Handle quoted keys like "log level"
    if key.starts_with('"') && key.ends_with('"') && key.len() > 2 {
        return key[1..key.len() - 1].replace(' ', "_");
    }

    // Replace spaces in unquoted keys with underscores
    key.replace(' ', "_")
}

/// Parses a value string into the appropriate LinoValue type.
fn parse_value_string(s: &str) -> LinoValue {
    let s = s.trim();

    // Handle quoted strings
    if ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
        && s.len() >= 2
    {
        return LinoValue::String(s[1..s.len() - 1].to_string());
    }

    // Handle booleans
    match s.to_lowercase().as_str() {
        "true" | "yes" | "on" => return LinoValue::Bool(true),
        "false" | "no" | "off" => return LinoValue::Bool(false),
        "null" | "nil" | "none" => return LinoValue::Null,
        _ => {}
    }

    // Handle numbers
    if let Ok(n) = s.parse::<f64>() {
        return LinoValue::Number(n);
    }

    // Default to string
    LinoValue::String(s.to_string())
}

/// Flattens a LinoValue into environment variable style key-value pairs.
///
/// This is useful for converting hierarchical config to flat environment variables.
///
/// # Example
///
/// Given this config:
/// ```text
/// settings
///   log_level info
/// ```
///
/// Produces:
/// ```text
/// SETTINGS_LOG_LEVEL=info
/// ```
pub fn flatten_to_env(value: &LinoValue, prefix: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    flatten_recursive(value, prefix, &mut result);
    result
}

fn flatten_recursive(value: &LinoValue, prefix: &str, result: &mut Vec<(String, String)>) {
    match value {
        LinoValue::Object(obj) => {
            for (key, val) in obj {
                let new_prefix = if prefix.is_empty() {
                    key.to_uppercase()
                } else {
                    format!("{}_{}", prefix, key.to_uppercase())
                };
                flatten_recursive(val, &new_prefix, result);
            }
        }
        LinoValue::List(list) => {
            for (i, node) in list.iter().enumerate() {
                let new_prefix = format!("{}_{}", prefix, i);
                flatten_recursive(&node.value, &new_prefix, result);
            }
        }
        LinoValue::String(s) => {
            if !prefix.is_empty() {
                result.push((prefix.to_string(), s.clone()));
            }
        }
        LinoValue::Number(n) => {
            if !prefix.is_empty() {
                result.push((prefix.to_string(), n.to_string()));
            }
        }
        LinoValue::Bool(b) => {
            if !prefix.is_empty() {
                result.push((prefix.to_string(), b.to_string()));
            }
        }
        LinoValue::Null => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key_value() {
        let content = r"
key value
another_key another_value
";
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_str("key"), Some("value"));
        assert_eq!(result.get_str("another_key"), Some("another_value"));
    }

    #[test]
    fn test_parse_nested_structure() {
        let content = r"
settings
  log_level info
  verbose false
";
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_str("settings/log_level"), Some("info"));
        assert_eq!(result.get_bool("settings/verbose"), Some(false));
    }

    #[test]
    fn test_parse_deeply_nested() {
        let content = r"
users
  user1
    name John
    accounts
      main
        balance 1000
";
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_str("users/user1/name"), Some("John"));
        assert_eq!(
            result.get_number("users/user1/accounts/main/balance"),
            Some(1000.0)
        );
    }

    #[test]
    fn test_parse_with_colons() {
        let content = r"
settings:
  log_level: info
  verbose: false
";
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_str("settings/log_level"), Some("info"));
    }

    #[test]
    fn test_parse_allocation_config() {
        let content = r"
allocation
  SBER 30
  LKOH 30
  GAZP 40
";
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_number("allocation/SBER"), Some(30.0));
        assert_eq!(result.get_number("allocation/LKOH"), Some(30.0));
        assert_eq!(result.get_number("allocation/GAZP"), Some(40.0));
    }

    #[test]
    fn test_parse_comments() {
        let content = r"
# This is a comment
key value
# Another comment
another_key value2
";
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_str("key"), Some("value"));
        assert_eq!(result.get_str("another_key"), Some("value2"));
    }

    #[test]
    fn test_flatten_to_env() {
        let content = r"
settings
  log_level info
  verbose true
";
        let result = parse_lino(content).unwrap();
        let env_vars = flatten_to_env(&result, "");

        assert!(env_vars.contains(&("SETTINGS_LOG_LEVEL".to_string(), "info".to_string())));
        assert!(env_vars.contains(&("SETTINGS_VERBOSE".to_string(), "true".to_string())));
    }

    #[test]
    fn test_parse_list_items() {
        let content = r"
users
  (
    name John
    broker tbank
  )
  (
    name Jane
    broker binance
  )
";
        let result = parse_lino(content).unwrap();
        if let Some(LinoValue::List(users)) = result.get("users") {
            assert_eq!(users.len(), 2);
        } else {
            panic!("Expected users to be a list");
        }
    }

    #[test]
    fn test_parse_quoted_keys() {
        let content = r#"
"log level" info
"dry run" false
"#;
        let result = parse_lino(content).unwrap();
        assert_eq!(result.get_str("log_level"), Some("info"));
        assert_eq!(result.get_bool("dry_run"), Some(false));
    }
}
