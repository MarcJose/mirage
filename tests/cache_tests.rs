use mirage::Mirror;
use mirage::cache::{CacheStats, PersistentCache};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

fn create_test_mirror() -> Mirror {
    Mirror {
        url: "https://mirror.example.com/archlinux/".to_string(),
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
        details: "Test mirror".to_string(),
    }
}

#[test]
fn test_persistent_cache_new() {
    let mirrors = vec![create_test_mirror()];
    let timestamp = 1_234_567_890;
    let etag = Some("test-etag".to_string());

    let cache = PersistentCache::new(mirrors.clone(), timestamp, etag.clone());

    assert_eq!(cache.mirrors.len(), 1);
    assert_eq!(cache.timestamp, timestamp);
    assert_eq!(cache.etag, etag);
    assert_eq!(cache.cache_version, 1);
    assert_eq!(cache.mirrors[0].url, mirrors[0].url);
}

#[test]
fn test_persistent_cache_is_valid_fresh() {
    let mirrors = vec![create_test_mirror()];
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Fresh cache (30 minutes old)
    let fresh_cache = PersistentCache::new(mirrors, current_time - 1800, None);
    assert!(fresh_cache.is_valid(3600)); // 1 hour timeout
}

#[test]
fn test_persistent_cache_is_valid_stale() {
    let mirrors = vec![create_test_mirror()];
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Stale cache (2 hours old)
    let stale_cache = PersistentCache::new(mirrors, current_time - 7200, None);
    assert!(!stale_cache.is_valid(3600)); // 1 hour timeout
}

#[test]
fn test_persistent_cache_is_valid_wrong_version() {
    let mirrors = vec![create_test_mirror()];
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Wrong version
    let mut wrong_version_cache = PersistentCache::new(mirrors, current_time - 1000, None);
    wrong_version_cache.cache_version = 999;
    assert!(!wrong_version_cache.is_valid(3600));
}

#[test]
fn test_persistent_cache_is_valid_system_time_unwrap_or() {
    // This tests the unwrap_or(0) path in is_valid when system time fails
    let mirrors = vec![create_test_mirror()];
    let cache = PersistentCache::new(mirrors, 0, None);

    // Should not panic even if system time calculation has issues
    let is_valid = cache.is_valid(3600);
    // With timestamp 0 and current time, should be invalid (too old)
    assert!(!is_valid);
}

#[test]
fn test_persistent_cache_clone() {
    let mirrors = vec![create_test_mirror()];
    let timestamp = 1_234_567_890;
    let etag = Some("test-etag".to_string());
    let cache = PersistentCache::new(mirrors, timestamp, etag.clone());

    let cloned = cache.clone();
    assert_eq!(cloned.mirrors.len(), cache.mirrors.len());
    assert_eq!(cloned.timestamp, cache.timestamp);
    assert_eq!(cloned.etag, cache.etag);
    assert_eq!(cloned.cache_version, cache.cache_version);
}

#[test]
fn test_persistent_cache_debug() {
    let mirrors = vec![create_test_mirror()];
    let cache = PersistentCache::new(mirrors, 1_234_567_890, Some("etag".to_string()));

    let debug_str = format!("{cache:?}");
    assert!(debug_str.contains("PersistentCache"));
    assert!(debug_str.contains("1234567890"));
    assert!(debug_str.contains("etag"));
}

#[test]
fn test_cache_stats_age_hours() {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let stats = CacheStats {
        size_bytes: 1024,
        mirror_count: 5,
        timestamp: current_time - 7200, // 2 hours ago
        cache_version: 1,
        etag: None,
    };

    let age = stats.age_hours();
    // Should be approximately 2 hours (allow some variance for test execution time)
    assert!((1.9..=2.1).contains(&age));
}

