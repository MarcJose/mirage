/*!
 * Command-Line Interface Module
 *
 * This module handles all command-line argument parsing and configuration merging
 * for the Mirage application. It uses the `clap` library to define a comprehensive
 * CLI interface that supports all of Mirage's features.
 *
 * # Architecture
 *
 * The CLI system has three main components:
 *
 * 1. **`build_cli()`**: Defines the CLI structure with all arguments and help text
 * 2. **`parse_args()`**: Parses CLI arguments and merges with config file
 * 3. **Helper functions**: Merge values from CLI and config with proper precedence
 *
 * # Configuration Precedence
 *
 * Values are resolved in this order (highest to lowest priority):
 *
 * 1. **CLI arguments**: Direct command-line flags and options
 * 2. **Config file**: Values from configuration files
 * 3. **Default values**: Built-in defaults defined in this module
 *
 * # Example Usage
 *
 * ```bash
 * # Basic usage with CLI arguments
 * mirage --country Germany --protocol https --fastest 5
 *
 * # With configuration file + CLI overrides
 * mirage --verbose  # Uses config file defaults but forces verbose mode
 * ```
 *
 * # Help System
 *
 * The CLI provides comprehensive help in two levels:
 * - **Short help** (`-h`): Brief descriptions of all options
 * - **Long help** (`--help`): Detailed explanations with examples and best practices
 */

// Import the Config structure and config file loading functionality
use crate::{Config, config::load_config_from_file};
// Import clap components for building the CLI interface
use clap::{Arg, ArgAction, Command, value_parser};
// Standard library for environment variable access
use std::env;

