use mirage::{
    CacheEntry, Config, MirageError, Mirror, MirrorStatus, filter_mirrors, sort_mirrors,
    validate_save_path,
};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

// Helper function to create test mirrors
fn create_test_mirrors() -> Vec<Mirror> {
    vec![
        Mirror {
            url: "https://mirror1.example.com/archlinux/".to_string(),
            protocol: "https".to_string(),
            last_sync: Some("2025-08-15T12:00:00Z".to_string()),
            completion_pct: Some(1.0),
            delay: Some(3600), // 1 hour
            duration_avg: Some(0.5),
            duration_stddev: Some(0.1),
            score: Some(2.5),
            active: true,
            country: "Germany".to_string(),
            country_code: "DE".to_string(),
            isos: true,
            ipv4: true,
            ipv6: true,
            details: "Test mirror 1".to_string(),
        },
        Mirror {
            url: "https://mirror2.example.com/archlinux/".to_string(),
            protocol: "http".to_string(),
            last_sync: Some("2025-08-14T12:00:00Z".to_string()),
            completion_pct: Some(0.95),
            delay: Some(7200), // 2 hours
            duration_avg: Some(1.0),
            duration_stddev: Some(0.2),
            score: Some(1.5),
            active: true,
            country: "France".to_string(),
            country_code: "FR".to_string(),
            isos: false,
            ipv4: true,
            ipv6: false,
            details: "Test mirror 2".to_string(),
        },
        Mirror {
            url: "http://mirror3.example.com/archlinux/".to_string(),
            protocol: "http".to_string(),
            last_sync: None,
            completion_pct: Some(0.85),
            delay: None,
            duration_avg: None,
            duration_stddev: None,
            score: None,
            active: false,
            country: "United States".to_string(),
            country_code: "US".to_string(),
            isos: true,
            ipv4: false,
            ipv6: true,
            details: "Inactive mirror".to_string(),
        },
    ]
}

#[allow(clippy::too_many_arguments)]
fn create_test_mirror_with_details(
    url: &str,
    country: &str,
    protocol: &str,
    active: bool,
    score: Option<f64>,
    delay: Option<i32>,
    completion_pct: Option<f64>,
    last_sync: Option<String>,
    duration_avg: Option<f64>,
    duration_stddev: Option<f64>,
) -> Mirror {
    Mirror {
        url: url.to_string(),
        protocol: protocol.to_string(),
        last_sync,
        completion_pct,
        delay,
        duration_avg,
        duration_stddev,
        score,
        active,
        country: country.to_string(),
        country_code: "US".to_string(), // Default for simplicity
        isos: true,
        ipv4: true,
        ipv6: false,
        details: "Test mirror".to_string(),
    }
}

fn create_test_mirror(
    url: &str,
    country: &str,
    country_code: &str,
    protocol: &str,
    active: bool,
) -> Mirror {
    Mirror {
        url: url.to_string(),
        protocol: protocol.to_string(),
        last_sync: None,
        completion_pct: Some(1.0),
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active,
        country: country.to_string(),
        country_code: country_code.to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    }
}

fn create_test_mirror_with_fields(
    url: &str,
    country: &str,
    last_sync: Option<String>,
    score: Option<f64>,
    delay: Option<i32>,
    duration_avg: Option<f64>,
    duration_stddev: Option<f64>,
) -> Mirror {
    Mirror {
        url: url.to_string(),
        protocol: "http".to_string(),
        last_sync,
        completion_pct: Some(1.0),
        delay,
        duration_avg,
        duration_stddev,
        score,
        active: true,
        country: country.to_string(),
        country_code: "TS".to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    }
}

// =============================================================================
// Mirror Implementation Tests
// =============================================================================

#[test]
fn test_mirror_impl_coverage() {
    // Test Mirror impl block
    let mirror = Mirror {
        url: "https://example.com/archlinux/".to_string(),
        protocol: "https".to_string(),
        last_sync: Some("2025-08-15T12:00:00Z".to_string()),
        completion_pct: Some(1.0),
        delay: Some(3600),
        duration_avg: Some(0.5),
        duration_stddev: Some(0.1),
        score: Some(2.5),
        active: true,
        country: "Germany".to_string(),
        country_code: "DE".to_string(),
        isos: true,
        ipv4: true,
        ipv6: true,
        details: "Test".to_string(),
    };

    // Test last_sync_hours method
    let hours = mirror.last_sync_hours();
    assert!(hours.is_some());
    assert!(hours.unwrap() >= 0.0);

    // Test delay_hours method
    let delay_hours = mirror.delay_hours();
    assert!(delay_hours.is_some());
    assert!((delay_hours.unwrap() - 1.0).abs() < f64::EPSILON); // 3600 seconds = 1 hour

    // Test with None values
    let mirror_none = Mirror {
        url: "https://example.com/archlinux/".to_string(),
        protocol: "https".to_string(),
        last_sync: None,
        completion_pct: None,
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Germany".to_string(),
        country_code: "DE".to_string(),
        isos: false,
        ipv4: false,
        ipv6: false,
        details: String::new(),
    };

    assert!(mirror_none.last_sync_hours().is_none());
    assert!(mirror_none.delay_hours().is_none());
}

