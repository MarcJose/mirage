/*
 * Configuration Module
 *
 * This module provides configuration file discovery, parsing, and loading functionality
 * for mirage. It implements XDG Base Directory Specification compliance and supports
 * a flexible configuration file format that mirrors command-line arguments.
 *
 * # Key Features
 *
 * - **XDG Compliance**: Configuration files follow XDG Base Directory Specification
 * - **Multiple Locations**: Searches multiple standard config directories in priority order
 * - **CLI-Compatible Format**: Config file format matches command-line argument syntax
 * - **Quote Support**: Handles quoted arguments with spaces (e.g., `--country "United States"`)
 * - **Comment Support**: Allows `#` comments and empty lines for documentation
 * - **Error Resilience**: Graceful handling of missing or malformed config files
 *
 * # Configuration File Format
 *
 * Configuration files use a simple format where each line contains command-line arguments:
 *
 * ```text
 * # mirage configuration file
 * --country Germany
 * --country "United States"
 * --protocol https
 * --age 12
 * --completion-percent 95
 * --fastest 10
 * --verbose
 * ```
 *
 * # Directory Search Priority
 *
 * Configuration files are searched in the following order:
 * 1. `$XDG_CONFIG_HOME/mirage/config` (if XDG_CONFIG_HOME is set)
 * 2. `$HOME/.config/mirage/config` (standard XDG fallback)
 * 3. `$HOME/.mirage/config` (legacy location)
 *
 * The first existing and readable file is used; subsequent locations are ignored.
 *
 * # Integration with CLI
 *
 * Configuration files are loaded and parsed into argument vectors that are then
 * processed by the CLI argument parser (clap). This ensures:
 * - Consistent validation between CLI and config file arguments
 * - CLI arguments override config file settings
 * - Same error handling and type conversion for both sources
 *
 * # Parsing Rules
 *
 * - **Comments**: Lines starting with `#` are ignored
 * - **Empty lines**: Blank lines are ignored
 * - **Quotes**: Arguments containing spaces must be quoted (e.g., `"United States"`)
 * - **Whitespace**: Leading/trailing whitespace is trimmed
 * - **Multiple arguments**: Multiple arguments can be on the same line
 * - **Escaping**: Quotes are the only special characters (no escape sequences)
 *
 * # Error Handling
 *
 * The module follows a fail-safe approach:
 * - Missing config files result in empty argument lists (not errors)
 * - Parse errors are logged but don't prevent program execution
 * - Malformed files result in empty argument lists with warnings
 *
 * # Examples
 *
 * ```rust
 * use mirage::config::{find_config_file, load_config_from_file};
 *
 * // Find configuration file
 * if let Some(config_path) = find_config_file() {
 *     println!("Using config: {}", config_path.display());
 * }
 *
 * // Load configuration arguments
 * let config_args = load_config_from_file(true); // verbose=true
 * println!("Loaded {} arguments from config", config_args.len());
 * ```
 *
 * # Thread Safety
 *
 * All functions in this module are thread-safe as they only read from the filesystem
 * and environment variables. However, if configuration files are modified during
 * program execution, behavior is undefined.
 */

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns a list of potential configuration file paths in search priority order.
///
/// This function generates the complete list of paths where mirage looks for
/// configuration files, following XDG Base Directory Specification. The paths
/// are returned in search order - the first existing file will be used.
///
/// # Search Priority
///
/// 1. **`$XDG_CONFIG_HOME/mirage/config`** - User's preferred config directory
/// 2. **`$HOME/.config/mirage/config`** - Standard XDG config location  
/// 3. **`$HOME/.mirage/config`** - Legacy config location for compatibility
///
/// # Environment Dependencies
///
/// - **`XDG_CONFIG_HOME`**: Optional XDG environment variable for config directory
/// - **`HOME`**: Required POSIX environment variable for user's home directory
///
/// If environment variables are not set, those paths are simply omitted from
/// the returned list (no errors are generated).
///
/// # Returns
///
/// A `Vec<PathBuf>` containing all potential config file paths in priority order.
/// The vector may be empty if no environment variables are available, but this
/// is unlikely on standard systems.
///
/// # Examples
///
/// ```rust
/// let paths = get_config_file_paths();
/// println!("Will search for config files in:");
/// for (i, path) in paths.iter().enumerate() {
///     println!("  {}. {}", i + 1, path.display());
/// }
/// ```
///
/// # Typical Output
///
/// On a standard Linux system with `HOME=/home/user`:
/// - `/home/user/.config/mirage/config`
/// - `/home/user/.mirage/config`
///
/// With `XDG_CONFIG_HOME=/custom/config`:
/// - `/custom/config/mirage/config`
/// - `/home/user/.config/mirage/config`
/// - `/home/user/.mirage/config`
///
/// # XDG Compliance
///
/// This implementation follows the XDG Base Directory Specification v0.8,
/// ensuring compatibility with other XDG-compliant applications and desktop
/// environments that respect these standards.
///
/// # Performance
///
/// This function only constructs paths and does not perform filesystem operations.
/// It's safe to call repeatedly, though the returned paths should be cached if
/// used multiple times.
#[must_use]
pub fn get_config_file_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Priority 1: XDG_CONFIG_HOME/mirage/config (user's preferred config directory)
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(xdg_config_home);
        path.push("mirage");
        path.push("config");
        paths.push(path);
    }

    // Priority 2: $HOME/.config/mirage/config (standard XDG fallback)
    if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("mirage");
        path.push("config");
        paths.push(path);
    }

    // Priority 3: $HOME/.mirage/config (legacy location for backward compatibility)
    if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".mirage");
        path.push("config");
        paths.push(path);
    }

    paths
}

