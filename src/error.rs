/*
 * Error Handling Module
 *
 * This module defines a comprehensive error handling system for mirage using the
 * `thiserror` crate. It provides a unified error type that covers all possible
 * failure scenarios in the application, from network issues to configuration
 * problems and validation failures.
 *
 * # Design Philosophy
 *
 * The error system follows these principles:
 * - **Comprehensive Coverage**: All error scenarios have specific variants
 * - **Contextual Information**: Errors include relevant context for debugging
 * - **User-Friendly Messages**: Error messages are clear and actionable
 * - **Automatic Conversions**: Common error types are automatically converted
 * - **Chain Preservation**: Error chains are preserved for full context
 * - **Type Safety**: Strong typing prevents error category confusion
 *
 * # Error Categories
 *
 * - **Network Errors**: HTTP requests, connectivity, timeouts
 * - **I/O Errors**: File operations, filesystem access
 * - **Configuration Errors**: Config file problems, invalid settings
 * - **Parse Errors**: Data parsing, format issues
 * - **Validation Errors**: Input validation, constraint violations
 * - **Cache Errors**: Cache operations, storage issues
 * - **Time Errors**: System time, timestamp operations
 * - **JSON Errors**: Serialization/deserialization problems
 * - **Regex Errors**: Regular expression compilation and usage
 * - **Mirror Test Errors**: Mirror connectivity and performance testing
 *
 * # Error Propagation
 *
 * The module uses Rust's `?` operator extensively with automatic conversions:
 *
 * ```ignore
 * use mirage::error::Result;
 *
 * fn example_operation() -> Result<String> {
 *     let response = reqwest::get("https://api.example.com").await?; // Auto-converts
 *     let data: ApiResponse = response.json().await?; // Auto-converts
 *     let file_content = std::fs::read_to_string("config.txt")?; // Auto-converts
 *
 *     Ok(process_data(data, file_content))
 * }
 * ```
 *
 * # Validation Helpers
 *
 * The module provides extension traits for common validation patterns:
 *
 * ```ignore
 * use mirage::error::ValidationExt;
 *
 * // Validate Option<T>
 * let value = some_option.validate("Value is required")?;
 *
 * // Validate bool conditions
 * (count > 0).validate("Count must be positive")?;
 * ```
 *
 * # Error Construction
 *
 * Errors can be constructed in multiple ways:
 *
 * ```ignore
 * // Direct construction
 * return Err(MirageError::Config("Invalid port number".to_string()));
 *
 * // Using helper methods (preferred)
 * return Err(MirageError::config("Invalid port number"));
 *
 * // Automatic conversion
 * let file = File::open("missing.txt")?; // std::io::Error -> MirageError
 * ```
 *
 * # Integration with Libraries
 *
 * The error system integrates seamlessly with common Rust libraries:
 * - **reqwest**: HTTP client errors are automatically converted
 * - **serde_json**: JSON serialization errors are automatically converted
 * - **std::io**: File I/O errors are automatically converted
 * - **regex**: Regular expression errors are handled with context
 * - **std::time**: System time errors are automatically converted
 *
 * # Thread Safety
 *
 * All error types are thread-safe and can be sent across thread boundaries.
 * This is important for mirage's concurrent mirror testing functionality.
 *
 * # Performance
 *
 * Error construction is designed to be lightweight, with string formatting
 * only occurring when errors are actually created (not in success paths).
 */

use thiserror::Error;