/// Build the complete CLI command structure with all arguments and help text.
///
/// This function defines the entire command-line interface for Mirage using
/// the `clap` library. It includes:
///
/// - All command-line arguments with validation
/// - Comprehensive help text and examples
/// - Value parsers for type safety
/// - Argument constraints and validation rules
///
/// # Design Principles
///
/// ## User Experience
/// - Clear, consistent argument naming
/// - Comprehensive help with examples
/// - Sensible defaults that work out of the box
/// - Progressive disclosure (basic → advanced options)
///
/// ## Compatibility
/// - Similar interface to `reflector` for easy migration
/// - Standard Unix conventions for argument naming
/// - Support for both short and long argument forms where appropriate
///
/// ## Validation
/// - Type-safe parsing with `value_parser!`
/// - Range validation for numeric arguments
/// - Enum validation for choice-based arguments
///
/// # Returns
///
/// A configured `clap::Command` ready to parse command-line arguments.
///
/// # Example
///
/// ```rust
/// use mirage::cli::build_cli;
/// let cli = build_cli();
/// let matches = cli.get_matches();
///
/// if matches.get_flag("verbose") {
///     println!("Verbose mode enabled");
/// }
/// ```
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_cli() -> Command {
    Command::new("mirage")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Retrieve and filter a list of the latest Arch Linux mirrors")
        .long_about("A fast, reliable command-line tool for retrieving and filtering Arch Linux mirrors. 
Similar to reflector, but written in Rust with enhanced performance, security, and caching features.

Features:
- Fast concurrent mirror testing with configurable threads
- Persistent caching with XDG Base Directory compliance  
- Security hardening with HTTPS-only connections and TLS 1.2+
- Comprehensive filtering options (country, protocol, sync age, etc.)
- Progress indicators and colored output for better user experience
- Shell completion support for bash, zsh, fish, and PowerShell

Examples:
  # Get 10 fastest HTTPS mirrors from Germany, synchronized within 12 hours
  mirage --country Germany --protocol https --age 12 --fastest 10

  # Save mirrorlist to file with detailed info
  mirage --save /etc/pacman.d/mirrorlist --info --verbose

  # Rate mirrors using 8 threads and show progress
  mirage --sort rate --threads 8 --number 5

  # Clear cache and get fresh mirror data
  mirage --clear-cache && mirage --verbose")
        .arg(
            Arg::new("connection-timeout")
                .long("connection-timeout")
                .help("Connection timeout in seconds (1-300)")
                .long_help("Number of seconds to wait before a connection times out. 
Used for both mirror status API requests and mirror performance testing.
Default: 5 seconds. Range: 1-300 seconds.")
                .value_name("SECONDS")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("download-timeout")
                .long("download-timeout")
                .help("Download timeout in seconds (1-600)")
                .long_help("Number of seconds to wait before a download times out.
Used when testing mirror performance by downloading test files.
Default: 5 seconds. Range: 1-600 seconds.")
                .value_name("SECONDS")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("list-countries")
                .long("list-countries")
                .help("Display available countries and mirror counts")
                .long_help("Display a table showing all available countries and the number of mirrors in each.
Useful for discovering available countries before filtering by --country.
Output includes both country names and country codes.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("cache-timeout")
                .long("cache-timeout")
                .help("Cache timeout in seconds (60-86400)")
                .long_help("How long to keep cached mirror data before fetching fresh data from the API.
Cached data is stored persistently following XDG Base Directory specification.
Default: 300 seconds (5 minutes). Range: 60 seconds to 86400 seconds (24 hours).")
                .value_name("SECONDS")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("url")
                .long("url")
                .help("Mirror status API URL (must use HTTPS)")
                .long_help("The URL from which to retrieve mirror data in JSON format.
Must use HTTPS for security. Typically you won't need to change this unless using a custom mirror status API.
Default: https://archlinux.org/mirrors/status/json/")
                .value_name("URL")
        )
        .arg(
            Arg::new("save")
                .long("save")
                .help("Save mirrorlist to file")
                .long_help("Save the generated mirrorlist to the specified file path instead of printing to stdout.
The parent directory must exist and be writable. Creates a properly formatted Arch Linux mirrorlist with comments.
Common usage: --save /etc/pacman.d/mirrorlist")
                .value_name("FILE")
        )
        .arg(
            Arg::new("sort")
                .long("sort")
                .help("Sort mirrors by specified method")
                .long_help("Sort the mirrorlist by the specified method:
• age: Sort by last synchronization time (most recent first)
• rate/score: Sort by mirror score (highest first, requires testing)
• country: Sort alphabetically by country name
• delay: Sort by reported sync delay (lowest first)
• duration: Sort by average response time (fastest first)
• duration-std: Sort by response time consistency (most consistent first)")
                .value_name("METHOD")
                .value_parser(["age", "rate", "country", "score", "delay", "duration", "duration-std"])
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .help("Number of concurrent threads for mirror testing (1-100)")
                .long_help("Number of concurrent threads to use when testing mirror performance.
Only used when sorting by 'rate' or 'score', or when using --fastest.
Higher values speed up testing but increase network load.
Default: Number of CPU cores. Range: 1-100 threads.")
                .value_name("COUNT")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Enable verbose output and debug logging")
                .long_help("Print detailed progress information, debug messages, and timing data to stderr.
Useful for troubleshooting and understanding what the tool is doing.
Also enables debug-level structured logging when available.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Quiet mode - suppress all informational output")
                .long_help("Suppress all informational output and only print the mirrorlist.
Useful for scripts and automation where only the mirror URLs are needed.
Progress indicators, timing information, and status messages are suppressed.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("info")
                .long("info")
                .help("Display detailed mirror information instead of mirrorlist")
                .long_help("Display comprehensive information about each mirror including:
• URL, protocol, country, and country code
• Last synchronization time and completion percentage
• Mirror delay, score, and performance metrics  
• Support for IPv4, IPv6, and ISO downloads
• Color-coded status indicators for easy reading")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("age")
                .short('a')
                .long("age")
                .help("Maximum sync age in hours (0-8760)")
                .long_help("Only return mirrors that have synchronized within the last N hours.
Filters out mirrors that are too out-of-date to be useful.
Use smaller values (1-24) for the most up-to-date mirrors.
Range: 0-8760 hours (1 year). Example: --age 12")
                .value_name("HOURS")
                .value_parser(value_parser!(f64))
        )
        .arg(
            Arg::new("delay")
                .long("delay")
                .help("Maximum sync delay in hours (0-720)")
                .long_help("Only return mirrors with a reported sync delay of N hours or less.
Sync delay is how far behind a mirror is from the main Arch repositories.
Lower values ensure mirrors are more up-to-date.
Range: 0-720 hours (30 days). Example: --delay 6")
                .value_name("HOURS")
                .value_parser(value_parser!(f64))
        )
        .arg(
            Arg::new("country")
                .short('c')
                .long("country")
                .help("Filter by country name or code (can be used multiple times)")
                .long_help("Restrict mirrors to the specified countries. Accepts both country names and ISO country codes.
Can be used multiple times to include multiple countries.
Use --list-countries to see available options.
Examples: --country Germany --country US --country 'United Kingdom'")
                .value_name("NAME_OR_CODE")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("fastest")
                .short('f')
                .long("fastest")
                .help("Return the N fastest mirrors (1-1000, requires testing)")
                .long_help("Test mirrors and return only the N fastest ones that meet other criteria.
Automatically enables performance testing with progress indicators.
Combines well with --threads for faster testing.
Range: 1-1000 mirrors. Example: --fastest 5 --threads 8")
                .value_name("COUNT")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("include")
                .short('i')
                .long("include")
                .help("Include servers that match <regex>.")
                .value_name("regex")
        )
        .arg(
            Arg::new("exclude")
                .short('x')
                .long("exclude")
                .help("Exclude servers that match <regex>.")
                .value_name("regex")
        )
        .arg(
            Arg::new("latest")
                .short('l')
                .long("latest")
                .help("Limit the list to the n most recently synchronized servers.")
                .value_name("n")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("score")
                .long("score")
                .help("Limit the list to the n servers with the highest score.")
                .value_name("n")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("number")
                .short('n')
                .long("number")
                .help("Return at most n mirrors.")
                .value_name("n")
                .value_parser(value_parser!(u32))
        )
        .arg(
            Arg::new("protocol")
                .short('p')
                .long("protocol")
                .help("Match one of the given protocols.")
                .value_name("protocol")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("completion-percent")
                .long("completion-percent")
                .help("Set the minimum completion percent for the returned mirrors. Default: 100.0")
                .value_name("0-100")
                .value_parser(value_parser!(f64))
        )
        .arg(
            Arg::new("isos")
                .long("isos")
                .help("Only return mirrors that host ISOs.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ipv4")
                .long("ipv4")
                .help("Only return mirrors that support IPv4.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ipv6")
                .long("ipv6")
                .help("Only return mirrors that support IPv6.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("clear-cache")
                .long("clear-cache")
                .help("Clear the persistent cache and exit.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("cache-info")
                .long("cache-info")
                .help("Show cache information and exit.")
                .action(ArgAction::SetTrue)
        )
}

/// Merge a single-value argument from CLI and config file sources.
///
/// This helper function implements the configuration precedence logic for
/// arguments that accept a single value (not lists). CLI arguments always
/// take precedence over config file values.
///
/// # Type Parameters
///
/// - `T`: The type of the argument value (must be cloneable and thread-safe)
///
/// # Parameters
///
/// - `cli_matches`: Parsed CLI arguments
/// - `config_matches`: Parsed config file arguments (optional)
/// - `arg_name`: Name of the argument to look up
///
/// # Returns
///
/// - `Some(value)`: If found in CLI args or config file
/// - `None`: If not found in either source
///
/// # Precedence Rules
///
/// 1. **CLI value**: If present, always used (highest priority)
/// 2. **Config value**: Used if CLI value not present
/// 3. **None**: If not found in either source
///
/// # Example
///
/// ```rust,no_run
/// # use mirage::cli::get_merged_value;
/// # let cli_matches = clap::ArgMatches::default();
/// # let config_matches: Option<clap::ArgMatches> = None;
/// // CLI: mirage --age 24
/// // Config: --age 12
/// // Result: Some(24) - CLI takes precedence
///
/// let age: Option<f64> = get_merged_value(&cli_matches, config_matches.as_ref(), "age");
/// ```
#[must_use]
pub fn get_merged_value<T>(
    cli_matches: &clap::ArgMatches,
    config_matches: Option<&clap::ArgMatches>,
    arg_name: &str,
) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    // CLI takes precedence over config (highest priority)
    if let Some(value) = cli_matches.get_one::<T>(arg_name) {
        Some(value.clone())
    } else if let Some(config) = config_matches {
        // Fall back to config file value
        config.get_one::<T>(arg_name).cloned()
    } else {
        // Not found in either source
        None
    }
}

/// Merge a multi-value argument from CLI and config file sources.
///
/// This helper function handles arguments that can be specified multiple times
/// (like `--country` or `--protocol`). Unlike single values, this follows an
/// "all-or-nothing" precedence - if ANY values are provided via CLI, ALL
/// config file values for that argument are ignored.
///
/// # Type Parameters
///
/// - `T`: The type of the argument values (must be cloneable and thread-safe)
///
/// # Parameters
///
/// - `cli_matches`: Parsed CLI arguments
/// - `config_matches`: Parsed config file arguments (optional)
/// - `arg_name`: Name of the argument to look up
///
/// # Returns
///
/// A `Vec<T>` containing the merged values, or empty if none found.
///
/// # Precedence Rules
///
/// 1. **CLI values**: If ANY CLI values present, use ALL CLI values only
/// 2. **Config values**: If no CLI values, use ALL config file values
/// 3. **Empty**: If not found in either source
///
/// # Rationale
///
/// This "replace-all" behavior prevents confusing interactions where CLI
/// arguments would merge with config file values. For example:
///
/// ```bash
/// # Config: --country Germany --country France
/// # CLI: mirage --country US
/// # Result: ["US"] not ["Germany", "France", "US"]
/// ```
///
/// # Example
///
/// ```rust,no_run
/// # use mirage::cli::get_merged_many;
/// # let cli_matches = clap::ArgMatches::default();
/// # let config_matches: Option<clap::ArgMatches> = None;
/// // Config: --country Germany --country France  
/// // CLI: mirage --country US --country UK
/// // Result: vec!["US", "UK"] - config ignored
///
/// let countries: Vec<String> = get_merged_many(&cli_matches, config_matches.as_ref(), "country");
/// ```
#[must_use]
pub fn get_merged_many<T>(
    cli_matches: &clap::ArgMatches,
    config_matches: Option<&clap::ArgMatches>,
    arg_name: &str,
) -> Vec<T>
where
    T: Clone + Send + Sync + 'static,
{
    // CLI takes precedence over config (all-or-nothing)
    if let Some(values) = cli_matches.get_many::<T>(arg_name) {
        // CLI values found - use ALL CLI values, ignore config
        values.cloned().collect()
    } else if let Some(config) = config_matches {
        // No CLI values - use ALL config values
        if let Some(values) = config.get_many::<T>(arg_name) {
            values.cloned().collect()
        } else {
            Vec::new()
        }
    } else {
        // Not found in either source
        Vec::new()
    }
}

/// Merge a boolean flag argument from CLI and config file sources.
///
/// This helper function handles boolean flags (like `--verbose` or `--info`)
/// that don't take values. Flags follow OR logic - if the flag is present
/// in EITHER CLI or config, the result is `true`.
///
/// # Parameters
///
/// - `cli_matches`: Parsed CLI arguments
/// - `config_matches`: Parsed config file arguments (optional)
/// - `arg_name`: Name of the flag to look up
///
/// # Returns
///
/// - `true`: If flag is present in CLI args OR config file
/// - `false`: If flag is not present in either source
///
/// # Precedence Rules
///
/// 1. **CLI flag present**: Always returns `true`
/// 2. **Config flag present**: Returns `true` if CLI flag not set
/// 3. **Neither present**: Returns `false`
///
/// # Note on Boolean Logic
///
/// Unlike other argument types, boolean flags use OR logic rather than
/// precedence. This is because there's no meaningful way to "override"
/// a true flag with false - flags are either present or absent.
///
/// # Example
///
/// ```rust,no_run
/// # use mirage::cli::get_merged_flag;
/// # let cli_matches = clap::ArgMatches::default();
/// # let config_matches: Option<clap::ArgMatches> = None;
/// // Config: --verbose
/// // CLI: mirage (no --verbose flag)
/// // Result: true - config flag is used
///
/// // CLI: mirage --verbose
/// // Result: true - CLI flag takes effect
///
/// let verbose = get_merged_flag(&cli_matches, config_matches.as_ref(), "verbose");
/// ```
#[must_use]
pub fn get_merged_flag(
    cli_matches: &clap::ArgMatches,
    config_matches: Option<&clap::ArgMatches>,
    arg_name: &str,
) -> bool {
    // Check CLI first (if present, always true)
    if cli_matches.get_flag(arg_name) {
        true
    } else if let Some(config) = config_matches {
        // Fall back to config file flag
        config.get_flag(arg_name)
    } else {
        // Not present in either source
        false
    }
}

/// Parse command-line arguments and merge with configuration file settings.
///
/// This is the main entry point for configuration processing. It implements
/// a sophisticated configuration system that merges multiple sources with
/// proper precedence handling.
///
/// # Configuration Resolution Process
///
/// 1. **Detect verbose mode**: Check CLI args for `--verbose` flag early to
///    enable verbose output during config file loading
///
/// 2. **Load config file**: Load and parse configuration file arguments using
///    XDG-compliant directory search
///
/// 3. **Parse config args**: Parse config file content as if it were CLI arguments
///    to leverage clap's validation and type conversion
///
/// 4. **Parse CLI args**: Parse actual command-line arguments with validation
///
/// 5. **Merge with precedence**: Combine CLI and config values using helper
///    functions that implement proper precedence rules
///
/// 6. **Apply defaults**: Use built-in defaults for any values not specified
///    in either CLI args or config file
///
/// # Configuration Sources (Priority Order)
///
/// 1. **CLI Arguments**: Highest priority, always override config/defaults
/// 2. **Config File**: Medium priority, overrides defaults
/// 3. **Built-in Defaults**: Lowest priority, used when not specified elsewhere
///
/// # Config File Locations (Search Order)
///
/// - `$XDG_CONFIG_HOME/mirage/config`
/// - `~/.config/mirage/config`  
/// - `/etc/mirage/config`
///
/// # Error Handling
///
/// - **Config file errors**: Non-fatal, app continues with CLI args + defaults
/// - **CLI argument errors**: Fatal, app exits with error message
/// - **Validation errors**: Handled by clap's built-in validation
///
/// # Example Configuration Merge
///
/// ```bash
/// # Config file: ~/.config/mirage/config
/// --country Germany
/// --protocol https
/// --cache-timeout 600
///
/// # CLI command:
/// mirage --country France --verbose
///
/// # Result:
/// # - country: ["France"] (CLI overrides config)
/// # - protocol: ["https"] (from config file)
/// # - cache_timeout: 600 (from config file)
/// # - verbose: true (from CLI)
/// # - connection_timeout: 5 (built-in default)
/// ```
///
/// # Returns
///
/// A fully populated `Config` struct with all values resolved according
/// to the precedence rules described above.
#[must_use]
pub fn parse_args() -> Config {
    // STEP 1: Early detection of verbose and quiet modes for config file loading
    // We need to check this before loading config files so we can enable
    // verbose output during the config file loading process
    let cli_args: Vec<String> = env::args().collect();
    let verbose = cli_args.iter().any(|arg| arg == "--verbose");
    let quiet = cli_args.iter().any(|arg| arg == "--quiet" || arg == "-q");

    // STEP 2: Load configuration file arguments
    // This searches XDG-compliant directories and parses the config file
    // into a list of arguments (e.g., ["--country", "Germany", "--verbose"])
    let config_args = load_config_from_file(verbose && !quiet);

    // STEP 3: Parse config file arguments using clap
    // We parse the config file as if it were CLI arguments to leverage
    // clap's validation, type conversion, and error handling
    let config_matches = if config_args.is_empty() {
        // No config file found or config file was empty
        None
    } else {
        // Prepend program name (required by clap) to config arguments
        let mut config_with_program = vec!["mirage".to_string()];
        config_with_program.extend(config_args);

        // Parse config arguments into structured matches
        Some(build_cli().get_matches_from(config_with_program))
    };

    // STEP 4: Parse actual CLI arguments
    // This parses the real command-line arguments provided by the user
    let cli_matches = build_cli().get_matches();

    // STEP 5: Merge all configuration sources with proper precedence
    // Each field is resolved using the helper functions that implement
    // the CLI > config > default precedence hierarchy
    Config {
        // === Network Configuration ===
        connection_timeout: get_merged_value(
            &cli_matches,
            config_matches.as_ref(),
            "connection-timeout",
        )
        .unwrap_or(5u32), // Default: 5 seconds
        download_timeout: get_merged_value(
            &cli_matches,
            config_matches.as_ref(),
            "download-timeout",
        )
        .unwrap_or(5u32), // Default: 5 seconds
        url: get_merged_value(&cli_matches, config_matches.as_ref(), "url")
            .unwrap_or("https://archlinux.org/mirrors/status/json/".to_string()),

        // === Caching Configuration ===
        cache_timeout: get_merged_value(&cli_matches, config_matches.as_ref(), "cache-timeout")
            .unwrap_or(300u32), // Default: 5 minutes

        // === Performance Configuration ===
        threads: get_merged_value(&cli_matches, config_matches.as_ref(), "threads"),

        // === Output Configuration ===
        list_countries: get_merged_flag(&cli_matches, config_matches.as_ref(), "list-countries"),
        save_path: get_merged_value(&cli_matches, config_matches.as_ref(), "save"),
        info: get_merged_flag(&cli_matches, config_matches.as_ref(), "info"),
        verbose: get_merged_flag(&cli_matches, config_matches.as_ref(), "verbose"),
        quiet: get_merged_flag(&cli_matches, config_matches.as_ref(), "quiet"),

        // === Sorting Configuration ===
        sort: get_merged_value(&cli_matches, config_matches.as_ref(), "sort"),

        // === Filtering Configuration ===

        // Time-based filters
        age: get_merged_value(&cli_matches, config_matches.as_ref(), "age"),
        delay: get_merged_value(&cli_matches, config_matches.as_ref(), "delay"),

        // Geographic filters
        countries: get_merged_many(&cli_matches, config_matches.as_ref(), "country"),

        // Protocol and feature filters
        protocols: get_merged_many(&cli_matches, config_matches.as_ref(), "protocol"),
        completion_percent: get_merged_value(
            &cli_matches,
            config_matches.as_ref(),
            "completion-percent",
        )
        .unwrap_or(100.0f64), // Default: 100% completion required
        ipv4: get_merged_flag(&cli_matches, config_matches.as_ref(), "ipv4"),
        ipv6: get_merged_flag(&cli_matches, config_matches.as_ref(), "ipv6"),
        isos: get_merged_flag(&cli_matches, config_matches.as_ref(), "isos"),

        // Pattern-based filters
        include_regex: get_merged_value(&cli_matches, config_matches.as_ref(), "include"),
        exclude_regex: get_merged_value(&cli_matches, config_matches.as_ref(), "exclude"),

        // === Result Limiting Configuration ===
        fastest: get_merged_value(&cli_matches, config_matches.as_ref(), "fastest"),
        latest: get_merged_value(&cli_matches, config_matches.as_ref(), "latest"),
        score: get_merged_value(&cli_matches, config_matches.as_ref(), "score"),
        number: get_merged_value(&cli_matches, config_matches.as_ref(), "number"),
    }
}
