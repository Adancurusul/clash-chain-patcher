//! Command-line proxy server for testing
//!
//! This example demonstrates how to use the proxy server core without GUI.
//!
//! # Usage
//!
//! ```bash
//! # Start proxy server with upstream
//! cargo run --example proxy_server -- \
//!   --listen 127.0.0.1:10808 \
//!   --upstream user:pass@host:port
//!
//! # Or use the alternative format
//! cargo run --example proxy_server -- \
//!   --listen 127.0.0.1:10808 \
//!   --upstream host:port:user:pass
//!
//! # Test with curl
//! curl --proxy socks5://127.0.0.1:10808 https://ifconfig.me
//! ```

use anyhow::{Context, Result};
use clash_chain_patcher::proxy::{ProxyConfig, ProxyServer, UpstreamConfig};
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "proxy-server")]
#[command(about = "Local SOCKS5 proxy server that forwards to upstream SOCKS5")]
struct Args {
    /// Local listen address
    #[arg(short, long, default_value = "127.0.0.1:10808")]
    listen: String,

    /// Upstream SOCKS5 proxy
    ///
    /// Supports two formats:
    /// 1. user:pass@host:port
    /// 2. host:port:user:pass
    #[arg(short, long)]
    upstream: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Parse command line arguments
    let args = Args::parse();

    info!("Starting proxy server...");
    info!("Listen address: {}", args.listen);
    info!("Upstream: {}", mask_credentials(&args.upstream));

    // Parse upstream configuration
    let upstream_config = UpstreamConfig::from_proxy_string(&args.upstream)
        .context("Failed to parse upstream proxy string")?;

    // Create proxy configuration
    let config = ProxyConfig {
        listen_addr: args.listen,
        upstream: upstream_config,
    };

    // Create and start proxy server
    let server = ProxyServer::new(config);

    info!("Proxy server started successfully");
    info!("Press Ctrl+C to stop");

    // Handle Ctrl+C gracefully
    tokio::select! {
        result = server.start() => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
    }

    Ok(())
}

/// Mask credentials in proxy string for logging
fn mask_credentials(proxy_str: &str) -> String {
    if let Some(_at_pos) = proxy_str.find('@') {
        // Format: user:pass@host:port
        let parts: Vec<&str> = proxy_str.split('@').collect();
        if parts.len() == 2 {
            return format!("***:***@{}", parts[1]);
        }
    }

    // Format: host:port:user:pass or host:port
    let parts: Vec<&str> = proxy_str.split(':').collect();
    if parts.len() >= 4 {
        format!("{}:{}:***:***", parts[0], parts[1])
    } else {
        proxy_str.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_credentials_format1() {
        let masked = mask_credentials("user:pass@host.com:1080");
        assert_eq!(masked, "***:***@host.com:1080");
    }

    #[test]
    fn test_mask_credentials_format2() {
        let masked = mask_credentials("host.com:1080:user:pass");
        assert_eq!(masked, "host.com:1080:***:***");
    }

    #[test]
    fn test_mask_credentials_no_auth() {
        let masked = mask_credentials("host.com:1080");
        assert_eq!(masked, "host.com:1080");
    }
}
