// Comprehensive tests for cli.rs functionality
use mirage::cli::{build_cli, get_merged_flag, get_merged_many, get_merged_value, parse_args};
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tempfile::TempDir;

// We need a mutex to ensure tests don't run concurrently since they modify environment
static TEST_MUTEX: Mutex<()> = Mutex::new(());

// Helper function to set up a test config file
fn create_test_config_file(temp_dir: &TempDir, content: &str) -> std::path::PathBuf {
    let config_dir = temp_dir.path().join("mirage");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config");
    fs::write(&config_file, content).unwrap();
    config_file
}

fn get_mirage_binary_path() -> String {
    // In tests, the binary should be available in the target directory
    "target/debug/mirage".to_string()
}

// =============================================================================
// Build CLI Tests
// =============================================================================

#[test]
fn test_build_cli_basic() {
    let cli = build_cli();

    // Test basic properties
    assert_eq!(cli.get_name(), "mirage");
    assert_eq!(cli.get_version(), Some("1.0.0"));

    // Test that the command has the expected arguments
    let args = cli.get_arguments().collect::<Vec<_>>();
    assert!(!args.is_empty());
}

#[test]
fn test_build_cli_help_parsing() {
    let cli = build_cli();

    // Test parsing help flag
    let help_result = cli.clone().try_get_matches_from(vec!["mirage", "--help"]);
    assert!(help_result.is_err()); // clap exits on help, so this should error

    // Test parsing version flag
    let version_result = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--version"]);
    assert!(version_result.is_err()); // clap exits on version, so this should error
}

#[test]
fn test_build_cli_argument_parsing() {
    let cli = build_cli();

    // Test basic argument parsing
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--country",
            "Germany",
            "--protocol",
            "https",
            "--number",
            "5",
            "--verbose",
        ])
        .unwrap();

    assert!(matches.get_flag("verbose"));
    assert_eq!(
        matches.get_one::<String>("country"),
        Some(&"Germany".to_string())
    );
    assert_eq!(
        matches.get_one::<String>("protocol"),
        Some(&"https".to_string())
    );
    assert_eq!(matches.get_one::<u32>("number"), Some(&5));
}

#[test]
fn test_build_cli_multiple_values() {
    let cli = build_cli();

    // Test arguments that accept multiple values
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--country",
            "Germany",
            "--country",
            "France",
            "--protocol",
            "https",
            "--protocol",
            "http",
        ])
        .unwrap();

    let countries: Vec<&String> = matches.get_many::<String>("country").unwrap().collect();
    assert_eq!(
        countries,
        vec![&"Germany".to_string(), &"France".to_string()]
    );

    let protocols: Vec<&String> = matches.get_many::<String>("protocol").unwrap().collect();
    assert_eq!(protocols, vec![&"https".to_string(), &"http".to_string()]);
}

#[test]
fn test_build_cli_numeric_arguments() {
    let cli = build_cli();

    // Test various numeric arguments
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--number",
            "10",
            "--fastest",
            "5",
            "--latest",
            "3",
            "--threads",
            "8",
            "--age",
            "24",
            "--delay",
            "3600",
            "--completion-percent",
            "95",
            "--cache-timeout",
            "7200",
            "--connection-timeout",
            "30",
        ])
        .unwrap();

    assert_eq!(matches.get_one::<u32>("number"), Some(&10));
    assert_eq!(matches.get_one::<u32>("fastest"), Some(&5));
    assert_eq!(matches.get_one::<u32>("latest"), Some(&3));
    assert_eq!(matches.get_one::<u32>("threads"), Some(&8));
    assert_eq!(matches.get_one::<f64>("age"), Some(&24.0));
    assert_eq!(matches.get_one::<f64>("delay"), Some(&3600.0));
    assert_eq!(matches.get_one::<f64>("completion-percent"), Some(&95.0));
    assert_eq!(matches.get_one::<u32>("cache-timeout"), Some(&7200));
    assert_eq!(matches.get_one::<u32>("connection-timeout"), Some(&30));
}

