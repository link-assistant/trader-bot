//! Tests for the lino-arguments module.

use super::*;

mod case_conversion {
    use super::*;

    #[test]
    fn test_to_upper_case() {
        assert_eq!(to_upper_case("apiKey"), "API_KEY");
        assert_eq!(to_upper_case("myVariableName"), "MY_VARIABLE_NAME");
        assert_eq!(to_upper_case("api-key"), "API_KEY");
        assert_eq!(to_upper_case("API_KEY"), "API_KEY");
        assert_eq!(to_upper_case("api_key"), "API_KEY");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("api-key"), "apiKey");
        assert_eq!(to_camel_case("API_KEY"), "apiKey");
        assert_eq!(to_camel_case("my_variable_name"), "myVariableName");
        assert_eq!(to_camel_case("my-variable-name"), "myVariableName");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("apiKey"), "api-key");
        assert_eq!(to_kebab_case("API_KEY"), "api-key");
        assert_eq!(to_kebab_case("MyVariableName"), "my-variable-name");
        assert_eq!(to_kebab_case("my_variable"), "my-variable");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("apiKey"), "api_key");
        assert_eq!(to_snake_case("api-key"), "api_key");
        assert_eq!(to_snake_case("API_KEY"), "api_key");
        assert_eq!(to_snake_case("MyVar"), "my_var");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("api-key"), "ApiKey");
        assert_eq!(to_pascal_case("api_key"), "ApiKey");
        assert_eq!(to_pascal_case("my-variable-name"), "MyVariableName");
        assert_eq!(to_pascal_case("API_KEY"), "ApiKey");
    }
}

mod getenv_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_getenv_with_default() {
        let result = getenv("NON_EXISTENT_VAR_12345_LINO_TEST", "default");
        assert_eq!(result, "default");
    }

    #[test]
    fn test_getenv_finds_var() {
        env::set_var("TEST_LINO_VAR_UNIQUE", "test_value");
        let result = getenv("TEST_LINO_VAR_UNIQUE", "default");
        assert_eq!(result, "test_value");
        env::remove_var("TEST_LINO_VAR_UNIQUE");
    }

    #[test]
    fn test_getenv_int() {
        env::set_var("TEST_LINO_PORT", "8080");
        let result = getenv_int("TEST_LINO_PORT", 3000);
        assert_eq!(result, 8080);
        env::remove_var("TEST_LINO_PORT");
    }

    #[test]
    fn test_getenv_int_invalid() {
        env::set_var("TEST_LINO_PORT_INVALID", "not_a_number");
        let result = getenv_int("TEST_LINO_PORT_INVALID", 3000);
        assert_eq!(result, 3000);
        env::remove_var("TEST_LINO_PORT_INVALID");
    }

    #[test]
    fn test_getenv_bool_true_values() {
        for value in &["true", "1", "yes", "on", "TRUE", "YES", "ON"] {
            env::set_var("TEST_LINO_DEBUG", value);
            let result = getenv_bool("TEST_LINO_DEBUG", false);
            assert!(result, "Expected true for value: {}", value);
            env::remove_var("TEST_LINO_DEBUG");
        }
    }

    #[test]
    fn test_getenv_bool_false_values() {
        for value in &["false", "0", "no", "off", "FALSE", "NO", "OFF"] {
            env::set_var("TEST_LINO_DEBUG", value);
            let result = getenv_bool("TEST_LINO_DEBUG", true);
            assert!(!result, "Expected false for value: {}", value);
            env::remove_var("TEST_LINO_DEBUG");
        }
    }

    #[test]
    fn test_getenv_bool_invalid() {
        env::set_var("TEST_LINO_DEBUG_INVALID", "maybe");
        let result = getenv_bool("TEST_LINO_DEBUG_INVALID", true);
        assert!(result);
        env::remove_var("TEST_LINO_DEBUG_INVALID");
    }
}

mod lenv_tests {
    use super::*;

    #[test]
    fn test_parse_lenv_simple() {
        let content = r"
API_KEY=my_secret
PORT=8080
";
        let vars = parse_lenv(content).unwrap();
        assert_eq!(vars.get("API_KEY"), Some(&"my_secret".to_string()));
        assert_eq!(vars.get("PORT"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_parse_lenv_with_comments() {
        let content = r"
# This is a comment
API_KEY=secret
# Another comment
DEBUG=true
";
        let vars = parse_lenv(content).unwrap();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars.get("API_KEY"), Some(&"secret".to_string()));
        assert_eq!(vars.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_lenv_quoted_values() {
        let content = r#"
MESSAGE="Hello, World!"
SINGLE='Single quoted'
PLAIN=NoQuotes
"#;
        let vars = parse_lenv(content).unwrap();
        assert_eq!(vars.get("MESSAGE"), Some(&"Hello, World!".to_string()));
        assert_eq!(vars.get("SINGLE"), Some(&"Single quoted".to_string()));
        assert_eq!(vars.get("PLAIN"), Some(&"NoQuotes".to_string()));
    }

    #[test]
    fn test_parse_lenv_empty_lines() {
        let content = r"

API_KEY=value

PORT=8080

";
        let vars = parse_lenv(content).unwrap();
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_parse_lenv_error_no_equals() {
        let content = "INVALID_LINE";
        let result = parse_lenv(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_lenv_error_empty_key() {
        let content = "=value";
        let result = parse_lenv(content);
        assert!(result.is_err());
    }
}