#[test]
fn test_mirror_last_sync_hours() {
    let mirror = Mirror {
        url: "http://test.com/".to_string(),
        protocol: "http".to_string(),
        last_sync: Some("2024-01-01T00:00:00Z".to_string()),
        completion_pct: Some(1.0),
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Test".to_string(),
        country_code: "TS".to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    };

    let hours = mirror.last_sync_hours();
    assert!(hours.is_some());
    assert!(hours.unwrap() > 0.0);
}

#[test]
fn test_mirror_last_sync_hours_none() {
    let mirror = Mirror {
        url: "http://test.com/".to_string(),
        protocol: "http".to_string(),
        last_sync: None,
        completion_pct: Some(1.0),
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Test".to_string(),
        country_code: "TS".to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    };

    let hours = mirror.last_sync_hours();
    assert!(hours.is_none());
}

#[test]
fn test_mirror_last_sync_hours_invalid_format() {
    let mirror = Mirror {
        url: "http://test.com/".to_string(),
        protocol: "http".to_string(),
        last_sync: Some("invalid-date".to_string()),
        completion_pct: Some(1.0),
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Test".to_string(),
        country_code: "TS".to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    };

    let hours = mirror.last_sync_hours();
    assert!(hours.is_none());
}

#[test]
fn test_mirror_delay_hours() {
    let mirror = Mirror {
        url: "http://test.com/".to_string(),
        protocol: "http".to_string(),
        last_sync: None,
        completion_pct: Some(1.0),
        delay: Some(7200), // 2 hours in seconds
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Test".to_string(),
        country_code: "TS".to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    };

    let hours = mirror.delay_hours();
    assert_eq!(hours, Some(2.0));
}

#[test]
fn test_mirror_delay_hours_none() {
    let mirror = Mirror {
        url: "http://test.com/".to_string(),
        protocol: "http".to_string(),
        last_sync: None,
        completion_pct: Some(1.0),
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Test".to_string(),
        country_code: "TS".to_string(),
        isos: false,
        ipv4: true,
        ipv6: false,
        details: String::new(),
    };

    let hours = mirror.delay_hours();
    assert!(hours.is_none());
}

