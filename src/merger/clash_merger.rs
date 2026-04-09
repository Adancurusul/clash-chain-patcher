//! Clash configuration merger — text-based, format-preserving
//!
//! Creates relay chain proxies that route traffic through a local SOCKS5 proxy
//! before reaching the target proxy nodes.
//!
//! Key design: uses serde_yaml for READ-ONLY analysis, but writes back using
//! text manipulation to preserve the original YAML formatting (flow-style,
//! indentation, quoting, comments).

use anyhow::{Context, Result};
use serde_yaml::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Configuration for the merger
#[derive(Debug, Clone)]
pub struct MergerConfig {
    pub proxy_name: String,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub create_backup: bool,
    pub insert_at_beginning: bool,
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
    pub proxy_added: bool,
    pub groups_updated: usize,
    pub chains_created: usize,
    pub backup_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

/// Clash configuration merger (format-preserving)
pub struct ClashConfigMerger {
    config: MergerConfig,
}

impl ClashConfigMerger {
    pub fn new() -> Self {
        Self { config: MergerConfig::default() }
    }

    pub fn with_config(config: MergerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &MergerConfig {
        &self.config
    }

    /// Merge local proxy configuration into Clash config file (format-preserving)
    pub fn merge<P: AsRef<Path>>(&self, config_path: P) -> Result<MergeResult> {
        let config_path = config_path.as_ref();

        if !config_path.exists() {
            anyhow::bail!("Config file does not exist: {}", config_path.display());
        }
        if let Ok(metadata) = fs::metadata(config_path) {
            if metadata.permissions().readonly() {
                anyhow::bail!(
                    "Config file is read-only: {}. Please run: chmod u+w \"{}\"",
                    config_path.display(), config_path.display()
                );
            }
        }

        info!("Starting Clash config merge for: {}", config_path.display());

        // Create single backup (delete old backups first)
        let backup_path = if self.config.create_backup {
            let bp = self.create_single_backup(config_path)?;
            info!("Backup: {}", bp.display());
            Some(bp)
        } else {
            None
        };

        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read: {}", config_path.display()))?;

        let mut result = MergeResult {
            proxy_added: false, groups_updated: 0, chains_created: 0,
            backup_path, warnings: Vec::new(),
        };

        let output = match self.merge_text(&content, &mut result) {
            Ok(out) => out,
            Err(e) => {
                if let Some(ref bp) = result.backup_path {
                    let _ = fs::copy(bp, config_path);
                }
                return Err(e);
            }
        };

        fs::write(config_path, &output)
            .with_context(|| format!("Failed to write: {}", config_path.display()))?;

        info!("Merge completed: proxy_added={}, chains={}, groups={}",
            result.proxy_added, result.chains_created, result.groups_updated);
        Ok(result)
    }

    /// Core text-based merge logic
    fn merge_text(&self, content: &str, result: &mut MergeResult) -> Result<String> {
        // Step 1: Parse with serde_yaml for READ-ONLY structure analysis
        let parsed: Value = serde_yaml::from_str(content)
            .context("YAML parse error")?;
        let config_map = parsed.as_mapping()
            .context("Config root must be a YAML mapping")?;

        let proxy_names = self.get_proxy_names(config_map)?;
        let main_group = self.detect_main_group(config_map);
        if let Some(ref name) = main_group {
            info!("Detected main entry group: {}", name);
        }

        // Step 2: Text-based manipulation
        let mut lines: Vec<String> = content.lines().map(String::from).collect();

        // Find section boundaries
        let proxies_range = Self::find_section_range(&lines, "proxies");
        let groups_range = Self::find_section_range(&lines, "proxy-groups");

        if proxies_range.is_none() || groups_range.is_none() {
            anyhow::bail!("Config must have both 'proxies' and 'proxy-groups' sections");
        }
        let (ps, pe) = proxies_range.unwrap();
        let (gs, ge) = groups_range.unwrap();

        // Detect indent style from original entries
        let indent = Self::detect_indent(&lines, ps);

        // Step 3: Process proxy-groups section (do AFTER proxies since line numbers shift)
        // We process groups first in reverse order to keep proxies indices valid
        let groups_lines = lines[gs + 1..ge].to_vec();
        let groups_entries = Self::split_entries(&groups_lines, &indent);

        let mut new_groups_content: Vec<String> = Vec::new();

        // Add Chain-Selector and Chain-Auto
        let chain_names: Vec<String> = proxy_names.iter()
            .map(|n| format!("'{}{}'", n, self.config.chain_suffix))
            .collect();
        let chain_list = chain_names.join(", ");

        new_groups_content.push(format!(
            "{}- {{ name: Chain-Selector, type: select, proxies: [{}] }}",
            indent, chain_list
        ));
        new_groups_content.push(format!(
            "{}- {{ name: Chain-Auto, type: url-test, proxies: [{}], url: 'http://www.gstatic.com/generate_204', interval: 300, tolerance: 50 }}",
            indent, chain_list
        ));

        // Keep original groups (skip old chain artifacts), modify main group
        for entry in &groups_entries {
            let name = Self::extract_entry_name(&entry.lines);
            if let Some(ref n) = name {
                if n == "Chain-Selector" || n == "Chain-Auto" || n.ends_with(&self.config.chain_suffix) {
                    continue; // skip old chain artifacts
                }
            }
            // If this is the main group, inject Chain-Selector/Chain-Auto into its proxies
            if main_group.as_deref() == name.as_deref() && name.is_some() {
                let modified = self.inject_into_group_proxies(&entry.lines);
                new_groups_content.extend(modified);
            } else {
                new_groups_content.extend(entry.lines.clone());
            }
        }

        // Add relay chain entries
        result.chains_created = proxy_names.len();
        for proxy_name in &proxy_names {
            new_groups_content.push(format!(
                "{}- {{ name: '{}{}', type: relay, proxies: ['{}', {}], benchmark-url: 'http://www.gstatic.com/generate_204', benchmark-timeout: 5 }}",
                indent, proxy_name, self.config.chain_suffix, proxy_name, self.config.proxy_name
            ));
        }

        // Step 4: Process proxies section
        let proxies_lines = lines[ps + 1..pe].to_vec();
        let proxies_entries = Self::split_entries(&proxies_lines, &indent);

        let mut new_proxies_content: Vec<String> = Vec::new();
        for entry in &proxies_entries {
            let name = Self::extract_entry_name(&entry.lines);
            if name.as_deref() == Some(&self.config.proxy_name) {
                continue; // remove old Local-Chain-Proxy
            }
            new_proxies_content.extend(entry.lines.clone());
        }

        // Append new Local-Chain-Proxy
        let proxy_line = self.format_proxy_entry(&indent);
        new_proxies_content.push(proxy_line);
        result.proxy_added = true;

        // Step 5: Rebuild the file
        // We need to replace sections while keeping everything else intact.
        // Process from bottom to top so line indices stay valid.
        // Replace proxy-groups section content
        let groups_content_start = gs + 1;
        let groups_content_end = ge;
        lines.splice(groups_content_start..groups_content_end, new_groups_content);

        // Replace proxies section content (indices still valid because proxies comes before groups)
        let proxies_content_start = ps + 1;
        let proxies_content_end = pe;
        lines.splice(proxies_content_start..proxies_content_end, new_proxies_content);

        result.groups_updated = if main_group.is_some() { 1 } else { 0 };

        // Ensure trailing newline
        let mut output = lines.join("\n");
        if !output.ends_with('\n') {
            output.push('\n');
        }
        Ok(output)
    }

    /// Find the line range [start, end) for a top-level YAML section
    fn find_section_range(lines: &[String], key: &str) -> Option<(usize, usize)> {
        let header = format!("{}:", key);
        let start = lines.iter().position(|l| {
            let t = l.trim_start();
            t == header || t.starts_with(&format!("{}: ", key))
        })?;

        // End is the next top-level key (line starting with a non-space char and containing ':')
        let end = lines[start + 1..].iter().position(|l| {
            !l.is_empty() && !l.starts_with(' ') && !l.starts_with('\t') && l.contains(':')
        }).map(|p| p + start + 1).unwrap_or(lines.len());

        Some((start, end))
    }

    /// Detect indentation from the first entry in a section
    fn detect_indent(lines: &[String], section_start: usize) -> String {
        for line in &lines[section_start + 1..] {
            if let Some(pos) = line.find("- ") {
                return " ".repeat(pos);
            }
        }
        String::new()
    }

    /// Split section content into individual entries
    fn split_entries(section_lines: &[String], indent: &str) -> Vec<TextEntry> {
        let prefix = format!("{}- ", indent);
        let mut entries: Vec<TextEntry> = Vec::new();
        let mut current: Vec<String> = Vec::new();

        for line in section_lines {
            if line.starts_with(&prefix) && !current.is_empty() {
                entries.push(TextEntry { lines: current });
                current = Vec::new();
            }
            if !line.is_empty() || !current.is_empty() {
                current.push(line.clone());
            }
        }
        if !current.is_empty() {
            entries.push(TextEntry { lines: current });
        }
        entries
    }

    /// Extract the `name:` value from an entry's lines
    fn extract_entry_name(entry_lines: &[String]) -> Option<String> {
        for line in entry_lines {
            if let Some(pos) = line.find("name:") {
                let after = line[pos + 5..].trim_start();
                if after.is_empty() { continue; }

                // Handle quoted names
                let first_char = after.chars().next()?;
                if first_char == '\'' || first_char == '"' {
                    let rest = &after[1..];
                    let end = rest.find(first_char)?;
                    return Some(rest[..end].to_string());
                }
                // Unquoted: ends at comma, space+}, or end of line
                let end = after.find(|c: char| c == ',' || c == '}')
                    .unwrap_or(after.len());
                return Some(after[..end].trim().to_string());
            }
        }
        None
    }

    /// Format the Local-Chain-Proxy entry in flow style
    fn format_proxy_entry(&self, indent: &str) -> String {
        let mut parts = vec![
            format!("name: {}", self.config.proxy_name),
            "type: socks5".to_string(),
            format!("server: {}", self.config.proxy_host),
            format!("port: {}", self.config.proxy_port),
        ];
        if let Some(ref u) = self.config.proxy_username {
            if !u.is_empty() { parts.push(format!("username: {}", u)); }
        }
        if let Some(ref p) = self.config.proxy_password {
            if !p.is_empty() { parts.push(format!("password: {}", p)); }
        }
        format!("{}- {{ {} }}", indent, parts.join(", "))
    }

    /// Inject Chain-Selector and Chain-Auto into a group's proxies list
    fn inject_into_group_proxies(&self, entry_lines: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        for line in entry_lines {
            // Flow style: proxies: [xxx, yyy]
            if line.contains("proxies:") && line.contains('[') {
                let replaced = line.replacen("proxies: [", "proxies: [Chain-Selector, Chain-Auto, ", 1);
                result.push(replaced);
            }
            // Block style: detect "proxies:" on its own line, insert before first sub-item
            else if line.trim() == "proxies:" {
                result.push(line.clone());
                // Detect sub-indent from context
                let sub_indent = format!(
                    "{}  ",
                    &line[..line.find("proxies:").unwrap_or(0)]
                );
                result.push(format!("{}- Chain-Selector", sub_indent));
                result.push(format!("{}- Chain-Auto", sub_indent));
            } else {
                result.push(line.clone());
            }
        }
        result
    }

    /// Get list of proxy names from parsed config (read-only)
    fn get_proxy_names(&self, config: &serde_yaml::Mapping) -> Result<Vec<String>> {
        let proxies = match config.get(&Value::String("proxies".to_string())) {
            Some(p) => p,
            None => return Ok(vec![]),
        };
        let proxies_seq = proxies.as_sequence()
            .context("Proxies section must be a sequence")?;

        let mut names = Vec::new();
        for proxy in proxies_seq {
            if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
                if name != self.config.proxy_name && !name.ends_with(&self.config.chain_suffix) {
                    names.push(name.to_string());
                }
            }
        }
        Ok(names)
    }

