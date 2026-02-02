//! Upstream proxy management

use crate::proxy::config::UpstreamConfig;
use anyhow::{Context, Result};
use fast_socks5::client::{Config as Socks5ClientConfig, Socks5Stream};
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::debug;

/// Upstream SOCKS5 proxy
pub struct UpstreamProxy {
    config: UpstreamConfig,
}

impl UpstreamProxy {
    /// Create a new upstream proxy
    pub fn new(config: UpstreamConfig) -> Self {
        Self { config }
    }

    /// Connect to target through upstream SOCKS5 proxy
    ///
    /// # Arguments
    /// * `target_addr` - Target domain or IP address
    /// * `target_port` - Target port
    ///
    /// # Returns
    /// * `Ok(Socks5Stream<TcpStream>)` - Connected stream to target through upstream
    /// * `Err` - Connection failed
    pub async fn connect(
        &self,
        target_addr: &str,
        target_port: u16,
    ) -> Result<Socks5Stream<TcpStream>> {
        debug!(
            "Connecting to {}:{} via upstream {}:{}",
            target_addr, target_port, self.config.host, self.config.port
        );

        let upstream_addr = format!("{}:{}", self.config.host, self.config.port);
        let socks_config = Socks5ClientConfig::default();

        // Use fast-socks5 to establish connection through upstream
        let socks_stream = if let (Some(username), Some(password)) =
            (&self.config.username, &self.config.password)
        {
            // With authentication
            Socks5Stream::connect_with_password(
                upstream_addr,
                target_addr.to_string(),
                target_port,
                username.clone(),
                password.clone(),
                socks_config,
            )
            .await
            .context("SOCKS5 connection with auth failed")?
        } else {
            // Without authentication
            Socks5Stream::connect(
                upstream_addr,
                target_addr.to_string(),
                target_port,
                socks_config,
            )
            .await
            .context("SOCKS5 connection failed")?
        };

        debug!(
            "Successfully connected to {}:{} through upstream",
            target_addr, target_port
        );

        Ok(socks_stream)
    }

    /// Get upstream configuration
    pub fn config(&self) -> &UpstreamConfig {
        &self.config
    }
}

impl From<UpstreamConfig> for UpstreamProxy {
    fn from(config: UpstreamConfig) -> Self {
        Self::new(config)
    }
}

/// Create Arc-wrapped UpstreamProxy from config
pub fn create_upstream(config: UpstreamConfig) -> Arc<UpstreamProxy> {
    Arc::new(UpstreamProxy::new(config))
}
