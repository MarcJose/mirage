// Tests to achieve 100% coverage for error.rs
use mirage::error::{MirageError, Result, ValidationExt};
use std::error::Error;

#[test]
fn test_mirage_error_display_messages() {
    // Test all error variants display correctly
    let network_err = MirageError::NetworkCustom("Connection timeout".to_string());
    assert_eq!(network_err.to_string(), "Network error: Connection timeout");

    let config_err = MirageError::Config("Invalid setting".to_string());
    assert_eq!(
        config_err.to_string(),
        "Configuration error: Invalid setting"
    );

    let parse_err = MirageError::Parse("Invalid number format".to_string());
    assert_eq!(parse_err.to_string(), "Parse error: Invalid number format");

    let validation_err = MirageError::Validation("Required field missing".to_string());
    assert_eq!(
        validation_err.to_string(),
        "Validation error: Required field missing"
    );

    let cache_err = MirageError::Cache("Cache file corrupted".to_string());
    assert_eq!(cache_err.to_string(), "Cache error: Cache file corrupted");

    let mirror_test_err = MirageError::MirrorTest {
        url: "https://example.com".to_string(),
        reason: "Connection failed".to_string(),
    };
    assert_eq!(
        mirror_test_err.to_string(),
        "Mirror test failed for https://example.com: Connection failed"
    );
}

#[test]
fn test_mirage_error_helper_functions() {
    // Test config() helper function
    let config_err = MirageError::config("Missing required field");
    match config_err {
        MirageError::Config(msg) => assert_eq!(msg, "Missing required field"),
        _ => panic!("Expected Config error"),
    }

    // Test network() helper function
    let network_err = MirageError::network("Request timeout");
    match network_err {
        MirageError::NetworkCustom(msg) => assert_eq!(msg, "Request timeout"),
        _ => panic!("Expected NetworkCustom error"),
    }

    // Test parse() helper function
    let parse_err = MirageError::parse("Invalid JSON format");
    match parse_err {
        MirageError::Parse(msg) => assert_eq!(msg, "Invalid JSON format"),
        _ => panic!("Expected Parse error"),
    }

    // Test validation() helper function (already covered but let's be thorough)
    let validation_err = MirageError::validation("Field is required");
    match validation_err {
        MirageError::Validation(msg) => assert_eq!(msg, "Field is required"),
        _ => panic!("Expected Validation error"),
    }

    // Test cache() helper function (already covered but let's be thorough)
    let cache_err = MirageError::cache("Unable to write cache");
    match cache_err {
        MirageError::Cache(msg) => assert_eq!(msg, "Unable to write cache"),
        _ => panic!("Expected Cache error"),
    }
}

#[test]
fn test_mirage_error_mirror_test_helper() {
    // Test mirror_test() helper function
    let mirror_err = MirageError::mirror_test("https://mirror.example.com", "HTTP 404");
    match mirror_err {
        MirageError::MirrorTest { url, reason } => {
            assert_eq!(url, "https://mirror.example.com");
            assert_eq!(reason, "HTTP 404");
        }
        _ => panic!("Expected MirrorTest error"),
    }

    // Test with different types that implement Into<String>
    let mirror_err2 = MirageError::mirror_test(
        String::from("https://other.mirror.com"),
        String::from("Timeout"),
    );
    match mirror_err2 {
        MirageError::MirrorTest { url, reason } => {
            assert_eq!(url, "https://other.mirror.com");
            assert_eq!(reason, "Timeout");
        }
        _ => panic!("Expected MirrorTest error"),
    }
}

#[test]
fn test_mirage_error_regex_helper() {
    // Test regex() helper function
    // Create a real regex error by using invalid regex pattern
    let invalid_pattern = "[";
    let regex_result = regex::Regex::new(invalid_pattern);

    match regex_result {
        Err(regex_err) => {
            let mirage_err = MirageError::regex("invalid_pattern", regex_err);
            let error_string = mirage_err.to_string();
            match mirage_err {
                MirageError::Regex {
                    ref pattern,
                    source: _,
                } => {
                    assert_eq!(pattern, "invalid_pattern");
                    assert!(error_string.contains("Invalid regex pattern 'invalid_pattern'"));
                }
                _ => panic!("Expected Regex error"),
            }
        }
        Ok(_) => panic!("Expected regex compilation to fail"),
    }
}

#[test]
fn test_mirage_error_from_conversions() {
    // Test automatic conversions from other error types

    // Test std::io::Error conversion
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let mirage_err: MirageError = io_err.into();
    match mirage_err {
        MirageError::Io(_) => {
            assert!(mirage_err.to_string().contains("IO error"));
        }
        _ => panic!("Expected Io error"),
    }

    // Test serde_json::Error conversion
    let json_str = "{invalid json";
    let json_result: serde_json::Result<serde_json::Value> = serde_json::from_str(json_str);
    match json_result {
        Err(json_err) => {
            let mirage_err: MirageError = json_err.into();
            match mirage_err {
                MirageError::Json(_) => {
                    assert!(mirage_err.to_string().contains("JSON serialization error"));
                }
                _ => panic!("Expected Json error"),
            }
        }
        Ok(_) => panic!("Expected JSON parsing to fail"),
    }
}

