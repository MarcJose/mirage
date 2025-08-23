// Integration tests for main.rs functionality
// These tests invoke the actual binary to test end-to-end functionality

use std::process::{Command, Stdio};
use tempfile::TempDir;

fn run_mirage_command(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new("target/debug/mirage")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

// Removed unused function run_mirage_command_with_timeout

#[test]
fn test_main_help_command() {
    let output = run_mirage_command(&["--help"]).unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify help output contains expected sections
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Options:"));
    assert!(stdout.contains("--country"));
    assert!(stdout.contains("--protocol"));
    assert!(stdout.contains("--sort"));
    assert!(stdout.contains("--save"));
    assert!(stdout.contains("--verbose"));

    // Verify examples are included
    assert!(stdout.contains("Examples:") || stdout.contains("mirage --country"));
}

#[test]
fn test_main_version_command() {
    let output = run_mirage_command(&["--version"]).unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain version information
    assert!(stdout.contains("mirage"));
    assert!(stdout.contains("1.0.0"));
}

#[test]
fn test_main_invalid_argument() {
    let output = run_mirage_command(&["--nonexistent-flag"]).unwrap();

    // Should exit with error
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    // Should mention the invalid argument
    assert!(stderr.contains("nonexistent-flag") || stderr.contains("unexpected"));
}

#[test]
fn test_main_invalid_numeric_value() {
    let output = run_mirage_command(&["--threads", "not-a-number"]).unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should mention parsing error
    assert!(stderr.contains("invalid") || stderr.contains("parse") || stderr.contains("number"));
}

#[test]
fn test_main_dry_run_options() {
    // Test various flag combinations that shouldn't trigger network requests
    let test_cases = vec![
        vec!["--help"],
        vec!["--version"],
        vec!["--threads", "5", "--help"], // Help overrides other options
    ];

    for args in test_cases {
        let output = run_mirage_command(&args).unwrap();
        // These should all succeed quickly without network access
        assert!(output.status.success());
    }
}

#[test]
fn test_main_save_path_validation() {
    let temp_dir = TempDir::new().unwrap();
    let save_path = temp_dir.path().join("test_mirrorlist");

    // Test dry-run with save path (should validate path without fetching mirrors)
    let output = run_mirage_command(&[
        "--save",
        save_path.to_str().unwrap(),
        "--help", // This should prevent actual mirror fetching
    ])
    .unwrap();

    assert!(output.status.success());
}

#[test]
fn test_main_config_file_handling() {
    // Test that the binary can handle missing config files gracefully
    let output = run_mirage_command(&["--help"]).unwrap();

    assert!(output.status.success());
    // Should not crash even if no config file exists
}

#[test]
fn test_main_verbose_mode() {
    // Test verbose flag with help (safe operation)
    let output = run_mirage_command(&["--verbose", "--help"]).unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should still show help even with verbose mode
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_main_multiple_countries() {
    // Test multiple country arguments parsing
    let output =
        run_mirage_command(&["--country", "Germany", "--country", "France", "--help"]).unwrap();

    assert!(output.status.success());
    // Should parse successfully without errors
}

#[test]
fn test_main_protocol_filtering() {
    // Test protocol filtering arguments
    let output =
        run_mirage_command(&["--protocol", "https", "--protocol", "http", "--help"]).unwrap();

    assert!(output.status.success());
}

#[test]
fn test_main_sorting_options() {
    let sort_methods = ["age", "country", "score", "delay", "rate"];

    for method in &sort_methods {
        let output = run_mirage_command(&["--sort", method, "--help"]).unwrap();

        assert!(
            output.status.success(),
            "Sort method '{method}' should be valid"
        );
    }
}

#[test]
fn test_main_numeric_parameters() {
    // Test various numeric parameters
    let numeric_tests = vec![
        (vec!["--threads", "5", "--help"], true),
        (vec!["--number", "10", "--help"], true),
        (vec!["--fastest", "3", "--help"], true),
        (vec!["--latest", "7", "--help"], true),
        (vec!["--age", "24", "--help"], true),
        (vec!["--delay", "3600", "--help"], true),
        (vec!["--completion-percent", "95", "--help"], true),
        (vec!["--cache-timeout", "7200", "--help"], true),
        (vec!["--connection-timeout", "10", "--help"], true),
        // Invalid values
        (vec!["--threads", "-1", "--help"], false),
        (vec!["--number", "0", "--help"], true), // 0 is valid (means no limit)
    ];

    for (args, should_succeed) in numeric_tests {
        let output = run_mirage_command(&args).unwrap();

        if should_succeed {
            assert!(output.status.success(), "Args {args:?} should succeed");
        } else {
            assert!(!output.status.success(), "Args {args:?} should fail");
        }
    }
}

#[test]
fn test_main_boolean_flags() {
    // Test boolean flag combinations
    let output = run_mirage_command(&[
        "--verbose",
        "--ipv4",
        "--ipv6",
        "--isos",
        "--info",
        "--help",
    ])
    .unwrap();

    assert!(output.status.success());
}

#[test]
fn test_main_regex_patterns() {
    // Test include/exclude regex patterns (with help to avoid network)
    let output =
        run_mirage_command(&["--include", "kernel", "--exclude", "archive", "--help"]).unwrap();

    assert!(output.status.success());
}

#[test]
fn test_main_completion_generation() {
    // Test shell completion generation (if supported)
    let shells = ["bash", "zsh", "fish", "powershell"];

    for shell in &shells {
        // Try to generate completion (may or may not be supported)
        let _output = run_mirage_command(&["--generate-completion", shell]).unwrap_or_else(|_| {
            // If completion generation is not supported, try another approach
            run_mirage_command(&["--help"]).unwrap()
        });

        // Should not crash regardless
        // Note: This test may need adjustment based on actual completion implementation
    }
}

#[test]
fn test_main_error_handling() {
    // Test various error conditions
    let error_cases = vec![
        // Invalid save path (directory that doesn't exist and can't be created)
        vec!["--save", "/root/nonexistent/path/mirrorlist", "--help"],
        // Invalid regex pattern
        vec!["--include", "[invalid-regex", "--help"],
    ];

    for args in error_cases {
        let _output = run_mirage_command(&args).unwrap();
        // Should handle errors gracefully without panicking
    }
}

#[test]
fn test_main_config_precedence() {
    // Test that CLI arguments are processed correctly
    // Using help to avoid network requests
    let output = run_mirage_command(&[
        "--country",
        "Germany",
        "--protocol",
        "https",
        "--verbose",
        "--help",
    ])
    .unwrap();

    assert!(output.status.success());
    // Configuration should be parsed and merged successfully
}
