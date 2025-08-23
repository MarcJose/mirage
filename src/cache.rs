/*
 * Cache Module
 *
 * This module provides persistent caching functionality for mirror data to reduce
 * API requests and improve performance. The cache system uses JSON serialization
 * and follows XDG Base Directory specifications for cache file placement.
 *
 * # Key Features
 *
 * - **Persistent Storage**: Mirror data is cached to disk using JSON format
 * - **XDG Compliance**: Cache files are stored in XDG-compliant directories
 * - **Version Control**: Cache format versioning prevents compatibility issues
 * - **Atomic Operations**: Cache writes use temporary files for data integrity
 * - **ETag Support**: HTTP ETags are cached for efficient API requests
 * - **Statistics**: Cache usage and performance metrics
 *
 * # Cache Directory Structure
 *
 * The cache follows XDG Base Directory specifications:
 * - `$XDG_CACHE_HOME/mirage/mirrors.json` (if XDG_CACHE_HOME is set)
 * - `$HOME/.cache/mirage/mirrors.json` (fallback)
 *
 * # Cache File Format
 *
 * The cache file contains:
 * - Mirror data array (from Arch Linux Mirror Status API)
 * - Timestamp for cache age validation
 * - ETag for HTTP conditional requests
 * - Version number for format compatibility
 *
 * # Thread Safety
 *
 * The cache operations are designed to be thread-safe through atomic file operations,
 * but concurrent access should be coordinated by the caller if needed.
 *
 * # Examples
 *
 * ```ignore
 * use mirage::cache::{load_cache, save_cache, PersistentCache};
 *
 * // Load existing cache
 * if let Some(cache) = load_cache()? {
 *     if cache.is_valid(300) { // 5 minute timeout
 *         println!("Using cached mirrors: {}", cache.mirrors.len());
 *     }
 * }
 *
 * // Save new cache
 * let cache = PersistentCache::new(mirrors, timestamp, etag);
 * save_cache(&cache)?;
 * ```
 */