/// Comprehensive error type for all mirage operations.
///
/// This enum covers all possible error scenarios that can occur during mirage
/// execution, providing specific variants for different error categories with
/// appropriate context and error chaining.
///
/// # Error Variants
///
/// Each variant is designed for specific error scenarios:
///
/// - [`Network`]: Automatic conversion from `reqwest::Error`
/// - [`NetworkCustom`]: Custom network error messages
/// - [`Io`]: Automatic conversion from `std::io::Error`
/// - [`Config`]: Configuration and setup problems
/// - [`Regex`]: Regular expression compilation errors with pattern context
/// - [`Parse`]: Data parsing and format errors
/// - [`Validation`]: Input validation and constraint violations
/// - [`Cache`]: Cache operations and storage issues
/// - [`Time`]: Automatic conversion from `std::time::SystemTimeError`
/// - [`Json`]: Automatic conversion from `serde_json::Error`
/// - [`MirrorTest`]: Mirror testing failures with URL and reason context
///
/// # Automatic Conversions
///
/// Several error types are automatically converted using the `#[from]` attribute:
///
/// ```ignore
/// // These conversions happen automatically with the ? operator
/// let response = reqwest::get(url).await?;          // reqwest::Error -> Network
/// let file_content = std::fs::read_to_string(path)?; // io::Error -> Io
/// let data: Value = serde_json::from_str(json)?;     // serde_json::Error -> Json
/// let duration = SystemTime::now().duration_since(UNIX_EPOCH)?; // SystemTimeError -> Time
/// ```
///
/// # Error Messages
///
/// All error messages are designed to be user-friendly and actionable:
///
/// ```ignore
/// MirageError::Config("Invalid port number: must be between 1 and 65535".to_string())
/// // Displays as: "Configuration error: Invalid port number: must be between 1 and 65535"
///
/// MirageError::MirrorTest {
///     url: "https://mirror.example.com".to_string(),
///     reason: "Connection timeout after 5 seconds".to_string()
/// }
/// // Displays as: "Mirror test failed for https://mirror.example.com: Connection timeout after 5 seconds"
/// ```
///
/// # Construction Helpers
///
/// Use the provided helper methods for consistent error construction:
///
/// ```ignore
/// // Preferred - uses helper methods
/// MirageError::config("Port must be positive")
/// MirageError::validation("Country code must be 2 characters")
/// MirageError::mirror_test("https://example.com", "Timeout")
///
/// // Also valid - direct construction
/// MirageError::Config("Port must be positive".to_string())
/// ```
#[derive(Error, Debug)]
pub enum MirageError {
    /// Network errors from HTTP requests and connectivity issues.
    ///
    /// This variant automatically converts from `reqwest::Error` and covers
    /// scenarios like connection timeouts, DNS resolution failures, SSL errors,
    /// and HTTP protocol issues when fetching mirror data.
    ///
    /// # Common Causes
    /// - Internet connectivity problems
    /// - Mirror server downtime or overload
    /// - DNS resolution failures
    /// - SSL/TLS certificate issues
    /// - HTTP protocol errors (4xx/5xx responses)
    ///
    /// # Example
    /// ```ignore
    /// let response = reqwest::get("https://mirror.example.com").await?;
    /// // If the request fails, it automatically becomes MirageError::Network
    /// ```
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Custom network error messages for application-specific network issues.
    ///
    /// Used when we need to provide custom context for network-related problems
    /// that aren't directly from reqwest, such as timeout configurations or
    /// application-level network validation failures.
    ///
    /// # Use Cases
    /// - Custom timeout messages
    /// - Network configuration validation
    /// - Application-specific connectivity checks
    /// - Custom retry logic failures
    ///
    /// # Example
    /// ```ignore
    /// return Err(MirageError::network("Mirror response took longer than configured timeout"));
    /// ```
    #[error("Network error: {0}")]
    NetworkCustom(String),

    /// I/O errors from filesystem operations.
    ///
    /// This variant automatically converts from `std::io::Error` and covers
    /// file operations like reading configuration files, writing mirror lists,
    /// cache operations, and other filesystem interactions.
    ///
    /// # Common Causes
    /// - File or directory not found
    /// - Insufficient permissions
    /// - Disk space exhausted
    /// - File system corruption
    /// - Network filesystem issues
    ///
    /// # Example
    /// ```ignore
    /// let config_content = std::fs::read_to_string(config_path)?;
    /// // If the file read fails, it automatically becomes MirageError::Io
    /// ```
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration errors including invalid settings and setup problems.
    ///
    /// Used for all configuration-related issues, from invalid config file
    /// syntax to logical configuration errors like invalid value ranges
    /// or conflicting options.
    ///
    /// # Common Causes
    /// - Invalid configuration file syntax
    /// - Out-of-range parameter values
    /// - Conflicting configuration options
    /// - Missing required configuration
    /// - Environment setup issues
    ///
    /// # Examples
    /// ```ignore
    /// MirageError::config("Port number must be between 1 and 65535")
    /// MirageError::config("Cannot specify both --fastest and --number options")
    /// MirageError::config("Invalid country code: must be 2 characters")
    /// ```
    #[error("Configuration error: {0}")]
    Config(String),

