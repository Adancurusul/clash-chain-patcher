//! Clash configuration merger implementation
//!
//! Automatically adds Local-Chain-Proxy node to Clash configuration
//! and inserts it into all select-type proxy groups.

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Configuration for the merger
#[derive(Debug, Clone)]
pub struct MergerConfig {
    /// Name of the local proxy node (default: "Local-Chain-Proxy")
    pub proxy_name: String,

    /// Local proxy server host (default: "127.0.0.1")
    pub proxy_host: String,

    /// Local proxy server port (default: 10808)
    pub proxy_port: u16,

    /// Whether to create backups (default: true)
    pub create_backup: bool,

    /// Whether to insert proxy at the beginning of groups (default: true)
    pub insert_at_beginning: bool,
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self {
            proxy_name: "Local-Chain-Proxy".to_string(),
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 10808,
            create_backup: true,
            insert_at_beginning: true,
        }
    }
}

/// Result of a merge operation
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// Whether the proxy node was added (false if it already existed)
    pub proxy_added: bool,

    /// Number of proxy groups the proxy was added to
    pub groups_updated: usize,

    /// Path to the backup file (if created)
    pub backup_path: Option<PathBuf>,

    /// Any warnings encountered during merge
    pub warnings: Vec<String>,
}

/// Clash configuration merger
///
/// Manages the process of adding a local SOCKS5 proxy node to Clash
/// configuration and updating all select-type proxy groups to include it.
pub struct ClashConfigMerger {
    config: MergerConfig,
}

impl ClashConfigMerger {
    /// Create a new merger with default configuration
    pub fn new() -> Self {
        Self {
            config: MergerConfig::default(),
        }
    }

    /// Create a new merger with custom configuration
    pub fn with_config(config: MergerConfig) -> Self {
        Self { config }
    }

    /// Merge local proxy configuration into Clash config file
    ///
    /// This will:
    /// 1. Create a backup of the original file (if enabled)
    /// 2. Add the local proxy node to the proxies list (if not exists)
    /// 3. Add the proxy to all select-type proxy groups
    ///
    /// Returns a MergeResult with details about what was changed.
    pub fn merge<P: AsRef<Path>>(&self, config_path: P) -> Result<MergeResult> {
        let config_path = config_path.as_ref();

        // Verify file exists
        if !config_path.exists() {
            anyhow::bail!("Config file does not exist: {}", config_path.display());
        }

        info!("Starting Clash config merge for: {}", config_path.display());

        // Create backup
        let backup_path = if self.config.create_backup {
            let backup_path = self.create_backup(config_path)?;
            info!("Backup created: {}", backup_path.display());
            Some(backup_path)
        } else {
            None
        };

        // Read and parse config
        let content = fs::read_to_string(config_path)
            .context("Failed to read config file")?;

        let mut config: Value = serde_yaml::from_str(&content)
            .context("Failed to parse YAML config")?;

        let mut result = MergeResult {
            proxy_added: false,
            groups_updated: 0,
            backup_path,
            warnings: Vec::new(),
        };

        // Merge the configuration
        if let Err(e) = self.merge_config(&mut config, &mut result) {
            // If merge fails and we have a backup, restore it
            if let Some(ref backup_path) = result.backup_path {
                warn!("Merge failed, restoring backup");
                fs::copy(backup_path, config_path)
                    .context("Failed to restore backup")?;
            }
            return Err(e);
        }

        // Write back to file
        let yaml_string = serde_yaml::to_string(&config)
            .context("Failed to serialize config")?;

        fs::write(config_path, yaml_string)
            .context("Failed to write config file")?;

        info!(
            "Merge completed: proxy_added={}, groups_updated={}",
            result.proxy_added, result.groups_updated
        );

        Ok(result)
    }

    /// Internal method to perform the actual merge logic
    fn merge_config(&self, config: &mut Value, result: &mut MergeResult) -> Result<()> {
        // Ensure config is a mapping
        let config_map = config.as_mapping_mut()
            .context("Config root must be a YAML mapping")?;

        // Add proxy node
        result.proxy_added = self.add_proxy_node(config_map)?;

        // Add to proxy groups
        result.groups_updated = self.add_to_proxy_groups(config_map, result)?;

        Ok(())
    }