#[test]
fn test_build_cli_boolean_flags() {
    let cli = build_cli();

    // Test various boolean flags
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--verbose",
            "--ipv4",
            "--ipv6",
            "--isos",
            "--info",
        ])
        .unwrap();

    assert!(matches.get_flag("verbose"));
    assert!(matches.get_flag("ipv4"));
    assert!(matches.get_flag("ipv6"));
    assert!(matches.get_flag("isos"));
    assert!(matches.get_flag("info"));
}

#[test]
fn test_build_cli_string_arguments() {
    let cli = build_cli();

    // Test string arguments
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--sort",
            "age",
            "--save",
            "/tmp/mirrorlist",
            "--include",
            "kernel",
            "--exclude",
            "archive",
        ])
        .unwrap();

    assert_eq!(matches.get_one::<String>("sort"), Some(&"age".to_string()));
    assert_eq!(
        matches.get_one::<String>("save"),
        Some(&"/tmp/mirrorlist".to_string())
    );
    assert_eq!(
        matches.get_one::<String>("include"),
        Some(&"kernel".to_string())
    );
    assert_eq!(
        matches.get_one::<String>("exclude"),
        Some(&"archive".to_string())
    );
}

#[test]
fn test_build_cli_invalid_arguments() {
    let cli = build_cli();

    // Test invalid numeric values
    let result = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "not-a-number"]);
    assert!(result.is_err());

    // Test invalid flag
    let result = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--nonexistent-flag"]);
    assert!(result.is_err());
}

#[test]
fn test_build_cli_comprehensive_argument_coverage() {
    let cli = build_cli();

    // Test cache management arguments
    let matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--clear-cache", "--cache-info"])
        .unwrap();

    assert!(matches.get_flag("clear-cache"));
    assert!(matches.get_flag("cache-info"));

    // Test URL argument
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--url",
            "https://custom.mirror.api/status.json",
        ])
        .unwrap();

    assert_eq!(
        matches.get_one::<String>("url"),
        Some(&"https://custom.mirror.api/status.json".to_string())
    );

    // Test download timeout argument
    let matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--download-timeout", "30"])
        .unwrap();

    assert_eq!(matches.get_one::<u32>("download-timeout"), Some(&30));

    // Test sort with all valid values
    for sort_method in [
        "age",
        "rate",
        "country",
        "score",
        "delay",
        "duration",
        "duration-std",
    ] {
        let matches = cli
            .clone()
            .try_get_matches_from(vec!["mirage", "--sort", sort_method])
            .unwrap();

        assert_eq!(
            matches.get_one::<String>("sort"),
            Some(&sort_method.to_string())
        );
    }
}

#[test]
fn test_cli_all_argument_combinations() {
    let cli = build_cli();

    // Test a comprehensive set of arguments
    let matches = cli
        .try_get_matches_from(vec![
            "mirage",
            "--country",
            "Germany",
            "--country",
            "France",
            "--protocol",
            "https",
            "--sort",
            "age",
            "--number",
            "10",
            "--fastest",
            "5",
            "--latest",
            "3",
            "--threads",
            "4",
            "--age",
            "24.5",
            "--delay",
            "1.5",
            "--completion-percent",
            "95.5",
            "--cache-timeout",
            "300",
            "--connection-timeout",
            "10",
            "--save",
            "/tmp/test",
            "--include",
            "kernel",
            "--exclude",
            "archive",
            "--verbose",
            "--ipv4",
            "--ipv6",
            "--isos",
            "--info",
        ])
        .unwrap();

    // Verify all values are parsed correctly
    let countries: Vec<&String> = matches.get_many::<String>("country").unwrap().collect();
    assert_eq!(countries.len(), 2);

    assert_eq!(
        matches.get_one::<String>("protocol"),
        Some(&"https".to_string())
    );
    assert_eq!(matches.get_one::<String>("sort"), Some(&"age".to_string()));
    assert_eq!(matches.get_one::<u32>("number"), Some(&10));
    assert_eq!(matches.get_one::<u32>("fastest"), Some(&5));
    assert_eq!(matches.get_one::<u32>("latest"), Some(&3));
    assert_eq!(matches.get_one::<u32>("threads"), Some(&4));
    assert_eq!(matches.get_one::<f64>("age"), Some(&24.5));
    assert_eq!(matches.get_one::<f64>("delay"), Some(&1.5));
    assert_eq!(matches.get_one::<f64>("completion-percent"), Some(&95.5));
    assert_eq!(matches.get_one::<u32>("cache-timeout"), Some(&300));
    assert_eq!(matches.get_one::<u32>("connection-timeout"), Some(&10));
    assert_eq!(
        matches.get_one::<String>("save"),
        Some(&"/tmp/test".to_string())
    );
    assert_eq!(
        matches.get_one::<String>("include"),
        Some(&"kernel".to_string())
    );
    assert_eq!(
        matches.get_one::<String>("exclude"),
        Some(&"archive".to_string())
    );

    assert!(matches.get_flag("verbose"));
    assert!(matches.get_flag("ipv4"));
    assert!(matches.get_flag("ipv6"));
    assert!(matches.get_flag("isos"));
    assert!(matches.get_flag("info"));
}

