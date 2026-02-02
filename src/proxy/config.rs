//! Configuration structures for the proxy server

use serde::{Deserialize, Serialize};

/// Configuration for the proxy server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Local listen address (e.g., "127.0.0.1:10808")
    pub listen_addr: String,

    /// Upstream SOCKS5 proxy configuration
    pub upstream: UpstreamConfig,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:10808".to_string(),
            upstream: UpstreamConfig::default(),
        }
    }
}

/// Configuration for upstream SOCKS5 proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Upstream proxy host
    pub host: String,

    /// Upstream proxy port
    pub port: u16,

    /// Optional username for authentication
    pub username: Option<String>,

    /// Optional password for authentication
    pub password: Option<String>,
}

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        }
    }
}

impl UpstreamConfig {
    /// Parse upstream config from proxy string
    ///
    /// Supports two formats:
    /// 1. `user:pass@host:port`
    /// 2. `host:port:user:pass`
    ///
    /// # Examples
    /// ```
    /// use clash_chain_patcher::proxy::UpstreamConfig;
    ///
    /// let config = UpstreamConfig::from_proxy_string("user:pass@host.com:1080");
    /// assert!(config.is_some());
    /// ```
    pub fn from_proxy_string(input: &str) -> Option<Self> {
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

            Some(Self {
                host: server_parts[0].to_string(),
                port,
                username: Some(auth_parts[0].to_string()),
                password: Some(auth_parts[1].to_string()),
            })
        } else {
            // Format 2: host:port:user:pass or host:port
            let parts: Vec<&str> = input.split(':').collect();

            if parts.len() >= 4 {
                let port = parts[1].parse::<u16>().ok()?;
                Some(Self {
                    host: parts[0].to_string(),
                    port,
                    username: Some(parts[2].to_string()),
                    password: Some(parts[3].to_string()),
                })
            } else if parts.len() >= 2 {
                let port = parts[1].parse::<u16>().ok()?;
                Some(Self {
                    host: parts[0].to_string(),
                    port,
                    username: None,
                    password: None,
                })
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_format1() {
        let config = UpstreamConfig::from_proxy_string("user:pass@host.com:1080").unwrap();
        assert_eq!(config.host, "host.com");
        assert_eq!(config.port, 1080);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
    }

    #[test]
    fn test_parse_format2() {
        let config = UpstreamConfig::from_proxy_string("64.32.179.160:60088:user:pass").unwrap();
        assert_eq!(config.host, "64.32.179.160");
        assert_eq!(config.port, 60088);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
    }

    #[test]
    fn test_parse_no_auth() {
        let config = UpstreamConfig::from_proxy_string("host.com:1080").unwrap();
        assert_eq!(config.host, "host.com");
        assert_eq!(config.port, 1080);
        assert_eq!(config.username, None);
        assert_eq!(config.password, None);
    }
}