/// Searches for and returns the path to the first existing configuration file.
///
/// This function searches through the standard configuration file locations
/// (as returned by [`get_config_file_paths()`]) and returns the first file that
/// both exists and is a regular file (not a directory or special file).
///
/// # Search Process
///
/// 1. Gets list of potential config paths in priority order
/// 2. Checks each path in sequence for existence and file type
/// 3. Returns the first path that passes both checks
/// 4. Returns `None` if no valid config file is found
///
/// # Return Values
///
/// - `Some(PathBuf)`: First existing and readable configuration file
/// - `None`: No configuration file found in any of the standard locations
///
/// # File Requirements
///
/// For a file to be returned, it must:
/// - **Exist**: Be present in the filesystem
/// - **Be a regular file**: Not a directory, symlink to directory, or special file
/// - **Be accessible**: Readable by the current process (checked during parsing)
///
/// # Examples
///
/// ```rust
/// match find_config_file() {
///     Some(path) => {
///         println!("Found config file: {}", path.display());
///         // Continue with loading configuration
///     }
///     None => {
///         println!("No config file found, using defaults");
///         // Continue with default configuration
///     }
/// }
/// ```
///
/// # Performance
///
/// This function performs filesystem operations (existence and type checks) for
/// each potential path until a valid file is found. In the common case where
/// a config file exists in the first location, only one filesystem check is performed.
///
/// # Error Handling
///
/// This function does not return errors for filesystem access issues - it simply
/// treats inaccessible paths as non-existent. Parse errors are handled later
/// during file loading by [`parse_config_file()`].
///
/// # Typical Behavior
///
/// - **First run**: Returns `None` (no config file exists yet)
/// - **After creating config**: Returns the path to the user's config file
/// - **Multiple configs**: Returns only the highest-priority existing config
///
/// # Integration
///
/// This function is typically used by [`load_config_from_file()`] to locate
/// the configuration file before attempting to parse it.
#[must_use]
pub fn find_config_file() -> Option<PathBuf> {
    get_config_file_paths()
        .into_iter()
        .find(|path| path.exists() && path.is_file())
}

