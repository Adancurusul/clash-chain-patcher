use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::upstream::UpstreamProxy;

/// Application configuration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Upstream proxy list
    pub upstream_proxies: Vec<UpstreamProxy>,

    /// Clash configuration
    pub clash: ClashConfig,

    /// Local proxy configuration
    pub local_proxy: LocalProxyConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            upstream_proxies: Vec::new(),
            clash: ClashConfig::default(),
            local_proxy: LocalProxyConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// Clash configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashConfig {
    /// Clash configuration file path
    pub config_path: Option<String>,

    /// Whether to enable automatic monitoring
    pub auto_monitor: bool,

    /// Whether to automatically merge configuration
    pub auto_merge: bool,

    /// Clash API configuration
    pub api: ClashApiConfig,
}

impl Default for ClashConfig {
    fn default() -> Self {
        Self {
            config_path: None,
            auto_monitor: false,
            auto_merge: true,
            api: ClashApiConfig::default(),
        }
    }
}

/// Clash API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashApiConfig {
    /// Whether API is enabled
    pub enabled: bool,

    /// API address
    pub host: String,

    /// API port
    pub port: u16,

    /// API secret key
    pub secret: Option<String>,
}

impl Default for ClashApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 9090,
            secret: None,
        }
    }
}

/// Local proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProxyConfig {
    /// Proxy name (in Clash configuration)
    pub name: String,

    /// Listen address
    pub listen: String,
}

impl Default for LocalProxyConfig {
    fn default() -> Self {
        Self {
            name: "Local-Chain-Proxy".to_string(),
            listen: "127.0.0.1:10808".to_string(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled
    pub enabled: bool,

    /// Check interval (seconds)
    pub interval_seconds: u64,

    /// Test URL
    pub test_url: String,

    /// Timeout duration (seconds)
    pub timeout_seconds: u64,

    /// Number of failures before marking as unhealthy
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            test_url: "http://www.gstatic.com/generate_204".to_string(),
            timeout_seconds: 10,
            failure_threshold: 3,
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// Create a new configuration manager
    ///
    /// Creates a default configuration if the config file doesn't exist
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        // Ensure the config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        // Load or create configuration
        let config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let default_config = AppConfig::default();
            Self::save_to_file(&config_path, &default_config)?;
            default_config
        };

        Ok(Self { config_path, config })
    }

    /// Create a configuration manager with a specific path (for testing)
    #[cfg(test)]
    pub(crate) fn new_with_path(config_path: PathBuf) -> Result<Self> {
        // Ensure the config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        // Load or create configuration
        let config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let default_config = AppConfig::default();
            Self::save_to_file(&config_path, &default_config)?;
            default_config
        };

        Ok(Self { config_path, config })
    }

    /// Get the configuration file path
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;