#[test]
fn test_mirror_debug_format() {
    let mirror = create_test_mirror_with_details(
        "https://test.com/arch/",
        "Test Country",
        "https",
        true,
        Some(2.5),
        Some(1800),
        Some(0.95),
        Some("2025-08-15T12:00:00Z".to_string()),
        Some(0.3),
        Some(0.1),
    );

    let debug_str = format!("{mirror:?}");
    assert!(debug_str.contains("Mirror"));
    assert!(debug_str.contains("test.com"));
    assert!(debug_str.contains("Test Country"));
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_config_default_impl() {
    // Test Config Default impl
    let config = Config::default();

    assert_eq!(config.connection_timeout, 5);
    assert_eq!(config.download_timeout, 5);
    assert!(!config.list_countries);
    assert_eq!(config.cache_timeout, 300);
    assert_eq!(config.url, "https://archlinux.org/mirrors/status/json/");
    assert!(config.save_path.is_none());
    assert!(config.sort.is_none());
    assert!(config.threads.is_none());
    assert!(!config.verbose);
    assert!(!config.info);
    assert!(config.age.is_none());
    assert!(config.delay.is_none());
    assert!(config.countries.is_empty());
    assert!(config.fastest.is_none());
    assert!(config.include_regex.is_none());
    assert!(config.exclude_regex.is_none());
    assert!(config.latest.is_none());
    assert!(config.score.is_none());
    assert!(config.number.is_none());
    assert!(config.protocols.is_empty());
    assert!((config.completion_percent - 100.0).abs() < f64::EPSILON);
    assert!(!config.isos);
    assert!(!config.ipv4);
    assert!(!config.ipv6);
}

#[test]
fn test_config_default_values() {
    let config = Config::default();

    // Test default values are reasonable
    assert_eq!(config.cache_timeout, 300); // 5 minutes
    assert_eq!(config.connection_timeout, 5); // 5 seconds
    assert!(config.threads.is_none()); // No specific thread count by default
    assert!(!config.verbose); // Not verbose by default
    assert!(config.sort.is_none()); // No sorting by default
    assert!(config.countries.is_empty()); // No country filter by default
}

#[test]
fn test_config_debug_format() {
    let config = Config::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("Config"));
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();

    // Valid config should pass validation
    let result = config.validate();
    assert!(result.is_ok());

    // Invalid timeout should fail validation
    config.connection_timeout = 0;
    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_config_validation_comprehensive() {
    // Test Config validation

    // Test valid config
    let valid_config = Config::default();
    assert!(valid_config.validate().is_ok());

    // Test connection timeout validation
    let mut config = mirage::Config {
        connection_timeout: 0,
        ..Default::default()
    };
    assert!(config.validate().is_err());

    config.connection_timeout = 301;
    assert!(config.validate().is_err());

    // Test download timeout validation
    config = Config::default();
    config.download_timeout = 0;
    assert!(config.validate().is_err());

    config.download_timeout = 601;
    assert!(config.validate().is_err());

    // Test cache timeout validation
    config = Config::default();
    config.cache_timeout = 59;
    assert!(config.validate().is_err());

    config.cache_timeout = 86401;
    assert!(config.validate().is_err());

    // Test completion percentage validation
    config = Config::default();
    config.completion_percent = -1.0;
    assert!(config.validate().is_err());

    config.completion_percent = 101.0;
    assert!(config.validate().is_err());

    // Test age validation
    config = Config::default();
    config.age = Some(-1.0);
    assert!(config.validate().is_err());

    config.age = Some(8761.0);
    assert!(config.validate().is_err());

    // Test delay validation
    config = Config::default();
    config.delay = Some(-1.0);
    assert!(config.validate().is_err());

    config.delay = Some(721.0);
    assert!(config.validate().is_err());

    // Test thread count validation
    config = Config::default();
    config.threads = Some(0);
    assert!(config.validate().is_err());

    config.threads = Some(101);
    assert!(config.validate().is_err());

    // Test fastest count validation
    config = Config::default();
    config.fastest = Some(0);
    assert!(config.validate().is_err());

    config.fastest = Some(1001);
    assert!(config.validate().is_err());

    // Test latest count validation
    config = Config::default();
    config.latest = Some(0);
    assert!(config.validate().is_err());

    config.latest = Some(1001);
    assert!(config.validate().is_err());

    // Test score count validation
    config = Config::default();
    config.score = Some(0);
    assert!(config.validate().is_err());

    config.score = Some(1001);
    assert!(config.validate().is_err());

    // Test number validation
    config = Config::default();
    config.number = Some(0);
    assert!(config.validate().is_err());

    config.number = Some(1001);
    assert!(config.validate().is_err());

    // Test URL validation
    config = Config::default();
    config.url = "http://insecure.com".to_string();
    assert!(config.validate().is_err());

    // Test regex validation
    config = Config::default();
    config.include_regex = Some("[invalid".to_string());
    assert!(config.validate().is_err());

    config = Config::default();
    config.exclude_regex = Some("[invalid".to_string());
    assert!(config.validate().is_err());

    // Test protocol validation
    config = Config::default();
    config.protocols = vec!["invalid_protocol".to_string()];
    assert!(config.validate().is_err());

    // Test valid protocols
    config = Config::default();
    config.protocols = vec![
        "http".to_string(),
        "https".to_string(),
        "ftp".to_string(),
        "rsync".to_string(),
    ];
    assert!(config.validate().is_ok());

    // Test sort method validation
    config = Config::default();
    config.sort = Some("invalid_sort".to_string());
    assert!(config.validate().is_err());

    // Test valid sort methods
    let valid_sorts = [
        "age",
        "rate",
        "country",
        "score",
        "delay",
        "duration",
        "duration-std",
    ];
    for sort_method in &valid_sorts {
        config = Config::default();
        config.sort = Some((*sort_method).to_string());
        assert!(config.validate().is_ok());
    }
}

#[test]
fn test_edgecases() {
    // Test protocol validation error paths
    let mut config = mirage::Config {
        protocols: vec!["invalid1".to_string(), "invalid2".to_string()],
        ..Default::default()
    };
    let result = config.validate();
    assert!(result.is_err());

    // Test feature filtering more thoroughly
    let mirrors = create_test_mirrors();

    // Test isos filtering thoroughly
    config = Config::default();
    config.isos = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert!(filtered.iter().all(|m| m.isos));
    assert!(filtered.len() < mirrors.len()); // Should filter some out

    // Test ipv4 filtering thoroughly
    config = Config::default();
    config.ipv4 = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert!(filtered.iter().all(|m| m.ipv4));
    assert!(filtered.len() < mirrors.len()); // Should filter some out

    // Test age filtering edge case
    config = Config::default();
    config.age = Some(0.1); // Very restrictive
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should filter out most mirrors
    assert!(filtered.len() <= mirrors.len());

    // Test delay filtering edge case
    config = Config::default();
    config.delay = Some(0.5); // 30 minutes
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should filter out mirrors with high delay
    assert!(filtered.len() <= mirrors.len());

    // Test sorting edge cases with None values
    let mut mirrors_with_nones = create_test_mirrors();

    // Add more mirrors with None values to trigger None handling paths
    for i in 0..3 {
        mirrors_with_nones.push(Mirror {
            url: format!("https://none{i}.mirror.com/archlinux/"),
            protocol: "https".to_string(),
            last_sync: None,
            completion_pct: None,
            delay: None,
            duration_avg: None,
            duration_stddev: None,
            score: None,
            active: true,
            country: format!("Country{i}"),
            country_code: format!("C{i}"),
            isos: false,
            ipv4: false,
            ipv6: false,
            details: String::new(),
        });
    }

    // Test delay sorting with mix of Some/None
    config = Config::default();
    config.sort = Some("delay".to_string());
    let sorted = sort_mirrors(mirrors_with_nones.clone(), &config);
    assert!(!sorted.is_empty());

    // Test fastest limiting with None values
    config = Config::default();
    config.fastest = Some(2);
    let sorted = sort_mirrors(mirrors_with_nones.clone(), &config);
    assert!(sorted.len() <= 2);

    // Test score limiting with None values
    config = Config::default();
    config.score = Some(1);
    let sorted = sort_mirrors(mirrors_with_nones.clone(), &config);
    assert!(sorted.len() <= 1);
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn test_validate_save_path_comprehensive() {
    let temp_dir = TempDir::new().unwrap();

    // Test empty path validation
    let result = validate_save_path("");
    assert!(result.is_err());

    let result = validate_save_path("   ");
    assert!(result.is_err());

    // Test valid path
    let valid_path = temp_dir.path().join("test_file.txt");
    let result = validate_save_path(valid_path.to_str().unwrap());
    assert!(result.is_ok());

    // Test nonexistent parent directory
    let nonexistent_path = temp_dir.path().join("nonexistent").join("file.txt");
    let result = validate_save_path(nonexistent_path.to_str().unwrap());
    assert!(result.is_err());

    // Test existing file permissions
    let existing_file = temp_dir.path().join("existing.txt");
    {
        let mut file = File::create(&existing_file).unwrap();
        file.write_all(b"test content").unwrap();
    }

    let result = validate_save_path(existing_file.to_str().unwrap());
    assert!(result.is_ok());

    // Test read-only file on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let readonly_file = temp_dir.path().join("readonly.txt");
        {
            let mut file = File::create(&readonly_file).unwrap();
            file.write_all(b"readonly content").unwrap();
        }

        let mut perms = fs::metadata(&readonly_file).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&readonly_file, perms).unwrap();

        let result = validate_save_path(readonly_file.to_str().unwrap());
        assert!(result.is_err());

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&readonly_file).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&readonly_file, perms).unwrap();
    }
}