/// Parses a single configuration line into command-line arguments.
///
/// This function handles the parsing of individual lines from configuration files,
/// processing quotes, whitespace, and multiple arguments. It implements a simple
/// shell-like parsing that's compatible with command-line argument syntax.
///
/// # Parsing Rules
///
/// - **Whitespace**: Spaces and tabs separate arguments
/// - **Quotes**: Double quotes preserve spaces within arguments
/// - **Multiple whitespace**: Consecutive spaces/tabs are treated as single separators
/// - **Empty arguments**: Not generated (e.g., `"arg1  arg2"` → `["arg1", "arg2"]`)
/// - **Quote handling**: Quotes are consumed and not included in output arguments
///
/// # Arguments
///
/// - `line`: The configuration line to parse (after comment/empty line filtering)
///
/// # Returns
///
/// - `Ok(Vec<String>)`: Successfully parsed arguments from the line
/// - `Err(String)`: Parse error with descriptive message
///
/// # Errors
///
/// This function returns errors for:
/// - **Unclosed quotes**: Line contains an opening quote without matching closing quote
///
/// # Examples
///
/// ```rust
/// // Simple arguments
/// let args = parse_config_line("--country Germany --protocol https")?;
/// assert_eq!(args, vec!["--country", "Germany", "--protocol", "https"]);
///
/// // Quoted arguments with spaces
/// let args = parse_config_line("--country \"United States\" --verbose")?;
/// assert_eq!(args, vec!["--country", "United States", "--verbose"]);
///
/// // Mixed whitespace
/// let args = parse_config_line("  --age   12   --fastest  5  ")?;
/// assert_eq!(args, vec!["--age", "12", "--fastest", "5"]);
///
/// // Error case
/// let result = parse_config_line("--country \"United States");
/// assert!(result.is_err());
/// ```
///
/// # Parsing Algorithm
///
/// 1. **Character iteration**: Process each character in sequence
/// 2. **Quote tracking**: Toggle quote state on `"` characters
/// 3. **Whitespace handling**: Split arguments on unquoted whitespace
/// 4. **Argument accumulation**: Build current argument character by character
/// 5. **Whitespace consolidation**: Skip multiple consecutive separators
/// 6. **Validation**: Ensure all quotes are properly closed
///
/// # Limitations
///
/// - **No escape sequences**: Cannot include literal quotes within quoted strings
/// - **Double quotes only**: Single quotes are treated as regular characters
/// - **No nested quotes**: Cannot have quotes within quoted strings
/// - **No empty quoted strings**: `""` results in no argument (not empty string)
///
/// # Performance
///
/// This function processes characters sequentially with minimal allocation.
/// Memory usage scales linearly with the number and length of arguments.
///
/// # Integration
///
/// This function is used by [`parse_config_file()`] to process each non-comment,
/// non-empty line from configuration files.
pub fn parse_config_line(line: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Toggle quote state - quotes are consumed, not included in output
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                // Unquoted whitespace separates arguments
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }

                // Skip multiple consecutive whitespace characters
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ' ' || next_ch == '\t' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => {
                // Regular character - add to current argument
                current_arg.push(ch);
            }
        }
    }

    // Check for unclosed quotes
    if in_quotes {
        return Err("Unclosed quote in config line".to_string());
    }

    // Add final argument if present
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    Ok(args)
}

/// Parses a complete configuration file and returns all arguments.
///
/// This function reads and parses a configuration file, processing each line
/// to extract command-line arguments while handling comments, empty lines,
/// and proper quote parsing. The result is a flat vector of arguments that
/// can be passed to a command-line argument parser.
///
/// # File Processing
///
/// 1. **Read file**: Load entire file content into memory
/// 2. **Line iteration**: Process each line individually
/// 3. **Filtering**: Skip empty lines and comment lines (starting with `#`)
/// 4. **Parsing**: Parse remaining lines using [`parse_config_line()`]
/// 5. **Accumulation**: Collect all arguments into a single vector
///
/// # Arguments
///
/// - `path`: Path to the configuration file to parse
///
/// # Returns
///
/// - `Ok(Vec<String>)`: All arguments extracted from the configuration file
/// - `Err(Box<dyn Error>)`: File read error or parse error with details
///
/// # Errors
///
/// This function returns errors for:
/// - **File I/O errors**: Cannot read file (missing, permissions, etc.)
/// - **Parse errors**: Invalid syntax in configuration file (e.g., unclosed quotes)
/// - **Encoding errors**: File contains invalid UTF-8 sequences
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
///
/// let config_path = Path::new("/home/user/.config/mirage/config");
/// match parse_config_file(config_path) {
///     Ok(args) => {
///         println!("Loaded {} arguments from config", args.len());
///         // args might be: ["--country", "Germany", "--protocol", "https", "--verbose"]
///     }
///     Err(e) => {
///         eprintln!("Failed to parse config file: {}", e);
///     }
/// }
/// ```
///
/// # Example Configuration File
///
/// ```text
/// # Mirage configuration file
/// # Select mirrors from multiple countries
/// --country Germany
/// --country France
/// --country "United States"
///
/// # Protocol and quality settings  
/// --protocol https
/// --age 12
/// --completion-percent 95
///
/// # Output preferences
/// --fastest 10
/// --verbose
/// ```
///
/// This would result in the argument vector:
/// ```rust
/// vec![
///     "--country", "Germany",
///     "--country", "France",
///     "--country", "United States",
///     "--protocol", "https",
///     "--age", "12",
///     "--completion-percent", "95",
///     "--fastest", "10",
///     "--verbose"
/// ]
/// ```
///
/// # Line Processing Rules
///
/// - **Comments**: Lines starting with `#` (after trimming) are ignored
/// - **Empty lines**: Blank lines or whitespace-only lines are ignored  
/// - **Trimming**: Leading and trailing whitespace is removed from each line
/// - **Multi-line**: Arguments from all lines are combined into a single vector
/// - **Order preservation**: Arguments maintain their order within and across lines
///
/// # Performance
///
/// This function loads the entire file into memory, so it's not suitable for
/// very large configuration files. However, typical config files are small
/// and this approach simplifies error handling and parsing.
///
/// # Integration
///
/// This function is used by [`load_config_from_file()`] to parse located
/// configuration files. The resulting argument vector is typically passed
/// to clap for validation and processing.
pub fn parse_config_file(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Read the entire configuration file into memory
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file '{}': {}", path.display(), e))?;

    let mut args = Vec::new();

    // Process each line in the configuration file
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse line into arguments, handling quotes and whitespace
        let line_args = parse_config_line(line)
            .map_err(|e| format!("Parse error on line {}: {}", line_num + 1, e))?;

        // Add all arguments from this line to the result
        args.extend(line_args);
    }

    Ok(args)
}