        Ok(config_dir
            .join("clash-chain-patcher")
            .join("config.json"))
    }

    /// Load configuration from file
    fn load_from_file(path: &Path) -> Result<AppConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: AppConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    fn save_to_file(path: &Path, config: &AppConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)
            .context("Failed to serialize config")?;

        fs::write(path, json)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Save the current configuration
    pub fn save(&self) -> Result<()> {
        Self::save_to_file(&self.config_path, &self.config)?;
        tracing::info!("Config saved to: {}", self.config_path.display());
        Ok(())
    }

    /// Reload the configuration
    pub fn reload(&mut self) -> Result<()> {
        self.config = Self::load_from_file(&self.config_path)?;
        tracing::info!("Config reloaded from: {}", self.config_path.display());
        Ok(())
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Get the configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    // ===== Upstream proxy management =====

    /// Add an upstream proxy
    pub fn add_upstream(&mut self, proxy: UpstreamProxy) -> Result<()> {
        // Check if ID already exists
        if self.config.upstream_proxies.iter().any(|p| p.id == proxy.id) {
            anyhow::bail!("Proxy with ID {} already exists", proxy.id);
        }

        self.config.upstream_proxies.push(proxy);
        self.save()?;
        Ok(())
    }

    /// Remove an upstream proxy
    pub fn remove_upstream(&mut self, id: &str) -> Result<()> {
        let original_len = self.config.upstream_proxies.len();
        self.config.upstream_proxies.retain(|p| p.id != id);

        if self.config.upstream_proxies.len() == original_len {
            anyhow::bail!("Proxy with ID {} not found", id);
        }

        self.save()?;
        Ok(())
    }

    /// Update an upstream proxy
    pub fn update_upstream(&mut self, proxy: UpstreamProxy) -> Result<()> {
        let pos = self.config.upstream_proxies
            .iter()
            .position(|p| p.id == proxy.id)
            .with_context(|| format!("Proxy with ID {} not found", proxy.id))?;

        self.config.upstream_proxies[pos] = proxy;
        self.save()?;
        Ok(())
    }

    /// Get an upstream proxy
    pub fn get_upstream(&self, id: &str) -> Option<&UpstreamProxy> {
        self.config.upstream_proxies.iter().find(|p| p.id == id)
    }

    /// Get all upstream proxies
    pub fn list_upstreams(&self) -> &[UpstreamProxy] {
        &self.config.upstream_proxies
    }

    /// Get all enabled upstream proxies
    pub fn list_enabled_upstreams(&self) -> Vec<&UpstreamProxy> {
        self.config.upstream_proxies
            .iter()
            .filter(|p| p.enabled)
            .collect()
    }

    /// Enable/disable an upstream proxy
    pub fn set_upstream_enabled(&mut self, id: &str, enabled: bool) -> Result<()> {
        let proxy = self.config.upstream_proxies
            .iter_mut()
            .find(|p| p.id == id)
            .with_context(|| format!("Proxy with ID {} not found", id))?;

        proxy.enabled = enabled;
        self.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::config::UpstreamConfig;
    use tempfile::TempDir;

    fn create_test_config_manager() -> (ConfigManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config = AppConfig::default();
        ConfigManager::save_to_file(&config_path, &config).unwrap();

        let manager = ConfigManager {
            config_path,
            config,
        };

        (manager, temp_dir)
    }

    #[test]
    fn test_config_save_and_load() {
        let (manager, _temp_dir) = create_test_config_manager();

        // Save
        manager.save().unwrap();

        // Load
        let loaded = ConfigManager::load_from_file(&manager.config_path).unwrap();
        assert_eq!(loaded.upstream_proxies.len(), 0);
    }

    #[test]
    fn test_add_upstream() {
        let (mut manager, _temp_dir) = create_test_config_manager();

        let config = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };

        let proxy = UpstreamProxy::new("Test Proxy".to_string(), config);
        let proxy_id = proxy.id.clone();

        manager.add_upstream(proxy).unwrap();

        assert_eq!(manager.list_upstreams().len(), 1);
        assert!(manager.get_upstream(&proxy_id).is_some());
    }

    #[test]
    fn test_remove_upstream() {
        let (mut manager, _temp_dir) = create_test_config_manager();

        let config = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        };

        let proxy = UpstreamProxy::new("Test".to_string(), config);
        let proxy_id = proxy.id.clone();

        manager.add_upstream(proxy).unwrap();
        assert_eq!(manager.list_upstreams().len(), 1);

        manager.remove_upstream(&proxy_id).unwrap();
        assert_eq!(manager.list_upstreams().len(), 0);
    }

    #[test]
    fn test_update_upstream() {
        let (mut manager, _temp_dir) = create_test_config_manager();

        let config = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        };

        let mut proxy = UpstreamProxy::new("Old Name".to_string(), config);
        let proxy_id = proxy.id.clone();

        manager.add_upstream(proxy.clone()).unwrap();

        // Update name
        proxy.name = "New Name".to_string();
        manager.update_upstream(proxy).unwrap();

        let updated = manager.get_upstream(&proxy_id).unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_list_enabled_upstreams() {
        let (mut manager, _temp_dir) = create_test_config_manager();

        // Add enabled proxy
        let config1 = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        };
        let proxy1 = UpstreamProxy::new("Enabled".to_string(), config1);
        manager.add_upstream(proxy1).unwrap();

        // Add disabled proxy
        let config2 = UpstreamConfig {
            host: "127.0.0.1".to_string(),
            port: 1081,
            username: None,
            password: None,
        };
        let mut proxy2 = UpstreamProxy::new("Disabled".to_string(), config2);
        proxy2.enabled = false;
        manager.add_upstream(proxy2).unwrap();

        let enabled = manager.list_enabled_upstreams();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "Enabled");
    }
}
