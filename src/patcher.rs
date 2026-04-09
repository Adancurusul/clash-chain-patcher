//! Clash configuration patcher - Core logic
//!
//! This module handles:
//! - Parsing Clash YAML configurations
//! - Creating SOCKS5 proxy nodes
//! - Creating relay (chain) proxy groups
//! - Patching configurations with new chain proxies

use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::collections::{HashMap, HashSet};

/// SOCKS5 proxy configuration
#[derive(Debug, Clone)]
pub struct Socks5Proxy {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Socks5Proxy {
    pub fn new(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        Self {
            name: "SOCKS5-FixedIP".to_string(),
            host,
            port,
            username,
            password,
        }
    }

    /// Convert to YAML Value for Clash config
    pub fn to_yaml_value(&self) -> Value {
        let mut map = Mapping::new();
        map.insert(Value::String("name".to_string()), Value::String(self.name.clone()));
        map.insert(Value::String("type".to_string()), Value::String("socks5".to_string()));
        map.insert(Value::String("server".to_string()), Value::String(self.host.clone()));
        map.insert(Value::String("port".to_string()), Value::Number(self.port.into()));
        map.insert(Value::String("udp".to_string()), Value::Bool(true));

        if let Some(ref username) = self.username {
            if !username.is_empty() {
                map.insert(Value::String("username".to_string()), Value::String(username.clone()));
            }
        }
        if let Some(ref password) = self.password {
            if !password.is_empty() {
                map.insert(Value::String("password".to_string()), Value::String(password.clone()));
            }
        }

        Value::Mapping(map)
    }
}

/// Patcher options
#[derive(Debug, Clone, Default)]
pub struct PatchOptions {
    pub filter_keywords: Vec<String>,
}

/// Result of patching operation
#[derive(Debug, Clone)]
pub struct PatchResult {
    pub success: bool,
    pub logs: Vec<String>,
    pub output: Option<String>,
    pub relay_names: Vec<String>,
}

/// Parse proxy string in two formats:
/// 1. user:pass@host:port
/// 2. host:port:user:pass
pub fn parse_proxy_string(input: &str) -> Option<Socks5Proxy> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Remove protocol prefix
    let input = input
        .strip_prefix("socks5://")
        .or_else(|| input.strip_prefix("http://"))
        .or_else(|| input.strip_prefix("https://"))
        .unwrap_or(input);

    if input.contains('@') {
        // Format 1: user:pass@host:port
        let parts: Vec<&str> = input.split('@').collect();
        if parts.len() != 2 {
            return None;
        }

        let auth_parts: Vec<&str> = parts[0].split(':').collect();
        let server_parts: Vec<&str> = parts[1].split(':').collect();

        if auth_parts.len() < 2 || server_parts.len() < 2 {
            return None;
        }

        let port = server_parts[1].parse::<u16>().ok()?;

        Some(Socks5Proxy::new(
            server_parts[0].to_string(),
            port,
            Some(auth_parts[0].to_string()),
            Some(auth_parts[1].to_string()),
        ))
    } else {
        // Format 2: host:port:user:pass or host:port
        let parts: Vec<&str> = input.split(':').collect();

        if parts.len() >= 4 {
            let port = parts[1].parse::<u16>().ok()?;
            Some(Socks5Proxy::new(
                parts[0].to_string(),
                port,
                Some(parts[2].to_string()),
                Some(parts[3].to_string()),
            ))
        } else if parts.len() >= 2 {
            let port = parts[1].parse::<u16>().ok()?;
            Some(Socks5Proxy::new(
                parts[0].to_string(),
                port,
                None,
                None,
            ))
        } else {
            None
        }
    }
}

/// Get existing proxy names from config
fn get_existing_proxy_names(config: &Value) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(proxies) = config.get("proxies").and_then(|v| v.as_sequence()) {
        for proxy in proxies {
            if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            }
        }
    }

    names
}

/// Clean proxy name for use in chain group name
fn clean_proxy_name(name: &str) -> String {
    let re = Regex::new(r"[\[\]（）\(\)]").unwrap();
    re.replace_all(name, "").replace(' ', "")
}

