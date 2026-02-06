//! Health checker implementation for upstream proxies

use crate::config::UpstreamProxy;
use crate::proxy::config::UpstreamConfig;
use anyhow::{Context, Result};
use fast_socks5::client::Socks5Stream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Timeout for each health check (default: 10 seconds)
    pub timeout: Duration,

    /// Test URL for HTTP validation (default: http://www.gstatic.com/generate_204)
    pub test_url: String,

    /// Number of consecutive failures before marking unhealthy (default: 3)
    pub failure_threshold: u32,

    /// Interval between health checks (default: 5 minutes)
    pub check_interval: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            test_url: "http://www.gstatic.com/generate_204".to_string(),
            failure_threshold: 3,
            check_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Whether the proxy is healthy
    pub is_healthy: bool,

    /// Latency in milliseconds (None if unhealthy)
    pub latency_ms: Option<u64>,

    /// Error message (None if healthy)
    pub error: Option<String>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(latency_ms: u64) -> Self {
        Self {
            is_healthy: true,
            latency_ms: Some(latency_ms),
            error: None,
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(error: String) -> Self {
        Self {
            is_healthy: false,
            latency_ms: None,
            error: Some(error),
        }
    }
}

/// Health checker for upstream proxies
pub struct HealthChecker {
    config: HealthCheckConfig,
    #[allow(dead_code)] // Used for future HTTP-based health checks
    client: reqwest::Client,
}

impl HealthChecker {
    /// Create a new health checker with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(HealthCheckConfig::default())
    }

    /// Create a new health checker with custom configuration
    pub fn with_config(config: HealthCheckConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// Check the health of a single proxy
    ///
    /// This performs:
    /// 1. SOCKS5 connection test
    /// 2. HTTP request validation
    /// 3. Latency measurement
    pub async fn check_proxy(&self, proxy: &UpstreamProxy) -> HealthCheckResult {
        info!("Starting health check for proxy: {} ({}:{})",
              proxy.name, proxy.config.host, proxy.config.port);

        let start = Instant::now();

        // Step 1: Test SOCKS5 connection
        match self.test_socks5_connection(&proxy.config).await {
            Ok(_) => {
                debug!("SOCKS5 connection test passed for {}", proxy.name);
            }
            Err(e) => {
                let error_msg = format!("SOCKS5 connection failed: {}", e);
                warn!("{}", error_msg);
                return HealthCheckResult::unhealthy(error_msg);
            }
        }

        // Step 2: Test HTTP request through proxy
        match self.test_http_request(&proxy.config).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                info!("Health check passed for {} (latency: {}ms)", proxy.name, latency);
                HealthCheckResult::healthy(latency)
            }
            Err(e) => {
                let error_msg = format!("HTTP request failed: {}", e);
                error!("{}", error_msg);
                HealthCheckResult::unhealthy(error_msg)
            }
        }
    }

    /// Test SOCKS5 connection to the proxy
    async fn test_socks5_connection(&self, config: &UpstreamConfig) -> Result<()> {
        let proxy_addr = format!("{}:{}", config.host, config.port);

        // Perform SOCKS5 connection test with authentication
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            // Test connection with authentication
            let _socks_stream = timeout(
                self.config.timeout,
                Socks5Stream::connect_with_password(
                    &proxy_addr,
                    "www.google.com".to_string(),
                    443,
                    username.clone(),
                    password.clone(),
                    fast_socks5::client::Config::default(),
                )
            )
            .await
            .context("SOCKS5 connection timeout")?
            .context("SOCKS5 connection failed")?;

            debug!("SOCKS5 connection with auth successful to {}", proxy_addr);
        } else {
            // Test connection without authentication
            let _socks_stream = timeout(
                self.config.timeout,
                Socks5Stream::connect(
                    &proxy_addr,
                    "www.google.com".to_string(),
                    443,
                    fast_socks5::client::Config::default(),
                )
            )
            .await
            .context("SOCKS5 connection timeout")?
            .context("SOCKS5 connection failed")?;

            debug!("SOCKS5 connection without auth successful to {}", proxy_addr);
        }

        Ok(())
    }

    /// Test HTTP request through the proxy
    async fn test_http_request(&self, config: &UpstreamConfig) -> Result<()> {
        let proxy_url = if let (Some(username), Some(password)) = (&config.username, &config.password) {
            format!("socks5://{}:{}@{}:{}", username, password, config.host, config.port)
        } else {
            format!("socks5://{}:{}", config.host, config.port)
        };

        // Create a client with proxy
        let proxy = reqwest::Proxy::all(&proxy_url)
            .context("Failed to create proxy configuration")?;

        let client = reqwest::Client::builder()
            .proxy(proxy)
            .timeout(self.config.timeout)
            .build()
            .context("Failed to create HTTP client with proxy")?;

        // Make HTTP request
        let response = timeout(
            self.config.timeout,
            client.get(&self.config.test_url).send()
        )
        .await
        .context("HTTP request timeout")?
        .context("HTTP request failed")?;

        // Check response status
        if response.status().is_success() || response.status() == 204 {
            debug!("HTTP request successful, status: {}", response.status());
            Ok(())
        } else {
            anyhow::bail!("Unexpected HTTP status: {}", response.status())
        }
    }

    /// Get the health check configuration
    pub fn config(&self) -> &HealthCheckConfig {
        &self.config
    }

    /// Start a background task that periodically checks proxy health
    ///
    /// Returns a handle that can be used to stop the task.
    ///
    /// The callback function is called after each health check with the results.
    pub fn start_periodic_check<F>(
        self: Arc<Self>,
        proxies: Arc<RwLock<Vec<UpstreamProxy>>>,
        mut callback: F,
    ) -> tokio::task::JoinHandle<()>
    where
        F: FnMut(String, HealthCheckResult) + Send + 'static,
    {
        let check_interval = self.config.check_interval;

        tokio::spawn(async move {
            let mut interval = interval(check_interval);
            interval.tick().await; // Skip the first immediate tick

            loop {
                interval.tick().await;

                info!("Starting periodic health check");

                let proxies_snapshot = {
                    let proxies_guard = proxies.read().await;
                    proxies_guard.clone()
                };

                for proxy in proxies_snapshot.iter().filter(|p| p.enabled) {
                    let result = self.check_proxy(proxy).await;
                    callback(proxy.id.clone(), result);
                }

                info!("Periodic health check completed");
            }
        })
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new().expect("Failed to create default health checker")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UpstreamProxy;

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.test_url, "http://www.gstatic.com/generate_204");
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.check_interval, Duration::from_secs(300));
    }

    #[test]
    fn test_health_check_result_healthy() {
        let result = HealthCheckResult::healthy(120);
        assert!(result.is_healthy);
        assert_eq!(result.latency_ms, Some(120));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_health_check_result_unhealthy() {
        let result = HealthCheckResult::unhealthy("Connection timeout".to_string());
        assert!(!result.is_healthy);
        assert!(result.latency_ms.is_none());
        assert_eq!(result.error, Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert!(checker.is_ok());

        let checker = checker.unwrap();
        assert_eq!(checker.config().timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_health_checker_custom_config() {
        let config = HealthCheckConfig {
            timeout: Duration::from_secs(5),
            test_url: "http://example.com".to_string(),
            failure_threshold: 2,
            check_interval: Duration::from_secs(60),
        };

        let checker = HealthChecker::with_config(config);
        assert!(checker.is_ok());

        let checker = checker.unwrap();
        assert_eq!(checker.config().timeout, Duration::from_secs(5));
        assert_eq!(checker.config().test_url, "http://example.com");
    }

    // Integration test - requires a real proxy
    #[tokio::test]
    #[ignore] // Ignore by default, run with --ignored
    async fn test_check_real_proxy() {
        // This test requires the Hong Kong proxy from the project
        let config = UpstreamConfig {
            host: "64.32.179.160".to_string(),
            port: 60088,
            username: Some("ZUvGbvjcI52P".to_string()),
            password: Some("0UxQRzGfZoup".to_string()),
        };

        let proxy = UpstreamProxy::new("Test Proxy".to_string(), config);

        let checker = HealthChecker::new().unwrap();
        let result = checker.check_proxy(&proxy).await;

        println!("Health check result: {:?}", result);
        assert!(result.is_healthy);
        assert!(result.latency_ms.is_some());
    }

    #[tokio::test]
    async fn test_check_invalid_proxy() {
        // Test with an invalid proxy
        let config = UpstreamConfig {
            host: "192.0.2.1".to_string(), // TEST-NET-1, should fail
            port: 9999,
            username: None,
            password: None,
        };

        let proxy = UpstreamProxy::new("Invalid Proxy".to_string(), config);

        let checker = HealthChecker::new().unwrap();
        let result = checker.check_proxy(&proxy).await;

        assert!(!result.is_healthy);
        assert!(result.error.is_some());
    }
}
