# Clash Chain Patcher

<p align="center">
  <img src="logo/logo_32.png" alt="Logo" width="64" height="64">
</p>

<p align="center">
  A GUI tool to add SOCKS5 proxy chains to Clash configurations
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#download">Download</a> •
  <a href="#usage">Usage</a> •
  <a href="#building">Building</a> •
  <a href="README_CN.md">中文文档</a>
</p>

---

## Screenshot

<p align="center">
  <img src="img/main.png" alt="Main Interface" width="400">
</p>

## Features

- **Dual Chain Groups** - Creates both auto-select (Chain-Auto) and manual-select (Chain-Selector) groups
- **Latency Testing** - Chain-Auto uses url-test for automatic fastest node selection
- **Smart Detection** - Auto-detects main entry group from MATCH rule
- **Proxy Pool** - Manage multiple SOCKS5 upstream proxies
- **File Watching** - Auto re-apply when Clash config changes externally
- **Recent Files** - Quick access to recently used config files (with delete)
- **Cross-platform** - Windows, macOS, Linux

## How It Works

The tool creates relay proxy chains:

1. **Add your SOCKS5 proxy** to the Proxy Pool
2. **Select Clash config** file
3. **Click Apply** - The tool will:
   - Create a `Local-Chain-Proxy` SOCKS5 node
   - Create `-Chain` relay for each existing proxy (e.g., `Tokyo-01-Chain`)
   - Create **Chain-Selector** group (select type, for manual selection)
   - Create **Chain-Auto** group (url-test type, for auto fastest selection)
   - Add both groups to the main entry group (detected from MATCH rule)

Traffic flow: `VPN Node → Your SOCKS5 Proxy → Internet`

## Download

Download the latest release from [Releases](../../releases):

| Platform | File | Note |
|----------|------|------|
| Windows | `*-setup.exe` | NSIS Installer (Recommended) |
| Windows | `*-portable.zip` | Portable version (unzip and run) |
| macOS | `Clash-Chain-Patcher-macos.dmg` | Drag to Applications |
| macOS | `Clash-Chain-Patcher-macos.zip` | Contains .app bundle |
| Linux | `clash-chain-patcher-linux` | Make executable with `chmod +x` |

### macOS: First Launch

Since the app is not signed with an Apple Developer certificate, macOS Gatekeeper will block it.

**Solution: Run in Terminal**
```bash
xattr -cr /Applications/Clash\ Chain\ Patcher.app
```

### Linux: First Launch
```bash
chmod +x clash-chain-patcher-linux
./clash-chain-patcher-linux
```

## Usage

### Step 1: Add SOCKS5 Proxy

1. Fill in **Host**, **Port**, **User**, **Pass** fields
2. Click **+ Add** to add to Proxy Pool
3. Ensure the proxy shows ✓ (enabled)

### Step 2: Select Clash Config

1. Click **Select** to choose your Clash YAML config file
2. Recent files are saved for quick access (click ▼ to show)

### Step 3: Apply

1. Click **Apply** button
2. Wait for completion message
3. In Clash, refresh configuration

### Step 4: Use Chain Nodes

After applying, you'll see two new groups in Clash sidebar (at the top):

- **Chain-Selector** - Manual selection of chain nodes
- **Chain-Auto** - Auto-select fastest chain node (with latency display)

Select either group, or select them from your main proxy group.

### File Watch (Optional)

Enable **Watch** to auto re-apply when Clash config changes (e.g., subscription updates).

## Example

Original config:
```yaml
proxies:
  - name: "Tokyo-01"
    type: vmess
    server: example.com

proxy-groups:
  - name: "Proxy"
    type: select
    proxies: ["Tokyo-01"]

rules:
  - MATCH,Proxy
```

After Apply:
```yaml
proxies:
  - name: "Local-Chain-Proxy"
    type: socks5
    server: your-socks5-host
    port: 1080
    username: user
    password: pass

  - name: "Tokyo-01"
    type: vmess
    server: example.com

  - name: "Tokyo-01-Chain"
    type: relay
    proxies:
      - "Tokyo-01"
      - "Local-Chain-Proxy"

proxy-groups:
  - name: "Chain-Selector"
    type: select
    proxies: ["Tokyo-01-Chain"]

  - name: "Chain-Auto"
    type: url-test
    proxies: ["Tokyo-01-Chain"]
    url: "http://www.gstatic.com/generate_204"
    interval: 300

  - name: "Proxy"
    type: select
    proxies: ["Chain-Selector", "Chain-Auto", "Tokyo-01"]
```

## Building

### Prerequisites
- Rust 1.70+

### Build from source

```bash
git clone https://github.com/user/clash-chain-patcher.git
cd clash-chain-patcher
cargo build --release
```

## Tech Stack

- **GUI**: [Makepad](https://github.com/makepad/makepad) - Rust UI framework
- **YAML**: serde_yaml
- **File dialogs**: rfd

## Disclaimer

This software is provided for **educational and research purposes only**.

- This tool is intended solely for learning network technologies and personal research
- Users are responsible for ensuring their use complies with all applicable local laws and regulations
- The author assumes no liability for any misuse, damage, or legal consequences arising from the use of this software

**Use at your own risk.**

## License

MIT License