    /// Regular expression compilation errors with pattern context.
    ///
    /// This variant preserves both the invalid regex pattern and the underlying
    /// regex compilation error, providing full context for debugging regex
    /// issues in include/exclude filters.
    ///
    /// # Common Causes
    /// - Invalid regex syntax in --include/--exclude patterns
    /// - Unsupported regex features
    /// - Regex complexity limits exceeded
    /// - Character encoding issues in patterns
    ///
    /// # Example
    /// ```ignore
    /// let regex = regex::Regex::new(user_pattern)
    ///     .map_err(|e| MirageError::regex(user_pattern, e))?;
    /// // Provides both the invalid pattern and the specific regex error
    /// ```
    #[error("Invalid regex pattern '{pattern}': {source}")]
    Regex {
        /// The regex pattern that failed to compile
        pattern: String,
        /// The underlying regex compilation error
        #[source]
        source: regex::Error,
    },

    /// Data parsing errors for API responses and configuration data.
    ///
    /// Used when data doesn't match expected formats, such as malformed
    /// JSON responses from the mirror status API or invalid data structures.
    ///
    /// # Common Causes
    /// - API response format changes
    /// - Corrupted or truncated data
    /// - Unexpected data types or structures
    /// - Encoding or character set issues
    /// - Version compatibility problems
    ///
    /// # Examples
    /// ```ignore
    /// MirageError::parse("API response missing required field 'urls'")
    /// MirageError::parse("Invalid timestamp format in mirror data")
    /// MirageError::parse("Unexpected response format from mirror status API")
    /// ```
    #[error("Parse error: {0}")]
    Parse(String),

    /// Input validation errors for user-provided parameters.
    ///
    /// Used when user inputs fail validation checks, such as invalid country
    /// codes, out-of-range numeric values, or conflicting option combinations.
    ///
    /// # Common Causes
    /// - Invalid country codes or names
    /// - Numeric values outside acceptable ranges
    /// - Conflicting command-line options
    /// - Invalid URL formats
    /// - Unsupported protocol specifications
    ///
    /// # Examples
    /// ```ignore
    /// MirageError::validation("Country code 'XX' not found in mirror database")
    /// MirageError::validation("Number of mirrors must be between 1 and 100")
    /// MirageError::validation("Cannot combine --fastest with --sort score")
    /// ```
    #[error("Validation error: {0}")]
    Validation(String),

    /// Cache operation errors including storage and retrieval issues.
    ///
    /// Used for cache-related problems such as cache directory creation
    /// failures, corrupted cache files, or cache serialization issues.
    ///
    /// # Common Causes
    /// - Cache directory creation failures
    /// - Cache file corruption
    /// - Disk space issues
    /// - Permission problems
    /// - Serialization/deserialization failures
    ///
    /// # Examples
    /// ```ignore
    /// MirageError::cache("Failed to create cache directory: permission denied")
    /// MirageError::cache("Cache file is corrupted or from incompatible version")
    /// MirageError::cache("Insufficient disk space to write cache file")
    /// ```
    #[error("Cache error: {0}")]
    Cache(String),