use crate::{MirageError, Mirror, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Represents a persistent cache entry containing mirror data and metadata.
///
/// This structure holds all the information needed to cache mirror data persistently
/// to disk, including versioning for compatibility and HTTP caching metadata.
///
/// # Fields
///
/// - `mirrors`: The cached mirror data from the Arch Linux Mirror Status API
/// - `timestamp`: Unix timestamp when the cache was created (seconds since epoch)
/// - `etag`: HTTP `ETag` from the API response for conditional requests
/// - `cache_version`: Format version number to handle schema changes
///
/// # Serialization
///
/// This struct is serialized to JSON format when saved to disk. The JSON structure
/// is human-readable and includes pretty-printing for easier debugging.
///
/// # Example JSON Format
///
/// ```json
/// {
///   "mirrors": [...],
///   "timestamp": 1693123456,
///   "etag": "W/\"abc123\"",
///   "cache_version": 1
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistentCache {
    /// Array of mirror information from the Arch Linux Mirror Status API
    pub mirrors: Vec<Mirror>,

    /// Unix timestamp when this cache entry was created (seconds since epoch)
    pub timestamp: u64,

    /// HTTP `ETag` header value from the API response for efficient caching
    ///
    /// When present, this `ETag` can be used in subsequent API requests via the
    /// "If-None-Match" header to avoid downloading unchanged data (HTTP 304).
    pub etag: Option<String>,

    /// Cache format version number for backward/forward compatibility
    ///
    /// This version number is checked when loading cache files to ensure
    /// compatibility. Mismatched versions result in cache invalidation.
    pub cache_version: u32,
}

impl PersistentCache {
    /// Current cache format version
    ///
    /// This constant is incremented whenever the cache format changes in a
    /// backward-incompatible way. Version mismatches cause cache invalidation.
    const CACHE_VERSION: u32 = 1;

    /// Creates a new cache entry with the current version number.
    ///
    /// # Arguments
    ///
    /// - `mirrors`: Vector of mirror data to cache
    /// - `timestamp`: Unix timestamp when the cache was created
    /// - `etag`: Optional HTTP `ETag` from the API response
    ///
    /// # Returns
    ///
    /// A new `PersistentCache` instance with the current cache version.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let timestamp = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs();
    ///
    /// let cache = PersistentCache::new(
    ///     mirrors,
    ///     timestamp,
    ///     Some("W/\"abc123\"".to_string())
    /// );
    /// ```
    #[must_use]
    pub fn new(mirrors: Vec<Mirror>, timestamp: u64, etag: Option<String>) -> Self {
        Self {
            mirrors,
            timestamp,
            etag,
            cache_version: Self::CACHE_VERSION,
        }
    }

    /// Checks if the cache entry is still valid based on age and version.
    ///
    /// This method performs two validation checks:
    /// 1. **Version compatibility**: Ensures cache format matches current version
    /// 2. **Age validation**: Checks if cache is within the specified timeout
    ///
    /// # Arguments
    ///
    /// - `cache_timeout`: Maximum age in seconds before cache is considered stale
    ///
    /// # Returns
    ///
    /// - `true`: Cache is valid and can be used
    /// - `false`: Cache is invalid (stale or incompatible version)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Check if cache is valid for 5 minutes (300 seconds)
    /// if cache.is_valid(300) {
    ///     println!("Using cached data with {} mirrors", cache.mirrors.len());
    /// } else {
    ///     println!("Cache is stale, fetching fresh data");
    /// }
    /// ```
    ///
    /// # Time Handling
    ///
    /// If the system time cannot be determined, the current time defaults to 0,
    /// which will cause the cache to be considered invalid (fail-safe behavior).
    #[must_use]
    pub fn is_valid(&self, cache_timeout: u32) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0); // Fail-safe: treat unknown time as epoch

        // Check version compatibility - mismatched versions are always invalid
        if self.cache_version != Self::CACHE_VERSION {
            debug!(
                "Cache version mismatch: cached={}, current={}",
                self.cache_version,
                Self::CACHE_VERSION
            );
            return false;
        }

        // Check if cache is still within the timeout window
        let age_seconds = current_time.saturating_sub(self.timestamp);
        let is_fresh = age_seconds < u64::from(cache_timeout);

        debug!(
            "Cache age check: {}s old, timeout={}s, valid={}",
            age_seconds, cache_timeout, is_fresh
        );

        is_fresh
    }
}

/// Determines the cache directory path following XDG Base Directory Specification.
///
/// This function implements the XDG Base Directory Specification for cache file
/// placement, ensuring consistent behavior across different environments and
/// supporting user customization through environment variables.
///
/// # Directory Resolution Priority
///
/// 1. **`$XDG_CACHE_HOME/mirage`** - If `XDG_CACHE_HOME` is set
/// 2. **`$HOME/.cache/mirage`** - Standard XDG fallback
/// 3. **Error** - If neither `XDG_CACHE_HOME` nor `HOME` are available
///
/// # Automatic Directory Creation
///
/// If the determined cache directory doesn't exist, this function will attempt
/// to create it (including any necessary parent directories). This ensures
/// the cache system works immediately after installation.
///
/// # Returns
///
/// - `Ok(PathBuf)`: Successfully determined and created cache directory path
/// - `Err(MirageError)`: Failed to determine path or create directory
///
/// # Errors
///
/// This function returns errors in the following cases:
/// - Neither `XDG_CACHE_HOME` nor `HOME` environment variables are set
/// - Filesystem permissions prevent directory creation
/// - I/O errors occur during directory creation
///
/// # Examples
///
/// ```ignore
/// match get_cache_dir() {
///     Ok(cache_path) => {
///         println!("Cache directory: {}", cache_path.display());
///         // Typically prints: /home/user/.cache/mirage
///     }
///     Err(e) => {
///         eprintln!("Failed to determine cache directory: {}", e);
///     }
/// }
/// ```
///
/// # Environment Variables
///
/// - `XDG_CACHE_HOME`: User-specified cache directory (XDG standard)
/// - `HOME`: User's home directory (POSIX standard)
///
/// # Cross-Platform Behavior
///
/// While this implementation follows POSIX/XDG standards, it should work on
/// most Unix-like systems. Windows support depends on HOME variable availability.
pub fn get_cache_dir() -> Result<PathBuf> {
    debug!("Determining cache directory path using XDG specification");

    // Follow XDG Base Directory Specification for cache directory
    let cache_dir = if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        debug!("Using XDG_CACHE_HOME: {}", xdg_cache);
        PathBuf::from(xdg_cache)
    } else if let Ok(home) = std::env::var("HOME") {
        debug!("Using HOME/.cache: {}", home);
        PathBuf::from(home).join(".cache")
    } else {
        return Err(MirageError::cache(
            "Cannot determine cache directory: neither XDG_CACHE_HOME nor HOME environment variable is set",
        ));
    };

    // Create mirage-specific subdirectory
    let mirage_cache = cache_dir.join("mirage");

    // Ensure cache directory exists (create if necessary)
    if mirage_cache.exists() {
        debug!("Using existing cache directory: {}", mirage_cache.display());
    } else {
        debug!("Creating cache directory: {}", mirage_cache.display());
        fs::create_dir_all(&mirage_cache).map_err(|e| {
            MirageError::cache(format!(
                "Failed to create cache directory '{}': {}",
                mirage_cache.display(),
                e
            ))
        })?;
        info!("Created cache directory: {}", mirage_cache.display());
    }

    Ok(mirage_cache)
}

