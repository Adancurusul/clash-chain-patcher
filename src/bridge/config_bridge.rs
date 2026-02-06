//! Configuration management bridge
//!
//! Provides synchronous access interface to ConfigManager for GUI components

use crate::config::{ConfigManager, UpstreamProxy};
use super::{BridgeError, BridgeResult};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// Configuration management bridge
///
/// Wraps the asynchronous ConfigManager into a synchronous API for GUI use
pub struct ConfigBridge {
    runtime: Runtime,
    manager: Arc<RwLock<ConfigManager>>,
}

impl ConfigBridge {
    /// Create a new configuration bridge
    pub fn new() -> BridgeResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| BridgeError::Runtime(format!("Failed to create runtime: {}", e)))?;

        let manager = ConfigManager::new()
            .map_err(|e| BridgeError::Config(format!("Failed to create config manager: {}", e)))?;

        Ok(Self {
            runtime,
            manager: Arc::new(RwLock::new(manager)),
        })
    }

    /// Create a bridge from an existing ConfigManager
    pub fn from_manager(manager: ConfigManager) -> BridgeResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| BridgeError::Runtime(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            runtime,
            manager: Arc::new(RwLock::new(manager)),
        })
    }

    /// List all upstream proxies
    pub fn list_upstreams(&self) -> Vec<UpstreamProxy> {
        self.runtime.block_on(async {
            let manager = self.manager.read().await;
            manager.list_upstreams().to_vec()
        })
    }

    /// Add an upstream proxy
    pub fn add_upstream(&self, proxy: UpstreamProxy) -> BridgeResult<()> {
        self.runtime.block_on(async {
            let mut manager = self.manager.write().await;
            manager.add_upstream(proxy)
                .map_err(|e| BridgeError::Config(e.to_string()))
        })
    }

    /// Update an upstream proxy
    pub fn update_upstream(&self, proxy: UpstreamProxy) -> BridgeResult<()> {
        self.runtime.block_on(async {
            let mut manager = self.manager.write().await;
            manager.update_upstream(proxy)
                .map_err(|e| BridgeError::Config(e.to_string()))
        })
    }

    /// Remove an upstream proxy
    pub fn remove_upstream(&self, id: &str) -> BridgeResult<()> {
        self.runtime.block_on(async {
            let mut manager = self.manager.write().await;
            manager.remove_upstream(id)
                .map_err(|e| BridgeError::Config(e.to_string()))
        })
    }

    /// Get a specific upstream proxy
    pub fn get_upstream(&self, id: &str) -> Option<UpstreamProxy> {
        self.runtime.block_on(async {
            let manager = self.manager.read().await;
            manager.get_upstream(id).cloned()
        })
    }

    /// Enable an upstream proxy
    pub fn enable_upstream(&self, id: &str) -> BridgeResult<()> {
        self.runtime.block_on(async {
            let mut manager = self.manager.write().await;
            manager.set_upstream_enabled(id, true)
                .map_err(|e| BridgeError::Config(e.to_string()))
        })
    }

    /// Disable an upstream proxy
    pub fn disable_upstream(&self, id: &str) -> BridgeResult<()> {
        self.runtime.block_on(async {
            let mut manager = self.manager.write().await;
            manager.set_upstream_enabled(id, false)
                .map_err(|e| BridgeError::Config(e.to_string()))
        })
    }

    /// Get the configuration file path
    pub fn config_path(&self) -> std::path::PathBuf {
        self.runtime.block_on(async {
            let manager = self.manager.read().await;
            manager.config_path().to_path_buf()
        })
    }

    /// Get an Arc reference to the internal manager (for other bridge components)
    #[allow(dead_code)]
    pub(crate) fn get_manager_arc(&self) -> Arc<RwLock<ConfigManager>> {
        Arc::clone(&self.manager)
    }

    /// Add a recently used file
    pub fn add_recent_file(&self, path: String) -> BridgeResult<()> {
        self.runtime.block_on(async {
            let mut manager = self.manager.write().await;
            manager.add_recent_file(path)
                .map_err(|e| BridgeError::Config(e.to_string()))
        })
    }

    /// Get the list of recently used files
    pub fn get_recent_files(&self) -> Vec<String> {
        self.runtime.block_on(async {
            let manager = self.manager.read().await;
            manager.get_recent_files().to_vec()
        })
    }
}

impl Default for ConfigBridge {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigBridge")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::config::UpstreamConfig;
    use uuid::Uuid;

    fn create_test_manager() -> ConfigManager {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join(format!("test-config-{}.json", Uuid::new_v4()));
        ConfigManager::new_with_path(config_path).unwrap()
    }

    #[test]
    fn test_config_bridge_creation() {
        let bridge = ConfigBridge::new();
        assert!(bridge.is_ok());
    }

    #[test]
    fn test_add_and_list_upstreams() {
        let manager = create_test_manager();
        let bridge = ConfigBridge::from_manager(manager).unwrap();

        let proxy = UpstreamProxy {
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
        };

        // Add proxy
        let result = bridge.add_upstream(proxy.clone());
        assert!(result.is_ok());

        // List proxies
        let proxies = bridge.list_upstreams();
        assert_eq!(proxies.len(), 1);
        assert_eq!(proxies[0].id, "test-proxy");
    }

    #[test]
    fn test_update_upstream() {
        let manager = create_test_manager();
        let bridge = ConfigBridge::from_manager(manager).unwrap();

        let mut proxy = UpstreamProxy {
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
        };

        bridge.add_upstream(proxy.clone()).unwrap();

        // Update proxy name
        proxy.name = "Updated Proxy".to_string();
        let result = bridge.update_upstream(proxy);
        assert!(result.is_ok());

        // Verify update
        let updated = bridge.get_upstream("test-proxy").unwrap();
        assert_eq!(updated.name, "Updated Proxy");
    }

    #[test]
    fn test_enable_disable_upstream() {
        let manager = create_test_manager();
        let bridge = ConfigBridge::from_manager(manager).unwrap();

        let proxy = UpstreamProxy {
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
        };

        bridge.add_upstream(proxy).unwrap();

        // Disable
        bridge.disable_upstream("test-proxy").unwrap();
        let proxy = bridge.get_upstream("test-proxy").unwrap();
        assert!(!proxy.enabled);

        // Enable
        bridge.enable_upstream("test-proxy").unwrap();
        let proxy = bridge.get_upstream("test-proxy").unwrap();
        assert!(proxy.enabled);
    }

    #[test]
    fn test_remove_upstream() {
        let manager = create_test_manager();
        let bridge = ConfigBridge::from_manager(manager).unwrap();

        let proxy = UpstreamProxy {
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
        };

        bridge.add_upstream(proxy).unwrap();
        assert_eq!(bridge.list_upstreams().len(), 1);

        // Remove
        bridge.remove_upstream("test-proxy").unwrap();
        assert_eq!(bridge.list_upstreams().len(), 0);
    }
}