#[test]
fn test_validation_ext_for_option() {
    // Test ValidationExt trait implementation for Option<T>

    // Test Some variant (success case)
    let some_value: Option<i32> = Some(42);
    let result = some_value.validate("Should not fail");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Test None variant (error case)
    let none_value: Option<i32> = None;
    let result = none_value.validate("Value is required");
    assert!(result.is_err());
    match result.unwrap_err() {
        MirageError::Validation(msg) => assert_eq!(msg, "Value is required"),
        _ => panic!("Expected Validation error"),
    }

    // Test with different types to ensure the impl<T> generic works
    let string_option: Option<String> = None;
    let result = string_option.validate("String value required");
    assert!(result.is_err());

    let some_string: Option<String> = Some("test".to_string());
    let result = some_string.validate("Should not fail");
    assert!(result.is_ok());

    // Test with more types to thoroughly exercise the generic implementation
    let float_option: Option<f64> = Some(42.5);
    let result = float_option.validate("Should work");
    assert!(result.is_ok());
    assert!((result.unwrap() - 42.5).abs() < f64::EPSILON);

    let empty_float: Option<f64> = None;
    let result = empty_float.validate("Float required");
    assert!(result.is_err());
}

#[test]
fn test_validation_ext_for_bool() {
    // Test ValidationExt trait implementation for bool

    // Test true case (success) - exercises the impl ValidationExt<()> for bool
    let valid_condition = true;
    let result = valid_condition.validate("Should not fail");
    assert!(result.is_ok());

    // Test false case (error) - exercises the full implementation
    let invalid_condition = false;
    let result = invalid_condition.validate("Condition not met");
    assert!(result.is_err());
    match result.unwrap_err() {
        MirageError::Validation(msg) => assert_eq!(msg, "Condition not met"),
        _ => panic!("Expected Validation error"),
    }

    // Additional tests to ensure the implementation is fully exercised
    assert!(true.validate("test").is_ok());
    assert!(false.validate("test").is_err());

    // Test with various message types
    let result1 = false.validate("Static str");
    let result2 = false.validate(&String::from("String reference"));
    assert!(result1.is_err());
    assert!(result2.is_err());
}

#[test]
fn test_error_chain_and_source() {
    // Test error chain functionality for complex error types

    // Create a regex error chain
    #[allow(clippy::invalid_regex)]
    let regex_result = regex::Regex::new("[");
    match regex_result {
        Err(regex_err) => {
            let mirage_err = MirageError::regex("bad_pattern", regex_err);

            // Test that the source error is preserved
            assert!(mirage_err.source().is_some());

            // Test the full error display
            let error_string = mirage_err.to_string();
            assert!(error_string.contains("Invalid regex pattern 'bad_pattern'"));
        }
        Ok(_) => panic!("Expected regex compilation to fail"),
    }
}

#[test]
fn test_result_type_alias() {
    // Test that our Result type alias works correctly
    fn test_function() -> i32 {
        42
    }

    fn test_error_function() -> Result<i32> {
        Err(MirageError::validation("Test error"))
    }

    let success = test_function();
    assert_eq!(success, 42);

    let failure = test_error_function();
    assert!(failure.is_err());
}

#[test]
fn test_debug_formatting() {
    // Test that Debug trait is properly derived for all error types
    let errors = vec![
        MirageError::config("test"),
        MirageError::network("test"),
        MirageError::validation("test"),
        MirageError::parse("test"),
        MirageError::cache("test"),
        MirageError::mirror_test("url", "reason"),
    ];

    for error in errors {
        let debug_str = format!("{error:?}");
        // Should contain the variant name
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_comprehensive_error_scenarios() {
    // Test various real-world error scenarios that would use these error types

    // Config validation scenario
    let config_result: Result<()> = false.validate("Configuration is invalid");
    assert!(config_result.is_err());

    // Option validation scenario
    let optional_value: Option<String> = None;
    let validation_result = optional_value.validate("Required field missing");
    assert!(validation_result.is_err());

    // Combined error handling
    let errors = vec![
        MirageError::config("Bad config"),
        MirageError::network("Network failure"),
        MirageError::parse("Parse failure"),
        MirageError::validation("Validation failure"),
        MirageError::cache("Cache failure"),
        MirageError::mirror_test("http://example.com", "Test failure"),
    ];

    for error in errors {
        // Each error should have a meaningful string representation
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
        assert!(error_str.len() > 10); // Should be descriptive
    }
}

#[test]
fn test_validation_ext_trait_implementations_directly() {
    // Direct test of trait implementations

    // Test Option<T> implementation explicitly
    use mirage::error::ValidationExt;

    // Test the trait impl for Option<T>
    let opt: Option<i32> = Some(42);
    let result: Result<i32> = ValidationExt::validate(opt, "test message");
    assert!(result.is_ok());

    let opt_none: Option<i32> = None;
    let result: Result<i32> = ValidationExt::validate(opt_none, "test message");
    assert!(result.is_err());

    // Test the trait impl for bool
    let bool_val = true;
    let result: Result<()> = ValidationExt::validate(bool_val, "test message");
    assert!(result.is_ok());

    let bool_val = false;
    let result: Result<()> = ValidationExt::validate(bool_val, "test message");
    assert!(result.is_err());

    // Test method call syntax too
    assert!(Some(1).validate("test").is_ok());
    assert!(None::<i32>.validate("test").is_err());
    assert!(true.validate("test").is_ok());
    assert!(false.validate("test").is_err());
}