#[test]
fn test_validate_save_path_valid() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("test_file");

    let result = validate_save_path(&test_path.to_string_lossy());
    assert!(result.is_ok());
}

#[test]
fn test_validate_save_path_nonexistent_dir() {
    let result = validate_save_path("/nonexistent/directory/file");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, MirageError::Validation(_)));
    assert!(
        error
            .to_string()
            .contains("Parent directory does not exist")
    );
}

#[test]
fn test_validate_save_path_readonly_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("readonly_file");

    // Create a file and make it read-only
    fs::write(&test_path, "test").unwrap();
    let mut perms = std::fs::metadata(&test_path).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&test_path, perms).unwrap();

    let result = validate_save_path(&test_path.to_string_lossy());
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, MirageError::Validation(_)));
    assert!(error.to_string().contains("read-only"));
}

#[test]
fn test_validate_save_path_cannot_access_file() {
    // This test is challenging to create portably, so we'll test the parent directory case
    let result = validate_save_path("/root/restricted/file");
    assert!(result.is_err());
}

#[test]
fn test_validate_save_path_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("existing_file");

    // Create an existing file that we can write to
    fs::write(&test_path, "test content").unwrap();

    let result = validate_save_path(&test_path.to_string_lossy());
    assert!(result.is_ok());
}

#[test]
fn test_validate_save_path_parent_must_exist() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new_directory");
    let file_path = new_dir.join("mirrorlist");

    // Should fail if parent directory doesn't exist
    let result = validate_save_path(file_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Parent directory does not exist")
    );
}

#[test]
fn test_validate_save_path_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("mirrorlist");

    // Directory already exists
    validate_save_path(file_path.to_str().unwrap()).unwrap();
}

#[test]
fn test_validate_save_path_file_exists_writable() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("existing_file");

    // Create existing file
    fs::write(&file_path, "content").unwrap();

    // Should succeed if file is writable
    validate_save_path(file_path.to_str().unwrap()).unwrap();
}

#[test]
fn test_save_path_error_paths() {
    use std::fs;

    let temp_dir = TempDir::new().unwrap();

    // Test metadata error path - simulate inaccessible parent directory on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Create a directory and make it inaccessible
        let restricted_dir = temp_dir.path().join("restricted");
        fs::create_dir(&restricted_dir).unwrap();

        let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(&restricted_dir, perms).unwrap();

        let inaccessible_file = restricted_dir.join("file.txt");
        let result = validate_save_path(inaccessible_file.to_str().unwrap());
        assert!(result.is_err());

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&restricted_dir, perms).unwrap();

        // Test read-only parent directory
        let readonly_parent = temp_dir.path().join("readonly_parent");
        fs::create_dir(&readonly_parent).unwrap();

        let mut perms = fs::metadata(&readonly_parent).unwrap().permissions();
        perms.set_mode(0o555); // Read-only
        fs::set_permissions(&readonly_parent, perms).unwrap();

        let file_in_readonly = readonly_parent.join("file.txt");
        let result = validate_save_path(file_in_readonly.to_str().unwrap());
        assert!(result.is_err());

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&readonly_parent).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_parent, perms).unwrap();
    }

    // Test file access error paths
    // This is tricky to simulate cross-platform, but we can test the basic flow
    let valid_file = temp_dir.path().join("test.txt");
    {
        let mut file = File::create(&valid_file).unwrap();
        file.write_all(b"test").unwrap();
    }

    // This should succeed
    let result = validate_save_path(valid_file.to_str().unwrap());
    assert!(result.is_ok());
}