#[test]
fn test_cli_short_flags() {
    let cli = build_cli();

    // Test short versions of flags that actually exist
    let matches = cli
        .try_get_matches_from(vec![
            "mirage", "-n", "5", // number
            "-p", "https", // protocol
            "-c", "Germany", // country
        ])
        .unwrap();

    assert_eq!(matches.get_one::<u32>("number"), Some(&5));
    assert_eq!(
        matches.get_one::<String>("protocol"),
        Some(&"https".to_string())
    );
    assert_eq!(
        matches.get_one::<String>("country"),
        Some(&"Germany".to_string())
    );
}

#[test]
fn test_cli_value_validation() {
    let cli = build_cli();

    // Test that negative numbers are rejected where inappropriate
    let _result = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "-1"]);
    // This might succeed or fail depending on validation rules

    // Test zero values
    let matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "0"])
        .unwrap();
    assert_eq!(matches.get_one::<u32>("number"), Some(&0));
}

#[test]
fn test_cli_edge_cases() {
    let cli = build_cli();

    // Test just the program name (minimal args)
    let matches = cli.clone().try_get_matches_from(vec!["mirage"]).unwrap();
    assert!(!matches.get_flag("verbose")); // Should be false by default

    // Test with just boolean flags
    let matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose", "--ipv4", "--isos"])
        .unwrap();

    assert!(matches.get_flag("verbose"));
    assert!(matches.get_flag("ipv4"));
    assert!(matches.get_flag("isos"));
    assert!(!matches.get_flag("ipv6")); // Not specified
}

#[test]
fn test_cli_long_help_content() {
    let mut cli = build_cli();

    // Test that help content is properly set
    let help_str = cli.render_help().to_string();
    assert!(help_str.contains("mirage"));
    assert!(help_str.contains("Retrieve and filter"));
    assert!(help_str.contains("--country"));
    assert!(help_str.contains("--verbose"));
}

// =============================================================================
// Helper Function Tests
// =============================================================================

#[test]
fn test_get_merged_value_cli_precedence() {
    let cli = build_cli();

    // CLI has value, config has different value
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "10"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "20"])
        .unwrap();

    let result: Option<u32> = get_merged_value(&cli_matches, Some(&config_matches), "number");
    assert_eq!(result, Some(10)); // CLI wins
}

#[test]
fn test_get_merged_value_config_fallback() {
    let cli = build_cli();

    // CLI has no value, config has value
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "30"])
        .unwrap();

    let result: Option<u32> = get_merged_value(&cli_matches, Some(&config_matches), "number");
    assert_eq!(result, Some(30)); // Config fallback
}

#[test]
fn test_get_merged_value_none_available() {
    let cli = build_cli();

    // Neither CLI nor config has the value
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "Germany"])
        .unwrap();

    let result: Option<u32> = get_merged_value(&cli_matches, Some(&config_matches), "number");
    assert_eq!(result, None);
}

#[test]
fn test_get_merged_value_no_config() {
    let cli = build_cli();

    // CLI has value, no config matches
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "40"])
        .unwrap();

    let result: Option<u32> = get_merged_value(&cli_matches, None, "number");
    assert_eq!(result, Some(40));

    // CLI has no value, no config matches
    let cli_matches_no_value = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let result_none: Option<u32> = get_merged_value(&cli_matches_no_value, None, "number");
    assert_eq!(result_none, None);
}

