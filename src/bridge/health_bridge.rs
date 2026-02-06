//! Health check bridge
//!
//! Provides synchronous access interface to HealthChecker for GUI components

use crate::config::UpstreamProxy;
use crate::health::{HealthCheckConfig, HealthCheckResult, HealthChecker};
use super::{BridgeError, BridgeResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// Health check bridge
///
/// Wraps the asynchronous HealthChecker into a synchronous API for GUI use
pub struct HealthBridge {
    runtime: Runtime,
    checker: Arc<HealthChecker>,
}

impl HealthBridge {
    /// Create a new health check bridge (with default configuration)
    pub fn new() -> BridgeResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| BridgeError::Runtime(format!("Failed to create runtime: {}", e)))?;

        let checker = HealthChecker::new()
            .map_err(|e| BridgeError::Health(format!("Failed to create health checker: {}", e)))?;

        Ok(Self {
            runtime,
            checker: Arc::new(checker),
        })
    }

    /// Create a health check bridge with custom configuration
    pub fn with_config(config: HealthCheckConfig) -> BridgeResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| BridgeError::Runtime(format!("Failed to create runtime: {}", e)))?;

        let checker = HealthChecker::with_config(config)
            .map_err(|e| BridgeError::Health(format!("Failed to create health checker: {}", e)))?;

        Ok(Self {
            runtime,
            checker: Arc::new(checker),
        })
    }

    /// Check the health status of a single proxy
    pub fn check_proxy(&self, proxy: &UpstreamProxy) -> HealthCheckResult {
        self.runtime.block_on(async {
            self.checker.check_proxy(proxy).await
        })
    }

    /// Batch check the health status of multiple proxies
    pub fn check_proxies(&self, proxies: &[UpstreamProxy]) -> Vec<(String, HealthCheckResult)> {
        self.runtime.block_on(async {
            let mut results = Vec::new();
            for proxy in proxies {
                if !proxy.enabled {
                    continue;
                }
                let result = self.checker.check_proxy(proxy).await;
                results.push((proxy.id.clone(), result));
            }
            results
        })
    }

    /// Start a background health check task
    ///
    /// Returns a task handle that can be stopped via `abort()`
    ///
    /// # Parameters
    ///
    /// - `proxies` - The list of proxies to check (Arc<RwLock<Vec<UpstreamProxy>>>)
    /// - `callback` - Callback function to be called after check completion
    ///
    /// # Note
    ///
    /// This method starts a background task that needs to be stopped manually by calling `handle.abort()`
    pub fn start_background_check<F>(
        &self,
        proxies: Arc<RwLock<Vec<UpstreamProxy>>>,
        callback: F,
    ) -> tokio::task::JoinHandle<()>
    where
        F: Fn(String, HealthCheckResult) + Send + 'static,
    {
        let checker = Arc::clone(&self.checker);
        let handle = checker.start_periodic_check(proxies, callback);

        // Convert JoinHandle<Result<(), JoinError>> to JoinHandle<()>
        self.runtime.spawn(async move {
            let _ = handle.await;
        })
    }

    /// Get an Arc reference to the internal checker (for other bridge components)
    #[allow(dead_code)]
    pub(crate) fn get_checker_arc(&self) -> Arc<HealthChecker> {
        Arc::clone(&self.checker)
    }
}

impl Default for HealthBridge {
    fn default() -> Self {
        Self::new().expect("Failed to create HealthBridge")
    }
}

/// Health check callback type
///
/// Used by GUI components to subscribe to health check results
#[allow(dead_code)]
pub type HealthCheckCallback = Box<dyn Fn(String, HealthCheckResult) + Send>;

/// Configurable health check bridge builder
#[allow(dead_code)]
pub struct HealthBridgeBuilder {
    timeout: Duration,
    test_url: String,
    failure_threshold: u32,
    check_interval: Duration,
}

#[allow(dead_code)]
impl HealthBridgeBuilder {
    /// Create a new builder (with default values)
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            test_url: "http://www.gstatic.com/generate_204".to_string(),
            failure_threshold: 3,
            check_interval: Duration::from_secs(60),
        }
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the test URL
    pub fn test_url(mut self, url: impl Into<String>) -> Self {
        self.test_url = url.into();
        self
    }

    /// Set the failure threshold
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the check interval
    pub fn check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Build the HealthBridge
    pub fn build(self) -> BridgeResult<HealthBridge> {
        let config = HealthCheckConfig {
            timeout: self.timeout,
            test_url: self.test_url,
            failure_threshold: self.failure_threshold,
            check_interval: self.check_interval,
        };

        HealthBridge::with_config(config)
    }
}

impl Default for HealthBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::config::UpstreamConfig;

    fn create_test_proxy() -> UpstreamProxy {
        UpstreamProxy {
            id: "test-proxy".to_string(),
            name: "Test Proxy".to_string(),
            config: UpstreamConfig {
                host: "127.0.0.1".to_string(),
                port: 1080,
                username: None,
                password: None,
            },
            enabled: true,
            health: crate::config::upstream::ProxyHealth::default(),
        }
    }

    #[test]
    fn test_health_bridge_creation() {
        let bridge = HealthBridge::new();
        assert!(bridge.is_ok());
    }

    #[test]
    fn test_health_bridge_with_config() {
        let config = HealthCheckConfig {
            timeout: Duration::from_secs(5),
            test_url: "http://example.com".to_string(),
            failure_threshold: 2,
            check_interval: Duration::from_secs(30),
        };

        let bridge = HealthBridge::with_config(config);
        assert!(bridge.is_ok());
    }

    #[test]
    #[ignore] // Requires a real SOCKS5 proxy
    fn test_check_proxy() {
        let bridge = HealthBridge::new().unwrap();
        let proxy = create_test_proxy();

        let result = bridge.check_proxy(&proxy);
        // Should fail because there's no real proxy
        assert!(!result.is_healthy);
    }

    #[test]
    fn test_health_bridge_builder() {
        let bridge = HealthBridgeBuilder::new()
            .timeout(Duration::from_secs(15))
            .test_url("http://www.google.com/generate_204")
            .failure_threshold(5)
            .check_interval(Duration::from_secs(120))
            .build();

        assert!(bridge.is_ok());
    }
}