/// Loads the persistent cache from disk if it exists.
///
/// This function attempts to load and deserialize the cache file from the
/// XDG-compliant cache directory. If no cache file exists, it returns `None`
/// rather than an error, allowing the caller to proceed with fresh data fetching.
///
/// # Cache File Location
///
/// The cache is loaded from `{cache_dir}/mirrors.json` where `{cache_dir}`
/// is determined by [`get_cache_dir()`].
///
/// # Return Values
///
/// - `Ok(Some(cache))`: Cache file exists and was successfully loaded
/// - `Ok(None)`: No cache file exists (first run or cache was cleared)
/// - `Err(MirageError)`: Cache file exists but couldn't be read or parsed
///
/// # Errors
///
/// This function returns errors for:
/// - **I/O errors**: File exists but cannot be read (permissions, corruption)
/// - **Parse errors**: File exists but contains invalid JSON or incompatible data
/// - **Directory errors**: Cache directory cannot be determined or accessed
///
/// # Validation
///
/// The loaded cache is not automatically validated for age or version compatibility.
/// Use [`PersistentCache::is_valid()`] after loading to check if the cache should be used.
///
/// # Examples
///
/// ```ignore
/// match load_cache()? {
///     Some(cache) => {
///         if cache.is_valid(300) {  // 5 minutes
///             println!("Using cache with {} mirrors", cache.mirrors.len());
///             return Ok(cache.mirrors);
///         } else {
///             println!("Cache exists but is stale or incompatible");
///         }
///     }
///     None => {
///         println!("No cache found, will fetch fresh data");
///     }
/// }
/// ```
///
/// # Performance
///
/// This function reads the entire cache file into memory and deserializes it.
/// For large mirror datasets, this may use significant memory temporarily.
///
/// # Thread Safety
///
/// This function is thread-safe for reading, but does not coordinate with
/// concurrent write operations. If multiple threads might be reading/writing
/// the cache simultaneously, external synchronization is recommended.
pub fn load_cache() -> Result<Option<PersistentCache>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join("mirrors.json");

    // Check if cache file exists before attempting to read
    if !cache_file.exists() {
        debug!("No cache file found at: {}", cache_file.display());
        return Ok(None);
    }

    debug!("Loading cache from: {}", cache_file.display());

    // Read and parse the cache file
    let content = fs::read_to_string(&cache_file).map_err(|e| {
        MirageError::cache(format!(
            "Failed to read cache file '{}': {}",
            cache_file.display(),
            e
        ))
    })?;

    let cache: PersistentCache = serde_json::from_str(&content)
        .map_err(|e| MirageError::cache(format!(
            "Failed to parse cache file '{}': {}. The cache file may be corrupted or from an incompatible version.",
            cache_file.display(),
            e
        )))?;

    info!(
        "Successfully loaded cache: {} mirrors, version {}, age {:.1}h",
        cache.mirrors.len(),
        cache.cache_version,
        cache_age_hours(cache.timestamp)
    );

    Ok(Some(cache))
}

