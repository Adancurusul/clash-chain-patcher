#!/usr/bin/env python3
"""
Icon Generator for Clash Chain Patcher
Converts PNG logo to .icns (macOS) and .ico (Windows) formats

Usage:
    uv run python scripts/generate_icons.py

Requirements:
    pip install pillow
"""

import os
import sys
import subprocess
from pathlib import Path

def check_pillow():
    """Check if Pillow is available, install if needed"""
    try:
        from PIL import Image
        return True
    except ImportError:
        print("Installing Pillow...")
        subprocess.run([sys.executable, "-m", "pip", "install", "pillow"], check=True)
        return True

def generate_icns_macos(png_path: Path, output_path: Path):
    """Generate .icns file for macOS using iconutil"""
    from PIL import Image

    iconset_path = output_path.parent / "AppIcon.iconset"
    iconset_path.mkdir(exist_ok=True)

    # macOS icon sizes (standard sizes for .icns)
    sizes = [
        (16, "16x16"),
        (32, "16x16@2x"),
        (32, "32x32"),
        (64, "32x32@2x"),
        (128, "128x128"),
        (256, "128x128@2x"),
        (256, "256x256"),
        (512, "256x256@2x"),
        (512, "512x512"),
        (1024, "512x512@2x"),
    ]

    img = Image.open(png_path)

    # Ensure RGBA mode
    if img.mode != 'RGBA':
        img = img.convert('RGBA')

    for size, name in sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(iconset_path / f"icon_{name}.png", "PNG")
        print(f"  Created icon_{name}.png ({size}x{size})")

    # Use iconutil to create .icns (macOS only)
    if sys.platform == "darwin":
        result = subprocess.run(
            ["iconutil", "-c", "icns", str(iconset_path), "-o", str(output_path)],
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            print(f"  Created {output_path}")
            # Cleanup iconset
            import shutil
            shutil.rmtree(iconset_path)
            return True
        else:
            print(f"  Error: {result.stderr}")
            return False
    else:
        print("  Note: .icns can only be generated on macOS")
        print(f"  Iconset created at: {iconset_path}")
        return True

def generate_ico_windows(png_path: Path, output_path: Path):
    """Generate .ico file for Windows"""
    from PIL import Image

    img = Image.open(png_path)

    # Ensure RGBA mode
    if img.mode != 'RGBA':
        img = img.convert('RGBA')

    # Windows icon sizes (256 is max for ICO format)
    sizes = [16, 24, 32, 48, 64, 128, 256]

    icons = []
    for size in sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        icons.append(resized)
        print(f"  Prepared {size}x{size}")

    # Save as .ico with multiple sizes embedded
    img.save(
        output_path,
        format='ICO',
        sizes=[(icon.width, icon.height) for icon in icons]
    )

    # Verify file size
    file_size = output_path.stat().st_size
    print(f"  Created {output_path} ({file_size / 1024:.1f} KB)")
    return True

def main():
    # Paths
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    logo_dir = project_root / "logo"
    png_path = logo_dir / "clash-chain-patcher.png"

    if not png_path.exists():
        print(f"Error: Logo not found at {png_path}")
        sys.exit(1)

    print(f"Source: {png_path}")
    print(f"Size: {png_path.stat().st_size / 1024:.1f} KB")
    print()

    check_pillow()

    # Generate macOS icon
    print("Generating macOS .icns...")
    icns_path = logo_dir / "AppIcon.icns"
    generate_icns_macos(png_path, icns_path)
    print()

    # Generate Windows icon
    print("Generating Windows .ico...")
    ico_path = logo_dir / "app.ico"
    generate_ico_windows(png_path, ico_path)
    print()

    print("Done!")
    print()
    print("Next steps:")
    print("  1. macOS: The .icns will be bundled automatically with app bundle")
    print("  2. Windows: Add build.rs to embed .ico in executable")
    print("  3. GUI: Logo image can be embedded using makepad's image widget")

if __name__ == "__main__":
    main()
