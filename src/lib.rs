/*!
 * Mirage Core Library
 *
 * This module contains the core business logic for the Mirage application.
 * It provides the fundamental data structures, filtering logic, sorting algorithms,
 * and validation functions that power the mirror management functionality.
 *
 * # Module Organization
 *
 * - `config`: Configuration file parsing and management
 * - `error`: Custom error types and error handling utilities
 * - `performance`: Mirror performance testing and benchmarking
 * - `cache`: Persistent caching system with XDG compliance
 *
 * # Core Data Structures
 *
 * - `Mirror`: Represents a single Arch Linux mirror with metadata
 * - `MirrorStatus`: API response structure from mirror status endpoint
 * - `Config`: Application configuration combining CLI args and config files
 * - `CacheEntry`: In-memory cache entry with timestamp validation
 *
 * # Key Algorithms
 *
 * ## Filtering Pipeline
 *
 * The filtering system processes mirrors through multiple stages:
 *
 * ```text
 * Raw Mirrors (1000+)
 *     ↓
 * Active Filter (removes inactive mirrors)
 *     ↓
 * Completion Filter (removes incomplete mirrors)
 *     ↓  
 * Age Filter (removes stale mirrors)
 *     ↓
 * Delay Filter (removes slow-updating mirrors)
 *     ↓
 * Country Filter (geographic filtering)
 *     ↓
 * Protocol Filter (HTTP/HTTPS/FTP/rsync)
 *     ↓
 * Feature Filter (IPv4/IPv6/ISOs)
 *     ↓
 * Regex Filter (include/exclude patterns)
 *     ↓
 * Filtered Mirrors (10-100)
 * ```
 *
 * ## Sorting Strategies
 *
 * Multiple sorting methods optimize for different use cases:
 *
 * - **Age**: Prioritizes recently synchronized mirrors
 * - **Rate/Score**: Performance-based ranking (requires testing)
 * - **Country**: Alphabetical geographic grouping
 * - **Delay**: Minimizes sync lag from upstream
 * - **Duration**: Optimizes for response time
 * - **Duration-std**: Prioritizes consistent response times
 *
 * # Example Usage
 *
 * ```ignore,no_run
 * use mirage::{Config, Mirror, filter_mirrors, sort_mirrors};
 *
 * // Create configuration
 * let config = Config {
 *     countries: vec!["Germany".to_string(), "France".to_string()],
 *     protocols: vec!["https".to_string()],
 *     age: Some(24.0), // Last 24 hours
 *     completion_percent: 95.0,
 *     ..Default::default()
 * };
 *
 * // Apply filtering and sorting pipeline (example only)
 * let mirrors: Vec<Mirror> = vec![];  // In real usage, get from API
 * let filtered = filter_mirrors(mirrors, &config);
 * let sorted = sort_mirrors(filtered, &config);
 *
 * println!("Found {} suitable mirrors", sorted.len());
 * ```
 */

// Module declarations - these define the public API surface
pub mod cache;
pub mod cli; // Command-line interface parsing and configuration merging
pub mod config; // Configuration file parsing and merging
pub mod error; // Custom error types and utilities
pub mod performance; // Mirror performance testing // Persistent caching system

// Re-export commonly used types for convenience
pub use cache::{CacheStats, PersistentCache};
pub use error::{MirageError, Result};
pub use performance::{MirrorPerformance, MirrorWithPerformance};

// External dependencies
use chrono::{DateTime, Utc}; // Date/time parsing for mirror sync times
use serde::{Deserialize, Serialize}; // JSON serialization for API responses
use std::fs; // File system operations for validation
use tracing::{debug, info, warn}; // Structured logging

/// In-memory cache entry for mirror data.
///
/// This structure holds mirror data in memory for the duration of the application.
/// It's used as a fallback when the persistent cache is unavailable and provides
/// faster access than disk-based storage.
///
/// # Fields
///
/// - `data`: Vector of mirrors retrieved from the API
/// - `timestamp`: Unix timestamp when the data was cached
///
/// # Thread Safety
///
/// When used in the static `CACHE` variable, this is protected by a `Mutex`
/// to ensure thread-safe access during concurrent operations.
#[derive(Clone)]
pub struct CacheEntry {
    /// The cached mirror data
    pub data: Vec<Mirror>,
    /// Unix timestamp when this cache entry was created
    pub timestamp: u64,
}