/// Create a relay group combining first proxy and SOCKS5 proxy
fn create_relay_group(first_proxy: &str, socks5_name: &str) -> Value {
    let clean_name = clean_proxy_name(first_proxy);
    let group_name = format!("Chain-{}", clean_name);

    let mut map = Mapping::new();
    map.insert(Value::String("name".to_string()), Value::String(group_name));
    map.insert(Value::String("type".to_string()), Value::String("relay".to_string()));

    let proxies = vec![
        Value::String(first_proxy.to_string()),
        Value::String(socks5_name.to_string()),
    ];
    map.insert(Value::String("proxies".to_string()), Value::Sequence(proxies));

    Value::Mapping(map)
}

/// Filter proxies by keywords
fn filter_proxies_by_keywords(proxy_names: &[String], keywords: &[String]) -> Vec<String> {
    if keywords.is_empty() {
        return proxy_names.to_vec();
    }

    proxy_names
        .iter()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            keywords.iter().any(|kw| name_lower.contains(&kw.to_lowercase()))
        })
        .cloned()
        .collect()
}

/// Preview the patch operation without modifying the config
pub fn preview_patch(
    config_content: &str,
    proxy: &Socks5Proxy,
    options: &PatchOptions,
) -> PatchResult {
    let config: Value = match serde_yaml::from_str(config_content) {
        Ok(v) => v,
        Err(e) => {
            return PatchResult {
                success: false,
                logs: vec![format!("[ERROR] YAML parsing failed: {}", e)],
                output: None,
                relay_names: vec![],
            };
        }
    };

    let existing_names = get_existing_proxy_names(&config);

    // Skip patterns
    let skip_patterns = vec!["若节点超时", "Emby", "SOCKS5"];
    let valid_proxies: Vec<String> = existing_names
        .iter()
        .filter(|name| !skip_patterns.iter().any(|pat| name.contains(pat)))
        .cloned()
        .collect();

    // Apply keyword filter
    let valid_proxies = filter_proxies_by_keywords(&valid_proxies, &options.filter_keywords);

    if valid_proxies.is_empty() {
        return PatchResult {
            success: false,
            logs: vec!["[ERROR] No available nodes for chain proxy".to_string()],
            output: None,
            relay_names: vec![],
        };
    }

    let relay_names: Vec<String> = valid_proxies
        .iter()
        .map(|name| format!("Chain-{}", clean_proxy_name(name)))
        .collect();

    let logs = vec![
        format!("Found {} proxy nodes", existing_names.len()),
        format!("After filtering: {} available nodes", valid_proxies.len()),
        format!("SOCKS5 proxy: {}:{}", proxy.host, proxy.port),
    ];

    PatchResult {
        success: true,
        logs,
        output: None,
        relay_names,
    }
}

