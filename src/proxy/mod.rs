//! Proxy module - Core SOCKS5 proxy server implementation
//!
//! This module provides a local SOCKS5 proxy server that forwards traffic
//! to upstream SOCKS5 proxies. It supports multiple upstreams, health checking,
//! and automatic failover.

pub mod config;
pub mod server;
pub mod upstream;
pub mod relay;

// Re-export commonly used types
pub use config::{ProxyConfig, UpstreamConfig};
pub use server::ProxyServer;
pub use upstream::UpstreamProxy;