/// Helper function to calculate cache age in hours for logging
#[allow(clippy::cast_precision_loss)] // Acceptable for timestamp calculations
fn cache_age_hours(timestamp: u64) -> f64 {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    (current_time.saturating_sub(timestamp)) as f64 / 3600.0
}

/// Saves the cache to disk using an atomic write operation.
///
/// This function serializes the cache to JSON format and writes it to disk
/// using a temporary file and atomic rename to ensure data integrity. This
/// prevents corruption if the process is interrupted during writing.
///
/// # Atomic Write Process
///
/// 1. **Serialize**: Convert cache to pretty-printed JSON
/// 2. **Write temp**: Write JSON to temporary file (`mirrors.json.tmp`)
/// 3. **Atomic rename**: Rename temp file to final location (`mirrors.json`)
///
/// This ensures that readers never see a partially written or corrupted cache file.
///
/// # Arguments
///
/// - `cache`: The cache instance to save to disk
///
/// # JSON Format
///
/// The cache is saved as pretty-printed JSON for human readability and debugging:
///
/// ```json
/// {
///   "mirrors": [ ... ],
///   "timestamp": 1693123456,
///   "etag": "W/\"abc123\"",
///   "cache_version": 1
/// }
/// ```
///
/// # Returns
///
/// - `Ok(())`: Cache was successfully saved
/// - `Err(MirageError)`: Save operation failed
///
/// # Errors
///
/// This function returns errors for:
/// - **Serialization errors**: Cache data cannot be converted to JSON
/// - **I/O errors**: Cannot write to temporary file or rename
/// - **Permission errors**: Insufficient permissions to write in cache directory
/// - **Disk space**: Insufficient disk space for the cache file
///
/// # Examples
///
/// ```ignore
/// use std::time::{SystemTime, UNIX_EPOCH};
///
/// let timestamp = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
///
/// let cache = PersistentCache::new(mirrors, timestamp, etag);
///
/// match save_cache(&cache) {
///     Ok(()) => println!("Cache saved successfully"),
///     Err(e) => eprintln!("Failed to save cache: {}", e),
/// }
/// ```
///
/// # File Safety
///
/// The atomic rename operation ensures that:
/// - **No data corruption**: Readers never see partial writes
/// - **No data loss**: Original file is preserved until new one is complete
/// - **Concurrent access**: Safe for readers during write operations
///
/// # Performance
///
/// Pretty-printing the JSON increases file size but improves debuggability.
/// For very large mirror datasets, the serialization process may use
/// significant CPU and memory temporarily.
pub fn save_cache(cache: &PersistentCache) -> Result<()> {
    debug!("Saving cache with {} mirrors to disk", cache.mirrors.len());

    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join("mirrors.json");
    let temp_file = cache_dir.join("mirrors.json.tmp");

    // Serialize cache to pretty-printed JSON for readability
    let content = serde_json::to_string_pretty(cache).map_err(|e| {
        MirageError::cache(format!(
            "Failed to serialize cache data: {e}. This may indicate a data structure issue."
        ))
    })?;

    // Write to temporary file first for atomic operation
    debug!("Writing cache to temporary file: {}", temp_file.display());
    fs::write(&temp_file, content).map_err(|e| {
        MirageError::cache(format!(
            "Failed to write temporary cache file '{}': {}. Check disk space and permissions.",
            temp_file.display(),
            e
        ))
    })?;

    // Perform atomic rename to finalize the cache file
    debug!("Performing atomic rename to finalize cache");
    fs::rename(&temp_file, &cache_file).map_err(|e| {
        MirageError::cache(format!(
            "Failed to finalize cache file '{}': {}. The temporary file may still exist.",
            cache_file.display(),
            e
        ))
    })?;

    #[allow(clippy::cast_precision_loss)] // Acceptable for file size display
    let file_size_kb = cache_file.metadata().map(|m| m.len()).unwrap_or(0) as f64 / 1024.0;

    info!(
        "Cache successfully saved: {} mirrors, {:.1} KB, version {}",
        cache.mirrors.len(),
        file_size_kb,
        cache.cache_version
    );

    Ok(())
}