/// Loads configuration arguments from the first available configuration file.
///
/// This is the main entry point for configuration file loading in mirage. It
/// combines file discovery, parsing, and error handling into a single function
/// that can be safely called even when no configuration file exists.
///
/// # Process Overview
///
/// 1. **Discovery**: Search for configuration files using [`find_config_file()`]
/// 2. **Parsing**: Parse the found file using [`parse_config_file()`]
/// 3. **Error handling**: Log warnings for parse errors but continue execution
/// 4. **Fallback**: Return empty vector if no config file or parse errors occur
///
/// # Arguments
///
/// - `verbose`: Whether to print informational messages about config file loading
///
/// # Returns
///
/// A `Vec<String>` containing all configuration arguments. This vector may be empty if:
/// - No configuration file exists in any standard location
/// - Configuration file exists but contains parse errors
/// - Configuration file exists but is empty (after filtering comments/blank lines)
///
/// # Behavior
///
/// ## Verbose Mode (`verbose = true`)
/// - Prints config file location when successfully loaded
/// - Prints warning messages for parse errors
/// - Helps with debugging configuration issues
///
/// ## Quiet Mode (`verbose = false`)  
/// - Only prints warning messages for parse errors
/// - Suitable for normal program operation
/// - Reduces output clutter
///
/// # Error Handling Philosophy
///
/// This function follows a fail-safe approach:
/// - **Missing config**: Not an error - returns empty vector
/// - **Parse errors**: Warning logged - returns empty vector  
/// - **I/O errors**: Warning logged - returns empty vector
///
/// This ensures the program can always continue execution even with
/// configuration problems, falling back to command-line arguments and defaults.
///
/// # Examples
///
/// ```rust
/// // Load config with verbose output
/// let config_args = load_config_from_file(true);
/// if config_args.is_empty() {
///     println!("No configuration loaded, using defaults");
/// } else {
///     println!("Loaded {} arguments from config", config_args.len());
/// }
///
/// // Load config quietly (production use)
/// let config_args = load_config_from_file(false);
/// // Continue with CLI argument processing...
/// ```
///
/// # Integration with CLI Parsing
///
/// The returned argument vector is designed to be merged with CLI arguments:
///
/// ```rust
/// let config_args = load_config_from_file(verbose);
///
/// // In CLI parsing (pseudo-code):
/// let config_matches = if !config_args.is_empty() {
///     let mut args_with_program = vec!["mirage".to_string()];
///     args_with_program.extend(config_args);
///     Some(cli_parser.get_matches_from(args_with_program))
/// } else {
///     None
/// };
/// ```
///
/// # File Location Logging
///
/// When `verbose = true` and a config file is successfully loaded, the output shows
/// which specific file was used:
/// ```text
/// Loaded config from: /home/user/.config/mirage/config
/// ```
///
/// This helps users understand which configuration file is active when multiple
/// potential locations exist.
///
/// # Warning Messages
///
/// Parse errors always produce warning messages (regardless of verbose setting):
/// ```text
/// Warning: Failed to parse config file /home/user/.config/mirage/config: Parse error on line 5: Unclosed quote in config line
/// ```
///
/// This ensures users are aware of configuration problems that might affect program behavior.
#[must_use]
pub fn load_config_from_file(verbose: bool) -> Vec<String> {
    // Search for configuration file in standard locations
    if let Some(config_path) = find_config_file() {
        // Attempt to parse the found configuration file
        match parse_config_file(&config_path) {
            Ok(args) => {
                // Successfully parsed - optionally report location
                if verbose {
                    eprintln!("Loaded config from: {}", config_path.display());
                }
                args
            }
            Err(e) => {
                // Parse error - always warn user but continue execution
                eprintln!(
                    "Warning: Failed to parse config file {}: {}",
                    config_path.display(),
                    e
                );
                Vec::new()
            }
        }
    } else {
        // No configuration file found - return empty vector (not an error)
        Vec::new()
    }
}