    /// Add the local proxy node to the proxies list
    fn add_proxy_node(&self, config: &mut Mapping) -> Result<bool> {
        // Ensure proxies section exists
        if !config.contains_key(&Value::String("proxies".to_string())) {
            config.insert(
                Value::String("proxies".to_string()),
                Value::Sequence(vec![]),
            );
        }

        let proxies = config
            .get_mut(&Value::String("proxies".to_string()))
            .context("Failed to get proxies section")?;

        let proxies_seq = proxies.as_sequence_mut()
            .context("Proxies section must be a sequence")?;

        // Check if proxy already exists
        for proxy in proxies_seq.iter() {
            if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
                if name == self.config.proxy_name {
                    debug!("Proxy '{}' already exists", self.config.proxy_name);
                    return Ok(false);
                }
            }
        }

        // Create the proxy node
        let mut proxy_node = Mapping::new();
        proxy_node.insert(
            Value::String("name".to_string()),
            Value::String(self.config.proxy_name.clone()),
        );
        proxy_node.insert(
            Value::String("type".to_string()),
            Value::String("socks5".to_string()),
        );
        proxy_node.insert(
            Value::String("server".to_string()),
            Value::String(self.config.proxy_host.clone()),
        );
        proxy_node.insert(
            Value::String("port".to_string()),
            Value::Number(self.config.proxy_port.into()),
        );

        proxies_seq.push(Value::Mapping(proxy_node));

        info!("Added proxy node: {}", self.config.proxy_name);
        Ok(true)
    }

    /// Add the local proxy to all select-type proxy groups
    fn add_to_proxy_groups(&self, config: &mut Mapping, result: &mut MergeResult) -> Result<usize> {
        // Check if proxy-groups section exists
        if !config.contains_key(&Value::String("proxy-groups".to_string())) {
            result.warnings.push("No proxy-groups section found in config".to_string());
            return Ok(0);
        }

        let proxy_groups = config
            .get_mut(&Value::String("proxy-groups".to_string()))
            .context("Failed to get proxy-groups section")?;

        let groups_seq = proxy_groups.as_sequence_mut()
            .context("Proxy-groups section must be a sequence")?;

        let mut updated_count = 0;

        for group in groups_seq.iter_mut() {
            let group_map = match group.as_mapping_mut() {
                Some(m) => m,
                None => {
                    result.warnings.push("Invalid proxy group format".to_string());
                    continue;
                }
            };

            // Check if this is a select-type group
            let group_type = match group_map.get(&Value::String("type".to_string())) {
                Some(Value::String(t)) => t.as_str(),
                _ => continue,
            };

            if group_type != "select" {
                continue;
            }

            let group_name = group_map
                .get(&Value::String("name".to_string()))
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string();

            // Ensure proxies array exists
            if !group_map.contains_key(&Value::String("proxies".to_string())) {
                group_map.insert(
                    Value::String("proxies".to_string()),
                    Value::Sequence(vec![]),
                );
            }

            let group_proxies = group_map
                .get_mut(&Value::String("proxies".to_string()))
                .context("Failed to get group proxies")?;

            let group_proxies_seq = group_proxies.as_sequence_mut()
                .context("Group proxies must be a sequence")?;

            // Check if proxy already in group
            let already_exists = group_proxies_seq.iter().any(|p| {
                p.as_str() == Some(&self.config.proxy_name)
            });

            if already_exists {
                debug!("Proxy already in group: {}", group_name);
                continue;
            }

            // Add proxy to group
            let proxy_value = Value::String(self.config.proxy_name.clone());
            if self.config.insert_at_beginning {
                group_proxies_seq.insert(0, proxy_value);
            } else {
                group_proxies_seq.push(proxy_value);
            }

            info!("Added proxy to group: {}", group_name);
            updated_count += 1;
        }

        Ok(updated_count)
    }

    /// Create a backup of the config file
    fn create_backup<P: AsRef<Path>>(&self, config_path: P) -> Result<PathBuf> {
        let config_path = config_path.as_ref();
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup_path = config_path.with_file_name(format!(
            "{}.backup-{}",
            config_path.file_name().unwrap().to_string_lossy(),
            timestamp
        ));

        fs::copy(config_path, &backup_path)
            .context("Failed to create backup")?;

        Ok(backup_path)
    }

    /// Get the merger configuration
    pub fn config(&self) -> &MergerConfig {
        &self.config
    }
}

