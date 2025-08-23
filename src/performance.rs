/*
 * Performance Testing Module
 *
 * This module provides mirror performance testing capabilities for mirage, allowing
 * the application to measure and evaluate mirror responsiveness, availability, and
 * download speeds. It's designed to help users identify the fastest and most reliable
 * mirrors for their specific network conditions.
 *
 * # Key Features
 *
 * - **Latency Testing**: Measures response time using HTTP HEAD requests
 * - **Availability Testing**: Checks if mirrors respond with successful HTTP status codes
 * - **Download Speed Testing**: Measures actual data transfer rates
 * - **Concurrent Testing**: Supports testing multiple mirrors simultaneously
 * - **Timeout Management**: Configurable timeouts to prevent hanging on slow mirrors
 * - **Error Resilience**: Graceful handling of network failures and timeouts
 *
 * # Testing Strategy
 *
 * The module uses two complementary testing approaches:
 *
 * ## Basic Performance Testing
 * - Uses HTTP HEAD requests to minimize bandwidth usage
 * - Tests with `core.db` file which should be available on all Arch mirrors
 * - Measures latency and HTTP response status
 * - Fast and lightweight for initial mirror screening
 *
 * ## Enhanced Speed Testing
 * - Uses HTTP GET requests to download small test files
 * - Tests with `core.db.sig` signature files (typically <1KB)
 * - Measures both latency and actual download throughput
 * - More comprehensive but uses slightly more bandwidth
 *
 * # Performance Metrics
 *
 * The module captures several key performance indicators:
 *
 * - **Latency**: Round-trip time for HTTP requests (milliseconds)
 * - **Status Code**: HTTP response status for availability assessment
 * - **Availability**: Boolean flag indicating if mirror responded successfully
 * - **Download Speed**: Actual data transfer rate in KB/s (optional)
 * - **Test Timestamp**: When the performance test was conducted
 *
 * # File Selection for Testing
 *
 * Test files are carefully chosen to balance accuracy with efficiency:
 *
 * - **`core.db`**: Database file present on all mirrors, moderate size
 * - **`core.db.sig`**: Signature file, very small size (~512 bytes)
 * - Both are part of the core Arch Linux repository structure
 * - Files are updated regularly, ensuring cache behavior is realistic
 *
 * # Error Handling
 *
 * Performance testing is designed to be fault-tolerant:
 * - Network errors don't fail the entire operation
 * - Timeouts are treated as performance data (slow mirrors)
 * - HTTP errors are recorded but don't prevent testing other mirrors
 * - Partial failures still provide useful latency information
 *
 * # Integration with Mirror Selection
 *
 * Performance data integrates with mirage's mirror selection logic:
 * - Supports `--fastest` option for speed-based selection
 * - Enables performance-based sorting and filtering
 * - Provides data for user-informed mirror choices
 * - Can be used to eliminate consistently slow or unreliable mirrors
 *
 * # Concurrency and Threading
 *
 * The module is designed for concurrent operation:
 * - All async functions support parallel execution
 * - HTTP client is shared for connection reuse
 * - No shared mutable state between tests
 * - Safe for use in multi-threaded environments
 *
 * # Examples
 *
 * ```rust
 * use mirage::performance::{test_mirror_performance, MirrorWithPerformance};
 * use reqwest::Client;
 *
 * // Basic performance test
 * let client = Client::new();
 * let performance = test_mirror_performance(&client, &mirror, false).await?;
 * println!("Mirror latency: {}ms", performance.latency_ms);
 *
 * // Enhanced speed test
 * let performance = test_mirror_download_speed(&client, &mirror, true).await?;
 * if let Some(speed) = performance.download_speed_kbps {
 *     println!("Download speed: {} KB/s", speed);
 * }
 * ```
 */

