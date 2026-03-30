# Clash Chain Patcher

<p align="center">
  <img src="logo/logo_32.png" alt="Logo" width="64" height="64">
</p>

<p align="center">
  A GUI & CLI tool to add SOCKS5 proxy chains to Clash configurations
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.3.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/rust-1.70%2B-orange" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#download">Download</a> •
  <a href="#gui-usage">GUI</a> •
  <a href="#cli-usage">CLI</a> •
  <a href="#building">Building</a>
</p>

---

## Screenshot

<p align="center">
  <img src="img/main.png" alt="Main Interface" width="400">
</p>

## Features

- **Dual Chain Groups** - Creates both Chain-Auto (fastest auto-select) and Chain-Selector (manual) groups
- **Rules Rewrite** - Selectively redirect Clash rules to use chain proxies (Chain-Selector / Chain-Auto)
- **Proxy Pool** - Manage multiple SOCKS5 upstream proxies with health checking
- **File Watching** - Auto re-apply when Clash config changes externally
- **CLI Support** - Full command-line interface (`ccp`) for scripting and automation
- **Cross-platform** - Windows, macOS, Linux

## How It Works

```
Traffic flow: You → Clash → VPN Node → Your SOCKS5 Proxy → Internet
```

1. **Add your SOCKS5 proxy** to the Proxy Pool
2. **Select Clash config** file
3. **Configure Rules Rewrite** - Choose which rule groups to redirect through chain proxies
4. **Click Apply** - Creates chain relays, selector groups, and rewrites rules

## Download

Download the latest release from [Releases](../../releases):

| Platform | File | Note |
|----------|------|------|
| Windows | `*-setup.exe` | NSIS Installer (Recommended) |
| Windows | `*-portable.zip` | Portable version |
| macOS | `*.dmg` | Drag to Applications |
| Linux | `clash-chain-patcher-linux` | `chmod +x` and run |

### macOS: First Launch

```bash
xattr -cr /Applications/Clash\ Chain\ Patcher.app
```

## GUI Usage

### Step 1: Add SOCKS5 Proxy

Fill in **Host**, **Port**, **User**, **Pass**, click **+ Add**.

### Step 2: Select Clash Config

Click **Select** to choose your Clash YAML config file.

### Step 3: Rules Rewrite (Optional)

After loading a config, the **Rules Rewrite** panel auto-detects all proxy groups referenced in rules:

- Click the **check (✓)** to enable/disable a group for rewriting
- Click the **target button** to cycle through: `Keep` → `Chain-Selector` → `Chain-Auto`
- Non-DIRECT/REJECT groups are auto-checked with Chain-Selector by default

### Step 4: Apply

Click **Apply**. The tool will:
- Create relay chains for each proxy node
- Create Chain-Selector and Chain-Auto groups
- Rewrite checked rules to point to the selected chain group

## CLI Usage

The CLI binary is called `ccp` (Clash Chain Patcher).

### Show config info

```bash
ccp info config.yaml
```

Output:
```
Rules: 3 groups, 630 total rules
Group                                       Rules
--------------------------------------------------
Proxy                                         371
DIRECT                                        233
REJECT                                         26

Proxy nodes: 45
Proxy groups: 5
```

### Apply chain patch + rewrite rules

```bash
# Auto-detect main group and replace with Chain-Selector
ccp apply config.yaml -p host:port:user:pass -r auto

# Specify target explicitly
ccp apply config.yaml -p host:port -r "Proxy=Chain-Selector"

# Multiple rewrites
ccp apply config.yaml -p user:pass@host:port \
  -r "Proxy=Chain-Selector" \
  -r "Streaming=Chain-Auto"
```

### Rewrite rules only (no chain creation)

```bash
ccp rules config.yaml -r auto
ccp rules config.yaml -r "Proxy=Chain-Auto"
```

### Options

```
ccp apply [OPTIONS] --proxy <PROXY> <CONFIG>

Options:
  -p, --proxy <PROXY>      SOCKS5 proxy string
  -r, --rewrite <REWRITE>  Rule rewrite (repeatable), or "auto"
      --no-backup          Skip creating backup
      --suffix <SUFFIX>    Chain suffix [default: -Chain]
```

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
  - DOMAIN,google.com,Proxy
  - MATCH,Proxy
```

After `ccp apply config.yaml -p 1.2.3.4:1080 -r auto`:
```yaml
proxies:
  - name: "Local-Chain-Proxy"
    type: socks5
    server: 1.2.3.4
    port: 1080

  - name: "Tokyo-01"
    type: vmess
    server: example.com

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

  - name: "Tokyo-01-Chain"
    type: relay
    proxies: ["Tokyo-01", "Local-Chain-Proxy"]

rules:
  - DOMAIN,google.com,Chain-Selector    # was: Proxy
  - MATCH,Chain-Selector                # was: Proxy
```

## Building

### Prerequisites
- Rust 1.70+

### Build from source

```bash
git clone https://github.com/user/clash-chain-patcher.git
cd clash-chain-patcher

# Build GUI
cargo build --release --bin clash-chain-patcher

# Build CLI
cargo build --release --bin ccp
```

## Tech Stack

- **GUI**: [Makepad](https://github.com/makepad/makepad) - Rust native UI framework
- **CLI**: [clap](https://github.com/clap-rs/clap) - Command-line argument parser
- **YAML**: serde_yaml
- **File dialogs**: rfd

## Disclaimer

This software is provided for **educational and research purposes only**.

- Intended solely for learning network technologies and personal research
- Users are responsible for ensuring compliance with applicable laws
- The author assumes no liability for misuse or consequences

**Use at your own risk.**

## License

MIT License