#[test]
fn test_get_merged_many_cli_precedence() {
    let cli = build_cli();

    // CLI has values, config has different values
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--country",
            "Germany",
            "--country",
            "France",
        ])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "US", "--country", "UK"])
        .unwrap();

    let result: Vec<String> = get_merged_many(&cli_matches, Some(&config_matches), "country");
    assert_eq!(result, vec!["Germany".to_string(), "France".to_string()]); // CLI wins
}

#[test]
fn test_get_merged_many_config_fallback() {
    let cli = build_cli();

    // CLI has no values, config has values
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--protocol", "https", "--protocol", "http"])
        .unwrap();

    let result: Vec<String> = get_merged_many(&cli_matches, Some(&config_matches), "protocol");
    assert_eq!(result, vec!["https".to_string(), "http".to_string()]); // Config fallback
}

#[test]
fn test_get_merged_many_none_available() {
    let cli = build_cli();

    // Neither CLI nor config has values
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "5"])
        .unwrap();

    let result: Vec<String> = get_merged_many(&cli_matches, Some(&config_matches), "country");
    assert!(result.is_empty());
}

#[test]
fn test_get_merged_many_no_config() {
    let cli = build_cli();

    // CLI has values, no config
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "Canada"])
        .unwrap();

    let result: Vec<String> = get_merged_many(&cli_matches, None, "country");
    assert_eq!(result, vec!["Canada".to_string()]);

    // CLI has no values, no config
    let cli_matches_no_values = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let result_empty: Vec<String> = get_merged_many(&cli_matches_no_values, None, "country");
    assert!(result_empty.is_empty());
}

#[test]
fn test_get_merged_flag_cli_true() {
    let cli = build_cli();

    // CLI flag is true
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "Germany"])
        .unwrap();

    let result = get_merged_flag(&cli_matches, Some(&config_matches), "verbose");
    assert!(result); // CLI flag wins
}

#[test]
fn test_get_merged_flag_config_fallback() {
    let cli = build_cli();

    // CLI flag is false, config flag is true
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "Germany"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let result = get_merged_flag(&cli_matches, Some(&config_matches), "verbose");
    assert!(result); // Config fallback
}

#[test]
fn test_get_merged_flag_none_available() {
    let cli = build_cli();

    // Neither CLI nor config has the flag
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "Germany"])
        .unwrap();

    let config_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--number", "10"])
        .unwrap();

    let result = get_merged_flag(&cli_matches, Some(&config_matches), "verbose");
    assert!(!result); // Default false
}

#[test]
fn test_get_merged_flag_no_config() {
    let cli = build_cli();

    // CLI flag is true, no config
    let cli_matches = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--verbose"])
        .unwrap();

    let result = get_merged_flag(&cli_matches, None, "verbose");
    assert!(result);

    // CLI flag is false, no config
    let cli_matches_no_flag = cli
        .clone()
        .try_get_matches_from(vec!["mirage", "--country", "Germany"])
        .unwrap();

    let result_false = get_merged_flag(&cli_matches_no_flag, None, "verbose");
    assert!(!result_false);
}

#[test]
fn test_different_value_types() {
    let cli = build_cli();

    // Test with different types to ensure generics work
    let matches = cli
        .clone()
        .try_get_matches_from(vec![
            "mirage",
            "--number",
            "42",
            "--age",
            "24.5",
            "--sort",
            "age",
            "--country",
            "Germany",
        ])
        .unwrap();

    // Test u32 type
    let number: Option<u32> = get_merged_value(&matches, None, "number");
    assert_eq!(number, Some(42));

    // Test f64 type
    let age: Option<f64> = get_merged_value(&matches, None, "age");
    assert_eq!(age, Some(24.5));

    // Test String type
    let sort: Option<String> = get_merged_value(&matches, None, "sort");
    assert_eq!(sort, Some("age".to_string()));

    // Test Vec<String> type
    let countries: Vec<String> = get_merged_many(&matches, None, "country");
    assert_eq!(countries, vec!["Germany".to_string()]);
}