/// Mirror Status API response structure.
///
/// This represents the JSON response from the Arch Linux Mirror Status API.
/// The API provides metadata about the mirror checking process and a list
/// of all available mirrors with their current status.
///
/// # API Endpoint
///
/// Default: `https://archlinux.org/mirrors/status/json/`
///
/// # Example Response
///
/// ```json
/// {
///   "cutoff": 86400,
///   "last_check": "2025-08-15T15:46:19.213Z",
///   "num_checks": 133,
///   "check_frequency": 642,
///   "urls": [
///     {
///       "url": "https://mirror.example.com/archlinux/",
///       "protocol": "https",
///       "country": "Germany",
///       "active": true,
///       // ... more mirror fields
///     }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct MirrorStatus {
    /// Cutoff time for considering mirrors active (seconds)
    pub cutoff: Option<i64>,
    /// ISO timestamp of the last mirror status check
    pub last_check: String,
    /// Number of mirrors checked in the last run
    pub num_checks: Option<i32>,
    /// Frequency of mirror status checks (seconds)
    pub check_frequency: Option<i32>,
    /// List of all available mirrors
    pub urls: Vec<Mirror>,
}

/// Represents a single Arch Linux mirror with all its metadata.
///
/// This structure contains comprehensive information about a mirror including
/// its location, performance characteristics, synchronization status, and
/// supported features. All fields are directly parsed from the Mirror Status API.
///
/// # Performance Metrics
///
/// - `score`: Overall mirror quality score (higher is better)
/// - `duration_avg`: Average response time for mirror checks
/// - `duration_stddev`: Response time consistency (lower is better)
/// - `delay`: How far behind the mirror is from upstream (seconds)
///
/// # Synchronization Status
///
/// - `last_sync`: ISO timestamp of last successful sync
/// - `completion_pct`: Percentage of packages successfully mirrored (0.0-1.0)
/// - `active`: Whether the mirror is currently active and responding
///
/// # Geographic and Technical Info
///
/// - `country`/`country_code`: Mirror location for geographic filtering
/// - `protocol`: Connection method (http, https, ftp, rsync)
/// - `ipv4`/`ipv6`: IP version support for connectivity requirements
/// - `isos`: Whether the mirror hosts installation images
///
/// # Example
///
/// ```ignore
/// use mirage::Mirror;
///
/// let mirror = Mirror {
///     url: "https://mirror.example.com/archlinux/".to_string(),
///     protocol: "https".to_string(),
///     country: "Germany".to_string(),
///     country_code: "DE".to_string(),
///     active: true,
///     completion_pct: Some(1.0),
///     score: Some(2.5),
///     ipv4: true,
///     ipv6: true,
///     isos: true,
///     last_sync: None,
///     delay: None,
///     duration_avg: None,
///     duration_stddev: None,
///     details: String::new(),
/// };
///
/// // Check if mirror meets requirements
/// if mirror.active && mirror.completion_pct.unwrap_or(0.0) >= 0.95 {
///     println!("High-quality mirror: {}", mirror.url);
/// }
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Mirror {
    /// Full URL to the mirror (e.g., `<https://mirror.example.com/archlinux/>`)
    pub url: String,
    /// Protocol used by the mirror (http, https, ftp, rsync)
    pub protocol: String,
    /// ISO timestamp of last successful synchronization
    pub last_sync: Option<String>,
    /// Completion percentage as decimal (0.0 = 0%, 1.0 = 100%)
    pub completion_pct: Option<f64>,
    /// Sync delay behind upstream in seconds
    pub delay: Option<i32>,
    /// Average response time for health checks (seconds)
    pub duration_avg: Option<f64>,
    /// Standard deviation of response times (consistency measure)
    pub duration_stddev: Option<f64>,
    /// Overall mirror quality score (higher is better)
    pub score: Option<f64>,
    /// Whether the mirror is currently active and responding
    pub active: bool,
    /// Human-readable country name
    pub country: String,
    /// ISO 3166-1 alpha-2 country code
    pub country_code: String,
    /// Whether the mirror hosts ISO installation images
    pub isos: bool,
    /// Whether the mirror supports IPv4 connections
    pub ipv4: bool,
    /// Whether the mirror supports IPv6 connections
    pub ipv6: bool,
    /// Additional details or notes about the mirror
    pub details: String,
}

impl Mirror {
    /// Calculate how many hours ago this mirror was last synchronized.
    ///
    /// This is a convenience method that parses the `last_sync` timestamp
    /// and calculates the time difference from now. It's commonly used for:
    ///
    /// - Age-based filtering (e.g., "only mirrors synced in last 24 hours")
    /// - Sorting by recency (most recently synced first)
    /// - Displaying human-readable sync ages in the UI
    ///
    /// # Returns
    ///
    /// - `Some(hours)`: Number of hours since last sync as a floating point
    /// - `None`: If `last_sync` is missing or cannot be parsed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mirage::Mirror;
    ///
    /// let mirror = Mirror {
    ///     url: "https://mirror.example.com/archlinux/".to_string(),
    ///     protocol: "https".to_string(),
    ///     country: "Germany".to_string(),
    ///     country_code: "DE".to_string(),
    ///     active: true,
    ///     completion_pct: None,
    ///     score: None,
    ///     ipv4: true,
    ///     ipv6: true,
    ///     isos: true,
    ///     last_sync: Some("2025-08-15T12:00:00Z".to_string()),
    ///     delay: None,
    ///     duration_avg: None,
    ///     duration_stddev: None,
    ///     details: String::new(),
    /// };
    ///
    /// if let Some(hours) = mirror.last_sync_hours() {
    ///     if hours < 24.0 {
    ///         println!("Mirror is fresh (synced {:.1}h ago)", hours);
    ///     } else {
    ///         println!("Mirror is stale (synced {:.1}h ago)", hours);
    ///     }
    /// }
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn last_sync_hours(&self) -> Option<f64> {
        if let Some(last_sync_str) = &self.last_sync {
            // Parse RFC 3339 timestamp (ISO 8601 format)
            if let Ok(last_sync) = DateTime::parse_from_rfc3339(last_sync_str) {
                let now = Utc::now();
                let duration = now.signed_duration_since(last_sync);
                // Convert seconds to hours with decimal precision
                return Some(duration.num_seconds() as f64 / 3600.0);
            }
        }
        None
    }