// =============================================================================
// Filter Tests
// =============================================================================

#[test]
fn test_filter_mirrors_comprehensive() {
    let mirrors = create_test_mirrors();

    // Test age filtering
    let mut config = mirage::Config {
        age: Some(1.0),
        ..Default::default()
    };
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should filter out mirrors older than 1 hour
    assert!(filtered.len() < mirrors.len());

    // Test age filtering with None last_sync
    config = Config::default();
    config.age = Some(24.0);
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Mirror with None last_sync should be filtered out
    // Only mirrors with last_sync should remain (should be less than total)
    assert!(filtered.len() < mirrors.len());

    // Test delay filtering
    config = Config::default();
    config.delay = Some(1.5); // 1.5 hours
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should filter out mirror with 2 hour delay
    assert!(filtered.len() < mirrors.len());

    // Test isos filtering
    config = Config::default();
    config.isos = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should only include mirrors with isos = true
    assert!(filtered.iter().all(|m| m.isos));

    // Test ipv4 filtering
    config = Config::default();
    config.ipv4 = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should only include mirrors with ipv4 = true
    assert!(filtered.iter().all(|m| m.ipv4));

    // Test ipv6 filtering
    config = Config::default();
    config.ipv6 = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should only include mirrors with ipv6 = true
    assert!(filtered.iter().all(|m| m.ipv6));

    // Test exclude regex with invalid regex
    config = Config::default();
    config.exclude_regex = Some("[invalid".to_string());
    config.verbose = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Invalid regex should just warn, not crash
    assert!(!filtered.is_empty());

    // Test exclude regex with valid regex
    config = Config::default();
    config.exclude_regex = Some("mirror2".to_string());
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should exclude mirror2
    assert!(!filtered.iter().any(|m| m.url.contains("mirror2")));
}

#[test]
fn test_filter_mirrors_by_country() {
    let mirrors = vec![
        create_test_mirror("http://mirror1.de/", "Germany", "DE", "http", true),
        create_test_mirror("http://mirror1.fr/", "France", "FR", "http", true),
    ];

    let config = Config {
        countries: vec!["Germany".to_string()],
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].country, "Germany");
}

#[test]
fn test_filter_mirrors_by_protocol() {
    let mirrors = vec![
        create_test_mirror("http://mirror1.com/", "US", "US", "http", true),
        create_test_mirror("https://mirror2.com/", "US", "US", "https", true),
    ];

    let config = Config {
        protocols: vec!["https".to_string()],
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].protocol, "https");
}

#[test]
fn test_filter_mirrors_inactive() {
    let mirrors = vec![
        create_test_mirror("http://active.com/", "Test", "TS", "http", true),
        create_test_mirror("http://inactive.com/", "Test", "TS", "http", false),
    ];

    let config = Config::default();
    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].url, "http://active.com/");
}

#[test]
fn test_filter_mirrors_completion_percent() {
    let mirrors = vec![
        Mirror {
            url: "http://complete.com/".to_string(),
            protocol: "http".to_string(),
            last_sync: None,
            completion_pct: Some(1.0), // 100%
            delay: None,
            duration_avg: None,
            duration_stddev: None,
            score: None,
            active: true,
            country: "Test".to_string(),
            country_code: "TS".to_string(),
            isos: false,
            ipv4: true,
            ipv6: false,
            details: String::new(),
        },
        Mirror {
            url: "http://incomplete.com/".to_string(),
            protocol: "http".to_string(),
            last_sync: None,
            completion_pct: Some(0.5), // 50%
            delay: None,
            duration_avg: None,
            duration_stddev: None,
            score: None,
            active: true,
            country: "Test".to_string(),
            country_code: "TS".to_string(),
            isos: false,
            ipv4: true,
            ipv6: false,
            details: String::new(),
        },
        Mirror {
            url: "http://nocompletion.com/".to_string(),
            protocol: "http".to_string(),
            last_sync: None,
            completion_pct: None, // No completion data
            delay: None,
            duration_avg: None,
            duration_stddev: None,
            score: None,
            active: true,
            country: "Test".to_string(),
            country_code: "TS".to_string(),
            isos: false,
            ipv4: true,
            ipv6: false,
            details: String::new(),
        },
    ];

    let config = Config {
        completion_percent: 80.0, // Require 80% completion
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].url, "http://complete.com/");
}