    /// System time errors from time-related operations.
    ///
    /// This variant automatically converts from `std::time::SystemTimeError`
    /// and typically occurs when working with timestamps, cache age calculations,
    /// or mirror synchronization time comparisons.
    ///
    /// # Common Causes
    /// - System clock running backwards
    /// - Time zone configuration issues
    /// - System time not synchronized
    /// - Timestamp overflow in calculations
    ///
    /// # Example
    /// ```ignore
    /// let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    /// // If system time is before UNIX_EPOCH, this becomes MirageError::Time
    /// ```
    #[error("Time error: {0}")]
    Time(#[from] std::time::SystemTimeError),

    /// JSON serialization/deserialization errors.
    ///
    /// This variant automatically converts from `serde_json::Error` and occurs
    /// during JSON parsing of API responses or cache file serialization.
    ///
    /// # Common Causes
    /// - Invalid JSON syntax in API responses
    /// - JSON structure doesn't match expected schema
    /// - Data type mismatches during deserialization
    /// - Cache file corruption
    /// - Memory issues during large JSON processing
    ///
    /// # Example
    /// ```ignore
    /// let mirrors: Vec<Mirror> = serde_json::from_str(response_text)?;
    /// // If JSON parsing fails, it automatically becomes MirageError::Json
    /// ```
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Mirror connectivity test failures with detailed context.
    ///
    /// Used when testing mirror connectivity and performance, providing both
    /// the mirror URL that failed and the specific reason for the failure.
    /// This helps users understand why particular mirrors were rejected.
    ///
    /// # Common Causes
    /// - Mirror server connectivity issues
    /// - Slow response times exceeding thresholds
    /// - HTTP errors from mirror servers
    /// - DNS resolution failures for mirror domains
    /// - Network routing problems
    ///
    /// # Example
    /// ```ignore
    /// MirageError::mirror_test(
    ///     "https://mirror.example.com/archlinux/",
    ///     "Connection timeout after 5 seconds"
    /// )
    /// // Displays: "Mirror test failed for https://mirror.example.com/archlinux/: Connection timeout after 5 seconds"
    /// ```
    #[error("Mirror test failed for {url}: {reason}")]
    MirrorTest {
        /// The mirror URL that failed testing
        url: String,
        /// Specific reason for the test failure
        reason: String,
    },
}

impl MirageError {
    /// Creates a configuration error with the provided message.
    ///
    /// This is a convenience constructor for configuration-related errors.
    /// It accepts any type that can be converted to a String, making it
    /// easy to use with string literals, formatted strings, or owned strings.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message describing the configuration problem
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // With string literal
    /// let error = MirageError::config("Invalid port number");
    ///
    /// // With formatted string
    /// let error = MirageError::config(format!("Port {} is out of range", port));
    ///
    /// // With owned string
    /// let message = get_config_error_message();
    /// let error = MirageError::config(message);
    /// ```
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Creates a custom network error with the provided message.
    ///
    /// This is used for application-specific network errors that don't come
    /// directly from reqwest. It's useful for custom timeout logic, connection
    /// validation, or other network-related application logic.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message describing the network problem
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Custom timeout error
    /// let error = MirageError::network("Request exceeded custom timeout of 10 seconds");
    ///
    /// // Connection validation error
    /// let error = MirageError::network("Mirror server returned invalid response format");
    ///
    /// // Retry exhaustion error
    /// let error = MirageError::network("All retry attempts failed for mirror server");
    /// ```
    pub fn network(msg: impl Into<String>) -> Self {
        Self::NetworkCustom(msg.into())
    }