    /// Detect the main entry group from rules section (read-only)
    fn detect_main_group(&self, config: &serde_yaml::Mapping) -> Option<String> {
        // Skip our own chain groups
        let skip: HashSet<&str> = ["Chain-Selector", "Chain-Auto"].into();

        // Priority 1: MATCH rule
        if let Some(rules) = config.get(&Value::String("rules".to_string())) {
            if let Some(rules_seq) = rules.as_sequence() {
                for rule in rules_seq {
                    if let Some(rule_str) = rule.as_str() {
                        if rule_str.starts_with("MATCH,") {
                            let parts: Vec<&str> = rule_str.split(',').collect();
                            if parts.len() >= 2 {
                                let group = parts[1].trim();
                                if group != "DIRECT" && group != "REJECT" && !skip.contains(group) {
                                    return Some(group.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Priority 2: first select-type group
        if let Some(groups) = config.get(&Value::String("proxy-groups".to_string())) {
            if let Some(seq) = groups.as_sequence() {
                for group in seq {
                    if group.get("type").and_then(|v| v.as_str()) == Some("select") {
                        if let Some(name) = group.get("name").and_then(|v| v.as_str()) {
                            if !name.starts_with("Chain-") && !name.ends_with("-Chain") {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Create a single backup, removing any old backups for this file
    fn create_single_backup<P: AsRef<Path>>(&self, config_path: P) -> Result<PathBuf> {
        let config_path = config_path.as_ref();
        let file_name = config_path.file_name().unwrap().to_string_lossy().to_string();
        let backup_name = format!("{}.backup", file_name);
        let backup_path = config_path.with_file_name(&backup_name);

        // Delete old timestamped backups
        if let Some(parent) = config_path.parent() {
            if let Ok(entries) = fs::read_dir(parent) {
                let prefix = format!("{}.backup", file_name);
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(&prefix) && name != backup_name {
                        let _ = fs::remove_file(entry.path());
                        info!("Removed old backup: {}", name);
                    }
                }
            }
        }

        fs::copy(config_path, &backup_path).context("Failed to create backup")?;
        Ok(backup_path)
    }
}

struct TextEntry {
    lines: Vec<String>,
}

impl Default for ClashConfigMerger {
    fn default() -> Self { Self::new() }
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

    fn create_flow_style_config() -> String {
        r#"mixed-port: 7890
allow-lan: true
mode: rule
log-level: info
proxies:
    - { name: '🇭🇰 香港01', type: ss, server: example.com, port: 443, cipher: aes-256-gcm, password: secret, udp: true }
    - { name: '🇯🇵 日本01', type: ss, server: example.jp, port: 443, cipher: aes-256-gcm, password: secret, udp: true }
    - { name: '🇺🇸 美国01', type: ss, server: example.us, port: 443, cipher: aes-256-gcm, password: secret, udp: true }
proxy-groups:
    - { name: 蓝海加速, type: select, proxies: [自动选择, '🇭🇰 香港01', '🇯🇵 日本01', '🇺🇸 美国01'] }
    - { name: 自动选择, type: url-test, proxies: ['🇭🇰 香港01', '🇯🇵 日本01', '🇺🇸 美国01'], url: 'http://www.gstatic.com/generate_204', interval: 86400 }
rules:
    - 'DOMAIN-SUFFIX,google.com,蓝海加速'
    - 'DOMAIN-SUFFIX,github.com,蓝海加速'
    - 'MATCH,蓝海加速'
"#
        .to_string()
    }

    #[test]
    fn test_merger_config_default() {
        let config = MergerConfig::default();
        assert_eq!(config.proxy_name, "Local-Chain-Proxy");
        assert_eq!(config.proxy_host, "127.0.0.1");
        assert_eq!(config.proxy_port, 10808);
    }

    #[test]
    fn test_merge_creates_chain_relays() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let merger = ClashConfigMerger::new();
        let result = merger.merge(&config_path).unwrap();

        assert!(result.proxy_added);
        assert_eq!(result.chains_created, 2);
        assert_eq!(result.groups_updated, 1);

        let content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_yaml::from_str(&content).unwrap();
        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 3);

        let groups = config["proxy-groups"].as_sequence().unwrap();
        let group_names: Vec<&str> = groups.iter()
            .filter_map(|g| g["name"].as_str()).collect();
        assert!(group_names.contains(&"HK-01-Chain"));
        assert!(group_names.contains(&"JP-01-Chain"));
        assert!(group_names.contains(&"Chain-Selector"));
        assert!(group_names.contains(&"Chain-Auto"));
    }

    #[test]
    fn test_merge_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let merger = ClashConfigMerger::new();
        merger.merge(&config_path).unwrap();
        let content1 = fs::read_to_string(&config_path).unwrap();

        merger.merge(&config_path).unwrap();
        let content2 = fs::read_to_string(&config_path).unwrap();

        // Parse both and compare structure (text may differ slightly due to re-processing)
        let c1: Value = serde_yaml::from_str(&content1).unwrap();
        let c2: Value = serde_yaml::from_str(&content2).unwrap();
        assert_eq!(
            c1["proxies"].as_sequence().unwrap().len(),
            c2["proxies"].as_sequence().unwrap().len()
        );
        assert_eq!(
            c1["proxy-groups"].as_sequence().unwrap().len(),
            c2["proxy-groups"].as_sequence().unwrap().len()
        );
    }

    #[test]
    fn test_flow_style_format_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        let original = create_flow_style_config();
        fs::write(&config_path, &original).unwrap();

        let merger = ClashConfigMerger::with_config(MergerConfig {
            proxy_host: "64.32.179.253".to_string(),
            proxy_port: 60088,
            proxy_username: Some("testuser".to_string()),
            proxy_password: Some("testpass".to_string()),
            create_backup: false,
            ..MergerConfig::default()
        });
        merger.merge(&config_path).unwrap();

        let output = fs::read_to_string(&config_path).unwrap();

        // Original flow-style lines must still be present (format preserved)
        assert!(output.contains("- { name: '🇭🇰 香港01', type: ss,"), "Original proxy format lost");
        assert!(output.contains("- 'DOMAIN-SUFFIX,google.com,蓝海加速'"), "Original rules format lost");

        // Chain content should be added
        assert!(output.contains("Chain-Selector"));
        assert!(output.contains("Chain-Auto"));
        assert!(output.contains("Local-Chain-Proxy"));
        assert!(output.contains("🇭🇰 香港01-Chain"));

        // Main group should have Chain-Selector injected
        assert!(output.contains("proxies: [Chain-Selector, Chain-Auto, 自动选择,"));

        // Verify parseable
        let config: Value = serde_yaml::from_str(&output).unwrap();
        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 4); // 3 original + Local-Chain-Proxy
    }

    #[test]
    fn test_multiple_apply_with_rules_rewrite_no_stacking() {
        use std::collections::HashMap;

        let original_config = r#"mixed-port: 7890
mode: rule
proxies:
    - { name: Trojan-VPS, type: trojan, server: 1.2.3.4, port: 443, password: secret }
proxy-groups:
    - { name: Proxy, type: select, proxies: [Trojan-VPS] }
rules:
    - 'DOMAIN-SUFFIX,claude.ai,Proxy'
    - 'MATCH,Proxy'
"#;
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let merger = ClashConfigMerger::with_config(MergerConfig {
            proxy_host: "64.32.179.253".to_string(),
            proxy_port: 60088,
            create_backup: false,
            ..MergerConfig::default()
        });

        // Apply 3 times with rules rewrite between each
        for _ in 0..3 {
            fs::write(&config_path, if config_path.exists() {
                fs::read_to_string(&config_path).unwrap()
            } else {
                original_config.to_string()
            }).ok();

            merger.merge(&config_path).unwrap();

            // Simulate rules rewrite (text-based)
            let content = fs::read_to_string(&config_path).unwrap();
            let mut replacements = HashMap::new();
            replacements.insert("Proxy".to_string(), "Chain-Selector".to_string());
            let (rewritten, _) = crate::patcher::rewrite_rules_text(&content, &replacements);
            fs::write(&config_path, rewritten).unwrap();
        }

        let final_content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_yaml::from_str(&final_content).unwrap();

        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 2); // Trojan-VPS + Local-Chain-Proxy

        let groups = config["proxy-groups"].as_sequence().unwrap();
        let names: Vec<&str> = groups.iter().filter_map(|g| g["name"].as_str()).collect();
        assert_eq!(names.len(), 4); // Chain-Selector, Chain-Auto, Proxy, Trojan-VPS-Chain

        // Chain-Selector must NOT contain itself
        let selector = groups.iter().find(|g| g["name"].as_str() == Some("Chain-Selector")).unwrap();
        let sel_proxies: Vec<&str> = selector["proxies"].as_sequence().unwrap()
            .iter().filter_map(|p| p.as_str()).collect();
        assert!(!sel_proxies.contains(&"Chain-Selector"));
    }

    #[test]
    fn test_single_backup() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_test_config()).unwrap();

        let merger = ClashConfigMerger::new();
        merger.merge(&config_path).unwrap();
        merger.merge(&config_path).unwrap();
        merger.merge(&config_path).unwrap();

        // Should only have 1 backup file
        let backups: Vec<_> = fs::read_dir(temp_dir.path()).unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("backup"))
            .collect();
        assert_eq!(backups.len(), 1);
    }
}