use crate::{Mirror, Result};
use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Performance metrics collected from mirror testing.
///
/// This structure captures comprehensive performance data from mirror connectivity
/// and speed tests. It provides both basic availability information and detailed
/// performance metrics that can be used for mirror ranking and selection.
///
/// # Fields
///
/// - `latency_ms`: Round-trip response time in milliseconds
/// - `status_code`: HTTP status code from the test request
/// - `available`: Whether the mirror responded successfully
/// - `download_speed_kbps`: Actual download speed in kilobytes per second (optional)
/// - `tested_at`: Timestamp when the performance test was conducted
///
/// # Performance Interpretation
///
/// ## Latency (`latency_ms`)
/// - **< 100ms**: Excellent - very responsive mirror
/// - **100-300ms**: Good - acceptable response time
/// - **300-1000ms**: Slow - noticeable delay
/// - **> 1000ms**: Very slow - may cause timeout issues
///
/// ## Status Codes (`status_code`)
/// - **200**: Success - mirror is fully functional
/// - **404**: File not found - mirror may be incomplete or misconfigured
/// - **503**: Service unavailable - mirror is temporarily down
/// - **0**: Network error - connection failed or timed out
///
/// ## Download Speed (`download_speed_kbps`)
/// - **None**: Speed test not performed or failed
/// - **< 100 KB/s**: Very slow - unsuitable for large downloads
/// - **100-1000 KB/s**: Acceptable - suitable for package updates
/// - **> 1000 KB/s**: Fast - excellent for all operations
///
/// # Examples
///
/// ```rust
/// use mirage::performance::MirrorPerformance;
///
/// // Example of a fast, responsive mirror
/// let performance = MirrorPerformance {
///     latency_ms: 45,
///     status_code: 200,
///     available: true,
///     download_speed_kbps: Some(2500),
///     tested_at: std::time::SystemTime::now(),
/// };
///
/// // Example of a slow or problematic mirror
/// let performance = MirrorPerformance {
///     latency_ms: 1200,
///     status_code: 503,
///     available: false,
///     download_speed_kbps: None,
///     tested_at: std::time::SystemTime::now(),
/// };
/// ```
///
/// # Usage in Mirror Selection
///
/// This data is typically used to:
/// - Sort mirrors by performance (`--fastest` option)
/// - Filter out unavailable or slow mirrors
/// - Provide user feedback about mirror quality
/// - Cache performance data for future reference
#[derive(Debug, Clone)]
pub struct MirrorPerformance {
    /// Round-trip response time in milliseconds.
    ///
    /// This measures the time from sending an HTTP request to receiving the
    /// response headers, providing an indication of network latency and server
    /// responsiveness. Lower values indicate better performance.
    pub latency_ms: u32,

    /// HTTP status code returned by the mirror.
    ///
    /// Standard HTTP status codes indicating the result of the test request:
    /// - 200: Success
    /// - 404: File not found  
    /// - 503: Service unavailable
    /// - 0: Network error or timeout
    pub status_code: u16,

    /// Whether the mirror is available and responding successfully.
    ///
    /// This is typically `true` when `status_code` is in the 2xx range,
    /// indicating the mirror is functional and ready to serve packages.
    pub available: bool,

    /// Download speed in kilobytes per second, if measured.
    ///
    /// This optional field contains actual data transfer rates measured
    /// during download speed tests. Not all performance tests measure
    /// download speed due to bandwidth considerations.
    pub download_speed_kbps: Option<u32>,

    /// Timestamp when this performance data was collected.
    ///
    /// Used to determine the age of performance data and decide when
    /// retesting is needed. Performance data may become stale as network
    /// conditions and mirror load change over time.
    pub tested_at: std::time::SystemTime,
}

impl Default for MirrorPerformance {
    /// Creates a default `MirrorPerformance` representing an untested or failed mirror.
    ///
    /// The default values indicate a mirror that hasn't been tested or failed testing:
    /// - Zero latency (placeholder value)
    /// - Zero status code (network error)
    /// - Not available
    /// - No download speed data
    /// - Current timestamp
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mirage::performance::MirrorPerformance;
    ///
    /// let performance = MirrorPerformance::default();
    /// assert_eq!(performance.latency_ms, 0);
    /// assert_eq!(performance.status_code, 0);
    /// assert_eq!(performance.available, false);
    /// assert_eq!(performance.download_speed_kbps, None);
    /// ```
    fn default() -> Self {
        Self {
            latency_ms: 0,
            status_code: 0,
            available: false,
            download_speed_kbps: None,
            tested_at: std::time::SystemTime::now(),
        }
    }
}