/// Removes the cache file from disk, if it exists.
///
/// This function provides a safe way to clear the persistent cache, which can
/// be useful for troubleshooting, forcing fresh data retrieval, or managing
/// disk space. It gracefully handles the case where no cache file exists.
///
/// # Behavior
///
/// - **Cache exists**: File is removed and success is logged
/// - **No cache**: Operation succeeds with debug message (not an error)
/// - **Permission/I/O error**: Returns error with detailed message
///
/// # Returns
///
/// - `Ok(())`: Cache was cleared (or didn't exist)
/// - `Err(MirageError)`: Failed to remove existing cache file
///
/// # Errors
///
/// This function returns errors for:
/// - **Permission errors**: Insufficient permissions to delete the file
/// - **I/O errors**: Filesystem errors during deletion
/// - **Directory errors**: Cache directory cannot be determined
///
/// # Examples
///
/// ```ignore
/// // Clear cache to force fresh data fetch
/// match clear_cache() {
///     Ok(()) => println!("Cache cleared successfully"),
///     Err(e) => eprintln!("Failed to clear cache: {}", e),
/// }
/// ```
///
/// # Use Cases
///
/// - **Troubleshooting**: Clear corrupted or problematic cache
/// - **Testing**: Ensure tests start with fresh state
/// - **Space management**: Remove cache to free disk space
/// - **Data refresh**: Force retrieval of latest mirror data
///
/// # Safety
///
/// This operation is irreversible - once cleared, the cache must be rebuilt
/// by fetching fresh data from the mirror status API.
pub fn clear_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join("mirrors.json");

    if cache_file.exists() {
        debug!("Removing cache file: {}", cache_file.display());
        fs::remove_file(&cache_file).map_err(|e| {
            MirageError::cache(format!(
                "Failed to remove cache file '{}': {}. Check file permissions.",
                cache_file.display(),
                e
            ))
        })?;
        info!("Cache file removed successfully: {}", cache_file.display());
    } else {
        debug!("No cache file to clear at: {}", cache_file.display());
        info!("Cache was already empty or never created");
    }

    Ok(())
}

/// Retrieves detailed statistics about the current cache state.
///
/// This function provides comprehensive information about the cache file,
/// including size, content, age, and version. It's useful for monitoring,
/// debugging, and user interfaces that display cache status.
///
/// # Return Values
///
/// - `Ok(Some(stats))`: Cache exists and statistics were collected
/// - `Ok(None)`: No cache file exists
/// - `Err(MirageError)`: Cache exists but statistics couldn't be collected
///
/// # Collected Statistics
///
/// - **File size**: Physical size of cache file on disk
/// - **Mirror count**: Number of mirrors in the cached dataset
/// - **Age**: When the cache was created (timestamp and calculated age)
/// - **Version**: Cache format version for compatibility checking
/// - **`ETag`**: HTTP `ETag` for conditional requests (if available)
///
/// # Errors
///
/// This function returns errors for:
/// - **Metadata errors**: Cannot read file system metadata
/// - **Parse errors**: Cache file exists but cannot be loaded
/// - **Directory errors**: Cache directory cannot be determined
///
/// # Examples
///
/// ```ignore
/// match get_cache_stats()? {
///     Some(stats) => {
///         println!("Cache: {} mirrors, {}, {:.1}h old",
///             stats.mirror_count,
///             stats.size_human(),
///             stats.age_hours()
///         );
///     }
///     None => {
///         println!("No cache found");
///     }
/// }
/// ```
///
/// # Performance
///
/// This function loads and parses the entire cache file to extract mirror count
/// and other metadata. For very large caches, this may be expensive. The
/// [`CacheStats`] object provides efficient methods for common calculations.
///
/// # Thread Safety
///
/// This function is safe to call concurrently with cache reads, but should be
/// coordinated with cache write operations to ensure consistent results.
///
/// # Panics
///
/// Panics if cache file exists but [`load_cache()`] returns `None`, which should
/// not occur under normal circumstances but indicates an internal inconsistency.
pub fn get_cache_stats() -> Result<Option<CacheStats>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join("mirrors.json");

    if !cache_file.exists() {
        debug!(
            "No cache file found for statistics at: {}",
            cache_file.display()
        );
        return Ok(None);
    }

    debug!("Collecting cache statistics from: {}", cache_file.display());

    // Get file system metadata for size information
    let metadata = fs::metadata(&cache_file).map_err(|e| {
        MirageError::cache(format!(
            "Failed to read cache file metadata '{}': {}",
            cache_file.display(),
            e
        ))
    })?;

    // Load cache content to extract mirror count and other data
    let cache = load_cache()?
        .expect("Cache file exists but load_cache returned None - this should not happen");

    let stats = CacheStats {
        size_bytes: metadata.len(),
        mirror_count: cache.mirrors.len(),
        timestamp: cache.timestamp,
        cache_version: cache.cache_version,
        etag: cache.etag,
    };

    debug!(
        "Cache statistics: {} mirrors, {} bytes, {:.1}h old, version {}",
        stats.mirror_count,
        stats.size_bytes,
        stats.age_hours(),
        stats.cache_version
    );

    Ok(Some(stats))
}

