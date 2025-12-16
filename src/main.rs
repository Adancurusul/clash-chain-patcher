//! Clash Chain Patcher - A Rust GUI tool for adding SOCKS5 proxy chains to Clash configurations
//!
//! This application provides the same functionality as the Python version:
//! - Load Clash YAML configuration files
//! - Configure SOCKS5 proxy settings (supports two input formats)
//! - Create relay (chain) proxy groups combining existing proxies with the SOCKS5 proxy
//! - Preview and apply changes
//! - Save the modified configuration
//!
//! Built with Makepad for a native GUI experience.

mod patcher;
mod app;

fn main() {
    app::app_main();
}