/// Apply the patch to the configuration
pub fn apply_patch(
    config_content: &str,
    proxy: &Socks5Proxy,
    options: &PatchOptions,
) -> PatchResult {
    let mut config: Value = match serde_yaml::from_str(config_content) {
        Ok(v) => v,
        Err(e) => {
            return PatchResult {
                success: false,
                logs: vec![format!("[ERROR] YAML parsing failed: {}", e)],
                output: None,
                relay_names: vec![],
            };
        }
    };

    let mut logs = Vec::new();

    let existing_names = get_existing_proxy_names(&config);

    // Skip patterns
    let skip_patterns = vec!["若节点超时", "Emby", "SOCKS5"];
    let valid_proxies: Vec<String> = existing_names
        .iter()
        .filter(|name| !skip_patterns.iter().any(|pat| name.contains(pat)))
        .cloned()
        .collect();

    // Apply keyword filter
    let valid_proxies = filter_proxies_by_keywords(&valid_proxies, &options.filter_keywords);

    if valid_proxies.is_empty() {
        return PatchResult {
            success: false,
            logs: vec!["[ERROR] No available nodes for chain proxy".to_string()],
            output: None,
            relay_names: vec![],
        };
    }

    logs.push(format!("Found {} available nodes", valid_proxies.len()));

    // Check if SOCKS5 node already exists
    let existing_socks5_names: HashSet<String> = existing_names.into_iter().collect();
    let socks5_name = &proxy.name;

    // Add SOCKS5 node to proxies
    if !existing_socks5_names.contains(socks5_name) {
        if let Some(proxies) = config.get_mut("proxies").and_then(|v| v.as_sequence_mut()) {
            proxies.push(proxy.to_yaml_value());
            logs.push(format!("[+] Added SOCKS5 node: {}", socks5_name));
        }
    } else {
        logs.push(format!("[=] SOCKS5 node already exists: {}", socks5_name));
    }

    // Create relay groups
    let mut relay_groups = Vec::new();
    let mut relay_names = Vec::new();

    for proxy_name in &valid_proxies {
        let relay = create_relay_group(proxy_name, socks5_name);
        if let Some(name) = relay.get("name").and_then(|v| v.as_str()) {
            relay_names.push(name.to_string());
        }
        relay_groups.push(relay);
    }

    // Create chain selector group
    let mut chain_selector = Mapping::new();
    chain_selector.insert(
        Value::String("name".to_string()),
        Value::String("Chain-Selector".to_string()),
    );
    chain_selector.insert(
        Value::String("type".to_string()),
        Value::String("select".to_string()),
    );
    chain_selector.insert(
        Value::String("proxies".to_string()),
        Value::Sequence(relay_names.iter().map(|n| Value::String(n.clone())).collect()),
    );

    // Update proxy-groups
    let proxy_groups = config
        .get_mut("proxy-groups")
        .and_then(|v| v.as_sequence_mut());

    if let Some(groups) = proxy_groups {
        // Remove existing Chain- groups
        groups.retain(|g| {
            if let Some(name) = g.get("name").and_then(|v| v.as_str()) {
                !name.starts_with("Chain-")
            } else {
                true
            }
        });

        // Add new relay groups
        for relay in relay_groups {
            groups.push(relay);
        }

        // Add chain selector
        groups.push(Value::Mapping(chain_selector));
    } else {
        // Create proxy-groups if not exists
        let mut groups = relay_groups;
        groups.push(Value::Mapping(chain_selector));
        if let Some(mapping) = config.as_mapping_mut() {
            mapping.insert(Value::String("proxy-groups".to_string()), Value::Sequence(groups));
        } else {
            return PatchResult {
                success: false,
                logs: vec!["[ERROR] Config root is not a YAML mapping".to_string()],
                output: None,
                relay_names: vec![],
            };
        }
    }

    logs.push(format!("[+] Added {} chain proxy groups", relay_names.len()));
    logs.push("[+] Added chain selector: Chain-Selector".to_string());

    // Serialize to YAML
    let output = match serde_yaml::to_string(&config) {
        Ok(s) => s,
        Err(e) => {
            return PatchResult {
                success: false,
                logs: vec![format!("[ERROR] YAML serialization failed: {}", e)],
                output: None,
                relay_names: vec![],
            };
        }
    };

    PatchResult {
        success: true,
        logs,
        output: Some(output),
        relay_names,
    }
}

/// A rule group found in the rules section, with its name and count
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleGroup {
    /// The proxy group name (e.g. "YepFast-椰皮加速", "DIRECT", "REJECT")
    pub name: String,
    /// How many rules reference this group
    pub count: usize,
}

/// Parse a single rule string and extract the proxy group name.
///
/// Rule formats:
/// - `TYPE,PATTERN,GROUP` (e.g. "DOMAIN,example.com,Proxy")
/// - `TYPE,PATTERN,GROUP,extra` (e.g. "IP-CIDR,1.1.1.1/32,Proxy,no-resolve")
/// - `MATCH,GROUP` (the default/fallback rule)
fn extract_group_from_rule(rule: &str) -> Option<String> {
    let parts: Vec<&str> = rule.split(',').collect();
    match parts.len() {
        // MATCH,GROUP
        2 if parts[0].trim() == "MATCH" => Some(parts[1].trim().to_string()),
        // TYPE,PATTERN,GROUP or TYPE,PATTERN,GROUP,extra
        n if n >= 3 => Some(parts[2].trim().to_string()),
        _ => None,
    }
}