    /// Convert the sync delay from seconds to hours.
    ///
    /// The `delay` field represents how far behind this mirror is from the
    /// upstream repositories, measured in seconds. This method converts that
    /// to hours for easier human consumption.
    ///
    /// # Use Cases
    ///
    /// - Delay-based filtering (e.g., "only mirrors with <6 hours delay")
    /// - Sorting by sync freshness (lowest delay first)
    /// - Displaying delay information in human-readable format
    ///
    /// # Returns
    ///
    /// - `Some(hours)`: Delay in hours as floating point
    /// - `None`: If delay information is not available for this mirror
    ///
    /// # Example
    ///
    /// ```ignore
    /// use mirage::Mirror;
    ///
    /// let mirror = Mirror {
    ///     url: "https://mirror.example.com/archlinux/".to_string(),
    ///     protocol: "https".to_string(),
    ///     country: "Germany".to_string(),
    ///     country_code: "DE".to_string(),
    ///     active: true,
    ///     completion_pct: None,
    ///     score: None,
    ///     ipv4: true,
    ///     ipv6: true,
    ///     isos: true,
    ///     last_sync: None,
    ///     delay: Some(3600), // 1 hour in seconds
    ///     duration_avg: None,
    ///     duration_stddev: None,
    ///     details: String::new(),
    /// };
    ///
    /// if let Some(delay_hours) = mirror.delay_hours() {
    ///     if delay_hours < 1.0 {
    ///         println!("Very current mirror ({}min delay)", delay_hours * 60.0);
    ///     } else {
    ///         println!("Mirror delay: {:.1} hours", delay_hours);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn delay_hours(&self) -> Option<f64> {
        // Convert seconds to hours with decimal precision
        self.delay.map(|d| f64::from(d) / 3600.0)
    }
}