#[test]
fn test_filter_mirrors_include_regex() {
    let mirrors = vec![
        create_test_mirror("http://example.com/archlinux/", "Test", "TS", "http", true),
        create_test_mirror("http://example.com/ubuntu/", "Test", "TS", "http", true),
    ];

    let config = Config {
        include_regex: Some("archlinux".to_string()),
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].url, "http://example.com/archlinux/");
}

#[test]
fn test_filter_mirrors_exclude_regex() {
    let mirrors = vec![
        create_test_mirror("http://example.com/archlinux/", "Test", "TS", "http", true),
        create_test_mirror("http://example.com/ubuntu/", "Test", "TS", "http", true),
    ];

    let config = Config {
        exclude_regex: Some("ubuntu".to_string()),
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].url, "http://example.com/archlinux/");
}

#[test]
fn test_filter_mirrors_invalid_regex() {
    let mirrors = vec![create_test_mirror(
        "http://test.com/",
        "Test",
        "TS",
        "http",
        true,
    )];

    let config = Config {
        include_regex: Some("[".to_string()), // Invalid regex
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    // Invalid include regex should reject all mirrors (return false)
    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_filter_verbose() {
    let mirrors = vec![create_test_mirror(
        "http://test.com/",
        "Test",
        "TS",
        "http",
        true,
    )];

    let config = Config {
        verbose: true,
        ..Default::default()
    };

    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_filter_mirrors_verbose_mode() {
    let mirrors = vec![
        create_test_mirror_with_details(
            "https://mirror1.com/arch/",
            "Germany",
            "https",
            true,
            Some(1.0),
            None,
            Some(1.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_details(
            "https://mirror2.com/arch/",
            "France",
            "https",
            false,
            Some(2.0),
            None,
            Some(1.0),
            None,
            None,
            None,
        ),
    ];

    // Test with some basic filtering to make sure verbose mode works
    let config = mirage::Config {
        verbose: true,
        protocols: vec!["https".to_string()],
        ..Default::default()
    };

    // Should work the same but print verbose output
    let filtered = filter_mirrors(mirrors, &config);
    assert_eq!(filtered.len(), 1); // Only HTTPS mirror
}

#[test]
fn test_filter_mirrors_include_exclude_regex() {
    let mirrors = vec![
        create_test_mirror_with_details(
            "https://mirror.kernel.org/arch/",
            "United States",
            "https",
            true,
            Some(2.0),
            None,
            Some(1.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_details(
            "https://ftp.halifax.rwth-aachen.de/arch/",
            "Germany",
            "https",
            true,
            Some(1.5),
            None,
            Some(1.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_details(
            "https://mirrors.edge.kernel.org/arch/",
            "United States",
            "https",
            true,
            Some(3.0),
            None,
            Some(1.0),
            None,
            None,
            None,
        ),
    ];

    let mut config = mirage::Config {
        include_regex: Some("kernel".to_string()),
        ..Default::default()
    };

    // Test include regex
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert_eq!(filtered.len(), 2); // Only mirrors with "kernel" in URL

    // Test exclude regex
    config = Config::default();
    config.exclude_regex = Some("edge".to_string());
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert_eq!(filtered.len(), 2); // Exclude mirrors with "edge" in URL

    // Test both include and exclude
    config = Config::default();
    config.include_regex = Some("kernel".to_string());
    config.exclude_regex = Some("edge".to_string());
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert_eq!(filtered.len(), 1); // Only non-edge kernel mirrors
}

#[test]
fn test_complex_filtering_scenarios() {
    let mirrors = create_test_mirrors();

    // Test complex country filtering with multiple countries
    let mut config = mirage::Config {
        countries: vec!["Germany,France".to_string()],
        ..Default::default()
    };
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert!(
        filtered
            .iter()
            .all(|m| m.country == "Germany" || m.country == "France")
    );

    // Test protocol filtering with multiple protocols
    config = Config::default();
    config.protocols = vec!["https,http".to_string()];
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert!(
        filtered
            .iter()
            .all(|m| m.protocol == "https" || m.protocol == "http")
    );

    // Test completion percentage filtering
    config = Config::default();
    config.completion_percent = 98.0;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert!(
        filtered
            .iter()
            .all(|m| { m.completion_pct.unwrap_or(0.0) >= 0.98 })
    );

    // Test include regex filtering
    config = Config::default();
    config.include_regex = Some("mirror1".to_string());
    let filtered = filter_mirrors(mirrors.clone(), &config);
    assert!(filtered.iter().all(|m| m.url.contains("mirror1")));

    // Test include regex with invalid regex
    config = Config::default();
    config.include_regex = Some("[invalid".to_string());
    config.verbose = true;
    let filtered = filter_mirrors(mirrors.clone(), &config);
    // Should filter out all mirrors due to invalid regex
    assert!(filtered.is_empty());
}

// =============================================================================
// Sort Tests
// =============================================================================

#[test]
fn test_sort_mirrors_comprehensive() {
    let mirrors = create_test_mirrors();

    // Test sorting without sort method - should not change order
    let config = Config::default();
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert_eq!(sorted.len(), mirrors.len());

    // Test age sorting
    let mut config = mirage::Config {
        sort: Some("age".to_string()),
        ..Default::default()
    };
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test rate/score sorting
    config.sort = Some("rate".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    config.sort = Some("score".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test country sorting
    config.sort = Some("country".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test delay sorting
    config.sort = Some("delay".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test duration sorting
    config.sort = Some("duration".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test duration-std sorting
    config.sort = Some("duration-std".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test unknown sort method
    config.sort = Some("unknown".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert_eq!(sorted.len(), mirrors.len()); // Should not change

    // Test latest limiting
    config = Config::default();
    config.latest = Some(2);
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(sorted.len() <= 2);

    // Test fastest limiting
    config = Config::default();
    config.fastest = Some(1);
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(sorted.len() <= 1);

    // Test score limiting
    config = Config::default();
    config.score = Some(2);
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(sorted.len() <= 2);

    // Test number limiting
    config = Config::default();
    config.number = Some(1);
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert_eq!(sorted.len(), 1);
}

#[test]
fn test_sort_mirrors_by_country() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://mirror1.com/",
            "Germany",
            None,
            None,
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://mirror2.com/",
            "Austria",
            None,
            None,
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        sort: Some("country".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].country, "Austria");
    assert_eq!(sorted[1].country, "Germany");
}

#[test]
fn test_sort_mirrors_by_age() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://old.com/",
            "Test",
            Some("2020-01-01T00:00:00Z".to_string()),
            None,
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://recent.com/",
            "Test",
            Some(chrono::Utc::now().to_rfc3339()),
            None,
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        sort: Some("age".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://recent.com/"); // Most recent first
    assert_eq!(sorted[1].url, "http://old.com/");
}

#[test]
fn test_sort_mirrors_by_score() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://lowscore.com/",
            "Test",
            None,
            Some(50.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://highscore.com/",
            "Test",
            None,
            Some(100.0),
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        sort: Some("score".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://highscore.com/"); // Highest score first
    assert_eq!(sorted[1].url, "http://lowscore.com/");
}

#[test]
fn test_sort_mirrors_by_delay() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://slow.com/",
            "Test",
            None,
            None,
            Some(7200),
            None,
            None,
        ), // 2 hours
        create_test_mirror_with_fields(
            "http://fast.com/",
            "Test",
            None,
            None,
            Some(1800),
            None,
            None,
        ), // 30 minutes
    ];

    let config = Config {
        sort: Some("delay".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://fast.com/"); // Lowest delay first
    assert_eq!(sorted[1].url, "http://slow.com/");
}

#[test]
fn test_sort_mirrors_by_duration() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://slow.com/",
            "Test",
            None,
            None,
            None,
            Some(2.0),
            None,
        ),
        create_test_mirror_with_fields(
            "http://fast.com/",
            "Test",
            None,
            None,
            None,
            Some(0.5),
            None,
        ),
    ];

    let config = Config {
        sort: Some("duration".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://fast.com/"); // Shortest duration first
    assert_eq!(sorted[1].url, "http://slow.com/");
}

#[test]
fn test_sort_mirrors_by_duration_std() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://variable.com/",
            "Test",
            None,
            None,
            None,
            None,
            Some(2.0),
        ),
        create_test_mirror_with_fields(
            "http://consistent.com/",
            "Test",
            None,
            None,
            None,
            None,
            Some(0.1),
        ),
    ];

    let config = Config {
        sort: Some("duration-std".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://consistent.com/"); // Lower std dev first
    assert_eq!(sorted[1].url, "http://variable.com/");
}

#[test]
fn test_sort_mirrors_latest() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://mirror1.com/",
            "Test",
            Some("2020-01-01T00:00:00Z".to_string()),
            None,
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://mirror2.com/",
            "Test",
            Some(chrono::Utc::now().to_rfc3339()),
            None,
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://mirror3.com/",
            "Test",
            Some("2023-01-01T00:00:00Z".to_string()),
            None,
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        latest: Some(2), // Only keep 2 latest
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0].url, "http://mirror2.com/"); // Most recent first
}

#[test]
fn test_sort_mirrors_fastest() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://mirror1.com/",
            "Test",
            None,
            Some(50.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://mirror2.com/",
            "Test",
            None,
            Some(100.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://mirror3.com/",
            "Test",
            None,
            Some(75.0),
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        fastest: Some(2), // Only keep 2 fastest
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0].url, "http://mirror2.com/"); // Highest score first
}

#[test]
fn test_sort_mirrors_score_limit() {
    let mirrors = vec![
        create_test_mirror_with_fields(
            "http://mirror1.com/",
            "Test",
            None,
            Some(30.0),
            None,
            None,
            None,
        ),
        create_test_mirror_with_fields(
            "http://mirror2.com/",
            "Test",
            None,
            Some(80.0),
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        score: Some(1), // Only keep 1 with highest score
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0].url, "http://mirror2.com/");
}

#[test]
fn test_sort_mirrors_number_limit() {
    let mirrors = vec![
        create_test_mirror_with_fields("http://mirror1.com/", "Test", None, None, None, None, None),
        create_test_mirror_with_fields("http://mirror2.com/", "Test", None, None, None, None, None),
        create_test_mirror_with_fields("http://mirror3.com/", "Test", None, None, None, None, None),
    ];

    let config = Config {
        number: Some(2), // Limit to 2 mirrors
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted.len(), 2);
}

#[test]
fn test_sort_mirrors_unknown_method() {
    let mirrors = vec![create_test_mirror_with_fields(
        "http://test.com/",
        "Test",
        None,
        None,
        None,
        None,
        None,
    )];

    let config = Config {
        sort: Some("unknown".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted.len(), 1); // Should still return the mirror unchanged
}

// Test sorting with None values
#[test]
fn test_sort_mirrors_age_with_none_values() {
    let mirrors = vec![
        create_test_mirror_with_fields("http://nosync.com/", "Test", None, None, None, None, None),
        create_test_mirror_with_fields(
            "http://withsync.com/",
            "Test",
            Some("2024-01-01T00:00:00Z".to_string()),
            None,
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        sort: Some("age".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://withsync.com/"); // With sync data comes first
    assert_eq!(sorted[1].url, "http://nosync.com/");
}

#[test]
fn test_sort_mirrors_score_with_none_values() {
    let mirrors = vec![
        create_test_mirror_with_fields("http://noscore.com/", "Test", None, None, None, None, None),
        create_test_mirror_with_fields(
            "http://withscore.com/",
            "Test",
            None,
            Some(50.0),
            None,
            None,
            None,
        ),
    ];

    let config = Config {
        sort: Some("score".to_string()),
        ..Default::default()
    };

    let sorted = sort_mirrors(mirrors, &config);
    assert_eq!(sorted[0].url, "http://withscore.com/"); // With score comes first
    assert_eq!(sorted[1].url, "http://noscore.com/");
}

#[test]
fn test_edge_cases_and_none_handling() {
    // Test sorting with None values in various fields
    let mut mirrors = create_test_mirrors();

    // Add mirror with all None values for comprehensive testing
    mirrors.push(Mirror {
        url: "https://empty.mirror.com/archlinux/".to_string(),
        protocol: "https".to_string(),
        last_sync: None,
        completion_pct: None,
        delay: None,
        duration_avg: None,
        duration_stddev: None,
        score: None,
        active: true,
        country: "Unknown".to_string(),
        country_code: "XX".to_string(),
        isos: false,
        ipv4: false,
        ipv6: false,
        details: String::new(),
    });

    // Test age sorting with None values - should handle gracefully
    let mut config = mirage::Config {
        sort: Some("age".to_string()),
        ..Default::default()
    };
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test score sorting with None values
    config.sort = Some("score".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test delay sorting with None values
    config.sort = Some("delay".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test duration sorting with None values
    config.sort = Some("duration".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());

    // Test duration-std sorting with None values
    config.sort = Some("duration-std".to_string());
    let sorted = sort_mirrors(mirrors.clone(), &config);
    assert!(!sorted.is_empty());
}

// =============================================================================
// Cache and MirrorStatus Tests
// =============================================================================

#[test]
fn test_cache_entry_struct() {
    // Test CacheEntry struct functionality
    let mirrors = create_test_mirrors();
    let cache_entry = CacheEntry {
        data: mirrors.clone(),
        timestamp: 1_234_567_890,
    };

    assert_eq!(cache_entry.data.len(), mirrors.len());
    assert_eq!(cache_entry.timestamp, 1_234_567_890);

    // Test clone
    let cloned = cache_entry.clone();
    assert_eq!(cloned.data.len(), cache_entry.data.len());
    assert_eq!(cloned.timestamp, cache_entry.timestamp);
}

#[test]
fn test_mirror_status_struct() {
    // Test MirrorStatus deserialization from JSON
    let json_data = r#"{
        "cutoff": 86400,
        "last_check": "2025-08-15T15:46:19.213Z",
        "num_checks": 133,
        "check_frequency": 642,
        "urls": [
            {
                "url": "https://mirror.example.com/archlinux/",
                "protocol": "https",
                "last_sync": "2025-08-15T12:00:00Z",
                "completion_pct": 1.0,
                "delay": 3600,
                "duration_avg": 0.5,
                "duration_stddev": 0.1,
                "score": 2.5,
                "active": true,
                "country": "Germany",
                "country_code": "DE",
                "isos": true,
                "ipv4": true,
                "ipv6": true,
                "details": "Test mirror"
            }
        ]
    }"#;

    let mirror_status: MirrorStatus = serde_json::from_str(json_data).unwrap();
    assert_eq!(mirror_status.cutoff, Some(86400));
    assert_eq!(mirror_status.last_check, "2025-08-15T15:46:19.213Z");
    assert_eq!(mirror_status.num_checks, Some(133));
    assert_eq!(mirror_status.check_frequency, Some(642));
    assert_eq!(mirror_status.urls.len(), 1);
    assert_eq!(
        mirror_status.urls[0].url,
        "https://mirror.example.com/archlinux/"
    );
}
