//! Bridge Layer - Connecting asynchronous backend with synchronous GUI
//!
//! This module provides a set of bridge components that allow calling
//! tokio-based asynchronous backend functionality in Makepad's synchronous GUI environment.
//!
//! ## Architecture Overview
//!
//! Makepad GUI components run in a synchronous context, while our backend modules (config, health, watcher, merger)
//! are all based on tokio's asynchronous API. The Bridge layer solves this problem in the following ways:
//!
//! 1. Each Bridge component holds a tokio runtime internally
//! 2. Uses `runtime.block_on()` to convert asynchronous calls to synchronous calls
//! 3. Uses `Arc<RwLock<T>>` to manage shared state
//!
//! ## Components
//!
//! - `ConfigBridge` - Configuration management bridge
//! - `HealthBridge` - Health check bridge
//! - `WatcherBridge` - File watcher bridge
//! - `MergerBridge` - Configuration merger bridge

mod config_bridge;
mod health_bridge;
mod merger_bridge;
mod watcher_bridge;

pub use config_bridge::ConfigBridge;
pub use health_bridge::HealthBridge;
pub use merger_bridge::MergerBridge;
pub use watcher_bridge::WatcherBridge;

/// Common error type for the bridge layer
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Bridge layer error
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Health check error: {0}")]
    Health(String),

    #[error("File watcher error: {0}")]
    Watcher(String),

    #[error("Configuration merger error: {0}")]
    Merger(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