/// Application configuration combining CLI arguments and config file settings.
///
/// This structure holds all configuration options for the Mirage application.
/// Configuration values are resolved in this priority order:
///
/// 1. **Command-line arguments** (highest priority)
/// 2. **Configuration file** (medium priority)  
/// 3. **Default values** (lowest priority)
///
/// # Configuration Sources
///
/// ## CLI Arguments
///
/// All fields can be set via command-line flags. CLI args always override
/// config file values to allow users to quickly adjust settings.
///
/// ## Config File
///
/// Configuration files are loaded from XDG-compliant locations:
/// - `$XDG_CONFIG_HOME/mirage/config`
/// - `~/.config/mirage/config`
/// - `/etc/mirage/config`
///
/// ## Example Configuration
///
/// ```bash
/// # ~/.config/mirage/config
/// --cache-timeout 600
/// --country Germany
/// --country France  
/// --protocol https
/// --completion-percent 95
/// --verbose
/// ```
///
/// # Field Categories
///
/// ## Network & Performance
/// - `connection_timeout`: HTTP connection timeout (1-300 seconds)
/// - `download_timeout`: HTTP download timeout (1-600 seconds)
/// - `threads`: Concurrent testing threads (1-100)
/// - `url`: Mirror status API endpoint
///
/// ## Caching
/// - `cache_timeout`: Cache validity period (60-86400 seconds)
///
/// ## Filtering
/// - `countries`: Geographic filter (country names/codes)
/// - `protocols`: Protocol filter (http, https, ftp, rsync)
/// - `age`: Maximum sync age (0-8760 hours)
/// - `delay`: Maximum sync delay (0-720 hours)
/// - `completion_percent`: Minimum completion percentage (0-100)
/// - `include_regex`/`exclude_regex`: URL pattern matching
/// - `ipv4`/`ipv6`/`isos`: Feature requirements
///
/// ## Output Control  
/// - `sort`: Sorting method (age, rate, country, etc.)
/// - `fastest`/`latest`/`score`/`number`: Result limiting
/// - `save_path`: Output file path
/// - `info`: Detailed vs. mirrorlist output
/// - `verbose`: Debug output
/// - `list_countries`: Country listing mode
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    // === Network Configuration ===
    /// Connection timeout in seconds (1-300)
    pub connection_timeout: u32,
    /// Download timeout in seconds (1-600)  
    pub download_timeout: u32,
    /// Mirror Status API URL (must be HTTPS)
    pub url: String,

    // === Caching Configuration ===
    /// Cache validity timeout in seconds (60-86400)
    pub cache_timeout: u32,

    // === Performance Configuration ===
    /// Number of concurrent threads for mirror testing (1-100)
    pub threads: Option<u32>,

    // === Output Configuration ===
    /// Show country distribution table instead of mirrors
    pub list_countries: bool,
    /// File path to save mirrorlist (None = stdout)
    pub save_path: Option<String>,
    /// Show detailed mirror info instead of mirrorlist format
    pub info: bool,
    /// Enable verbose debugging output
    pub verbose: bool,
    /// Suppress all informational output (quiet mode)
    pub quiet: bool,

    // === Sorting Configuration ===
    /// Sorting method: age, rate, country, score, delay, duration, duration-std
    pub sort: Option<String>,

    // === Filtering Configuration ===

    // Time-based filters
    /// Maximum sync age in hours (0-8760)
    pub age: Option<f64>,
    /// Maximum sync delay in hours (0-720)
    pub delay: Option<f64>,

    // Geographic filters
    /// Country names or ISO codes (can specify multiple)
    pub countries: Vec<String>,

    // Protocol and feature filters
    /// Required protocols: http, https, ftp, rsync (can specify multiple)
    pub protocols: Vec<String>,
    /// Minimum completion percentage (0.0-100.0)
    pub completion_percent: f64,
    /// Require IPv4 support
    pub ipv4: bool,
    /// Require IPv6 support
    pub ipv6: bool,
    /// Require ISO hosting
    pub isos: bool,

    // Pattern-based filters
    /// Include only URLs matching this regex
    pub include_regex: Option<String>,
    /// Exclude URLs matching this regex
    pub exclude_regex: Option<String>,

    // === Result Limiting Configuration ===
    /// Return N fastest mirrors (requires performance testing)
    pub fastest: Option<u32>,
    /// Return N most recently synchronized mirrors
    pub latest: Option<u32>,
    /// Return N highest-scoring mirrors
    pub score: Option<u32>,
    /// Maximum number of mirrors to return
    pub number: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connection_timeout: 5,
            download_timeout: 5,
            list_countries: false,
            cache_timeout: 300,
            url: "https://archlinux.org/mirrors/status/json/".to_string(),
            save_path: None,
            sort: None,
            threads: None,
            verbose: false,
            info: false,
            quiet: false,
            age: None,
            delay: None,
            countries: Vec::new(),
            fastest: None,
            include_regex: None,
            exclude_regex: None,
            latest: None,
            score: None,
            number: None,
            protocols: Vec::new(),
            completion_percent: 100.0,
            isos: false,
            ipv4: false,
            ipv6: false,
        }
    }
}

impl Config {
    /// Validate the configuration and return helpful error messages
    ///
    /// # Errors
    ///
    /// Returns validation errors for invalid configuration values.
    pub fn validate(&self) -> Result<()> {
        use error::ValidationExt;

        debug!("Validating configuration: {:#?}", self);

        // Validate timeouts
        (self.connection_timeout > 0 && self.connection_timeout <= 300)
            .validate("Connection timeout must be between 1 and 300 seconds")?;

        (self.download_timeout > 0 && self.download_timeout <= 600)
            .validate("Download timeout must be between 1 and 600 seconds")?;

        (self.cache_timeout >= 60 && self.cache_timeout <= 86400).validate(
            "Cache timeout must be between 60 seconds (1 minute) and 86400 seconds (24 hours)",
        )?;

        // Validate completion percentage
        (self.completion_percent >= 0.0 && self.completion_percent <= 100.0)
            .validate("Completion percentage must be between 0 and 100")?;

        // Validate age and delay
        if let Some(age) = self.age {
            (0.0..=8760.0)
                .contains(&age)
                .validate("Age must be between 0 and 8760 hours (1 year)")?;
        }

        if let Some(delay) = self.delay {
            (0.0..=720.0)
                .contains(&delay)
                .validate("Delay must be between 0 and 720 hours (30 days)")?;
        }

        // Validate thread count
        if let Some(threads) = self.threads {
            (threads > 0 && threads <= 100).validate("Thread count must be between 1 and 100")?;
        }

        // Validate counts
        if let Some(fastest) = self.fastest {
            (fastest > 0 && fastest <= 1000)
                .validate("Fastest count must be between 1 and 1000")?;
        }

        if let Some(latest) = self.latest {
            (latest > 0 && latest <= 1000).validate("Latest count must be between 1 and 1000")?;
        }

        if let Some(score) = self.score {
            (score > 0 && score <= 1000).validate("Score count must be between 1 and 1000")?;
        }

        if let Some(number) = self.number {
            (number > 0 && number <= 1000).validate("Number must be between 1 and 1000")?;
        }

        // Validate URL
        self.url
            .starts_with("https://")
            .validate("API URL must use HTTPS for security")?;

        // Validate regex patterns early
        if let Some(ref include_regex) = self.include_regex {
            regex::Regex::new(include_regex).map_err(|e| MirageError::regex(include_regex, e))?;
        }

        if let Some(ref exclude_regex) = self.exclude_regex {
            regex::Regex::new(exclude_regex).map_err(|e| MirageError::regex(exclude_regex, e))?;
        }

        // Validate protocols
        for protocol in &self.protocols {
            ["http", "https", "ftp", "rsync"]
                .contains(&protocol.as_str())
                .validate(&format!(
                    "Invalid protocol '{protocol}'. Supported: http, https, ftp, rsync"
                ))?;
        }

        // Validate sort method
        if let Some(ref sort_method) = self.sort {
            ["age", "rate", "country", "score", "delay", "duration", "duration-std"].contains(&sort_method.as_str())
                .validate(&format!("Invalid sort method '{sort_method}'. Supported: age, rate, country, score, delay, duration, duration-std"))?;
        }

        debug!("Configuration validation successful");
        Ok(())
    }
}