/// A mirror combined with its performance testing results.
///
/// This structure pairs mirror metadata (from the Arch Linux Mirror Status API)
/// with performance data collected through direct testing. It allows the application
/// to work with both static mirror information and dynamic performance metrics.
///
/// # Fields
///
/// - `mirror`: Complete mirror metadata from the API
/// - `performance`: Optional performance test results
///
/// # Performance Data Lifecycle
///
/// The `performance` field follows this lifecycle:
/// 1. **Initial state**: `None` - mirror hasn't been tested
/// 2. **Testing**: Performance tests are run asynchronously
/// 3. **Tested state**: `Some(MirrorPerformance)` - results are available
/// 4. **Aging**: Performance data becomes stale over time
/// 5. **Retesting**: New performance tests update the data
///
/// # Examples
///
/// ```rust
/// use mirage::performance::MirrorWithPerformance;
///
/// // Create from a mirror without performance data
/// let mirror_with_perf = MirrorWithPerformance::from(mirror);
/// assert!(mirror_with_perf.performance.is_none());
///
/// // After testing
/// let mut mirror_with_perf = MirrorWithPerformance::from(mirror);
/// let performance = test_mirror_performance(&client, &mirror_with_perf.mirror, false).await?;
/// mirror_with_perf.performance = Some(performance);
/// ```
///
/// # Usage Patterns
///
/// This structure is commonly used in:
/// - Concurrent mirror testing operations
/// - Performance-based mirror ranking
/// - Caching performance data with mirror information
/// - User interfaces showing mirror status and performance
#[derive(Debug, Clone)]
pub struct MirrorWithPerformance {
    /// The mirror metadata from the Arch Linux Mirror Status API.
    ///
    /// Contains all static information about the mirror including URL,
    /// country, protocols, synchronization status, and other metadata.
    pub mirror: Mirror,

    /// Optional performance test results.
    ///
    /// This field is `None` for untested mirrors and `Some(MirrorPerformance)`
    /// for mirrors that have been tested. Performance data should be considered
    /// temporal and may need refreshing based on the `tested_at` timestamp.
    pub performance: Option<MirrorPerformance>,
}

impl From<Mirror> for MirrorWithPerformance {
    /// Creates a `MirrorWithPerformance` from a Mirror with no performance data.
    ///
    /// This conversion is used when initializing mirror testing operations.
    /// The resulting structure has `performance` set to `None`, indicating
    /// that performance testing hasn't been conducted yet.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mirage::performance::MirrorWithPerformance;
    ///
    /// // Convert from Mirror
    /// let mirror_with_perf = MirrorWithPerformance::from(mirror);
    ///
    /// // Or use Into trait
    /// let mirror_with_perf: MirrorWithPerformance = mirror.into();
    /// ```
    fn from(mirror: Mirror) -> Self {
        Self {
            mirror,
            performance: None,
        }
    }
}

