//! Build script for Clash Chain Patcher
//!
//! Embeds the application icon for Windows builds
//!
//! Reference: https://github.com/BenjaminRi/winresource

fn main() {
    // Only compile Windows resources when building on/for Windows
    // The winresource crate is only available as a build dependency on Windows
    #[cfg(windows)]
    {
        // Use winresource to embed icon (maintained fork of winres for Rust 1.61+)
        if std::path::Path::new("logo/app.ico").exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("logo/app.ico");
            res.compile().expect("Failed to compile Windows resources");
        }
    }
}