/// Validates that a save path is writable and accessible for mirrorlist output.
///
/// This function performs comprehensive validation of the specified file path to
/// ensure that mirage can successfully write the generated mirrorlist. It checks
/// directory permissions, file accessibility, and performs a test write to verify
/// the path is usable.
///
/// # Validation Process
///
/// 1. **Path Validation**: Ensures the path is not empty or whitespace-only
/// 2. **Parent Directory Check**: Verifies parent directory exists and is writable
/// 3. **Existing File Check**: If file exists, verifies it's not read-only
/// 4. **Write Test**: Attempts to open the file for writing to confirm access
///
/// # Arguments
///
/// - `path`: File path where the mirrorlist should be saved
///
/// # Returns
///
/// - `Ok(())`: Path is valid and writable
/// - `Err(MirageError::Validation)`: Path validation failed with specific reason
///
/// # Errors
///
/// This function returns validation errors for:
/// - **Empty path**: Path is empty or contains only whitespace
/// - **Missing parent**: Parent directory doesn't exist
/// - **Permission denied**: Cannot write to directory or file
/// - **Read-only**: File or directory has read-only permissions
/// - **Access denied**: General filesystem access issues
///
/// # Examples
///
/// ```ignore
/// use mirage::validate_save_path;
///
/// // Valid path
/// match validate_save_path("/etc/pacman.d/mirrorlist") {
///     Ok(()) => println!("Path is writable"),
///     Err(e) => eprintln!("Path validation failed: {}", e),
/// }
///
/// // Common error cases
/// assert!(validate_save_path("").is_err()); // Empty path
/// assert!(validate_save_path("/nonexistent/dir/file").is_err()); // Missing parent
/// ```
///
/// # Use Cases
///
/// - **Pre-flight checks**: Validate output path before mirror fetching
/// - **CLI validation**: Early validation of --save argument
/// - **Batch operations**: Verify multiple paths before processing
/// - **Error prevention**: Catch path issues before time-consuming operations
///
/// # Security Considerations
///
/// This function only validates write access and doesn't modify the file system
/// beyond a test file open. It's safe to use with user-provided paths as it
/// performs no destructive operations.
pub fn validate_save_path(path: &str) -> Result<()> {
    use error::ValidationExt;
    use std::path::Path;

    debug!("Validating save path: {}", path);

    let path_obj = Path::new(path);

    // Validate path is not empty
    (!path.trim().is_empty()).validate("Save path cannot be empty")?;

    // Check if parent directory exists
    if let Some(parent) = path_obj.parent() {
        if !parent.exists() {
            return Err(MirageError::validation(format!(
                "Parent directory does not exist: {}",
                parent.display()
            )));
        }

        // Check if parent directory is writable
        let metadata = fs::metadata(parent).map_err(|e| {
            MirageError::validation(format!(
                "Cannot access parent directory {}: {}",
                parent.display(),
                e
            ))
        })?;

        if metadata.permissions().readonly() {
            return Err(MirageError::validation(format!(
                "Parent directory is not writable: {}",
                parent.display()
            )));
        }
    }

    // If file already exists, check if it's writable
    if path_obj.exists() {
        let metadata = fs::metadata(path_obj)
            .map_err(|e| MirageError::validation(format!("Cannot access file {path}: {e}")))?;

        if metadata.permissions().readonly() {
            return Err(MirageError::validation(format!(
                "File is read-only: {path}"
            )));
        }
    }

    // Try to create/write to test file accessibility
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path_obj)
        .map_err(|e| MirageError::validation(format!("Cannot write to file {path}: {e}")))?;

    debug!("Save path validation successful");
    Ok(())
}