/// Extract all unique proxy groups referenced in the rules section,
/// along with their rule counts. Results are sorted by count (descending).
pub fn extract_rule_groups(config_content: &str) -> Vec<RuleGroup> {
    let config: Value = match serde_yaml::from_str(config_content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    extract_rule_groups_from_value(&config)
}

/// Extract rule groups from a parsed YAML Value
pub fn extract_rule_groups_from_value(config: &Value) -> Vec<RuleGroup> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    if let Some(rules) = config.get("rules").and_then(|v| v.as_sequence()) {
        for rule in rules {
            if let Some(rule_str) = rule.as_str() {
                if let Some(group) = extract_group_from_rule(rule_str) {
                    *counts.entry(group).or_insert(0) += 1;
                }
            }
        }
    }

    let mut groups: Vec<RuleGroup> = counts
        .into_iter()
        .map(|(name, count)| RuleGroup { name, count })
        .collect();

    // Sort by count descending, then name ascending
    groups.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    groups
}

/// Rewrite rules in-place: for each rule whose group is a key in `replacements`,
/// replace the group with the corresponding value.
///
/// Returns the number of rules rewritten.
pub fn rewrite_rules(config_content: &str, replacements: &HashMap<String, String>) -> Result<(String, usize), String> {
    if replacements.is_empty() {
        return Ok((config_content.to_string(), 0));
    }

    let mut config: Value = serde_yaml::from_str(config_content)
        .map_err(|e| format!("YAML parse error: {}", e))?;

    let rewritten = rewrite_rules_in_value(&mut config, replacements);

    let output = serde_yaml::to_string(&config)
        .map_err(|e| format!("YAML serialize error: {}", e))?;

    Ok((output, rewritten))
}

/// Rewrite rules in a mutable YAML Value. Returns count of rewritten rules.
pub fn rewrite_rules_in_value(config: &mut Value, replacements: &HashMap<String, String>) -> usize {
    let mut rewritten = 0;

    let rules = match config.get_mut("rules").and_then(|v| v.as_sequence_mut()) {
        Some(r) => r,
        None => return 0,
    };

    for rule in rules.iter_mut() {
        if let Some(rule_str) = rule.as_str() {
            let parts: Vec<&str> = rule_str.split(',').collect();
            let (group_idx, group) = if parts.len() == 2 && parts[0].trim() == "MATCH" {
                (1, parts[1].trim())
            } else if parts.len() >= 3 {
                (2, parts[2].trim())
            } else {
                continue;
            };

            if let Some(new_group) = replacements.get(group) {
                // Rebuild the rule string with the new group
                let mut new_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                new_parts[group_idx] = new_group.clone();
                *rule = Value::String(new_parts.join(","));
                rewritten += 1;
            }
        }
    }

    rewritten
}