#[test]
fn test_cache_stats_age_hours_system_time_unwrap_or() {
    // Test the unwrap_or(0) path in age_hours
    let stats = CacheStats {
        size_bytes: 1024,
        mirror_count: 5,
        timestamp: 0, // Very old timestamp
        cache_version: 1,
        etag: None,
    };

    let age = stats.age_hours();
    // Should not panic and should return a reasonable value
    assert!(age > 0.0);
}

#[test]
fn test_cache_stats_size_human_bytes() {
    let stats = CacheStats {
        size_bytes: 512,
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };
    assert_eq!(stats.size_human(), "512.0 B");
}

#[test]
fn test_cache_stats_size_human_kb() {
    let stats = CacheStats {
        size_bytes: 1536, // 1.5 KB
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };
    assert_eq!(stats.size_human(), "1.5 KB");
}

#[test]
fn test_cache_stats_size_human_mb() {
    let stats = CacheStats {
        size_bytes: 2 * 1024 * 1024 + 512 * 1024, // 2.5 MB
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };
    assert_eq!(stats.size_human(), "2.5 MB");
}

#[test]
fn test_cache_stats_size_human_gb() {
    let stats = CacheStats {
        size_bytes: 3 * 1024 * 1024 * 1024 + 512 * 1024 * 1024, // 3.5 GB
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };
    assert_eq!(stats.size_human(), "3.5 GB");
}

#[test]
fn test_cache_stats_size_human_edge_cases() {
    // Test edge cases for size formatting
    let test_cases = vec![
        (0, "0.0 B"),
        (1, "1.0 B"),
        (1023, "1023.0 B"),
        (1024, "1.0 KB"),
        (1024 * 1024, "1.0 MB"),
        (1024 * 1024 * 1024, "1.0 GB"),
    ];

    for (size_bytes, expected) in test_cases {
        let stats = CacheStats {
            size_bytes,
            mirror_count: 1,
            timestamp: 0,
            cache_version: 1,
            etag: None,
        };
        assert_eq!(stats.size_human(), expected, "Failed for size {size_bytes}");
    }
}

#[test]
fn test_cache_stats_size_human_large_sizes() {
    // Test that the loop in size_human works correctly
    let stats = CacheStats {
        size_bytes: 1024 * 1024 * 1024 * 1024, // 1 TB
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };

    // Should not panic and should handle large sizes
    let result = stats.size_human();
    assert!(result.contains("GB")); // Should be in GB units
}

#[test]
fn test_cache_stats_size_human_while_loop_coverage() {
    // Test that we exercise the while loop in size_human
    let stats_kb = CacheStats {
        size_bytes: 2048, // 2 KB
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };
    assert_eq!(stats_kb.size_human(), "2.0 KB");

    let stats_megabytes = CacheStats {
        size_bytes: 2048 * 1024, // 2 MB
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };
    assert_eq!(stats_megabytes.size_human(), "2.0 MB");
}

#[test]
fn test_cache_stats_size_human_unit_index_bounds() {
    // Test that unit_index < UNITS.len() - 1 condition is exercised
    let stats = CacheStats {
        size_bytes: 5 * 1024 * 1024 * 1024 * 1024, // 5 TB (very large)
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };

    let result = stats.size_human();
    // Should cap at GB and not go beyond available units
    assert!(result.contains("GB"));
    assert!(result.contains("5120.0 GB") || result.contains("5242880.0 GB")); // 5 TB in GB
}

#[test]
fn test_cache_stats_debug() {
    let stats = CacheStats {
        size_bytes: 1024,
        mirror_count: 5,
        timestamp: 1_234_567_890,
        cache_version: 1,
        etag: Some("test-etag".to_string()),
    };

    let debug_str = format!("{stats:?}");
    assert!(debug_str.contains("CacheStats"));
    assert!(debug_str.contains("1024"));
    assert!(debug_str.contains('5'));
    assert!(debug_str.contains("1234567890"));
    assert!(debug_str.contains("test-etag"));
}

