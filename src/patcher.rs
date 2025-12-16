//! Clash configuration patcher - Core logic
//!
//! This module handles:
//! - Parsing Clash YAML configurations
//! - Creating SOCKS5 proxy nodes
//! - Creating relay (chain) proxy groups
//! - Patching configurations with new chain proxies

use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::collections::HashSet;

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
        config
            .as_mapping_mut()
            .unwrap()
            .insert(Value::String("proxy-groups".to_string()), Value::Sequence(groups));
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
}