/// Text-based rules rewrite that preserves original YAML formatting.
/// Unlike `rewrite_rules`, this does NOT parse/serialize YAML, so flow style,
/// quoting, indentation, and comments are all preserved.
pub fn rewrite_rules_text(content: &str, replacements: &HashMap<String, String>) -> (String, usize) {
    if replacements.is_empty() {
        return (content.to_string(), 0);
    }

    let mut count = 0;
    let mut in_rules_section = false;

    let lines: Vec<String> = content.lines().map(|line| {
        let trimmed = line.trim_start();

        // Detect top-level section changes (line starts at column 0 with "key:")
        if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
            in_rules_section = trimmed.starts_with("rules:");
        }

        if !in_rules_section {
            return line.to_string();
        }

        // Only process rule lines (contain comma-separated rule parts)
        // Rule lines look like: "    - 'DOMAIN-SUFFIX,google.com,Proxy'"
        // or: "- DOMAIN-SUFFIX,google.com,Proxy"
        if !trimmed.starts_with("- ") {
            return line.to_string();
        }

        for (old_group, new_group) in replacements {
            // Match ",oldgroup" possibly followed by quote or end
            let pattern = format!(",{}", old_group);
            if line.contains(&pattern) {
                count += 1;
                return line.replacen(&pattern, &format!(",{}", new_group), 1);
            }
        }

        line.to_string()
    }).collect();

    // Preserve trailing newline if original had one
    let mut output = lines.join("\n");
    if content.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    (output, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proxy_string_format1() {
        let result = parse_proxy_string("user:pass@host.com:1080");
        assert!(result.is_some());
        let proxy = result.unwrap();
        assert_eq!(proxy.host, "host.com");
        assert_eq!(proxy.port, 1080);
        assert_eq!(proxy.username, Some("user".to_string()));
        assert_eq!(proxy.password, Some("pass".to_string()));
    }

    #[test]
    fn test_parse_proxy_string_format2() {
        let result = parse_proxy_string("64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup");
        assert!(result.is_some());
        let proxy = result.unwrap();
        assert_eq!(proxy.host, "64.32.179.160");
        assert_eq!(proxy.port, 60088);
        assert_eq!(proxy.username, Some("ZUvGbvjcI52P".to_string()));
        assert_eq!(proxy.password, Some("0UxQRzGfZoup".to_string()));
    }

    #[test]
    fn test_clean_proxy_name() {
        assert_eq!(clean_proxy_name("[US] Node (Test)"), "USNodeTest");
        assert_eq!(clean_proxy_name("香港节点"), "香港节点");
    }

    #[test]
    fn test_extract_group_from_rule() {
        assert_eq!(extract_group_from_rule("MATCH,Proxy"), Some("Proxy".to_string()));
        assert_eq!(extract_group_from_rule("DOMAIN,example.com,MyProxy"), Some("MyProxy".to_string()));
        assert_eq!(extract_group_from_rule("IP-CIDR,1.1.1.1/32,Proxy,no-resolve"), Some("Proxy".to_string()));
        assert_eq!(extract_group_from_rule("DOMAIN-SUFFIX,cn,DIRECT"), Some("DIRECT".to_string()));
        assert_eq!(extract_group_from_rule("invalid"), None);
    }

    #[test]
    fn test_extract_rule_groups() {
        let yaml = r#"
rules:
  - DOMAIN,a.com,ProxyA
  - DOMAIN,b.com,ProxyA
  - DOMAIN,c.com,DIRECT
  - IP-CIDR,1.1.1.1/32,ProxyA,no-resolve
  - MATCH,ProxyA
"#;
        let groups = extract_rule_groups(yaml);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "ProxyA");
        assert_eq!(groups[0].count, 4);
        assert_eq!(groups[1].name, "DIRECT");
        assert_eq!(groups[1].count, 1);
    }

    #[test]
    fn test_rewrite_rules() {
        let yaml = r#"
rules:
  - DOMAIN,a.com,OldProxy
  - DOMAIN,b.com,DIRECT
  - IP-CIDR,1.1.1.1/32,OldProxy,no-resolve
  - MATCH,OldProxy
"#;
        let mut replacements = HashMap::new();
        replacements.insert("OldProxy".to_string(), "Chain-Selector".to_string());

        let (output, count) = rewrite_rules(yaml, &replacements).unwrap();
        assert_eq!(count, 3);
        assert!(output.contains("Chain-Selector"));
        assert!(!output.contains("OldProxy"));
        // DIRECT should remain unchanged
        assert!(output.contains("DIRECT"));
    }

    #[test]
    fn test_rewrite_rules_match_rule() {
        // Ensure MATCH rule is also rewritten correctly
        let yaml = r#"
rules:
  - DOMAIN,a.com,Proxy
  - MATCH,Proxy
"#;
        let mut replacements = HashMap::new();
        replacements.insert("Proxy".to_string(), "Chain-Auto".to_string());

        let (output, count) = rewrite_rules(yaml, &replacements).unwrap();
        assert_eq!(count, 2);
        assert!(output.contains("MATCH,Chain-Auto"));
        assert!(output.contains("Chain-Auto"));
    }

    #[test]
    fn test_rewrite_rules_no_replacements() {
        let yaml = "rules:\n  - DOMAIN,a.com,Proxy\n";
        let (output, count) = rewrite_rules(yaml, &HashMap::new()).unwrap();
        assert_eq!(count, 0);
        assert_eq!(output, yaml);
    }
}