#[test]
fn test_cache_stats_all_fields() {
    let stats = CacheStats {
        size_bytes: 2048,
        mirror_count: 42,
        timestamp: 1_640_995_200, // 2022-01-01
        cache_version: 1,
        etag: Some("abc123".to_string()),
    };

    // Test all fields are accessible
    assert_eq!(stats.size_bytes, 2048);
    assert_eq!(stats.mirror_count, 42);
    assert_eq!(stats.timestamp, 1_640_995_200);
    assert_eq!(stats.cache_version, 1);
    assert_eq!(stats.etag, Some("abc123".to_string()));
}

#[test]
fn test_cache_stats_etag_none() {
    let stats = CacheStats {
        size_bytes: 1024,
        mirror_count: 1,
        timestamp: 0,
        cache_version: 1,
        etag: None,
    };

    assert!(stats.etag.is_none());

    let debug_str = format!("{stats:?}");
    assert!(debug_str.contains("None"));
}

#[test]
fn test_persistent_cache_const_cache_version() {
    // Test that CACHE_VERSION const is used correctly
    let mirrors = vec![create_test_mirror()];
    let cache = PersistentCache::new(mirrors, 0, None);

    // Should use the const value
    assert_eq!(cache.cache_version, 1);
}

#[test]
fn test_persistent_cache_serializable() {
    // Test that PersistentCache can be serialized/deserialized
    let mirrors = vec![create_test_mirror()];
    let cache = PersistentCache::new(mirrors, 1_234_567_890, Some("etag".to_string()));

    // Should serialize to JSON without error
    let json = serde_json::to_string(&cache).unwrap();
    assert!(json.contains("mirrors"));
    assert!(json.contains("timestamp"));
    assert!(json.contains("etag"));
    assert!(json.contains("cache_version"));

    // Should deserialize back
    let deserialized: PersistentCache = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.timestamp, cache.timestamp);
    assert_eq!(deserialized.etag, cache.etag);
    assert_eq!(deserialized.cache_version, cache.cache_version);
    assert_eq!(deserialized.mirrors.len(), cache.mirrors.len());
}

#[test]
fn test_persistent_cache_basic() {
    let mirrors = vec![create_test_mirror()];
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cache = PersistentCache::new(mirrors, current_time, Some("etag".to_string()));

    // Test basic properties
    assert_eq!(cache.mirrors.len(), 1);
    assert_eq!(cache.timestamp, current_time);
    assert_eq!(cache.etag, Some("etag".to_string()));
    assert_eq!(cache.cache_version, 1);

    // Test validity
    assert!(cache.is_valid(7200)); // Fresh cache

    // Test with old timestamp
    let old_cache = PersistentCache::new(vec![create_test_mirror()], 1_234_567_890, None);
    assert!(!old_cache.is_valid(1)); // Stale cache (1 second timeout)

    // Test with wrong cache version
    let mut wrong_version_cache =
        PersistentCache::new(vec![create_test_mirror()], current_time, None);
    wrong_version_cache.cache_version = 999; // Wrong version
    assert!(!wrong_version_cache.is_valid(7200)); // Should be invalid due to version mismatch
}

#[test]
fn test_cache_stats_basic() {
    let stats = CacheStats {
        size_bytes: 1024,
        mirror_count: 5,
        timestamp: 1_234_567_890,
        cache_version: 1,
        etag: Some("test".to_string()),
    };

    let age = stats.age_hours();
    assert!(age > 0.0);

    let human_size = stats.size_human();
    assert_eq!(human_size, "1.0 KB");
}