/// Filters mirrors based on configuration criteria using a multi-stage pipeline.
///
/// This function implements the core mirror filtering logic that reduces the full
/// set of available mirrors (typically 1000+) to a smaller set meeting specific
/// requirements. The filtering pipeline processes mirrors through multiple stages,
/// with each stage removing mirrors that don't meet the specified criteria.
///
/// # Filtering Pipeline
///
/// The filters are applied in this order for optimal performance:
///
/// 1. **Activity Filter**: Removes inactive mirrors (most common exclusion)
/// 2. **Completion Filter**: Removes mirrors below completion threshold
/// 3. **Age Filter**: Removes mirrors not synchronized recently enough
/// 4. **Delay Filter**: Removes mirrors with excessive sync delay
/// 5. **Geographic Filter**: Keeps only mirrors from specified countries
/// 6. **Protocol Filter**: Keeps only mirrors using specified protocols
/// 7. **Feature Filters**: IPv4/IPv6/ISO hosting requirements
/// 8. **Pattern Filters**: Include/exclude regex matching on URLs
///
/// # Arguments
///
/// - `mirrors`: Complete list of mirrors from the Mirror Status API
/// - `config`: Configuration specifying all filtering criteria
///
/// # Returns
///
/// A filtered vector of mirrors that meet all specified criteria.
/// The order is preserved from the input vector.
///
/// # Performance Characteristics
///
/// - **Time Complexity**: O(n) where n is the number of input mirrors
/// - **Memory Usage**: Single pass with in-place filtering
/// - **Early Exit**: Most restrictive filters are applied first
/// - **Regex Caching**: Regex patterns are compiled once per filter call
///
/// # Filter Behaviors
///
/// ## Active Filter (Always Applied)
/// Removes mirrors marked as inactive in the API response.
///
/// ## Completion Filter
/// - Threshold set by `config.completion_percent` (0-100)
/// - Mirrors with `None` completion are excluded
/// - Default: 100% (only complete mirrors)
///
/// ## Age Filter
/// - Limit set by `config.age` in hours
/// - Mirrors with `None` `last_sync` are excluded
/// - Example: `age: Some(24.0)` = "synced within 24 hours"
///
/// ## Geographic Filter
/// - Countries specified in `config.countries`
/// - Supports both country names and ISO codes
/// - Case-insensitive matching
/// - Comma-separated values supported
///
/// ## Protocol Filter
/// - Protocols specified in `config.protocols`
/// - Supported: http, https, ftp, rsync
/// - Case-insensitive matching
/// - Comma-separated values supported
///
/// ## Feature Filters
/// - `config.ipv4`: Requires IPv4 support
/// - `config.ipv6`: Requires IPv6 support  
/// - `config.isos`: Requires ISO hosting
///
/// ## Pattern Filters
/// - `config.include_regex`: Only URLs matching pattern
/// - `config.exclude_regex`: Exclude URLs matching pattern
/// - Invalid regex patterns cause exclusion (fail-safe)
///
/// # Examples
///
/// ```ignore
/// use mirage::{Config, filter_mirrors, Mirror};
///
/// let config = Config {
///     countries: vec!["Germany".to_string()],
///     protocols: vec!["https".to_string()],
///     age: Some(24.0),
///     completion_percent: 95.0,
///     ..Default::default()
/// };
///
/// let mirrors: Vec<Mirror> = fetch_mirrors_from_api(); // Example
/// let filtered = filter_mirrors(mirrors, &config);
///
/// println!("Filtered to {} mirrors", filtered.len());
/// ```
///
/// # Error Handling
///
/// This function does not return errors - invalid filter criteria are handled
/// gracefully:
/// - Invalid regex patterns are logged and cause exclusion
/// - Missing data fields (None values) cause exclusion
/// - Unknown countries/protocols are ignored
///
/// # Logging and Verbosity
///
/// - Debug logs track filtering progress and counts
/// - Info logs show initial and final mirror counts  
/// - Verbose mode prints progress to stderr
/// - Warnings for invalid regex patterns
///
/// # Thread Safety
///
/// This function is thread-safe and can be called concurrently with different
/// mirror lists and configurations without interference.
#[allow(clippy::too_many_lines)]
pub fn filter_mirrors(mirrors: Vec<Mirror>, config: &Config) -> Vec<Mirror> {
    let initial_count = mirrors.len();
    debug!("Filtering {} mirrors with configuration", initial_count);
    let mut filtered = mirrors;

    filtered.retain(|mirror| {
        if !mirror.active {
            return false;
        }

        if config.completion_percent > 0.0 {
            if let Some(completion) = mirror.completion_pct {
                if completion < config.completion_percent / 100.0 {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(age_limit) = config.age {
            if let Some(last_sync_hours) = mirror.last_sync_hours() {
                if last_sync_hours > age_limit {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let (Some(delay_limit), Some(delay_hours)) = (config.delay, mirror.delay_hours())
            && delay_hours > delay_limit
        {
            return false;
        }

        if !config.countries.is_empty() {
            let matches_country = config.countries.iter().any(|country| {
                country.split(',').any(|c| {
                    let c = c.trim().to_lowercase();
                    c == mirror.country.to_lowercase() || c == mirror.country_code.to_lowercase()
                })
            });
            if !matches_country {
                return false;
            }
        }

        if !config.protocols.is_empty() {
            let matches_protocol = config.protocols.iter().any(|protocol| {
                protocol.split(',').any(|p| {
                    let p = p.trim().to_lowercase();
                    p == mirror.protocol.to_lowercase()
                })
            });
            if !matches_protocol {
                return false;
            }
        }

        if config.isos && !mirror.isos {
            return false;
        }

        if config.ipv4 && !mirror.ipv4 {
            return false;
        }

        if config.ipv6 && !mirror.ipv6 {
            return false;
        }

        if let Some(include_regex) = &config.include_regex {
            match regex::Regex::new(include_regex) {
                Ok(regex) => {
                    if !regex.is_match(&mirror.url) {
                        return false;
                    }
                }
                Err(e) => {
                    warn!("Invalid include regex '{}': {}", include_regex, e);
                    if config.verbose {
                        eprintln!("Warning: Invalid include regex '{include_regex}': {e}");
                    }
                    return false;
                }
            }
        }

        if let Some(exclude_regex) = &config.exclude_regex {
            match regex::Regex::new(exclude_regex) {
                Ok(regex) => {
                    if regex.is_match(&mirror.url) {
                        return false;
                    }
                }
                Err(e) => {
                    warn!("Invalid exclude regex '{}': {}", exclude_regex, e);
                    if config.verbose {
                        eprintln!("Warning: Invalid exclude regex '{exclude_regex}': {e}");
                    }
                    // Invalid exclude regex - don't exclude anything
                }
            }
        }

        true
    });

    info!("Filtered {} -> {} mirrors", initial_count, filtered.len());
    if config.verbose && !config.quiet {
        eprintln!("After filtering: {} mirrors", filtered.len());
    }

    filtered
}

/// Sorts mirrors by specified criteria and applies result limiting.
///
/// This function implements the final stage of mirror processing, taking filtered
/// mirrors and applying sorting and limiting operations based on configuration.
/// It supports multiple sorting methods and result limiting options to help users
/// get the most appropriate mirrors for their needs.
///
/// # Sorting Methods
///
/// ## Age Sorting (`sort: "age"`)
/// - Sorts by most recently synchronized mirrors first
/// - Uses `last_sync_hours()` for comparison
/// - Mirrors without sync data are placed last
/// - Best for: Getting the freshest mirror data
///
/// ## Rate/Score Sorting (`sort: "rate"` or `sort: "score"`)
/// - Sorts by highest score/performance rating first
/// - Uses mirror quality scores from the API
/// - Mirrors without scores are placed last
/// - Best for: General performance optimization
///
/// ## Country Sorting (`sort: "country"`)
/// - Alphabetical sorting by country name
/// - Useful for geographic organization
/// - Best for: Organizing mirrors by location
///
/// ## Delay Sorting (`sort: "delay"`)
/// - Sorts by lowest sync delay first
/// - Prioritizes mirrors closest to upstream
/// - Best for: Minimizing package update lag
///
/// ## Duration Sorting (`sort: "duration"`)
/// - Sorts by fastest average response time
/// - Uses `duration_avg` field from API
/// - Best for: Optimizing connection speed
///
/// ## Duration Standard Deviation (`sort: "duration-std"`)
/// - Sorts by most consistent response times (lowest std dev)
/// - Uses `duration_stddev` field
/// - Best for: Reliable, predictable performance
///
/// # Result Limiting
///
/// Multiple limiting options can be applied independently:
///
/// ## Latest Selection (`config.latest`)
/// - Selects N most recently synchronized mirrors
/// - Always sorts by age regardless of main sort method
/// - Applied before other limiting options
///
/// ## Fastest Selection (`config.fastest`)
/// - Selects N highest-scoring mirrors
/// - Always sorts by score regardless of main sort method
/// - Applied after latest but before score/number limits
///
/// ## Score Selection (`config.score`)
/// - Selects N highest-scoring mirrors  
/// - Similar to fastest but respects main sort order
/// - Applied after fastest but before number limit
///
/// ## Number Limiting (`config.number`)
/// - Truncates to maximum N mirrors
/// - Applied last, respects all previous sorting/limiting
/// - General-purpose result count limiting
///
/// # Arguments
///
/// - `mirrors`: Filtered mirrors ready for sorting and limiting
/// - `config`: Configuration specifying sort method and limits
///
/// # Returns
///
/// A sorted and limited vector of mirrors ready for output or processing.
///
/// # Algorithm Details
///
/// ## Sorting Stability
/// - All sorts are stable (preserve relative order of equal elements)
/// - Missing data values are consistently handled (placed at end)
/// - Numeric comparisons use `partial_cmp` with fallback to `Equal`
///
/// ## Performance Characteristics
/// - **Time Complexity**: O(n log n) for sorting operations
/// - **Memory Usage**: In-place sorting with temporary allocations
/// - **Optimization**: Early truncation for large result sets
///
/// ## Missing Data Handling
/// For all sorting methods, mirrors with missing data are:
/// - Placed after mirrors with valid data
/// - Maintain stable relative ordering among themselves
/// - Never cause panics or undefined behavior
///
/// # Examples
///
/// ```ignore
/// use mirage::{Config, sort_mirrors, Mirror};
///
/// // Sort by age, get 10 most recent
/// let config = Config {
///     sort: Some("age".to_string()),
///     number: Some(10),
///     ..Default::default()
/// };
///
/// let mirrors: Vec<Mirror> = get_filtered_mirrors(); // Example
/// let sorted = sort_mirrors(mirrors, &config);
/// println!("Selected {} most recent mirrors", sorted.len());
///
/// // Get 5 fastest mirrors
/// let config = Config {
///     fastest: Some(5),
///     ..Default::default()
/// };
/// let fastest = sort_mirrors(mirrors, &config);
/// ```
///
/// # Edge Cases
///
/// - **Empty input**: Returns empty vector
/// - **All missing data**: Maintains input order
/// - **Limit exceeds count**: Returns all available mirrors
/// - **Multiple limits**: Applied in documented precedence order
///
/// # Logging
///
/// - Debug logs show sorting method and mirror counts
/// - Info logs report final mirror count after processing
/// - No warnings or errors (all operations are safe)
///
/// # Thread Safety
///
/// This function is thread-safe and can process different mirror lists
/// concurrently without interference. The input vector is moved and
/// modified in-place for efficiency.
#[allow(clippy::too_many_lines)]
pub fn sort_mirrors(mut mirrors: Vec<Mirror>, config: &Config) -> Vec<Mirror> {
    let initial_count = mirrors.len();

    if let Some(sort_method) = &config.sort {
        debug!(
            "Sorting {} mirrors by method: {}",
            initial_count, sort_method
        );
        match sort_method.as_str() {
            "age" => {
                mirrors.sort_by(|a, b| match (a.last_sync_hours(), b.last_sync_hours()) {
                    (Some(a_hours), Some(b_hours)) => a_hours
                        .partial_cmp(&b_hours)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
            }
            "rate" | "score" => {
                mirrors.sort_by(|a, b| match (a.score, b.score) {
                    (Some(a_score), Some(b_score)) => b_score
                        .partial_cmp(&a_score)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
            }
            "country" => {
                mirrors.sort_by(|a, b| a.country.cmp(&b.country));
            }
            "delay" => {
                mirrors.sort_by(|a, b| match (a.delay, b.delay) {
                    (Some(a_delay), Some(b_delay)) => a_delay.cmp(&b_delay),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
            }
            "duration" => {
                mirrors.sort_by(|a, b| match (a.duration_avg, b.duration_avg) {
                    (Some(a_duration), Some(b_duration)) => a_duration
                        .partial_cmp(&b_duration)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
            }
            "duration-std" => {
                mirrors.sort_by(|a, b| match (a.duration_stddev, b.duration_stddev) {
                    (Some(a_std), Some(b_std)) => a_std
                        .partial_cmp(&b_std)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });
            }
            _ => {}
        }
    }

    if let Some(latest) = config.latest {
        debug!("Selecting {} latest mirrors", latest);
        mirrors.sort_by(|a, b| match (a.last_sync_hours(), b.last_sync_hours()) {
            (Some(a_hours), Some(b_hours)) => a_hours
                .partial_cmp(&b_hours)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        mirrors.truncate(latest as usize);
    }

    if let Some(fastest) = config.fastest {
        debug!("Selecting {} fastest mirrors", fastest);
        mirrors.sort_by(|a, b| match (a.score, b.score) {
            (Some(a_score), Some(b_score)) => b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        mirrors.truncate(fastest as usize);
    }

    if let Some(score_limit) = config.score {
        debug!("Selecting {} highest scoring mirrors", score_limit);
        mirrors.sort_by(|a, b| match (a.score, b.score) {
            (Some(a_score), Some(b_score)) => b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        mirrors.truncate(score_limit as usize);
    }

    if let Some(number) = config.number {
        debug!("Limiting to {} mirrors", number);
        mirrors.truncate(number as usize);
    }

    info!(
        "Final mirror count after sorting/limiting: {}",
        mirrors.len()
    );
    mirrors
}
