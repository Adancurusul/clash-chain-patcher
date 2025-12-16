# Logo Setup Guide

This document explains how to set up and modify the application logo for Clash Chain Patcher.

## Quick Start

### macOS
```bash
# Build and create .app bundle with icon
./scripts/bundle_macos.sh

# Run (with custom Dock icon)
open "target/bundle/Clash Chain Patcher.app"

# Install to Applications
cp -r "target/bundle/Clash Chain Patcher.app" /Applications/
```

### Windows
```cmd
:: Build (icon automatically embedded)
cargo build --release

:: Run
target\release\clash-chain-patcher.exe
```

---

## File Structure

```
logo/
├── clash-chain-patcher.png   # Original high-res logo (source)
├── logo_32.png               # 32x32 PNG for GUI display
├── AppIcon.icns              # macOS app icon (Dock, Finder)
└── app.ico                   # Windows app icon (taskbar, exe)

scripts/
├── generate_icons.py         # PNG → .icns/.ico converter
└── bundle_macos.sh           # macOS .app bundle creator
```

---

## Platform Differences

| Feature | Windows | macOS |
|---------|---------|-------|
| Icon in EXE/Binary | ✅ Embedded at compile time | ❌ Not possible |
| Taskbar/Dock Icon | ✅ Automatic | ⚠️ Requires .app bundle |
| GUI Logo | ✅ Embedded | ✅ Embedded |
| Single file distribution | ✅ Yes | ❌ Need .app folder |

### Why macOS requires .app bundle?

macOS uses a **file system based** icon system:
- The system reads `Info.plist` to find `CFBundleIconFile`
- Icons must be in `.app/Contents/Resources/`
- A standalone binary has no metadata for icons
- This is Apple's design, not a technical limitation

### Windows icon embedding

Windows allows embedding resources (icons, version info) directly into `.exe` files using the PE format. We use the `winres` crate in `build.rs` to do this automatically at compile time.

---

## Updating the Logo

### Step 1: Replace source image
```bash
# Replace with your new logo (PNG, 1024x1024 recommended)
cp your-new-logo.png logo/clash-chain-patcher.png
```

### Step 2: Regenerate icons
```bash
uv run python scripts/generate_icons.py
```

This generates:
- `logo/AppIcon.icns` - macOS icon (multiple sizes)
- `logo/app.ico` - Windows icon (multiple sizes)
- `logo/logo_32.png` - GUI display

### Step 3: Rebuild

**macOS:**
```bash
./scripts/bundle_macos.sh
```

**Windows:**
```cmd
cargo build --release
```

---

## macOS Distribution

### Option 1: .app Bundle (Recommended)
```bash
# Create bundle
./scripts/bundle_macos.sh

# Result: target/bundle/Clash Chain Patcher.app
```

Features:
- ✅ Custom Dock icon
- ✅ Proper app name in menu bar
- ✅ Can be added to Applications
- ✅ File type associations

### Option 2: DMG Installer
```bash
# Create DMG (requires create-dmg)
brew install create-dmg

create-dmg \
  --volname "Clash Chain Patcher" \
  --volicon "logo/AppIcon.icns" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "Clash Chain Patcher.app" 175 190 \
  --app-drop-link 425 190 \
  "Clash Chain Patcher.dmg" \
  "target/bundle/"
```

### Option 3: Homebrew Cask
Create a cask formula for distribution via Homebrew.

---

## Windows Distribution

### Option 1: Single EXE
```cmd
:: Just distribute the exe (icon embedded)
target\release\clash-chain-patcher.exe
```

### Option 2: Installer (NSIS/Inno Setup)
For professional distribution, use an installer creator.

---

## Technical Details

### GUI Logo (app.rs)
```rust
logo_image = <Image> {
    width: 32,
    height: 32,
    source: dep("crate://self/logo/logo_32.png")
}
```

### Windows Icon Embedding (build.rs)
```rust
#[cfg(target_os = "windows")]
{
    let mut res = winres::WindowsResource::new();
    res.set_icon("logo/app.ico");
    res.compile().expect("Failed to compile Windows resources");
}
```

### macOS Info.plist
```xml
<key>CFBundleIconFile</key>
<string>AppIcon</string>
```

---

## Troubleshooting

### macOS: Dock icon not showing
1. Make sure you're running the `.app` bundle, not the binary directly
2. Clear icon cache:
   ```bash
   rm -rf ~/Library/Caches/com.apple.iconservices.store
   killall Dock
   ```
3. Re-run the `.app`

### macOS: "App is damaged" warning
```bash
xattr -cr "target/bundle/Clash Chain Patcher.app"
```

### Windows: Icon not showing in exe
1. Ensure `logo/app.ico` exists
2. Clean and rebuild:
   ```cmd
   cargo clean
   cargo build --release
   ```

### GUI logo not showing
1. Ensure `logo/logo_32.png` exists
2. File path must match: `source: dep("crate://self/logo/logo_32.png")`
3. Rebuild the application

---

## Scripts Reference

### generate_icons.py
Converts `logo/clash-chain-patcher.png` to platform-specific formats:
- Uses Pillow for image processing
- macOS: Creates iconset then uses `iconutil` to generate `.icns`
- Windows: Creates multi-resolution `.ico`

### bundle_macos.sh
Creates macOS `.app` bundle:
1. Builds release binary
2. Creates bundle directory structure
3. Copies binary and icon
4. Generates `Info.plist` and `PkgInfo`

---

## CI/CD Release

GitHub Actions workflow is configured in `.github/workflows/release.yml`.

### How to release:
```bash
# Tag a new version
git tag v1.0.0
git push origin v1.0.0
```

This will automatically:
1. Build for Windows (.exe with embedded icon)
2. Build for macOS (.app bundle + .dmg)
3. Build for Linux (binary)
4. Create GitHub Release with all artifacts

### Release artifacts:
| Platform | File | Icon |
|----------|------|------|
| Windows | `clash-chain-patcher-windows.exe` | ✅ Embedded |
| macOS | `Clash-Chain-Patcher-macos.dmg` | ✅ In bundle |
| macOS | `Clash-Chain-Patcher-macos.zip` | ✅ In bundle |
| Linux | `clash-chain-patcher-linux` | ❌ N/A |

### Manual trigger:
Go to Actions → Build and Release → Run workflow
