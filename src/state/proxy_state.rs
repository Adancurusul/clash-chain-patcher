//! Proxy-related application state
//!
//! Manages upstream proxies, health checks, monitoring, and other state

use crate::bridge::{ConfigBridge, HealthBridge, MergerBridge, WatcherBridge};
use crate::config::UpstreamProxy;
use std::path::PathBuf;

/// Proxy-related application state
///
/// Contains all state and bridge objects related to proxy management
#[derive(Default)]
pub struct ProxyState {
    // ===== Bridge objects =====
    /// Configuration management bridge
    config_bridge: Option<ConfigBridge>,

    /// Health check bridge
    health_bridge: Option<HealthBridge>,

    /// Configuration merger bridge
    merger_bridge: Option<MergerBridge>,

    /// File watcher bridge
    #[allow(dead_code)]
    watcher_bridge: Option<WatcherBridge>,

    // ===== UI state =====
    /// Currently selected proxy ID
    selected_proxy_id: Option<String>,

    /// Whether a health check is in progress
    is_checking: bool,

    /// Whether config file monitoring is active
    is_watching: bool,

    /// Clash configuration file path
    clash_config_path: Option<PathBuf>,

    /// Error message
    error_message: Option<String>,

    /// Success message
    success_message: Option<String>,
}

impl ProxyState {
    /// Create a new proxy state
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize state (create all bridge objects)
    pub fn initialize(&mut self) -> Result<(), String> {
        // Create configuration bridge
        self.config_bridge = Some(
            ConfigBridge::new()
                .map_err(|e| format!("Failed to create config bridge: {}", e))?,
        );

        // Create health check bridge
        self.health_bridge = Some(
            HealthBridge::new()
                .map_err(|e| format!("Failed to create health bridge: {}", e))?,
        );

        // Create merger bridge
        self.merger_bridge = Some(MergerBridge::new());

        Ok(())
    }

    // ===== Config Bridge related methods =====

    /// Get all upstream proxy list
    pub fn list_upstreams(&self) -> Vec<UpstreamProxy> {
        self.config_bridge
            .as_ref()
            .map(|bridge| bridge.list_upstreams())
            .unwrap_or_default()
    }

    /// Add an upstream proxy
    pub fn add_upstream(&mut self, proxy: UpstreamProxy) -> Result<(), String> {
        self.config_bridge
            .as_ref()
            .ok_or("Config bridge not initialized")?
            .add_upstream(proxy)
            .map_err(|e| e.to_string())
    }

    /// Update an upstream proxy
    pub fn update_upstream(&mut self, proxy: UpstreamProxy) -> Result<(), String> {
        self.config_bridge
            .as_ref()
            .ok_or("Config bridge not initialized")?
            .update_upstream(proxy)
            .map_err(|e| e.to_string())
    }

    /// Remove an upstream proxy
    pub fn remove_upstream(&mut self, id: &str) -> Result<(), String> {
        self.config_bridge
            .as_ref()
            .ok_or("Config bridge not initialized")?
            .remove_upstream(id)
            .map_err(|e| e.to_string())
    }

    /// Enable an upstream proxy
    pub fn enable_upstream(&mut self, id: &str) -> Result<(), String> {
        self.config_bridge
            .as_ref()
            .ok_or("Config bridge not initialized")?
            .enable_upstream(id)
            .map_err(|e| e.to_string())
    }

    /// Disable an upstream proxy
    pub fn disable_upstream(&mut self, id: &str) -> Result<(), String> {
        self.config_bridge
            .as_ref()
            .ok_or("Config bridge not initialized")?
            .disable_upstream(id)
            .map_err(|e| e.to_string())
    }

    /// Get a specific proxy
    pub fn get_upstream(&self, id: &str) -> Option<UpstreamProxy> {
        self.config_bridge
            .as_ref()
            .and_then(|bridge| bridge.get_upstream(id))
    }

    // ===== Health Bridge related methods =====

    /// Check the health status of a single proxy
    pub fn check_proxy_health(&mut self, id: &str) -> Result<(), String> {
        let proxy = self
            .get_upstream(id)
            .ok_or_else(|| format!("Proxy {} not found", id))?;

        let result = self
            .health_bridge
            .as_ref()
            .ok_or("Health bridge not initialized")?
            .check_proxy(&proxy);

        // Update proxy health status
        if let Some(mut proxy) = self.get_upstream(id) {
            if result.is_healthy {
                if let Some(latency) = result.latency_ms {
                    proxy.health.mark_healthy(latency);
                }
            } else {
                proxy
                    .health
                    .mark_unhealthy(result.error.unwrap_or_else(|| "Unknown error".to_string()));
            }

            self.update_upstream(proxy)?;
        }

        Ok(())
    }

