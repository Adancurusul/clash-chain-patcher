//! Configuration merger bridge
//!
//! Provides synchronous access interface to ClashConfigMerger for GUI components

use crate::merger::{ClashConfigMerger, MergeResult, MergerConfig};
use super::{BridgeError, BridgeResult};
use std::path::Path;

/// Configuration merger bridge
///
/// Wraps ClashConfigMerger into a more user-friendly API for GUI use
pub struct MergerBridge {
    merger: ClashConfigMerger,
}

impl MergerBridge {
    /// Create a new configuration merger bridge (with default configuration)
    pub fn new() -> Self {
        Self {
            merger: ClashConfigMerger::new(),
        }
    }

    /// Create a configuration merger bridge with custom configuration
    pub fn with_config(config: MergerConfig) -> Self {
        Self {
            merger: ClashConfigMerger::with_config(config),
        }
    }

    /// Merge configuration
    ///
    /// Add local proxy nodes to the Clash configuration
    pub fn merge(&self, config_path: impl AsRef<Path>) -> BridgeResult<MergeResult> {
        self.merger
            .merge(config_path.as_ref())
            .map_err(|e| BridgeError::Merger(e.to_string()))
    }

    /// Validate if the configuration file is valid
    pub fn validate_config(&self, config_path: impl AsRef<Path>) -> BridgeResult<()> {
        let path = config_path.as_ref();

        if !path.exists() {
            return Err(BridgeError::Merger(format!(
                "Configuration file does not exist: {}",
                path.display()
            )));
        }

        // Try to read YAML
        let content = std::fs::read_to_string(path)
            .map_err(|e| BridgeError::Merger(format!("Unable to read configuration file: {}", e)))?;

        serde_yaml::from_str::<serde_yaml::Value>(&content)
            .map_err(|e| BridgeError::Merger(format!("YAML format error: {}", e)))?;

        Ok(())
    }

    /// Get the merger configuration
    pub fn config(&self) -> &MergerConfig {
        self.merger.config()
    }

    /// Update the merger configuration
    pub fn update_config(&mut self, config: MergerConfig) {
        self.merger = ClashConfigMerger::with_config(config);
    }
}

impl Default for MergerBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Configurable configuration merger bridge builder
#[allow(dead_code)]
pub struct MergerBridgeBuilder {
    proxy_name: String,
    proxy_host: String,
    proxy_port: u16,
    create_backup: bool,
    insert_at_beginning: bool,
}

#[allow(dead_code)]
impl MergerBridgeBuilder {
    /// Create a new builder (with default values)
    pub fn new() -> Self {
        Self {
            proxy_name: "Local-Chain-Proxy".to_string(),
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 10808,
            create_backup: true,
            insert_at_beginning: true,
        }
    }

    /// Set the proxy name
    pub fn proxy_name(mut self, name: impl Into<String>) -> Self {
        self.proxy_name = name.into();
        self
    }

    /// Set the proxy host
    pub fn proxy_host(mut self, host: impl Into<String>) -> Self {
        self.proxy_host = host.into();
        self
    }

    /// Set the proxy port
    pub fn proxy_port(mut self, port: u16) -> Self {
        self.proxy_port = port;
        self
    }

    /// Set whether to create a backup
    pub fn create_backup(mut self, create: bool) -> Self {
        self.create_backup = create;
        self
    }

    /// Set whether to insert at the beginning
    pub fn insert_at_beginning(mut self, insert: bool) -> Self {
        self.insert_at_beginning = insert;
        self
    }

    /// Build the MergerBridge
    pub fn build(self) -> MergerBridge {
        let config = MergerConfig {
            proxy_name: self.proxy_name,
            proxy_host: self.proxy_host,
            proxy_port: self.proxy_port,
            proxy_username: None,
            proxy_password: None,
            create_backup: self.create_backup,
            insert_at_beginning: self.insert_at_beginning,
            chain_suffix: "-Chain".to_string(),
        };

        MergerBridge::with_config(config)
    }
}

impl Default for MergerBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_config() -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join(format!("test-clash-merge-config-{}.yaml", uuid::Uuid::new_v4()));

        fs::write(
            &config_path,
            r#"
proxies:
  - name: "Test Proxy"
    type: ss
    server: example.com
    port: 443

proxy-groups:
  - name: "Select"
    type: select
    proxies:
      - "Test Proxy"
  - name: "Auto"
    type: url-test
    proxies:
      - "Test Proxy"
"#,
        )
        .unwrap();

        config_path
    }

    #[test]
    fn test_merger_bridge_creation() {
        let bridge = MergerBridge::new();
        assert_eq!(bridge.config().proxy_name, "Local-Chain-Proxy");
        assert_eq!(bridge.config().proxy_port, 10808);
    }

    #[test]
    fn test_merger_bridge_with_config() {
        let config = MergerConfig {
            proxy_name: "Custom-Proxy".to_string(),
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 9999,
            proxy_username: None,
            proxy_password: None,
            create_backup: false,
            insert_at_beginning: false,
            chain_suffix: "-Chain".to_string(),
        };

        let bridge = MergerBridge::with_config(config);
        assert_eq!(bridge.config().proxy_name, "Custom-Proxy");
        assert_eq!(bridge.config().proxy_port, 9999);
        assert!(!bridge.config().create_backup);
    }

    #[test]
    fn test_validate_config() {
        let config_path = create_test_config();
        let bridge = MergerBridge::new();

        let result = bridge.validate_config(&config_path);
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(config_path);
    }

    #[test]
    fn test_validate_nonexistent_config() {
        let bridge = MergerBridge::new();
        let result = bridge.validate_config("/nonexistent/config.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_merge() {
        let config_path = create_test_config();
        let bridge = MergerBridge::new();

        let result = bridge.merge(&config_path);
        assert!(result.is_ok());

        let merge_result = result.unwrap();
        assert!(merge_result.proxy_added); // Should add on first time
        assert!(merge_result.groups_updated > 0); // Should have updated proxy groups

        // Second merge, proxy should already exist
        let result2 = bridge.merge(&config_path);
        assert!(result2.is_ok());
        let merge_result2 = result2.unwrap();
        assert!(!merge_result2.proxy_added); // Should not add duplicates

        // Cleanup
        let _ = fs::remove_file(config_path);
    }

    #[test]
    fn test_merger_bridge_builder() {
        let bridge = MergerBridgeBuilder::new()
            .proxy_name("Custom-Chain")
            .proxy_host("192.168.1.100")
            .proxy_port(8888)
            .create_backup(false)
            .insert_at_beginning(false)
            .build();

        assert_eq!(bridge.config().proxy_name, "Custom-Chain");
        assert_eq!(bridge.config().proxy_host, "192.168.1.100");
        assert_eq!(bridge.config().proxy_port, 8888);
        assert!(!bridge.config().create_backup);
        assert!(!bridge.config().insert_at_beginning);
    }

    #[test]
    fn test_update_config() {
        let mut bridge = MergerBridge::new();
        assert_eq!(bridge.config().proxy_port, 10808);

        let new_config = MergerConfig {
            proxy_name: "Updated-Proxy".to_string(),
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 7777,
            proxy_username: Some("user".to_string()),
            proxy_password: Some("pass".to_string()),
            create_backup: true,
            insert_at_beginning: true,
            chain_suffix: "-Chain".to_string(),
        };

        bridge.update_config(new_config);
        assert_eq!(bridge.config().proxy_name, "Updated-Proxy");
        assert_eq!(bridge.config().proxy_port, 7777);
    }
}
