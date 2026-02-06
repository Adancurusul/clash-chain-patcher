//! Clash Chain Patcher Library
//!
//! This library provides:
//! 1. YAML configuration modification (patcher module)
//! 2. Dynamic SOCKS5 proxy server (proxy module)
//! 3. Configuration management (config module)
//! 4. Health checking for upstream proxies (health module)
//! 5. File watching for Clash configuration (watcher module)
//! 6. Configuration merging for Clash configs (merger module)
//! 7. Bridge layer for GUI integration (bridge module)
//! 8. Application state management (state module)

// Re-export commonly used modules
pub mod bridge;
pub mod config;
pub mod health;
pub mod merger;
pub mod patcher;
pub mod proxy;
pub mod state;
pub mod watcher;
