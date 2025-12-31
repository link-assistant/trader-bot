//! Case conversion utilities for variable name handling.
//!
//! Provides functions to convert between different naming conventions:
//! - UPPER_CASE (environment variable style)
//! - camelCase (JavaScript style)
//! - kebab-case (CLI argument style)
//! - snake_case (Rust/Python style)
//! - PascalCase (type name style)

/// Converts a string to UPPER_CASE (SCREAMING_SNAKE_CASE).
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::to_upper_case;
///
/// assert_eq!(to_upper_case("apiKey"), "API_KEY");
/// assert_eq!(to_upper_case("myVariableName"), "MY_VARIABLE_NAME");
/// assert_eq!(to_upper_case("api-key"), "API_KEY");
/// assert_eq!(to_upper_case("API_KEY"), "API_KEY");
/// ```
#[must_use]
pub fn to_upper_case(s: &str) -> String {
    // If already uppercase with underscores, just replace hyphens
    if s.chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c == '-')
    {
        return s.replace('-', "_");
    }

    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        if *c == '-' || *c == ' ' {
            result.push('_');
        } else {
            result.push(c.to_ascii_uppercase());
        }
    }

    // Clean up leading underscores and double underscores
    result = result.trim_start_matches('_').to_string();
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    result
}

/// Converts a string to camelCase.
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::to_camel_case;
///
/// assert_eq!(to_camel_case("api-key"), "apiKey");
/// assert_eq!(to_camel_case("API_KEY"), "apiKey");
/// assert_eq!(to_camel_case("my_variable_name"), "myVariableName");
/// ```
#[must_use]
pub fn to_camel_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in lower.chars() {
        if c == '-' || c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    // Ensure first character is lowercase
    if let Some(first) = result.chars().next() {
        if first.is_uppercase() {
            result = first.to_lowercase().to_string() + &result[1..];
        }
    }

    result
}

/// Converts a string to kebab-case.
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::to_kebab_case;
///
/// assert_eq!(to_kebab_case("apiKey"), "api-key");
/// assert_eq!(to_kebab_case("API_KEY"), "api-key");
/// assert_eq!(to_kebab_case("MyVariableName"), "my-variable-name");
/// ```
#[must_use]
pub fn to_kebab_case(s: &str) -> String {
    // If all uppercase with underscores, convert directly
    if s.chars().all(|c| c.is_ascii_uppercase() || c == '_') && s.contains('_') {
        return s.replace('_', "-").to_lowercase();
    }

    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('-');
        }
        if *c == '_' || *c == ' ' {
            result.push('-');
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    // Clean up leading hyphens and double hyphens
    result = result.trim_start_matches('-').to_string();
    while result.contains("--") {
        result = result.replace("--", "-");
    }

    result
}

/// Converts a string to snake_case.
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::to_snake_case;
///
/// assert_eq!(to_snake_case("apiKey"), "api_key");
/// assert_eq!(to_snake_case("api-key"), "api_key");
/// assert_eq!(to_snake_case("API_KEY"), "api_key");
/// ```
#[must_use]
pub fn to_snake_case(s: &str) -> String {
    // If all uppercase with underscores, just lowercase it
    if s.chars().all(|c| c.is_ascii_uppercase() || c == '_') && s.contains('_') {
        return s.to_lowercase();
    }

    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        if *c == '-' || *c == ' ' {
            result.push('_');
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    // Clean up leading underscores and double underscores
    result = result.trim_start_matches('_').to_string();
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    result
}

/// Converts a string to PascalCase.
///
/// # Examples
///
/// ```
/// use trader_bot::lino_args::to_pascal_case;
///
/// assert_eq!(to_pascal_case("api-key"), "ApiKey");
/// assert_eq!(to_pascal_case("api_key"), "ApiKey");
/// assert_eq!(to_pascal_case("my-variable-name"), "MyVariableName");
/// ```
#[must_use]
pub fn to_pascal_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in lower.chars() {
        if c == '-' || c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