/// Tests a mirror's basic performance using a lightweight HTTP HEAD request.
///
/// This function performs a basic performance test by sending an HTTP HEAD request
/// to a known file on the mirror. It measures response latency and checks availability
/// without downloading any content, making it bandwidth-efficient for testing many mirrors.
///
/// # Test Method
///
/// The test uses the `core.db` file from the core repository, which should be present
/// on all properly configured Arch Linux mirrors. The HEAD request method is used to:
/// - Minimize bandwidth usage (no content download)
/// - Check mirror availability and responsiveness
/// - Measure network latency and server response time
/// - Verify the mirror serves the expected content structure
///
/// # Arguments
///
/// - `client`: Shared HTTP client for connection reuse and configuration
/// - `mirror`: Mirror to test, containing URL and metadata
/// - `verbose`: Whether to print progress messages to stderr
///
/// # Returns
///
/// Always returns `Ok(MirrorPerformance)` with the test results. Network errors
/// and timeouts are recorded in the performance data rather than propagated as errors.
/// This design ensures that testing one mirror doesn't prevent testing others.
///
/// # Performance Data
///
/// The returned `MirrorPerformance` contains:
/// - **Latency**: Time from request start to response headers
/// - **Status Code**: HTTP status code (200 for success, 0 for network errors)
/// - **Available**: Whether the request succeeded (2xx status codes)
/// - **Download Speed**: Always `None` for this test type
/// - **Test Timestamp**: When the test was performed
///
/// # Timeout and Error Handling
///
/// - **Timeout**: 5 second timeout prevents hanging on slow mirrors
/// - **Network errors**: Treated as performance data, not fatal errors
/// - **HTTP errors**: Recorded in status code and availability fields
/// - **Partial results**: Even failed tests provide latency information
///
/// # Examples
///
/// ```rust
/// use mirage::performance::test_mirror_performance;
/// use reqwest::Client;
///
/// let client = Client::new();
/// let performance = test_mirror_performance(&client, &mirror, false).await?;
///
/// if performance.available {
///     println!("Mirror is responsive: {}ms latency", performance.latency_ms);
/// } else {
///     println!("Mirror failed: status {}", performance.status_code);
/// }
/// ```
///
/// # Concurrent Usage
///
/// This function is designed for concurrent execution:
///
/// ```rust
/// use futures::future::join_all;
///
/// let tasks: Vec<_> = mirrors.iter()
///     .map(|mirror| test_mirror_performance(&client, mirror, false))
///     .collect();
///
/// let results = join_all(tasks).await;
/// ```
///
/// # Logging and Verbosity
///
/// - **Debug logs**: Always generated for debugging and troubleshooting
/// - **Verbose output**: Progress messages printed to stderr when enabled
/// - **Warning logs**: Generated for failures and error responses
/// - **Success logs**: Generated for successful tests with timing information
///
/// # Errors
///
/// Returns [`MirageError`] for critical failures that prevent testing:
/// - Network client configuration errors
/// - URL parsing or construction failures
/// - Internal timing calculation errors
///
/// Note: HTTP errors, timeouts, and mirror unavailability are captured as performance
/// data rather than returned as errors, allowing the caller to make availability decisions.
#[allow(clippy::cast_possible_truncation)]
pub async fn test_mirror_performance(
    client: &Client,
    mirror: &Mirror,
    verbose: bool,
) -> Result<MirrorPerformance> {
    // Test with core.db file which should be available on all properly configured mirrors
    // This file is part of the core repository and is typically 1-2 MB in size
    let test_url = format!("{}core/os/x86_64/core.db", mirror.url);

    debug!("Testing mirror performance: {}", mirror.url);
    if verbose {
        eprintln!("Testing mirror: {}", mirror.url);
    }

    let start = Instant::now();

    // Use HTTP HEAD request to check availability without downloading content
    // This minimizes bandwidth usage while still testing connectivity and response time
    let response = client
        .head(&test_url)
        .timeout(Duration::from_secs(5)) // 5-second timeout prevents hanging
        .send()
        .await;

    let latency = start.elapsed();

    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let available = resp.status().is_success();

            if available {
                debug!(
                    "Mirror {} test successful: {}ms latency",
                    mirror.url,
                    latency.as_millis()
                );
            } else {
                warn!(
                    "Mirror {} returned error status: {}",
                    mirror.url, status_code
                );
                if verbose {
                    eprintln!("Mirror {} returned status: {}", mirror.url, status_code);
                }
            }

            Ok(MirrorPerformance {
                latency_ms: latency.as_millis() as u32,
                status_code,
                available,
                download_speed_kbps: None, // HEAD requests don't measure download speed
                tested_at: std::time::SystemTime::now(),
            })
        }
        Err(e) => {
            // Network errors are treated as performance data, not fatal errors
            // This allows testing to continue with other mirrors
            warn!("Failed to test mirror {}: {}", mirror.url, e);
            if verbose {
                eprintln!("Failed to test mirror {}: {}", mirror.url, e);
            }

            Ok(MirrorPerformance {
                latency_ms: latency.as_millis() as u32,
                status_code: 0, // 0 indicates network error
                available: false,
                download_speed_kbps: None,
                tested_at: std::time::SystemTime::now(),
            })
        }
    }
}