impl Default for ClashConfigMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> String {
        r#"
proxies:
  - name: "Proxy 1"
    type: ss
    server: example.com
    port: 443

proxy-groups:
  - name: "Auto"
    type: select
    proxies:
      - "Proxy 1"
  - name: "Manual"
    type: select
    proxies:
      - "Proxy 1"
  - name: "Fallback"
    type: fallback
    proxies:
      - "Proxy 1"
"#
        .to_string()
    }

    #[test]
    fn test_merger_config_default() {
        let config = MergerConfig::default();
        assert_eq!(config.proxy_name, "Local-Chain-Proxy");
        assert_eq!(config.proxy_host, "127.0.0.1");
        assert_eq!(config.proxy_port, 10808);
        assert!(config.create_backup);
        assert!(config.insert_at_beginning);
    }

    #[test]
    fn test_merger_creation() {
        let merger = ClashConfigMerger::new();
        assert_eq!(merger.config().proxy_name, "Local-Chain-Proxy");

        let custom_config = MergerConfig {
            proxy_name: "Custom-Proxy".to_string(),
            proxy_port: 9999,
            ..Default::default()
        };

        let merger = ClashConfigMerger::with_config(custom_config);
        assert_eq!(merger.config().proxy_name, "Custom-Proxy");
        assert_eq!(merger.config().proxy_port, 9999);
    }

    #[test]
    fn test_merge_nonexistent_file() {
        let merger = ClashConfigMerger::new();
        let result = merger.merge("/nonexistent/file.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let merger = ClashConfigMerger::new();
        let result = merger.merge(&config_path).unwrap();

        assert!(result.proxy_added);
        assert_eq!(result.groups_updated, 2); // Only select-type groups
        assert!(result.backup_path.is_some());
        assert!(result.warnings.is_empty());

        // Verify the merge
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Local-Chain-Proxy"));

        let config: Value = serde_yaml::from_str(&content).unwrap();
        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 2); // Original + Local-Chain-Proxy

        // Check Auto group
        let auto_group = &config["proxy-groups"][0];
        let auto_proxies = auto_group["proxies"].as_sequence().unwrap();
        assert_eq!(auto_proxies[0].as_str().unwrap(), "Local-Chain-Proxy");
    }

    #[test]
    fn test_merge_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let merger = ClashConfigMerger::new();

        // First merge
        let result1 = merger.merge(&config_path).unwrap();
        assert!(result1.proxy_added);
        assert_eq!(result1.groups_updated, 2);

        // Second merge (should be no-op)
        let result2 = merger.merge(&config_path).unwrap();
        assert!(!result2.proxy_added);
        assert_eq!(result2.groups_updated, 0);
    }

    #[test]
    fn test_merge_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let custom_config = MergerConfig {
            proxy_name: "My-Custom-Proxy".to_string(),
            proxy_port: 7777,
            create_backup: false,
            insert_at_beginning: false,
            ..Default::default()
        };

        let merger = ClashConfigMerger::with_config(custom_config);
        let result = merger.merge(&config_path).unwrap();

        assert!(result.proxy_added);
        assert!(result.backup_path.is_none());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("My-Custom-Proxy"));
        assert!(content.contains("7777"));

        let config: Value = serde_yaml::from_str(&content).unwrap();
        let auto_group = &config["proxy-groups"][0];
        let auto_proxies = auto_group["proxies"].as_sequence().unwrap();

        // Should be at the end, not beginning
        assert_eq!(
            auto_proxies.last().unwrap().as_str().unwrap(),
            "My-Custom-Proxy"
        );
    }

    #[test]
    fn test_merge_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, "{}").unwrap();

        let merger = ClashConfigMerger::new();
        let result = merger.merge(&config_path).unwrap();

        assert!(result.proxy_added);
        assert_eq!(result.groups_updated, 0);
        assert_eq!(result.warnings.len(), 1);

        let content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_yaml::from_str(&content).unwrap();
        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 1);
    }
}