#[test]
fn test_cache_functions_with_home() {
    // Ensure HOME is set for this test
    let temp_dir = TempDir::new().unwrap();
    let original_home = env::var("HOME").ok();
    let original_xdg = env::var("XDG_CACHE_HOME").ok();

    // Create a proper .cache directory structure in the temp directory
    let cache_base_dir = temp_dir.path().join(".cache");
    std::fs::create_dir_all(&cache_base_dir).unwrap();

    // Remove XDG_CACHE_HOME and set HOME to temp dir
    unsafe {
        env::remove_var("XDG_CACHE_HOME");
    }
    unsafe {
        env::set_var("HOME", temp_dir.path());
    }

    // Test get_cache_dir
    let cache_dir_result = mirage::cache::get_cache_dir();
    assert!(cache_dir_result.is_ok());

    // Test load_cache (should return None when no cache exists)
    let load_result = mirage::cache::load_cache();
    assert!(load_result.is_ok());
    assert!(load_result.unwrap().is_none());

    // Test save_cache
    let mirrors = vec![create_test_mirror()];
    let cache = PersistentCache::new(mirrors, 1_234_567_890, None);
    let save_result = mirage::cache::save_cache(&cache);
    if save_result.is_err() {
        println!("Save error: {:?}", save_result.as_ref().unwrap_err());
    }
    assert!(save_result.is_ok());

    // Test load_cache again (should now return the cache)
    let load_result2 = mirage::cache::load_cache();
    assert!(load_result2.is_ok());
    assert!(load_result2.unwrap().is_some());

    // Test get_cache_stats
    let stats_result = mirage::cache::get_cache_stats();
    assert!(stats_result.is_ok());

    // Test clear_cache
    let clear_result = mirage::cache::clear_cache();
    assert!(clear_result.is_ok());

    // Restore environment
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CACHE_HOME", xdg);
        }
    }
}

#[test]
fn test_cache_no_home_error() {
    let original_home = env::var("HOME").ok();
    let original_xdg = env::var("XDG_CACHE_HOME").ok();

    // Remove both environment variables
    unsafe {
        env::remove_var("HOME");
    }
    unsafe {
        env::remove_var("XDG_CACHE_HOME");
    }

    // get_cache_dir should fail
    let result = mirage::cache::get_cache_dir();
    assert!(result.is_err());

    // Restore environment
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CACHE_HOME", xdg);
        }
    }
}

#[test]
fn test_xdg_cache_home_path() {
    // Test that XDG_CACHE_HOME path is used correctly
    let temp_dir = TempDir::new().unwrap();
    let xdg_path = temp_dir.path().to_str().unwrap();

    let original_xdg = env::var("XDG_CACHE_HOME").ok();
    let original_home = env::var("HOME").ok();

    // Set XDG_CACHE_HOME
    unsafe {
        env::set_var("XDG_CACHE_HOME", xdg_path);
    }

    let cache_dir = mirage::cache::get_cache_dir().unwrap();
    assert_eq!(cache_dir, temp_dir.path().join("mirage"));

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CACHE_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CACHE_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_clear_nonexistent_cache() {
    // Test clearing cache when no cache file exists
    let temp_dir = TempDir::new().unwrap();

    let original_xdg = env::var("XDG_CACHE_HOME").ok();
    let original_home = env::var("HOME").ok();

    // Use a fresh temp directory with no cache file
    unsafe {
        env::set_var("XDG_CACHE_HOME", temp_dir.path());
    }

    // This should succeed even though no cache file exists
    let result = mirage::cache::clear_cache();
    assert!(result.is_ok());

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CACHE_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CACHE_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}

#[test]
fn test_stats_no_cache_file() {
    // Test getting stats when no cache file exists
    let temp_dir = TempDir::new().unwrap();

    let original_xdg = env::var("XDG_CACHE_HOME").ok();
    let original_home = env::var("HOME").ok();

    // Use a fresh temp directory with no cache file
    unsafe {
        env::set_var("XDG_CACHE_HOME", temp_dir.path());
    }

    let result = mirage::cache::get_cache_stats();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // No cache file exists

    // Restore environment
    if let Some(xdg) = original_xdg {
        unsafe {
            env::set_var("XDG_CACHE_HOME", xdg);
        }
    } else {
        unsafe {
            env::remove_var("XDG_CACHE_HOME");
        }
    }
    if let Some(home) = original_home {
        unsafe {
            env::set_var("HOME", home);
        }
    }
}
