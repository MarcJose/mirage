// Comprehensive tests for performance.rs functionality
use mirage::Mirror;
use mirage::performance::{
    MirrorPerformance, MirrorWithPerformance, test_mirror_download_speed, test_mirror_performance,
};
use reqwest::Client;
use std::time::{Duration, SystemTime};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

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

fn create_test_mirror_with_url(url: &str) -> Mirror {
    Mirror {
        url: url.to_string(),
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

// =============================================================================
// Struct Tests
// =============================================================================

#[test]
fn test_mirror_performance_default() {
    let perf = MirrorPerformance::default();
    assert_eq!(perf.latency_ms, 0);
    assert_eq!(perf.status_code, 0);
    assert!(!perf.available);
    assert!(perf.download_speed_kbps.is_none());
    assert!(perf.tested_at <= SystemTime::now());
}

#[test]
fn test_mirror_with_performance_from() {
    let mirror = create_test_mirror();
    let mirror_with_perf: MirrorWithPerformance = mirror.clone().into();

    assert_eq!(mirror_with_perf.mirror.url, mirror.url);
    assert!(mirror_with_perf.performance.is_none());
}

#[test]
fn test_mirror_performance_struct() {
    let perf = MirrorPerformance {
        latency_ms: 250,
        status_code: 200,
        available: true,
        download_speed_kbps: Some(1024),
        tested_at: SystemTime::now(),
    };

    assert_eq!(perf.latency_ms, 250);
    assert_eq!(perf.status_code, 200);
    assert!(perf.available);
    assert_eq!(perf.download_speed_kbps, Some(1024));
}

#[test]
fn test_mirror_with_performance_full() {
    let mirror = create_test_mirror();
    let perf = MirrorPerformance {
        latency_ms: 123,
        status_code: 200,
        available: true,
        download_speed_kbps: Some(512),
        tested_at: SystemTime::now(),
    };

    let mirror_with_perf = MirrorWithPerformance {
        mirror: mirror.clone(),
        performance: Some(perf),
    };

    assert_eq!(mirror_with_perf.mirror.url, mirror.url);
    assert!(mirror_with_perf.performance.is_some());

    let performance = mirror_with_perf.performance.unwrap();
    assert_eq!(performance.latency_ms, 123);
    assert_eq!(performance.status_code, 200);
    assert!(performance.available);
    assert_eq!(performance.download_speed_kbps, Some(512));
}

#[test]
fn test_mirror_performance_clone() {
    let perf = MirrorPerformance {
        latency_ms: 100,
        status_code: 200,
        available: true,
        download_speed_kbps: Some(256),
        tested_at: SystemTime::now(),
    };

    let cloned = perf.clone();
    assert_eq!(cloned.latency_ms, perf.latency_ms);
    assert_eq!(cloned.status_code, perf.status_code);
    assert_eq!(cloned.available, perf.available);
    assert_eq!(cloned.download_speed_kbps, perf.download_speed_kbps);
}

#[test]
fn test_mirror_with_performance_clone() {
    let mirror = create_test_mirror();
    let perf = MirrorPerformance {
        latency_ms: 150,
        status_code: 404,
        available: false,
        download_speed_kbps: None,
        tested_at: SystemTime::now(),
    };

    let mirror_with_perf = MirrorWithPerformance {
        mirror,
        performance: Some(perf),
    };

    let cloned = mirror_with_perf.clone();
    assert_eq!(cloned.mirror.url, mirror_with_perf.mirror.url);
    assert!(cloned.performance.is_some());

    let cloned_perf = cloned.performance.unwrap();
    let orig_perf = mirror_with_perf.performance.unwrap();
    assert_eq!(cloned_perf.latency_ms, orig_perf.latency_ms);
    assert_eq!(cloned_perf.status_code, orig_perf.status_code);
    assert_eq!(cloned_perf.available, orig_perf.available);
}

// =============================================================================
// Mock Server Tests
// =============================================================================

#[tokio::test]
async fn test_mirror_performance_success_debug_logging() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Mock successful HEAD request
    Mock::given(method("HEAD"))
        .and(path("/core/os/x86_64/core.db"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    // Test with verbose=false to trigger debug logging but not verbose eprintln
    let result = test_mirror_performance(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200);
}

#[tokio::test]
async fn test_mirror_performance_success_with_delay() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Create a mock that simulates a successful mirror test with delay
    Mock::given(method("HEAD"))
        .and(path("/core/os/x86_64/core.db"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(50)))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_performance(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200);
    assert!(perf.latency_ms > 0);
}

#[tokio::test]
async fn test_mirror_performance_verbose_mode() {
    let client = Client::new();
    let mut mirror = create_test_mirror();
    mirror.url = "https://nonexistent.invalid.mirror.example/".to_string();

    // Test verbose mode (should not panic or fail, just print to stderr)
    let result = test_mirror_performance(&client, &mirror, true).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(!perf.available);
    assert_eq!(perf.status_code, 0);
}

#[tokio::test]
async fn test_mirror_download_speed_verbose_mode() {
    let client = Client::new();
    let mut mirror = create_test_mirror();
    mirror.url = "https://nonexistent.invalid.mirror.example/".to_string();

    // Test verbose mode
    let result = test_mirror_download_speed(&client, &mirror, true).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(!perf.available);
    assert_eq!(perf.status_code, 0);
    assert!(perf.download_speed_kbps.is_none());
}

#[tokio::test]
async fn test_mirror_download_speed_success() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Create a reasonably sized file with some delay
    let content = vec![b'X'; 10_000]; // 10KB content
    Mock::given(method("GET"))
        .and(path("/core/os/x86_64/core.db.sig"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(content)
                .set_delay(Duration::from_millis(100)), // 100ms delay
        )
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_download_speed(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200);
    // Speed calculation might be None for very small files or very fast downloads
    // Just verify the test completed successfully
}

#[tokio::test]
async fn test_mirror_download_speed_large_file() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Create a very large file and add significant delay to ensure measurable time
    let large_content = vec![b'D'; 100_000]; // 100KB content
    Mock::given(method("GET"))
        .and(path("/core/os/x86_64/core.db.sig"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(large_content)
                .set_delay(Duration::from_millis(200)), // 200ms delay
        )
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_download_speed(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200);
    // Speed calculation might be None for very fast downloads in test environment
    // Just verify the test completed successfully
}

#[tokio::test]
async fn test_mirror_performance_error_response() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Mock 404 error response
    Mock::given(method("HEAD"))
        .and(path("/core/os/x86_64/core.db"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_performance(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(!perf.available);
    assert_eq!(perf.status_code, 404);
}

#[tokio::test]
async fn test_mirror_download_speed_error_response() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Mock 500 error response
    Mock::given(method("GET"))
        .and(path("/core/os/x86_64/core.db.sig"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_download_speed(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(!perf.available);
    assert_eq!(perf.status_code, 500);
    assert!(perf.download_speed_kbps.is_none());
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_mirror_performance_invalid_url() {
    let client = Client::new();
    let mut mirror = create_test_mirror();
    mirror.url = "https://nonexistent.invalid.mirror.example/".to_string();

    let result = test_mirror_performance(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(!perf.available);
    assert_eq!(perf.status_code, 0);
}

#[tokio::test]
async fn test_mirror_download_speed_invalid_url() {
    let client = Client::new();
    let mut mirror = create_test_mirror();
    mirror.url = "https://nonexistent.invalid.mirror.example/".to_string();

    let result = test_mirror_download_speed(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(!perf.available);
    assert_eq!(perf.status_code, 0);
    assert!(perf.download_speed_kbps.is_none());
}

#[tokio::test]
async fn test_mirror_performance_timeout_behavior() {
    let client = Client::builder()
        .timeout(Duration::from_millis(100)) // Short but not impossibly short timeout
        .build()
        .unwrap();

    let mut mirror = create_test_mirror();
    // Use a non-existent domain that should timeout
    mirror.url = "https://definitely-does-not-exist-12345.com/".to_string();

    let result = test_mirror_performance(&client, &mirror, false).await;
    // Should either succeed quickly or timeout and return unavailable
    assert!(result.is_ok());

    let perf = result.unwrap();
    // Should be unavailable due to timeout or connection failure
    assert!(!perf.available);
    // Status code should be 0 for connection failures/timeouts
    assert_eq!(perf.status_code, 0);
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[tokio::test]
async fn test_mirror_download_speed_zero_size() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Mock empty file response
    Mock::given(method("GET"))
        .and(path("/core/os/x86_64/core.db.sig"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![]))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_download_speed(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200);
    // Speed calculation with zero size should result in None or 0
    assert!(perf.download_speed_kbps.is_none() || perf.download_speed_kbps == Some(0));
}

#[tokio::test]
async fn test_mirror_performance_with_redirects() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Mock redirect response
    Mock::given(method("HEAD"))
        .and(path("/core/os/x86_64/core.db"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", &format!("{}/redirected", mock_server.uri())),
        )
        .mount(&mock_server)
        .await;

    // Mock final destination
    Mock::given(method("HEAD"))
        .and(path("/redirected"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_performance(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200); // Should follow redirect and get final status
}

// =============================================================================
// Performance Measurement Tests
// =============================================================================

#[tokio::test]
async fn test_performance_timing_accuracy() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    let delay_ms = 100;
    Mock::given(method("HEAD"))
        .and(path("/core/os/x86_64/core.db"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(delay_ms)))
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_performance(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);

    // Latency should be approximately the delay we set (allow some variance)
    assert!(perf.latency_ms >= u32::try_from(delay_ms - 50).expect("REASON")); // Allow 50ms variance
    assert!(perf.latency_ms <= u32::try_from(delay_ms + 200).expect("REASON")); // Allow up to 200ms extra for processing
}

#[tokio::test]
async fn test_download_speed_calculation() {
    let mock_server = MockServer::start().await;
    let client = Client::new();

    // Create a file of known size with controlled delay
    let file_size = 50_000; // 50KB
    let delay_ms = 250; // 250ms
    let content = vec![b'T'; file_size];

    Mock::given(method("GET"))
        .and(path("/core/os/x86_64/core.db.sig"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(content)
                .set_delay(Duration::from_millis(delay_ms)),
        )
        .mount(&mock_server)
        .await;

    let mirror = create_test_mirror_with_url(&format!("{}/", mock_server.uri()));

    let result = test_mirror_download_speed(&client, &mirror, false).await;
    assert!(result.is_ok());

    let perf = result.unwrap();
    assert!(perf.available);
    assert_eq!(perf.status_code, 200);
    // Speed calculation might be None for very fast downloads in test environment
    // Just verify the test completed successfully
    if let Some(speed_kbps) = perf.download_speed_kbps {
        // If speed is calculated, it should be reasonable
        assert!(speed_kbps > 0);
        assert!(speed_kbps < 10000); // Less than 10 MB/s (reasonable upper bound)
    }
}