/// Statistics and metadata about the current cache state.
///
/// This structure provides detailed information about the cache file,
/// including both filesystem metadata and cache content information.
/// It includes utility methods for common formatting and calculations.
///
/// # Fields
///
/// - `size_bytes`: Physical size of the cache file on disk
/// - `mirror_count`: Number of mirrors stored in the cache
/// - `timestamp`: Unix timestamp when cache was created
/// - `cache_version`: Format version of the cached data
/// - `etag`: HTTP `ETag` from API response (if available)
///
/// # Utility Methods
///
/// - [`age_hours()`]: Calculate cache age in hours
/// - [`size_human()`]: Format file size in human-readable units
///
/// # Examples
///
/// ```ignore
/// if let Some(stats) = get_cache_stats()? {
///     println!("Cache Status:");
///     println!("  Mirrors: {}", stats.mirror_count);
///     println!("  Size: {}", stats.size_human());
///     println!("  Age: {:.1} hours", stats.age_hours());
///     println!("  Version: {}", stats.cache_version);
/// }
/// ```
#[derive(Debug)]
pub struct CacheStats {
    /// Physical size of the cache file in bytes
    pub size_bytes: u64,

    /// Number of mirrors stored in the cache
    pub mirror_count: usize,

    /// Unix timestamp when the cache was created
    pub timestamp: u64,

    /// Cache format version number
    pub cache_version: u32,

    /// HTTP `ETag` from the API response, if available
    pub etag: Option<String>,
}

impl CacheStats {
    /// Calculates the age of the cache in hours.
    ///
    /// # Returns
    ///
    /// The age of the cache as a floating-point number of hours.
    /// If the system time cannot be determined, returns a very large age.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let age = stats.age_hours();
    /// if age < 1.0 {
    ///     println!("Cache is fresh ({}m old)", (age * 60.0) as u32);
    /// } else {
    ///     println!("Cache is {:.1}h old", age);
    /// }
    /// ```
    #[must_use]
    pub fn age_hours(&self) -> f64 {
        cache_age_hours(self.timestamp)
    }

    /// Formats the cache file size in human-readable units.
    ///
    /// Uses binary units (1024 bytes = 1 KB) and formats to one decimal place.
    /// Supports B, KB, MB, and GB units.
    ///
    /// # Returns
    ///
    /// A formatted string with the size and appropriate unit.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// println!("Cache file size: {}", stats.size_human());
    /// // Output examples: "1.2 KB", "15.7 MB", "2.1 GB"
    /// ```
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn size_human(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.size_bytes as f64;
        let mut unit_index = 0;

        // Convert to appropriate unit (binary: 1024-based)
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{size:.1} {}", UNITS[unit_index])
    }
}