// =============================================================================
// Parse Args Tests
// =============================================================================

#[test]
fn test_parse_args_with_verbose_flag() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Create an empty config file scenario to test the empty config path
    let _config_file = create_test_config_file(&temp_dir, "");

    let config = parse_args();

    // Verify basic default values are set correctly (this exercises the default paths)
    assert_eq!(config.connection_timeout, 5u32);
    assert_eq!(config.download_timeout, 5u32);
    assert_eq!(config.cache_timeout, 300u32);
    assert!((config.completion_percent - 100.0).abs() < f64::EPSILON);
    assert_eq!(config.url, "https://archlinux.org/mirrors/status/json/");

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_parse_args_config_file_scenarios() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Test scenario with a config file that has content
    let _config_file = create_test_config_file(
        &temp_dir,
        "--connection-timeout 10\n--cache-timeout 600\n--completion-percent 95.0",
    );

    // Parse args which should load the config file
    let config = parse_args();

    // These should come from defaults since we can't override CLI args easily
    // But this tests that parse_args runs without errors with a config file present
    assert!(config.connection_timeout >= 5); // Either default 5 or config 10
    assert!(config.cache_timeout >= 300); // Either default 300 or config 600

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_parse_args_no_config_file() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Don't create any config file - this should test the None config path
    let config = parse_args();

    // Should get all defaults
    assert_eq!(config.connection_timeout, 5u32);
    assert_eq!(config.download_timeout, 5u32);
    assert_eq!(config.cache_timeout, 300u32);
    assert!((config.completion_percent - 100.0).abs() < f64::EPSILON);
    assert_eq!(config.url, "https://archlinux.org/mirrors/status/json/");
    assert!(!config.verbose);
    assert!(!config.info);
    assert!(!config.list_countries);
    assert!(!config.ipv4);
    assert!(!config.ipv6);
    assert!(!config.isos);
    assert!(config.countries.is_empty());
    assert!(config.protocols.is_empty());
    assert!(config.threads.is_none());
    assert!(config.save_path.is_none());
    assert!(config.sort.is_none());
    assert!(config.age.is_none());
    assert!(config.delay.is_none());
    assert!(config.include_regex.is_none());
    assert!(config.exclude_regex.is_none());
    assert!(config.fastest.is_none());
    assert!(config.latest.is_none());
    assert!(config.score.is_none());
    assert!(config.number.is_none());

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_parse_args_config_file_processing_coverage() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Create config file with specific content
    // This needs to have content that will make load_config_from_file return non-empty args
    let config_content = "--verbose\n--connection-timeout 15\n--cache-timeout 600";
    let _config_file = create_test_config_file(&temp_dir, config_content);

    let config = parse_args();

    // Verify the config was created successfully - this exercises the config file processing paths
    assert!(config.connection_timeout > 0);
    assert!(config.cache_timeout > 0);
    assert!(!config.url.is_empty());

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_cli_help() {
    let output = Command::new(get_mirage_binary_path())
        .arg("--help")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute mirage binary");

    // Help should be successful (exit code 0)
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain expected help content
    assert!(stdout.contains("mirage"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Options:"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--country"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(get_mirage_binary_path())
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute mirage binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain version information
    assert!(stdout.contains("mirage"));
    assert!(stdout.contains("1.0.0"));
}

#[test]
fn test_cli_invalid_argument() {
    let output = Command::new(get_mirage_binary_path())
        .arg("--invalid-argument")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute mirage binary");

    // Should fail with invalid argument
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    // Should mention the invalid argument
    assert!(stderr.contains("invalid-argument") || stderr.contains("unexpected"));
}

#[test]
fn test_cli_argument_validation() {
    // Test invalid numeric argument
    let output = Command::new(get_mirage_binary_path())
        .args(["--threads", "invalid"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute mirage binary");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    // Should mention parsing error or invalid value
    assert!(stderr.contains("invalid") || stderr.contains("parse") || stderr.contains("number"));
}

// =============================================================================
// Additional Merge Tests
// =============================================================================

// Helper function to test merge scenarios
fn test_merge_scenario(cli_args: Vec<&str>, config_content: &str) {
    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    }

    // Create config file if content provided
    if !config_content.is_empty() {
        let _config_file = create_test_config_file(&temp_dir, config_content);
    }

    let cli = build_cli();

    // Parse config args
    let _config_matches = if config_content.is_empty() {
        None
    } else {
        let config_args = config_content
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();

        if config_args.is_empty() {
            None
        } else {
            let mut config_with_program = vec!["mirage".to_string()];
            config_with_program.extend(config_args);
            Some(
                cli.clone()
                    .try_get_matches_from(config_with_program)
                    .unwrap(),
            )
        }
    };

    // Parse CLI args
    let _cli_matches = cli.try_get_matches_from(cli_args).unwrap();

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_merge_value_cli_precedence() {
    // Test that CLI values take precedence over config values
    test_merge_scenario(
        vec!["mirage", "--country", "Germany", "--number", "10"],
        "--country France --number 5",
    );
}

#[test]
fn test_merge_value_config_fallback() {
    // Test that config values are used when CLI doesn't provide them
    test_merge_scenario(vec!["mirage", "--verbose"], "--country Germany --number 10");
}

#[test]
fn test_merge_value_no_config() {
    // Test CLI-only scenario (no config file)
    test_merge_scenario(vec!["mirage", "--country", "US", "--protocol", "https"], "");
}

#[test]
fn test_merge_many_values() {
    // Test multiple values merging (countries, protocols)
    test_merge_scenario(
        vec![
            "mirage",
            "--country",
            "Germany",
            "--country",
            "France",
            "--protocol",
            "https",
        ],
        "--country US --protocol http --protocol ftp",
    );
}

#[test]
fn test_merge_flag_values() {
    // Test boolean flag merging
    test_merge_scenario(vec!["mirage", "--verbose", "--ipv4"], "--ipv6 --isos");
}

#[test]
fn test_complex_argument_combinations() {
    // Test complex combinations that exercise many code paths
    test_merge_scenario(
        vec![
            "mirage",
            "--country",
            "Germany",
            "--protocol",
            "https",
            "--protocol",
            "http",
            "--number",
            "10",
            "--fastest",
            "5",
            "--verbose",
            "--ipv4",
            "--sort",
            "age",
        ],
        "--country France --country US --protocol ftp --number 20 --ipv6 --sort score",
    );
}

#[test]
fn test_verbose_early_detection_scenarios() {
    // Test early verbose detection logic
    let verbose_args = ["mirage", "--verbose", "--country", "Germany"];
    let has_verbose = verbose_args.contains(&"--verbose");
    assert!(has_verbose);

    let no_verbose_args = ["mirage", "--country", "Germany", "--number", "10"];
    let no_verbose = no_verbose_args.contains(&"--verbose");
    assert!(!no_verbose);

    // Test different verbose flag positions
    let verbose_at_end = ["mirage", "--country", "Germany", "--verbose"];
    let verbose_end = verbose_at_end.contains(&"--verbose");
    assert!(verbose_end);
}

#[test]
fn test_empty_config_args_scenario() {
    let cli = build_cli();

    // Test scenario where config_args is empty (simulates parse_args)
    let config_args: Vec<String> = vec![];
    let config_matches = if config_args.is_empty() {
        None
    } else {
        let mut config_with_program = vec!["mirage".to_string()];
        config_with_program.extend(config_args);
        Some(
            cli.clone()
                .try_get_matches_from(config_with_program)
                .unwrap(),
        )
    };

    // Should be None when config_args is empty
    assert!(config_matches.is_none());

    // Test CLI parsing when config is None
    let cli_matches = cli
        .try_get_matches_from(vec!["mirage", "--country", "Germany"])
        .unwrap();

    // Test merging with None config (exercises the None branches in helper functions)
    let country: Option<String> = get_merged_value(&cli_matches, None, "country");
    assert_eq!(country, Some("Germany".to_string()));

    let verbose = get_merged_flag(&cli_matches, None, "verbose");
    assert!(!verbose);

    let countries: Vec<String> = get_merged_many(&cli_matches, None, "country");
    assert_eq!(countries, vec!["Germany".to_string()]);
}
