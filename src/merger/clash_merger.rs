//! Clash configuration merger — text-based, format-preserving
//!
//! Creates chain proxies that route traffic through a local SOCKS5 proxy
//! before reaching the target proxy nodes.
//!
//! Implementation: clones each proxy node and adds the `dialer-proxy` field
//! pointing at the local SOCKS5 hop. The cloned nodes are named `<name>-Chain`
//! and referenced from `Chain-Selector` / `Chain-Auto` proxy-groups.
//! This replaces the legacy `type: relay` proxy-groups, which Mihomo removed.
//!
//! Key design: uses serde_yaml for READ-ONLY analysis, but writes back using
//! text manipulation to preserve the original YAML formatting (flow-style,
//! indentation, quoting, comments).

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
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

        let proxy_entries = self.get_proxy_entries(config_map)?;
        let proxy_names: Vec<String> = proxy_entries.iter().map(|(n, _)| n.clone()).collect();
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

        // Note: chain entries are no longer relay-type proxy-groups (removed in Mihomo).
        // They are emitted in the proxies section as cloned nodes with `dialer-proxy`.
        result.chains_created = proxy_names.len();

        // Step 4: Process proxies section
        let proxies_lines = lines[ps + 1..pe].to_vec();
        let proxies_entries = Self::split_entries(&proxies_lines, &indent);

        let mut new_proxies_content: Vec<String> = Vec::new();
        for entry in &proxies_entries {
            let name = Self::extract_entry_name(&entry.lines);
            if let Some(ref n) = name {
                // Drop old Local-Chain-Proxy and any prior cloned chain nodes
                if n == &self.config.proxy_name || n.ends_with(&self.config.chain_suffix) {
                    continue;
                }
            }
            new_proxies_content.extend(entry.lines.clone());
        }

        // Append cloned chain proxy nodes with dialer-proxy
        for (name, value) in &proxy_entries {
            let chain_name = format!("{}{}", name, self.config.chain_suffix);
            let cloned = build_cloned_proxy_flow(value, &chain_name, &self.config.proxy_name);
            new_proxies_content.push(format!("{}- {}", indent, cloned));
        }

        // Append new Local-Chain-Proxy
        let proxy_line = self.format_proxy_entry(&indent);
        new_proxies_content.push(proxy_line);
        result.proxy_added = true;

        // Step 5: Rebuild the file
        // Splice the LATER section first so earlier indices stay valid.
        if ps < gs {
            // proxies before groups (common case)
            lines.splice(gs + 1..ge, new_groups_content);
            lines.splice(ps + 1..pe, new_proxies_content);
        } else {
            // groups before proxies (uncommon but valid YAML)
            lines.splice(ps + 1..pe, new_proxies_content);
            lines.splice(gs + 1..ge, new_groups_content);
        }

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
        // Must exclude YAML list entries (starting with "- ") which are section content, not headers
        let end = lines[start + 1..].iter().position(|l| {
            !l.is_empty()
                && !l.starts_with(' ')
                && !l.starts_with('\t')
                && !l.starts_with("- ")
                && l.contains(':')
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
            // Match "name:" as a standalone key, not as part of "hostname:" etc.
            // Valid positions: start of line, after "- ", after "{ ", or after whitespace
            let pos = line.find("name:").and_then(|p| {
                if p == 0 { return Some(p); }
                let before = line.as_bytes()[p - 1];
                if before == b' ' || before == b'-' || before == b'{' || before == b'\t' {
                    Some(p)
                } else {
                    None
                }
            });
            if let Some(pos) = pos {
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

    /// Inject Chain-Selector and Chain-Auto into a group's proxies list.
    /// Idempotent: existing references to Chain-Selector / Chain-Auto are
    /// stripped first so re-applying the merge does not stack duplicates.
    fn inject_into_group_proxies(&self, entry_lines: &[String]) -> Vec<String> {
        // Pass 1: strip any existing block-style "- Chain-Selector" / "- Chain-Auto"
        // and clean inline flow-style proxies: [..] lists of those names.
        let cleaned: Vec<String> = entry_lines.iter().filter_map(|line| {
            let t = line.trim();
            if t == "- Chain-Selector" || t == "- Chain-Auto"
                || t == "- 'Chain-Selector'" || t == "- 'Chain-Auto'"
                || t == "- \"Chain-Selector\"" || t == "- \"Chain-Auto\""
            {
                return None;
            }
            if line.contains("proxies:") && line.contains('[') {
                if let (Some(open), Some(close)) = (line.find('['), line.rfind(']')) {
                    let inner = &line[open + 1..close];
                    let parts: Vec<String> = inner
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()
                            && s != "Chain-Selector" && s != "Chain-Auto"
                            && s != "'Chain-Selector'" && s != "'Chain-Auto'"
                            && s != "\"Chain-Selector\"" && s != "\"Chain-Auto\"")
                        .collect();
                    let new_line = format!("{}[{}]{}", &line[..open], parts.join(", "), &line[close + 1..]);
                    return Some(new_line);
                }
            }
            Some(line.clone())
        }).collect();

        // Pass 2: inject Chain-Selector / Chain-Auto at the top of the proxies list.
        let mut result = Vec::with_capacity(cleaned.len() + 2);
        for (i, line) in cleaned.iter().enumerate() {
            // Flow style: proxies: [xxx, yyy]
            if line.contains("proxies:") && line.contains('[') {
                if let (Some(open), Some(close)) = (line.find('['), line.rfind(']')) {
                    let inner = line[open + 1..close].trim();
                    let new_inner = if inner.is_empty() {
                        "Chain-Selector, Chain-Auto".to_string()
                    } else {
                        format!("Chain-Selector, Chain-Auto, {}", inner)
                    };
                    let new_line = format!("{}[{}]{}", &line[..open], new_inner, &line[close + 1..]);
                    result.push(new_line);
                    continue;
                }
            }
            // Block style: detect "proxies:" on its own line
            if line.trim() == "proxies:" {
                result.push(line.clone());

                // Detect sub-indent from the next existing list entry
                let sub_indent = cleaned[i + 1..]
                    .iter()
                    .find(|l| l.trim().starts_with("- "))
                    .map(|l| {
                        let trimmed = l.trim_start();
                        l[..l.len() - trimmed.len()].to_string()
                    })
                    .unwrap_or_else(|| {
                        let prefix_len = line.find("proxies:").unwrap_or(0);
                        line[..prefix_len].to_string()
                    });

                result.push(format!("{}- Chain-Selector", sub_indent));
                result.push(format!("{}- Chain-Auto", sub_indent));
                continue;
            }
            result.push(line.clone());
        }
        result
    }

    /// Get proxy (name, full Value) pairs from parsed config (read-only).
    /// Skips Local-Chain-Proxy and any previously-cloned chain nodes.
    fn get_proxy_entries(&self, config: &serde_yaml::Mapping) -> Result<Vec<(String, Value)>> {
        let proxies = match config.get(&Value::String("proxies".to_string())) {
            Some(p) => p,
            None => return Ok(vec![]),
        };
        let proxies_seq = proxies.as_sequence()
            .context("Proxies section must be a sequence")?;

        let mut out = Vec::new();
        for proxy in proxies_seq {
            if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
                if name != self.config.proxy_name && !name.ends_with(&self.config.chain_suffix) {
                    out.push((name.to_string(), proxy.clone()));
                }
            }
        }
        Ok(out)
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

/// Build a single-line flow-style YAML mapping for a cloned proxy node,
/// overriding `name` and appending `dialer-proxy: <socks5>`.
fn build_cloned_proxy_flow(original: &Value, new_name: &str, dialer_proxy: &str) -> String {
    let mut ordered = Mapping::new();
    ordered.insert(Value::String("name".to_string()), Value::String(new_name.to_string()));

    if let Some(orig_map) = original.as_mapping() {
        for (k, v) in orig_map.iter() {
            if let Some(ks) = k.as_str() {
                if ks == "name" || ks == "dialer-proxy" {
                    continue;
                }
            }
            ordered.insert(k.clone(), v.clone());
        }
    }

    ordered.insert(
        Value::String("dialer-proxy".to_string()),
        Value::String(dialer_proxy.to_string()),
    );

    yaml_to_flow_string(&Value::Mapping(ordered))
}

/// Serialize a YAML Value as a single-line flow-style string.
/// Handles nested mappings and sequences.
fn yaml_to_flow_string(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => quote_yaml_scalar(s),
        Value::Sequence(seq) => {
            let parts: Vec<String> = seq.iter().map(yaml_to_flow_string).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Mapping(m) => {
            let parts: Vec<String> = m.iter().map(|(k, v)| {
                let key = match k {
                    Value::String(s) => yaml_key_string(s),
                    other => yaml_to_flow_string(other),
                };
                format!("{}: {}", key, yaml_to_flow_string(v))
            }).collect();
            format!("{{ {} }}", parts.join(", "))
        }
        Value::Tagged(t) => yaml_to_flow_string(&t.value),
    }
}

/// Render a YAML mapping key. Most config keys are simple identifiers, possibly
/// with hyphens (e.g. `dialer-proxy`, `reality-opts`); quote only if needed.
fn yaml_key_string(s: &str) -> String {
    let safe = !s.is_empty()
        && !s.starts_with(' ')
        && !s.ends_with(' ')
        && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');
    if safe {
        s.to_string()
    } else {
        quote_yaml_scalar(s)
    }
}

/// Quote a scalar string for flow-style YAML output if it contains characters
/// that would break parsing or could be misinterpreted as another type.
fn quote_yaml_scalar(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.starts_with(|c: char| matches!(c, '!' | '&' | '*' | '?' | '|' | '>' | '%' | '@' | '`' | '[' | ']' | '{' | '}' | '#' | ',' | '\'' | '"'))
        || s.contains(|c: char| matches!(c, ':' | ',' | '{' | '}' | '[' | ']' | '#' | '\n' | '\t' | '\'' | '"' | '`'))
        || matches!(s.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~")
        || s.parse::<f64>().is_ok();
    if needs_quote {
        format!("'{}'", s.replace('\'', "''"))
    } else {
        s.to_string()
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
    fn test_merge_creates_chain_proxy_clones() {
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

        // proxies: 2 originals + 2 clones + 1 Local-Chain-Proxy = 5
        let proxies = config["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 5);

        // Cloned chain nodes must exist in proxies and carry dialer-proxy
        let proxy_names: Vec<&str> = proxies.iter()
            .filter_map(|p| p["name"].as_str()).collect();
        assert!(proxy_names.contains(&"HK-01-Chain"));
        assert!(proxy_names.contains(&"JP-01-Chain"));

        let chain = proxies.iter()
            .find(|p| p["name"].as_str() == Some("HK-01-Chain")).unwrap();
        assert_eq!(chain["dialer-proxy"].as_str(), Some("Local-Chain-Proxy"));
        assert_eq!(chain["type"].as_str(), Some("ss")); // type preserved from original

        // proxy-groups: only Chain-Selector, Chain-Auto, Proxy, Auto. No relay groups.
        let groups = config["proxy-groups"].as_sequence().unwrap();
        let group_names: Vec<&str> = groups.iter()
            .filter_map(|g| g["name"].as_str()).collect();
        assert!(group_names.contains(&"Chain-Selector"));
        assert!(group_names.contains(&"Chain-Auto"));
        // No more relay-type chain groups
        assert!(!group_names.contains(&"HK-01-Chain"));
        assert!(!group_names.contains(&"JP-01-Chain"));
        // No group should be type: relay
        for g in groups {
            assert_ne!(g["type"].as_str(), Some("relay"),
                "relay-type proxy-groups must not be emitted");
        }
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
        // 3 originals + 3 cloned chain nodes + 1 Local-Chain-Proxy
        assert_eq!(proxies.len(), 7);

        // Each clone must carry dialer-proxy pointing at Local-Chain-Proxy
        let clones: Vec<&Value> = proxies.iter()
            .filter(|p| p["name"].as_str().is_some_and(|n| n.ends_with("-Chain")))
            .collect();
        assert_eq!(clones.len(), 3);
        for c in &clones {
            assert_eq!(c["dialer-proxy"].as_str(), Some("Local-Chain-Proxy"));
        }
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
        // Trojan-VPS + Trojan-VPS-Chain (clone) + Local-Chain-Proxy
        assert_eq!(proxies.len(), 3);

        let groups = config["proxy-groups"].as_sequence().unwrap();
        let names: Vec<&str> = groups.iter().filter_map(|g| g["name"].as_str()).collect();
        // Chain-Selector, Chain-Auto, Proxy (no more relay groups)
        assert_eq!(names.len(), 3);
        assert!(!names.contains(&"Trojan-VPS-Chain")); // no longer a group, now a proxy

        // Chain-Selector must NOT contain itself
        let selector = groups.iter().find(|g| g["name"].as_str() == Some("Chain-Selector")).unwrap();
        let sel_proxies: Vec<&str> = selector["proxies"].as_sequence().unwrap()
            .iter().filter_map(|p| p.as_str()).collect();
        assert!(!sel_proxies.contains(&"Chain-Selector"));

        // Proxy group should reference Chain-Selector / Chain-Auto only ONCE each
        // (idempotent re-apply must not duplicate after dedup pass).
        let proxy_grp = groups.iter().find(|g| g["name"].as_str() == Some("Proxy")).unwrap();
        let p_proxies: Vec<&str> = proxy_grp["proxies"].as_sequence().unwrap()
            .iter().filter_map(|p| p.as_str()).collect();
        assert_eq!(p_proxies.iter().filter(|n| **n == "Chain-Selector").count(), 1);
        assert_eq!(p_proxies.iter().filter(|n| **n == "Chain-Auto").count(), 1);
    }

    #[test]
    fn test_no_relay_type_anywhere() {
        // Regression: new Mihomo removed `type: relay`. Output must not contain it.
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, create_flow_style_config()).unwrap();

        let merger = ClashConfigMerger::with_config(MergerConfig {
            create_backup: false,
            ..MergerConfig::default()
        });
        merger.merge(&config_path).unwrap();

        let output = fs::read_to_string(&config_path).unwrap();
        assert!(!output.contains("type: relay"),
            "Output must not contain `type: relay` (removed by Mihomo)");

        // Output must contain dialer-proxy entries for the chain clones
        assert!(output.contains("dialer-proxy: Local-Chain-Proxy"));
    }

    #[test]
    fn test_block_style_inject_dedup_on_reapply() {
        // Block-style proxies: list must not stack Chain-Selector / Chain-Auto on re-apply.
        let original = r#"proxies:
  - name: HK-01
    type: ss
    server: example.com
    port: 443
    cipher: aes-256-gcm
    password: secret
proxy-groups:
  - name: Proxy
    type: select
    proxies:
      - HK-01
rules:
  - MATCH,Proxy
"#;
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, original).unwrap();

        let merger = ClashConfigMerger::with_config(MergerConfig {
            create_backup: false,
            ..MergerConfig::default()
        });

        for _ in 0..3 {
            merger.merge(&config_path).unwrap();
        }

        let final_content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_yaml::from_str(&final_content).unwrap();

        let proxy_grp = config["proxy-groups"].as_sequence().unwrap()
            .iter().find(|g| g["name"].as_str() == Some("Proxy")).unwrap();
        let p_list: Vec<&str> = proxy_grp["proxies"].as_sequence().unwrap()
            .iter().filter_map(|p| p.as_str()).collect();
        assert_eq!(p_list.iter().filter(|n| **n == "Chain-Selector").count(), 1);
        assert_eq!(p_list.iter().filter(|n| **n == "Chain-Auto").count(), 1);
    }

    #[test]
    fn test_clone_preserves_nested_options() {
        // reality-opts (nested mapping) and other fields must round-trip into the clone.
        let original = r#"proxies:
  - name: HK-01
    type: vless
    server: cdn.example.com
    port: 30394
    uuid: 4807c050-5b46-4910-a101-73cc059079b9
    udp: true
    network: tcp
    tls: true
    flow: xtls-rprx-vision
    servername: www.apple.com
    client-fingerprint: chrome
    reality-opts:
      public-key: VoA8WXthSh6CroV4pHK7V6L5gYUw8MHTo2kIVknTXSE
      short-id: 693b25e06841ac7f
proxy-groups:
  - name: Proxy
    type: select
    proxies:
      - HK-01
rules:
  - MATCH,Proxy
"#;
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, original).unwrap();

        let merger = ClashConfigMerger::with_config(MergerConfig {
            create_backup: false,
            ..MergerConfig::default()
        });
        merger.merge(&config_path).unwrap();

        let final_content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_yaml::from_str(&final_content).unwrap();
        let proxies = config["proxies"].as_sequence().unwrap();
        let clone = proxies.iter()
            .find(|p| p["name"].as_str() == Some("HK-01-Chain")).unwrap();

        assert_eq!(clone["type"].as_str(), Some("vless"));
        assert_eq!(clone["server"].as_str(), Some("cdn.example.com"));
        assert_eq!(clone["port"].as_u64(), Some(30394));
        assert_eq!(clone["udp"].as_bool(), Some(true));
        assert_eq!(clone["dialer-proxy"].as_str(), Some("Local-Chain-Proxy"));
        // Nested mapping preserved
        let reality = clone["reality-opts"].as_mapping().unwrap();
        assert_eq!(reality.get("public-key").and_then(|v| v.as_str()),
            Some("VoA8WXthSh6CroV4pHK7V6L5gYUw8MHTo2kIVknTXSE"));
        assert_eq!(reality.get("short-id").and_then(|v| v.as_str()),
            Some("693b25e06841ac7f"));
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
