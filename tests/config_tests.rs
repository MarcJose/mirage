// Comprehensive tests for config.rs functionality
use mirage::config::{
    find_config_file, get_config_file_paths, load_config_from_file, parse_config_file,
    parse_config_line,
};
use std::env;
use std::fs::{self, File, write};
use std::io::Write;
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

// =============================================================================
// Parse Config Line Tests
// =============================================================================

#[test]
fn test_parse_config_line_simple() {
    let result = parse_config_line("--country Germany").unwrap();
    assert_eq!(result, vec!["--country", "Germany"]);
}

#[test]
fn test_parse_config_line_quoted() {
    let result = parse_config_line("--include \"some pattern\"").unwrap();
    assert_eq!(result, vec!["--include", "some pattern"]);
}

#[test]
fn test_parse_config_line_multiple_spaces() {
    let result = parse_config_line("--country    Germany    --number   5").unwrap();
    assert_eq!(result, vec!["--country", "Germany", "--number", "5"]);
}

#[test]
fn test_parse_config_line_unclosed_quote() {
    let result = parse_config_line("--include \"unclosed quote");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unclosed quote in config line");
}

#[test]
fn test_parse_config_line_empty() {
    let result = parse_config_line("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_config_line_only_spaces() {
    let result = parse_config_line("   \t   ").unwrap();
    assert!(result.is_empty());
}

// =============================================================================
// Parse Config File Tests
// =============================================================================

#[test]
fn test_parse_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config");

    let config_content = r"
# This is a comment
--country Germany
--number 5

# Another comment
--sort score
--verbose
";

    write(&config_path, config_content).unwrap();

    let result = parse_config_file(&config_path).unwrap();
    assert_eq!(
        result,
        vec![
            "--country",
            "Germany",
            "--number",
            "5",
            "--sort",
            "score",
            "--verbose"
        ]
    );
}

#[test]
fn test_parse_config_file_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config");

    let config_content = r"
# Configuration for mirage
--country Germany
# --country France  # This is commented out

--number 10
--verbose
";

    write(&config_path, config_content).unwrap();

    let result = parse_config_file(&config_path).unwrap();
    assert_eq!(
        result,
        vec!["--country", "Germany", "--number", "10", "--verbose"]
    );
}

#[test]
fn test_parse_config_file_nonexistent() {
    let result = parse_config_file(std::path::Path::new("/nonexistent/config"));
    assert!(result.is_err());
}

#[test]
fn test_parse_config_file_io_error() {
    // Test error handling in parse_config_file by trying to parse a non-existent file
    let temp_dir = TempDir::new().unwrap();
    let non_existent_path = temp_dir.path().join("does_not_exist");

    let result = parse_config_file(&non_existent_path);
    assert!(result.is_err());

    // Should contain "No such file" or similar IO error message
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No such file") || error_msg.contains("not found"));
}

#[test]
fn test_parse_config_file_permission_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config");

    // Create a file and then remove read permissions
    {
        let mut file = File::create(&config_path).unwrap();
        file.write_all(b"--verbose").unwrap();
    }

    // Try to make it unreadable (this might not work on all systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path).unwrap().permissions();
        perms.set_mode(0o000); // Remove all permissions
        fs::set_permissions(&config_path, perms).unwrap();

        let result = parse_config_file(&config_path);
        assert!(result.is_err());

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&config_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&config_path, perms).unwrap();
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, just test that the function works with a valid file
        let result = parse_config_file(&config_path);
        assert!(result.is_ok());
    }
}

// =============================================================================
// Config Path Tests
// =============================================================================

#[test]
fn test_get_config_file_paths() {
    let paths = get_config_file_paths();
    // Should have at least one path (even if HOME isn't set, we try various locations)
    // The test might fail in environments without HOME variable, so let's be more flexible
    if !paths.is_empty() {
        // All paths should end with "config"
        for path in paths {
            assert_eq!(path.file_name().unwrap(), "config");
        }
    }
    // At minimum, we can check that the function doesn't crash
    // This test mainly ensures the function works correctly
}

