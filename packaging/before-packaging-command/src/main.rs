use cargo_metadata::MetadataCommand;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const BINARY_NAME: &str = "clash-chain-patcher";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: before-packaging-command <before-packaging|before-each-package>");
        eprintln!("  before-packaging: Copy all Makepad resources to dist/resources/");
        eprintln!("  before-each-package [format]: Build with correct MAKEPAD_PACKAGE_DIR");
        eprintln!("    If format is not provided, reads from CARGO_PACKAGER_FORMAT env var");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "before-packaging" => before_packaging(),
        "before-each-package" => {
            // Try to get format from command line argument first, then from env var
            let format = args.get(2)
                .map(|s| s.to_string())
                .or_else(|| env::var("CARGO_PACKAGER_FORMAT").ok())
                .unwrap_or_else(|| {
                    eprintln!("Error: No format specified and CARGO_PACKAGER_FORMAT not set");
                    std::process::exit(1);
                });
            before_each_package(&format);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

/// Returns the MAKEPAD_PACKAGE_DIR value for each platform/format
/// Reference: https://book.makepad.rs/zh/guide/appendix/packaging-guide
fn makepad_package_dir_value(format: &str) -> &'static str {
    match format {
        // macOS: Use "." because apple_bundle cfg tells Makepad to use NSBundle API
        // to retrieve the resource path at runtime (Contents/Resources/)
        "app" | "dmg" => ".",
        // Linux deb/pacman: resources in /usr/lib/<binary-name>
        "deb" | "pacman" => concat!("/usr/lib/", "clash-chain-patcher"),
        // Linux AppImage: resources relative to binary (simulated working dir is usr/)
        "appimage" => concat!("lib/", "clash-chain-patcher"),
        // Windows: resources in same directory as exe
        "nsis" | "wix" => ".",
        _ => {
            eprintln!("Warning: Unknown format '{}', using '.' as fallback", format);
            "."
        }
    }
}

/// Returns whether MAKEPAD env var should include apple_bundle
fn needs_apple_bundle(format: &str) -> bool {
    matches!(format, "app" | "dmg")
}

/// Copy all Makepad-related resources to dist/resources/
fn before_packaging() {
    println!("=== Before Packaging: Copying Makepad resources ===");

    // Get cargo metadata to find all dependencies
    let metadata = MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .exec()
        .expect("Failed to get cargo metadata");

    let dist_resources = Path::new("dist/resources");

    // Clean and recreate dist/resources
    if dist_resources.exists() {
        fs::remove_dir_all(dist_resources).expect("Failed to remove old dist/resources");
    }
    fs::create_dir_all(dist_resources).expect("Failed to create dist/resources");

    // Find all makepad-related packages and copy their resources
    for package in &metadata.packages {
        // Only process makepad-related packages
        if !package.name.starts_with("makepad") {
            continue;
        }

        let package_path = package
            .manifest_path
            .parent()
            .expect("Package has no parent directory");

        let resources_dir = package_path.join("resources");

        // Convert Utf8PathBuf to std::path::Path for checking
        let resources_path = Path::new(resources_dir.as_str());

        if resources_path.exists() && resources_path.is_dir() {
            println!("Found resources in: {}", package.name);

            // Convert package name to use underscores (cargo-packager convention)
            // e.g., makepad-fonts-chinese-bold -> makepad_fonts_chinese_bold
            let normalized_name = package.name.replace('-', "_");

            // Destination: dist/resources/<normalized-package-name>/resources/
            // Makepad expects resources to be in <package>/resources/ subdirectory
            let dest_dir = dist_resources.join(&normalized_name).join("resources");

            copy_dir_recursive(resources_path, &dest_dir);
            println!("  Copied to: {}", dest_dir.display());
        }
    }

    // Also copy our own resources (logo, etc.)
    let own_resources = [
        ("logo/logo_32.png", "logo_32.png"),
        ("logo/logo_48.png", "logo_48.png"),
        ("logo/clash-chain-patcher.png", "clash-chain-patcher.png"),
    ];

    for (src, dst) in own_resources {
        let src_path = Path::new(src);
        if src_path.exists() {
            let dest_path = dist_resources.join(dst);
            fs::copy(src_path, &dest_path).expect(&format!("Failed to copy {}", src));
            println!("Copied own resource: {} -> {}", src, dest_path.display());
        }
    }

    println!("=== Resource copying complete ===");
}

/// Build the project with the correct MAKEPAD_PACKAGE_DIR for the given format
fn before_each_package(format: &str) {
    println!("=== Before Each Package: Building for format '{}' ===", format);

    let package_dir = makepad_package_dir_value(format);
    println!("Setting MAKEPAD_PACKAGE_DIR={}", package_dir);

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .env("MAKEPAD_PACKAGE_DIR", package_dir);

    // For macOS app/dmg, enable apple_bundle cfg so Makepad uses NSBundle API
    // to find resources at runtime from Contents/Resources/
    if needs_apple_bundle(format) {
        println!("Setting MAKEPAD=apple_bundle");
        cmd.env("MAKEPAD", "apple_bundle");
    }

    println!("Running: cargo build --release");

    let status = cmd
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        eprintln!("Build failed with status: {}", status);
        std::process::exit(1);
    }

    // Copy the built binary to the expected location
    let binary_src = if cfg!(windows) {
        format!("target/release/{}.exe", BINARY_NAME)
    } else {
        format!("target/release/{}", BINARY_NAME)
    };

    let binary_dest = if cfg!(windows) {
        format!("dist/{}.exe", BINARY_NAME)
    } else {
        format!("dist/{}", BINARY_NAME)
    };

    fs::create_dir_all("dist").expect("Failed to create dist directory");
    fs::copy(&binary_src, &binary_dest)
        .expect(&format!("Failed to copy binary from {} to {}", binary_src, binary_dest));

    println!("Copied binary: {} -> {}", binary_src, binary_dest);
    println!("=== Build complete for format '{}' ===", format);
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect(&format!("Failed to create directory: {}", dst.display()));

    for entry in fs::read_dir(src).expect(&format!("Failed to read directory: {}", src.display())) {
        let entry = entry.expect("Failed to read directory entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path)
                .expect(&format!("Failed to copy file: {}", src_path.display()));
        }
    }
}