    /// Creates a validation error with the provided message.
    ///
    /// This is used for input validation failures, parameter range checks,
    /// and other user input validation scenarios.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message describing what validation failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Range validation
    /// let error = MirageError::validation("Number of mirrors must be between 1 and 100");
    ///
    /// // Format validation
    /// let error = MirageError::validation("Country code must be exactly 2 characters");
    ///
    /// // Logic validation
    /// let error = MirageError::validation("Cannot combine --fastest with --sort options");
    /// ```
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Creates a parse error with the provided message.
    ///
    /// This is used when data parsing fails, such as API response parsing,
    /// configuration parsing, or any other data format issues.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message describing what failed to parse
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // API response parsing
    /// let error = MirageError::parse("API response missing required 'mirrors' field");
    ///
    /// // Data format error
    /// let error = MirageError::parse("Invalid timestamp format in mirror data");
    ///
    /// // Structure error
    /// let error = MirageError::parse("Expected array but found object in JSON response");
    /// ```
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Creates a cache error with the provided message.
    ///
    /// This is used for cache-related operations including file I/O,
    /// serialization issues, and cache management problems.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message describing the cache operation failure
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Cache directory issues
    /// let error = MirageError::cache("Failed to create cache directory: permission denied");
    ///
    /// // Cache file corruption
    /// let error = MirageError::cache("Cache file is corrupted or from incompatible version");
    ///
    /// // Storage issues
    /// let error = MirageError::cache("Insufficient disk space to write cache file");
    /// ```
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    /// Creates a mirror test error with URL and failure reason.
    ///
    /// This specialized constructor is used when mirror connectivity or
    /// performance tests fail, providing both the specific mirror URL
    /// and the reason for the failure.
    ///
    /// # Arguments
    ///
    /// - `url`: The mirror URL that failed testing
    /// - `reason`: Specific reason why the test failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Connection timeout
    /// let error = MirageError::mirror_test(
    ///     "https://mirror.example.com/archlinux/",
    ///     "Connection timeout after 5 seconds"
    /// );
    ///
    /// // HTTP error
    /// let error = MirageError::mirror_test(
    ///     "https://slow.mirror.com/archlinux/",
    ///     "HTTP 503: Service Unavailable"
    /// );
    ///
    /// // Performance issue
    /// let error = MirageError::mirror_test(
    ///     "https://distant.mirror.org/archlinux/",
    ///     "Response time 15.2s exceeds threshold of 10s"
    /// );
    /// ```
    pub fn mirror_test(url: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::MirrorTest {
            url: url.into(),
            reason: reason.into(),
        }
    }