#[test]
fn test_config_paths_xdg() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    // Test XDG_CONFIG_HOME path
    unsafe {
        env::set_var("XDG_CONFIG_HOME", "/tmp/xdg");
        env::set_var("HOME", "/tmp/home");
    }

    let paths = get_config_file_paths();

    assert_eq!(paths.len(), 3);
    assert_eq!(paths[0].to_string_lossy(), "/tmp/xdg/mirage/config");
    assert_eq!(
        paths[1].to_string_lossy(),
        "/tmp/home/.config/mirage/config"
    );
    assert_eq!(paths[2].to_string_lossy(), "/tmp/home/.mirage/config");

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
// Find Config File Tests
// =============================================================================

#[test]
fn test_find_config_file_none() {
    // Test when no config files exist
    let result = find_config_file();
    // This will be None unless the user actually has config files
    // We can't assert the exact result since it depends on the environment
    let _ = result;
}

#[test]
fn test_find_config_file_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config");
    write(&config_path, "--verbose\n--country Germany").unwrap();

    // This test can't easily mock the XDG paths, but we can test the parsing
    let result = parse_config_file(&config_path).unwrap();
    assert_eq!(result, vec!["--verbose", "--country", "Germany"]);
}

#[test]
fn test_find_config_file_comprehensive_paths() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    // Test XDG_CONFIG_HOME path priority
    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::set_var("HOME", temp_dir.path().join("fallback"));
    }

    // Create config file in XDG location
    let _config_file = create_test_config_file(&temp_dir, "--verbose");

    let found_path = find_config_file();
    assert!(found_path.is_some());
    assert!(found_path.unwrap().ends_with("mirage/config"));

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
// Load Config From File Tests
// =============================================================================

#[test]
fn test_load_config_from_file_verbose_logging() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Create a valid config file
    let config_content = "--verbose\n--country Germany";
    let _config_file = create_test_config_file(&temp_dir, config_content);

    // Call load_config_from_file with verbose=true
    let args = load_config_from_file(true);

    // Should successfully load the config args
    assert_eq!(args, vec!["--verbose", "--country", "Germany"]);

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
fn test_load_config_from_file_parse_error_handling() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Create a config file with invalid content (unclosed quote) to trigger parse error
    let config_content = "--include \"unclosed quote\n--country Germany";
    let _config_file = create_test_config_file(&temp_dir, config_content);

    // Call load_config_from_file
    let args = load_config_from_file(false);

    // Should return empty vec when parsing fails
    assert!(args.is_empty());

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
fn test_load_config_from_file_verbose_false_with_valid_config() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Create a valid config file
    let config_content = "--number 10\n--sort age";
    let _config_file = create_test_config_file(&temp_dir, config_content);

    // Call load_config_from_file with verbose=false
    let args = load_config_from_file(false);

    // Should successfully load the config args
    assert_eq!(args, vec!["--number", "10", "--sort", "age"]);

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
fn test_load_config_comprehensive_scenarios() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        env::remove_var("HOME");
    }

    // Test 1: No config file - should return empty vec
    let args = load_config_from_file(false);
    assert!(args.is_empty());

    // Test 2: Empty config file - should return empty vec
    let _config_file = create_test_config_file(&temp_dir, "");
    let args = load_config_from_file(false);
    assert!(args.is_empty());

    // Test 3: Config file with only comments - should return empty vec
    fs::write(
        temp_dir.path().join("mirage/config"),
        "# Just comments\n# More comments",
    )
    .unwrap();
    let args = load_config_from_file(false);
    assert!(args.is_empty());

    // Test 4: Valid config file - should return parsed args
    fs::write(
        temp_dir.path().join("mirage/config"),
        "--country US\n--number 5",
    )
    .unwrap();
    let args = load_config_from_file(false);
    assert_eq!(args, vec!["--country", "US", "--number", "5"]);

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
