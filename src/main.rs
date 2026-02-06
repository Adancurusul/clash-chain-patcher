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

// Prevents console window from appearing on Windows
// Reference: https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
#![windows_subsystem = "windows"]

mod app;
mod app_impl;

fn main() {
    app::app_main();
}
