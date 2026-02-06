//! Clash configuration merger implementation
//!
//! Creates relay chain proxies that route traffic through a local SOCKS5 proxy
//! before reaching the target proxy nodes.
//!
//! Example chain: User -> Clash -> Local SOCKS5 (127.0.0.1:10808) -> Target Node (香港 01)

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

    /// SOCKS5 username (optional)
    pub proxy_username: Option<String>,

    /// SOCKS5 password (optional)
    pub proxy_password: Option<String>,

    /// Whether to create backups (default: true)
    pub create_backup: bool,

    /// Whether to insert proxy at the beginning of groups (default: true)
    pub insert_at_beginning: bool,

    /// Suffix for chain proxy names (default: "-Chain")
    pub chain_suffix: String,
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self {
            proxy_name: "Local-Chain-Proxy".to_string(),
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 10808,
            proxy_username: None,
            proxy_password: None,
            create_backup: true,
            insert_at_beginning: true,
            chain_suffix: "-Chain".to_string(),
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

    /// Number of chain relay proxies created
    pub chains_created: usize,

    /// Path to the backup file (if created)
    pub backup_path: Option<PathBuf>,

    /// Any warnings encountered during merge
    pub warnings: Vec<String>,
}

/// Clash configuration merger
///
/// Creates relay chain proxies that route through a local SOCKS5 proxy.
/// This allows traffic to go: Local SOCKS5 -> Target Proxy Node
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
    /// 3. Create relay chain proxies for each existing proxy
    /// 4. Add the chain proxies to select-type proxy groups
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
            chains_created: 0,
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
            "Merge completed: proxy_added={}, chains_created={}, groups_updated={}",
            result.proxy_added, result.chains_created, result.groups_updated
        );

        Ok(result)
    }

    /// Internal method to perform the actual merge logic
    fn merge_config(&self, config: &mut Value, result: &mut MergeResult) -> Result<()> {
        // Ensure config is a mapping
        let config_map = config.as_mapping_mut()
            .context("Config root must be a YAML mapping")?;

        // Add local proxy node
        result.proxy_added = self.add_proxy_node(config_map)?;

        // Get list of existing proxy names (before adding chains)
        let existing_proxies = self.get_proxy_names(config_map)?;

        // Detect the main entry group from rules
        let main_group = self.detect_main_group(config_map);
        if let Some(ref name) = main_group {
            info!("Detected main entry group: {}", name);
        }

        // Create relay chain proxies for each existing proxy
        // This also creates Chain-Selector group
        result.chains_created = self.create_chain_proxies(config_map, &existing_proxies, result)?;

        // Add Chain-Selector and Chain-Auto to the main entry group
        result.groups_updated = self.add_selector_to_main_group(config_map, main_group.as_deref(), result)?;

        Ok(())
    }

    /// Detect the main entry group from rules section
    /// Priority: 1. MATCH rule (default route), 2. First proxy-group of select type
    fn detect_main_group(&self, config: &Mapping) -> Option<String> {
        // First, try to find MATCH rule (the default/fallback route)
        if let Some(rules) = config.get(&Value::String("rules".to_string())) {
            if let Some(rules_seq) = rules.as_sequence() {
                for rule in rules_seq {
                    if let Some(rule_str) = rule.as_str() {
                        // MATCH rule format: "MATCH,GROUP"
                        if rule_str.starts_with("MATCH,") {
                            let parts: Vec<&str> = rule_str.split(',').collect();
                            if parts.len() >= 2 {
                                let group = parts[1].trim();
                                if group != "DIRECT" && group != "REJECT" {
                                    return Some(group.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: find first select-type proxy group
        if let Some(proxy_groups) = config.get(&Value::String("proxy-groups".to_string())) {
            if let Some(groups_seq) = proxy_groups.as_sequence() {
                for group in groups_seq {
                    if let Some(group_type) = group.get("type").and_then(|v| v.as_str()) {
                        if group_type == "select" {
                            if let Some(name) = group.get("name").and_then(|v| v.as_str()) {
                                // Skip our own chain groups
                                if !name.starts_with("Chain-") && !name.ends_with("-Chain") {
                                    return Some(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Get list of proxy names from config
    fn get_proxy_names(&self, config: &Mapping) -> Result<Vec<String>> {
        let proxies = match config.get(&Value::String("proxies".to_string())) {
            Some(p) => p,
            None => return Ok(vec![]),
        };

        let proxies_seq = proxies.as_sequence()
            .context("Proxies section must be a sequence")?;

        let mut names = Vec::new();
        for proxy in proxies_seq {
            if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
                // Skip our own local proxy and any existing chain proxies
                if name != self.config.proxy_name && !name.ends_with(&self.config.chain_suffix) {
                    names.push(name.to_string());
                }
            }
        }

        Ok(names)
    }

    /// Add or update the local proxy node in the proxies list
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

        // Check if proxy already exists and update it
        for proxy in proxies_seq.iter_mut() {
            if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
                if name == self.config.proxy_name {
                    // Update existing proxy with new configuration
                    if let Some(proxy_map) = proxy.as_mapping_mut() {
                        proxy_map.insert(
                            Value::String("server".to_string()),
                            Value::String(self.config.proxy_host.clone()),
                        );
                        proxy_map.insert(
                            Value::String("port".to_string()),
                            Value::Number(self.config.proxy_port.into()),
                        );
                        // Update or remove username
                        if let Some(ref username) = self.config.proxy_username {
                            if !username.is_empty() {
                                proxy_map.insert(
                                    Value::String("username".to_string()),
                                    Value::String(username.clone()),
                                );
                            } else {
                                proxy_map.remove(&Value::String("username".to_string()));
                            }
                        } else {
                            proxy_map.remove(&Value::String("username".to_string()));
                        }
                        // Update or remove password
                        if let Some(ref password) = self.config.proxy_password {
                            if !password.is_empty() {
                                proxy_map.insert(
                                    Value::String("password".to_string()),
                                    Value::String(password.clone()),
                                );
                            } else {
                                proxy_map.remove(&Value::String("password".to_string()));
                            }
                        } else {
                            proxy_map.remove(&Value::String("password".to_string()));
                        }
                        info!("Updated existing proxy: {} ({}:{})",
                            self.config.proxy_name,
                            self.config.proxy_host,
                            self.config.proxy_port
                        );
                    }
                    return Ok(false); // Not added, but updated
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

        // Add username/password if provided
        if let Some(ref username) = self.config.proxy_username {
            if !username.is_empty() {
                proxy_node.insert(
                    Value::String("username".to_string()),
                    Value::String(username.clone()),
                );
            }
        }
        if let Some(ref password) = self.config.proxy_password {
            if !password.is_empty() {
                proxy_node.insert(
                    Value::String("password".to_string()),
                    Value::String(password.clone()),
                );
            }
        }

        proxies_seq.push(Value::Mapping(proxy_node));

        info!("Added proxy node: {} ({}:{})",
            self.config.proxy_name,
            self.config.proxy_host,
            self.config.proxy_port
        );
        Ok(true)
    }

    /// Create relay chain proxy groups for each existing proxy
    fn create_chain_proxies(
        &self,
        config: &mut Mapping,
        existing_proxies: &[String],
        _result: &mut MergeResult,
    ) -> Result<usize> {
        // Ensure proxy-groups section exists
        if !config.contains_key(&Value::String("proxy-groups".to_string())) {
            config.insert(
                Value::String("proxy-groups".to_string()),
                Value::Sequence(vec![]),
            );
        }

        let proxy_groups = config
            .get_mut(&Value::String("proxy-groups".to_string()))
            .context("Failed to get proxy-groups section")?;

        let groups_seq = proxy_groups.as_sequence_mut()
            .context("Proxy-groups section must be a sequence")?;

        // Build set of existing group names
        let existing_group_names: std::collections::HashSet<String> = groups_seq
            .iter()
            .filter_map(|g| g.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let mut chains_created = 0;

        // Create a relay chain for each existing proxy
        for proxy_name in existing_proxies {
            let chain_name = format!("{}{}", proxy_name, self.config.chain_suffix);

            // Skip if chain already exists
            if existing_group_names.contains(&chain_name) {
                debug!("Chain '{}' already exists", chain_name);
                continue;
            }

            // Create relay proxy group
            // Relay format: [first_proxy, second_proxy, ...]
            // Traffic flow: User -> Clash -> first_proxy -> second_proxy -> target
            // We want: User -> Clash -> VPN node -> SOCKS5 proxy -> target
            let mut relay_group = Mapping::new();
            relay_group.insert(
                Value::String("name".to_string()),
                Value::String(chain_name.clone()),
            );
            relay_group.insert(
                Value::String("type".to_string()),
                Value::String("relay".to_string()),
            );
            relay_group.insert(
                Value::String("proxies".to_string()),
                Value::Sequence(vec![
                    Value::String(proxy_name.clone()),            // First: VPN node (香港01 etc)
                    Value::String(self.config.proxy_name.clone()), // Second: SOCKS5 proxy
                ]),
            );
            // Add benchmark URL for latency testing (required for relay groups)
            relay_group.insert(
                Value::String("benchmark-url".to_string()),
                Value::String("http://www.gstatic.com/generate_204".to_string()),
            );
            relay_group.insert(
                Value::String("benchmark-timeout".to_string()),
                Value::Number(5.into()),
            );

            groups_seq.push(Value::Mapping(relay_group));
            info!("Created chain relay: {} -> {}", proxy_name, self.config.proxy_name);
            chains_created += 1;
        }

        // Create Chain-Auto and Chain-Selector groups
        // Chain-Auto: url-test type, auto select fastest chain
        // Chain-Selector: select type, manual selection
        if !existing_proxies.is_empty() {
            let auto_name = "Chain-Auto";
            let selector_name = "Chain-Selector";

            // Collect all chain names (both existing and newly created)
            let chain_names: Vec<Value> = existing_proxies
                .iter()
                .map(|name| Value::String(format!("{}{}", name, self.config.chain_suffix)))
                .collect();

            // Remove existing Chain-Auto and Chain-Selector if present
            groups_seq.retain(|g| {
                g.get("name")
                    .and_then(|v| v.as_str())
                    .map(|n| n != auto_name && n != selector_name)
                    .unwrap_or(true)
            });

            // Create Chain-Auto (url-test) - auto select fastest
            let mut auto_group = Mapping::new();
            auto_group.insert(
                Value::String("name".to_string()),
                Value::String(auto_name.to_string()),
            );
            auto_group.insert(
                Value::String("type".to_string()),
                Value::String("url-test".to_string()),
            );
            auto_group.insert(
                Value::String("proxies".to_string()),
                Value::Sequence(chain_names.clone()),
            );
            auto_group.insert(
                Value::String("url".to_string()),
                Value::String("http://www.gstatic.com/generate_204".to_string()),
            );
            auto_group.insert(
                Value::String("interval".to_string()),
                Value::Number(300.into()),
            );
            auto_group.insert(
                Value::String("tolerance".to_string()),
                Value::Number(50.into()),
            );

            // Create Chain-Selector (select) - manual selection
            let mut selector_group = Mapping::new();
            selector_group.insert(
                Value::String("name".to_string()),
                Value::String(selector_name.to_string()),
            );
            selector_group.insert(
                Value::String("type".to_string()),
                Value::String("select".to_string()),
            );
            selector_group.insert(
                Value::String("proxies".to_string()),
                Value::Sequence(chain_names),
            );

            // Insert at the beginning: Chain-Selector first, then Chain-Auto
            // So sidebar shows: Chain-Selector, Chain-Auto, ...
            groups_seq.insert(0, Value::Mapping(auto_group));
            groups_seq.insert(0, Value::Mapping(selector_group));
            info!("Created Chain-Selector (select) and Chain-Auto (url-test) at top with {} chain proxies", chains_created);
        }

        Ok(chains_created)
    }

    /// Add Chain-Selector and Chain-Auto to the main entry group
    /// The main group is auto-detected from rules, or falls back to first select-type group
    fn add_selector_to_main_group(
        &self,
        config: &mut Mapping,
        main_group_name: Option<&str>,
        result: &mut MergeResult,
    ) -> Result<usize> {
        let auto_name = "Chain-Auto";
        let selector_name = "Chain-Selector";

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

            // Get group name
            let group_name = group_map
                .get(&Value::String("name".to_string()))
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string();

            // Only process the main entry group (if detected)
            if let Some(main_name) = main_group_name {
                if group_name != main_name {
                    continue;
                }
            } else {
                // No main group detected, skip (chain groups are still created at top)
                continue;
            }

            // Check if this is a select-type group
            let group_type = match group_map.get(&Value::String("type".to_string())) {
                Some(Value::String(t)) => t.as_str(),
                _ => continue,
            };

            if group_type != "select" {
                warn!("Main group '{}' is not select type, skipping", group_name);
                continue;
            }

            // Ensure proxies array exists
            if !group_map.contains_key(&Value::String("proxies".to_string())) {
                continue;
            }

            let group_proxies = group_map
                .get_mut(&Value::String("proxies".to_string()))
                .context("Failed to get group proxies")?;

            let group_proxies_seq = group_proxies.as_sequence_mut()
                .context("Group proxies must be a sequence")?;

            // Check what's already in this group
            let has_selector = group_proxies_seq
                .iter()
                .any(|p| p.as_str() == Some(selector_name));
            let has_auto = group_proxies_seq
                .iter()
                .any(|p| p.as_str() == Some(auto_name));

            // Add Chain-Auto at position 0 (if not exists)
            if !has_auto {
                group_proxies_seq.insert(0, Value::String(auto_name.to_string()));
            }

            // Add Chain-Selector at position 0 (if not exists), so it's before Chain-Auto
            if !has_selector {
                group_proxies_seq.insert(0, Value::String(selector_name.to_string()));
            }

            if !has_selector || !has_auto {
                info!("Added Chain-Selector and Chain-Auto to main group: {}", group_name);
                updated_count += 1;
            }

            // Only process main group, break after found
            break;
        }

        Ok(updated_count)
    }

    /// Add chain proxies to select-type proxy groups (legacy, not currently used)
    #[allow(dead_code)]
    fn add_chains_to_groups(
        &self,
        config: &mut Mapping,
        existing_proxies: &[String],
        result: &mut MergeResult,
    ) -> Result<usize> {
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

        // Build map of original proxy -> chain proxy name
        let chain_map: std::collections::HashMap<String, String> = existing_proxies
            .iter()
            .map(|name| (name.clone(), format!("{}{}", name, self.config.chain_suffix)))
            .collect();

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
                continue;
            }

            let group_proxies = group_map
                .get_mut(&Value::String("proxies".to_string()))
                .context("Failed to get group proxies")?;

            let group_proxies_seq = group_proxies.as_sequence_mut()
                .context("Group proxies must be a sequence")?;

            // Build set of existing proxies in this group
            let existing_in_group: std::collections::HashSet<String> = group_proxies_seq
                .iter()
                .filter_map(|p| p.as_str().map(|s| s.to_string()))
                .collect();

            // For each proxy in this group, add its chain version if not already present
            let mut to_add = Vec::new();
            for proxy_value in group_proxies_seq.iter() {
                if let Some(proxy_name) = proxy_value.as_str() {
                    if let Some(chain_name) = chain_map.get(proxy_name) {
                        if !existing_in_group.contains(chain_name) {
                            to_add.push((proxy_name.to_string(), chain_name.clone()));
                        }
                    }
                }
            }

            if to_add.is_empty() {
                continue;
            }

            // Insert chain proxies right after their original proxies
            let mut new_seq: Vec<Value> = Vec::new();
            for proxy_value in group_proxies_seq.iter() {
                new_seq.push(proxy_value.clone());
                if let Some(proxy_name) = proxy_value.as_str() {
                    for (orig, chain) in &to_add {
                        if orig == proxy_name {
                            new_seq.push(Value::String(chain.clone()));
                        }
                    }
                }
            }

            *group_proxies_seq = new_seq;
            info!("Added {} chain proxies to group: {}", to_add.len(), group_name);
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
  - name: "HK-01"
    type: ss
    server: example.com
    port: 443
    cipher: aes-256-gcm
    password: secret
  - name: "JP-01"
    type: ss
    server: example.jp
    port: 443
    cipher: aes-256-gcm
    password: secret

proxy-groups:
  - name: "Proxy"
    type: select
    proxies:
      - "HK-01"
      - "JP-01"
  - name: "Auto"
    type: url-test
    proxies:
      - "HK-01"
      - "JP-01"
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
        assert_eq!(config.chain_suffix, "-Chain");
    }

    #[test]
    fn test_merge_creates_chain_relays() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let merger = ClashConfigMerger::new();
        let result = merger.merge(&config_path).unwrap();

        assert!(result.proxy_added);
        assert_eq!(result.chains_created, 2); // HK-01-Chain, JP-01-Chain
        assert_eq!(result.groups_updated, 1); // Only select-type group

        // Verify the config
        let content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_yaml::from_str(&content).unwrap();

        // Check proxies
        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 3); // HK-01, JP-01, Local-Chain-Proxy

        // Check proxy-groups for relay chains
        let groups = config["proxy-groups"].as_sequence().unwrap();
        let group_names: Vec<&str> = groups
            .iter()
            .filter_map(|g| g["name"].as_str())
            .collect();

        assert!(group_names.contains(&"HK-01-Chain"));
        assert!(group_names.contains(&"JP-01-Chain"));

        // Verify relay structure: VPN node first, then SOCKS5 proxy
        for group in groups {
            let name = group["name"].as_str().unwrap_or("");
            if name == "HK-01-Chain" {
                assert_eq!(group["type"].as_str().unwrap(), "relay");
                let proxies = group["proxies"].as_sequence().unwrap();
                assert_eq!(proxies[0].as_str().unwrap(), "HK-01"); // First: VPN node
                assert_eq!(proxies[1].as_str().unwrap(), "Local-Chain-Proxy"); // Second: SOCKS5
            }
        }

        // Check that chains were added to select group
        let proxy_group = &groups[0]; // "Proxy" select group
        let group_proxies = proxy_group["proxies"].as_sequence().unwrap();
        let proxy_names: Vec<&str> = group_proxies
            .iter()
            .filter_map(|p| p.as_str())
            .collect();

        assert!(proxy_names.contains(&"HK-01-Chain"));
        assert!(proxy_names.contains(&"JP-01-Chain"));
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
        assert_eq!(result1.chains_created, 2);

        // Second merge (should be no-op)
        let result2 = merger.merge(&config_path).unwrap();
        assert!(!result2.proxy_added);
        assert_eq!(result2.chains_created, 0);
        assert_eq!(result2.groups_updated, 0);
    }
}
