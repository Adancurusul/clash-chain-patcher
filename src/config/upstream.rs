use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

use crate::proxy::config::UpstreamConfig;

/// Upstream proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamProxy {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Whether enabled
    pub enabled: bool,

    /// Upstream proxy configuration
    pub config: UpstreamConfig,

    /// Health status
    pub health: ProxyHealth,
}

impl UpstreamProxy {
    /// Create a new upstream proxy
    pub fn new(name: String, config: UpstreamConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            enabled: true,
            config,
            health: ProxyHealth::default(),
        }
    }

    /// Create from a proxy string
    ///
    /// Supports two formats:
    /// - `host:port:user:pass`
    /// - `user:pass@host:port`
    pub fn from_proxy_string(name: String, proxy_str: &str) -> anyhow::Result<Self> {
        let config = UpstreamConfig::from_proxy_string(proxy_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid proxy string format: {}", proxy_str))?;
        Ok(Self::new(name, config))
    }
}

/// Proxy health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealth {
    /// Health status
    pub status: HealthStatus,

    /// Latency (milliseconds)
    pub latency_ms: Option<u64>,

    /// Last check time
    #[serde(with = "optional_system_time")]
    pub last_check: Option<SystemTime>,

    /// Error message
    pub error: Option<String>,

    /// Consecutive failure count
    pub consecutive_failures: u32,

    /// Exit IP address (from proxy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_ip: Option<String>,

    /// Geographic location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Country code (e.g., "US", "HK")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

impl Default for ProxyHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            latency_ms: None,
            last_check: None,
            error: None,
            consecutive_failures: 0,
            exit_ip: None,
            location: None,
            country_code: None,
        }
    }
}

impl ProxyHealth {
    /// Mark as healthy
    pub fn mark_healthy(&mut self, latency_ms: u64) {
        self.status = HealthStatus::Healthy;
        self.latency_ms = Some(latency_ms);
        self.last_check = Some(SystemTime::now());
        self.error = None;
        self.consecutive_failures = 0;
    }

    /// Mark as healthy with detailed information
    pub fn mark_healthy_with_details(
        &mut self,
        latency_ms: u64,
        exit_ip: Option<String>,
        location: Option<String>,
        country_code: Option<String>,
    ) {
        self.mark_healthy(latency_ms);
        self.exit_ip = exit_ip;
        self.location = location;
        self.country_code = country_code;
    }

    /// Mark as unhealthy
    pub fn mark_unhealthy(&mut self, error: String) {
        self.status = HealthStatus::Unhealthy;
        self.latency_ms = None;
        self.last_check = Some(SystemTime::now());
        self.error = Some(error);
        self.consecutive_failures += 1;
    }

    /// Mark as checking
    pub fn mark_checking(&mut self) {
        self.status = HealthStatus::Checking;
        self.last_check = Some(SystemTime::now());
    }

    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Healthy
    Healthy,

    /// Unhealthy
    Unhealthy,

    /// Checking
    Checking,

    /// Unknown (not checked)
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "✅ Healthy"),
            HealthStatus::Unhealthy => write!(f, "❌ Unhealthy"),
            HealthStatus::Checking => write!(f, "🔄 Checking"),
            HealthStatus::Unknown => write!(f, "⚫ Unknown"),
        }
    }
}

/// Serialization/deserialization helper module for SystemTime
mod optional_system_time {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(st) => {
                let dt: DateTime<Utc> = (*st).into();
                serializer.serialize_some(&dt.to_rfc3339())
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let dt = DateTime::parse_from_rfc3339(&s)
                    .map_err(serde::de::Error::custom)?;
                Ok(Some(SystemTime::from(dt)))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_proxy_creation() {
        let config = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };

        let proxy = UpstreamProxy::new("Test Proxy".to_string(), config);

        assert_eq!(proxy.name, "Test Proxy");
        assert!(proxy.enabled);
        assert_eq!(proxy.health.status, HealthStatus::Unknown);
    }

    #[test]
    fn test_upstream_proxy_from_string() {
        let proxy = UpstreamProxy::from_proxy_string(
            "HK Proxy".to_string(),
            "64.32.179.160:60088:user:pass"
        ).unwrap();

        assert_eq!(proxy.name, "HK Proxy");
        assert_eq!(proxy.config.host, "64.32.179.160");
        assert_eq!(proxy.config.port, 60088);
    }

    #[test]
    fn test_health_status_transitions() {
        let mut health = ProxyHealth::default();

        // Initially unknown
        assert_eq!(health.status, HealthStatus::Unknown);
        assert!(!health.is_healthy());

        // Mark as checking
        health.mark_checking();
        assert_eq!(health.status, HealthStatus::Checking);

        // Mark as healthy
        health.mark_healthy(120);
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency_ms, Some(120));
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.is_healthy());

        // Mark as unhealthy
        health.mark_unhealthy("Connection timeout".to_string());
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.consecutive_failures, 1);
        assert!(!health.is_healthy());

        // Second failure
        health.mark_unhealthy("Still failing".to_string());
        assert_eq!(health.consecutive_failures, 2);
    }

    #[test]
    fn test_serialization() {
        let config = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };

        let proxy = UpstreamProxy::new("Test".to_string(), config);

        // Serialize
        let json = serde_json::to_string_pretty(&proxy).unwrap();
        println!("Serialized: {}", json);

        // Deserialize
        let deserialized: UpstreamProxy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, proxy.name);
        assert_eq!(deserialized.config.host, proxy.config.host);
    }
}