/// Tests a mirror's download speed by downloading a small file and measuring throughput.
///
/// This function performs an enhanced performance test that measures actual download
/// speed in addition to latency and availability. It downloads a small signature file
/// to balance accuracy with bandwidth efficiency, providing comprehensive performance
/// metrics for mirror ranking and selection.
///
/// # Test Method
///
/// The test uses the `core.db.sig` signature file, which is typically very small
/// (around 512 bytes) but still representative of actual download performance:
/// - **HTTP GET request**: Downloads actual content to measure transfer rates
/// - **Small file size**: Minimizes bandwidth impact while providing speed data
/// - **Real transfer measurement**: Measures actual data transfer, not just headers
/// - **Network condition reflection**: Captures current network performance
///
/// # Arguments
///
/// - `client`: Shared HTTP client for connection reuse and configuration
/// - `mirror`: Mirror to test, containing URL and metadata
/// - `verbose`: Whether to print detailed progress and results to stderr
///
/// # Returns
///
/// Always returns `Ok(MirrorPerformance)` with comprehensive test results including
/// download speed measurements when successful. Like the basic performance test,
/// errors are captured in the performance data rather than propagated.
///
/// # Performance Data
///
/// The returned `MirrorPerformance` contains:
/// - **Latency**: Total time from request start to download completion
/// - **Status Code**: HTTP status code (200 for success, 0 for network errors)
/// - **Available**: Whether the request and download succeeded
/// - **Download Speed**: Measured transfer rate in KB/s (if successful)
/// - **Test Timestamp**: When the test was performed
///
/// # Speed Calculation
///
/// Download speed is calculated as:
/// ```text
/// speed_kbps = (bytes_downloaded * 1000) / (download_time_ms * 1024)
/// ```
///
/// The calculation accounts for:
/// - Actual bytes transferred (not file size)
/// - Pure download time (excluding initial connection latency)
/// - Conversion to kilobytes per second (KB/s)
/// - Protection against division by zero
///
/// # Timeout and Error Handling
///
/// - **Extended timeout**: 10 second timeout accommodates slower connections
/// - **Two-phase timing**: Separate measurements for connection and download
/// - **Graceful degradation**: Partial results provided even on download failures
/// - **Error isolation**: Download failures don't prevent testing other mirrors
///
/// # Examples
///
/// ```rust
/// use mirage::performance::test_mirror_download_speed;
/// use reqwest::Client;
///
/// let client = Client::new();
/// let performance = test_mirror_download_speed(&client, &mirror, true).await?;
///
/// match performance.download_speed_kbps {
///     Some(speed) if speed > 1000 => println!("Fast mirror: {} KB/s", speed),
///     Some(speed) => println!("Moderate speed: {} KB/s", speed),
///     None => println!("Speed test failed or unavailable"),
/// }
/// ```
///
/// # Bandwidth Considerations
///
/// This test uses minimal bandwidth while providing accurate speed measurements:
/// - Small file size (typically 512 bytes)
/// - Quick test duration (usually < 1 second)
/// - Efficient for testing many mirrors concurrently
/// - Suitable for bandwidth-conscious environments
///
/// # When to Use
///
/// This enhanced test is appropriate when:
/// - Accurate speed measurements are needed for ranking
/// - User requests `--fastest` option with speed-based selection
/// - Detailed performance analysis is required
/// - Network conditions vary significantly between mirrors
///
/// Use the basic [`test_mirror_performance`] function when:
/// - Only availability and latency are needed
/// - Bandwidth usage must be minimized
/// - Testing a large number of mirrors quickly
/// - Initial screening before detailed testing
///
/// # Verbose Output
///
/// When `verbose = true`, detailed information is printed including:
/// - Mirror URL being tested
/// - Measured latency in milliseconds
/// - Number of bytes downloaded
/// - Calculated download speed in KB/s
/// - Error messages for failed tests
///
/// # Errors
///
/// Returns [`MirageError`] for critical failures that prevent testing:
/// - Network client configuration errors
/// - URL parsing or construction failures
/// - Internal timing calculation errors
///
/// Note: HTTP errors, timeouts, and mirror unavailability are captured as performance
/// data rather than returned as errors, allowing the caller to make availability decisions.
#[allow(clippy::cast_possible_truncation)]
pub async fn test_mirror_download_speed(
    client: &Client,
    mirror: &Mirror,
    verbose: bool,
) -> Result<MirrorPerformance> {
    // Use a small signature file for speed testing - typically around 512 bytes
    // This provides accurate speed measurements with minimal bandwidth usage
    let test_url = format!("{}core/os/x86_64/core.db.sig", mirror.url);

    debug!("Speed testing mirror: {}", mirror.url);
    if verbose {
        eprintln!("Speed testing mirror: {}", mirror.url);
    }

    let start = Instant::now();

    // Use HTTP GET request to download content and measure transfer rate
    // Extended timeout accommodates slower connections for download tests
    let response = client
        .get(&test_url)
        .timeout(Duration::from_secs(10)) // 10-second timeout for download tests
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16();

            if resp.status().is_success() {
                // Measure pure download time (excluding initial connection latency)
                let content_start = Instant::now();
                let content = resp.bytes().await;
                let download_duration = content_start.elapsed();

                match content {
                    Ok(bytes) => {
                        let bytes_len = bytes.len();
                        let latency = start.elapsed(); // Total time including connection

                        // Calculate download speed in KB/s
                        // Formula: (bytes * 1000 / ms) / 1024 = KB/s
                        let speed_kbps = if download_duration.as_millis() > 0 {
                            Some(
                                (bytes_len as u64 * 1000
                                    / download_duration.as_millis() as u64
                                    / 1024) as u32,
                            )
                        } else {
                            None // Avoid division by zero for instantaneous downloads
                        };

                        if verbose {
                            eprintln!(
                                "Mirror {} - Latency: {}ms, Downloaded: {} bytes, Speed: {:?} KB/s",
                                mirror.url,
                                latency.as_millis(),
                                bytes_len,
                                speed_kbps
                            );
                        }

                        Ok(MirrorPerformance {
                            latency_ms: latency.as_millis() as u32,
                            status_code,
                            available: true,
                            download_speed_kbps: speed_kbps,
                            tested_at: std::time::SystemTime::now(),
                        })
                    }
                    Err(e) => {
                        // Download failed after successful connection
                        if verbose {
                            eprintln!("Failed to download from {}: {}", mirror.url, e);
                        }

                        Ok(MirrorPerformance {
                            latency_ms: start.elapsed().as_millis() as u32,
                            status_code,
                            available: false, // Download failure means not available
                            download_speed_kbps: None,
                            tested_at: std::time::SystemTime::now(),
                        })
                    }
                }
            } else {
                // HTTP error status - mirror responded but returned error
                Ok(MirrorPerformance {
                    latency_ms: start.elapsed().as_millis() as u32,
                    status_code,
                    available: false,
                    download_speed_kbps: None,
                    tested_at: std::time::SystemTime::now(),
                })
            }
        }
        Err(e) => {
            // Network error - couldn't connect to mirror
            if verbose {
                eprintln!("Failed to connect to {}: {}", mirror.url, e);
            }

            Ok(MirrorPerformance {
                latency_ms: start.elapsed().as_millis() as u32,
                status_code: 0, // 0 indicates network error
                available: false,
                download_speed_kbps: None,
                tested_at: std::time::SystemTime::now(),
            })
        }
    }
}