    /// Creates a regex error with pattern context and underlying error.
    ///
    /// This constructor preserves both the regex pattern that failed to compile
    /// and the underlying `regex::Error`, providing complete context for debugging
    /// regular expression issues.
    ///
    /// # Arguments
    ///
    /// - `pattern`: The regex pattern that failed to compile
    /// - `source`: The underlying `regex::Error` with specific details
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use regex::Regex;
    ///
    /// let user_pattern = "[invalid";
    /// let regex_error = Regex::new(user_pattern)
    ///     .map_err(|e| MirageError::regex(user_pattern, e))?;
    ///
    /// // This preserves both the pattern and the specific regex compilation error
    /// // Error display: "Invalid regex pattern '[invalid': missing closing bracket"
    /// ```
    pub fn regex(pattern: impl Into<String>, source: regex::Error) -> Self {
        Self::Regex {
            pattern: pattern.into(),
            source,
        }
    }
}

/// Convenient Result type alias for mirage operations.
///
/// This type alias simplifies function signatures throughout the mirage codebase
/// by providing a standard Result type that uses [`MirageError`] as the error type.
/// This promotes consistency and reduces boilerplate in error handling.
///
/// # Usage
///
/// Instead of writing `std::result::Result<T, MirageError>` throughout the codebase,
/// functions can simply return `Result<T>`:
///
/// ```ignore
/// use mirage::error::Result;
///
/// // Concise function signature
/// fn fetch_mirrors() -> Result<Vec<Mirror>> {
///     // Implementation that may return MirageError
/// }
///
/// // Equivalent to writing:
/// fn fetch_mirrors_verbose() -> std::result::Result<Vec<Mirror>, MirageError> {
///     // Same implementation
/// }
/// ```
///
/// # Error Propagation
///
/// This Result type works seamlessly with the `?` operator for error propagation:
///
/// ```ignore
/// fn example_operation() -> Result<String> {
///     let mirrors = fetch_mirrors()?;           // Propagates MirageError
///     let filtered = filter_mirrors(mirrors)?;  // Propagates MirageError  
///     let result = process_mirrors(filtered)?;  // Propagates MirageError
///     Ok(result)
/// }
/// ```
///
/// # Integration
///
/// This type is used consistently across all mirage modules for operations that
/// can fail, providing a uniform error handling experience for both internal
/// code and library users.
pub type Result<T> = std::result::Result<T, MirageError>;

/// Extension trait for validation operations on Option and bool types.
///
/// This trait provides convenient methods for converting validation failures
/// into [`MirageError::Validation`] variants. It's designed to make common
/// validation patterns more ergonomic and consistent throughout the codebase.
///
/// # Supported Types
///
/// - **`Option<T>`**: Validates that the option contains a value
/// - **`bool`**: Validates that a boolean condition is true
///
/// # Design Rationale
///
/// Many validation scenarios involve checking for required values (Option)
/// or boolean conditions, and converting failures to descriptive errors.
/// This trait provides a uniform interface for these common patterns.
///
/// # Examples
///
/// ```ignore
/// use mirage::error::{ValidationExt, Result};
///
/// fn validate_user_input(
///     name: Option<String>,
///     age: u32,
/// ) -> Result<(String, u32)> {
///     // Validate required field
///     let name = name.validate("Name is required")?;
///     
///     // Validate condition  
///     (age >= 18).validate("Age must be 18 or older")?;
///     
///     Ok((name, age))
/// }
/// ```
///
/// # Error Messages
///
/// The trait methods accept string slices for error messages, which are
/// converted to [`MirageError::Validation`] variants with descriptive text
/// that helps users understand what validation failed.
pub trait ValidationExt<T> {
    /// Validates that an Option contains a value, converting None to a validation error.
    ///
    /// This method provides a concise way to handle required fields and optional
    /// values that become required in certain contexts. It's particularly useful
    /// for API validation and configuration validation.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message to use if validation fails (Option is None)
    ///
    /// # Returns
    ///
    /// - `Ok(T)`: If the Option contains a value
    /// - `Err(MirageError::Validation)`: If the Option is None
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mirage::error::ValidationExt;
    ///
    /// // Required configuration value
    /// let config_value = config.get("required_setting")
    ///     .validate("required_setting must be specified in configuration")?;
    ///
    /// // Required command line argument
    /// let country = matches.get_one::<String>("country")
    ///     .validate("--country argument is required when using --fastest")?;
    ///
    /// // API response field
    /// let mirror_url = mirror.url
    ///     .validate("Mirror data missing required 'url' field")?;
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Validating required configuration fields
    /// - Ensuring mandatory command-line arguments are provided
    /// - Checking for required fields in API responses
    /// - Converting optional values to required in specific contexts
    ///
    /// # Errors
    ///
    /// Returns [`MirageError::Validation`] if validation fails (Option is None or condition is false).
    fn validate(self, msg: &str) -> Result<T>;
}

impl<T> ValidationExt<T> for Option<T> {
    fn validate(self, msg: &str) -> Result<T> {
        self.ok_or_else(|| MirageError::validation(msg))
    }
}

impl ValidationExt<()> for bool {
    /// Validates that a boolean condition is true, converting false to a validation error.
    ///
    /// This method provides a concise way to validate boolean conditions and convert
    /// failures into descriptive validation errors. It's particularly useful for
    /// range checks, logical constraints, and precondition validation.
    ///
    /// # Arguments
    ///
    /// - `msg`: Error message to use if validation fails (condition is false)
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the condition is true
    /// - `Err(MirageError::Validation)`: If the condition is false
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mirage::error::ValidationExt;
    ///
    /// // Range validation
    /// (port > 0 && port <= 65535).validate("Port must be between 1 and 65535")?;
    ///
    /// // String format validation
    /// (country_code.len() == 2).validate("Country code must be exactly 2 characters")?;
    ///
    /// // Logical constraint validation
    /// (!fastest_enabled || sort_method.is_none())
    ///     .validate("Cannot specify both --fastest and --sort options")?;
    ///
    /// // File existence validation
    /// config_path.exists().validate("Configuration file does not exist")?;
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Range and boundary checking for numeric values
    /// - String format and length validation
    /// - Logical constraint enforcement
    /// - Precondition and postcondition validation
    /// - File system state validation
    fn validate(self, msg: &str) -> Result<()> {
        if self {
            Ok(())
        } else {
            Err(MirageError::validation(msg))
        }
    }
}