    /// Check the health status of all enabled proxies
    pub fn check_all_proxies_health(&mut self) -> Result<(), String> {
        self.is_checking = true;

        let proxies = self.list_upstreams();
        let enabled_proxies: Vec<_> = proxies.into_iter().filter(|p| p.enabled).collect();

        let results = self
            .health_bridge
            .as_ref()
            .ok_or("Health bridge not initialized")?
            .check_proxies(&enabled_proxies);

        // Update all proxy health statuses
        for (proxy_id, result) in results {
            if let Some(mut proxy) = self.get_upstream(&proxy_id) {
                if result.is_healthy {
                    if let Some(latency) = result.latency_ms {
                        proxy.health.mark_healthy(latency);
                    }
                } else {
                    proxy.health.mark_unhealthy(
                        result.error.unwrap_or_else(|| "Unknown error".to_string()),
                    );
                }

                let _ = self.update_upstream(proxy);
            }
        }

        self.is_checking = false;
        Ok(())
    }

    // ===== Merger Bridge related methods =====

    /// Merge configuration to Clash config file
    pub fn merge_to_clash(&mut self) -> Result<(), String> {
        let clash_path = self
            .clash_config_path
            .as_ref()
            .ok_or("Clash config path not set")?;

        self.merger_bridge
            .as_ref()
            .ok_or("Merger bridge not initialized")?
            .merge(clash_path)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ===== UI state related methods =====

    /// Set the selected proxy
    pub fn set_selected_proxy(&mut self, id: Option<String>) {
        self.selected_proxy_id = id;
    }

    /// Get the selected proxy ID
    pub fn selected_proxy_id(&self) -> Option<&str> {
        self.selected_proxy_id.as_deref()
    }

    /// Get the selected proxy
    pub fn selected_proxy(&self) -> Option<UpstreamProxy> {
        self.selected_proxy_id
            .as_ref()
            .and_then(|id| self.get_upstream(id))
    }

    /// Set the Clash configuration file path
    pub fn set_clash_config_path(&mut self, path: PathBuf) {
        self.clash_config_path = Some(path);
    }

    /// Get the Clash configuration file path
    pub fn clash_config_path(&self) -> Option<&PathBuf> {
        self.clash_config_path.as_ref()
    }

    /// Check if a health check is in progress
    pub fn is_checking(&self) -> bool {
        self.is_checking
    }

    /// Check if monitoring is active
    pub fn is_watching(&self) -> bool {
        self.is_watching
    }

    /// Set an error message
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
        self.success_message = None;
    }

    /// Set a success message
    pub fn set_success(&mut self, message: impl Into<String>) {
        self.success_message = Some(message.into());
        self.error_message = None;
    }

    /// Clear messages
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    /// Get the error message
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Get the success message
    pub fn success_message(&self) -> Option<&str> {
        self.success_message.as_deref()
    }

    // ===== Recent files management =====

    /// Add a recently used configuration file
    pub fn add_recent_file(&mut self, path: String) -> Result<(), String> {
        self.config_bridge
            .as_ref()
            .ok_or("Config bridge not initialized")?
            .add_recent_file(path)
            .map_err(|e| e.to_string())
    }

    /// Get the list of recently used files
    pub fn get_recent_files(&self) -> Vec<String> {
        self.config_bridge
            .as_ref()
            .map(|bridge| bridge.get_recent_files())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_state_creation() {
        let state = ProxyState::new();
        assert!(state.config_bridge.is_none());
        assert!(state.selected_proxy_id.is_none());
        assert!(!state.is_checking);
        assert!(!state.is_watching);
    }

    #[test]
    fn test_proxy_state_initialization() {
        let mut state = ProxyState::new();
        let result = state.initialize();

        assert!(result.is_ok());
        assert!(state.config_bridge.is_some());
        assert!(state.health_bridge.is_some());
        assert!(state.merger_bridge.is_some());
    }

    #[test]
    fn test_proxy_state_selected_proxy() {
        let mut state = ProxyState::new();
        state.initialize().unwrap();

        // Initial state
        assert!(state.selected_proxy_id().is_none());
        assert!(state.selected_proxy().is_none());

        // Set selection
        state.set_selected_proxy(Some("test-id".to_string()));
        assert_eq!(state.selected_proxy_id(), Some("test-id"));
    }

    #[test]
    fn test_proxy_state_messages() {
        let mut state = ProxyState::new();

        // Initial state
        assert!(state.error_message().is_none());
        assert!(state.success_message().is_none());

        // Set error message
        state.set_error("Test error");
        assert_eq!(state.error_message(), Some("Test error"));
        assert!(state.success_message().is_none());

        // Set success message
        state.set_success("Test success");
        assert_eq!(state.success_message(), Some("Test success"));
        assert!(state.error_message().is_none());

        // Clear messages
        state.clear_messages();
        assert!(state.error_message().is_none());
        assert!(state.success_message().is_none());
    }

    #[test]
    fn test_proxy_state_clash_config_path() {
        let mut state = ProxyState::new();

        // Initial state
        assert!(state.clash_config_path().is_none());

        // Set path
        let path = PathBuf::from("/path/to/config.yaml");
        state.set_clash_config_path(path.clone());
        assert_eq!(state.clash_config_path(), Some(&path));
    }
}
