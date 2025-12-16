#!/bin/bash
# macOS App Bundle Creator for Clash Chain Patcher
#
# Usage: ./scripts/bundle_macos.sh
#
# Creates a .app bundle with proper icon

set -e

APP_NAME="Clash Chain Patcher"
BUNDLE_ID="com.clashchain.patcher"
# Extract version from Cargo.toml automatically
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/target/release"
BUNDLE_DIR="$PROJECT_DIR/target/bundle"
APP_DIR="$BUNDLE_DIR/$APP_NAME.app"

echo "Building release binary..."
cd "$PROJECT_DIR"
cargo build --release

echo "Creating app bundle..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy executable
cp "$BUILD_DIR/clash-chain-patcher" "$APP_DIR/Contents/MacOS/"

# Copy icon
if [ -f "$PROJECT_DIR/logo/AppIcon.icns" ]; then
    cp "$PROJECT_DIR/logo/AppIcon.icns" "$APP_DIR/Contents/Resources/"
    echo "Icon copied"
else
    echo "Warning: AppIcon.icns not found"
fi

# Create Info.plist
cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>clash-chain-patcher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>YAML File</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>yaml</string>
                <string>yml</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create PkgInfo
echo -n "APPL????" > "$APP_DIR/Contents/PkgInfo"

echo ""
echo "App bundle created: $APP_DIR"
echo ""
echo "To install:"
echo "  cp -r \"$APP_DIR\" /Applications/"
echo ""
echo "Or drag the .app to Applications folder in Finder"
