/// Configuration management module
///
/// This module is responsible for managing application configuration, including:
/// - Upstream proxy list
/// - Clash configuration path and settings
/// - Local proxy server configuration
/// - Health check configuration

pub mod manager;
pub mod upstream;

pub use manager::{
    AppConfig, ClashApiConfig, ClashConfig, ConfigManager, HealthCheckConfig, LocalProxyConfig,
};
pub use upstream::{HealthStatus, ProxyHealth, UpstreamProxy};
