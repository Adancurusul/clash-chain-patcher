# Clash Chain Patcher

<p align="center">
  <img src="logo/logo_32.png" alt="Logo" width="64" height="64">
</p>

<p align="center">
  为 Clash 配置添加 SOCKS5 代理链的图形化工具
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#下载">下载</a> •
  <a href="#使用方法">使用方法</a> •
  <a href="#编译构建">编译构建</a> •
  <a href="README.md">English</a>
</p>

---

## 界面截图

<p align="center">
  <img src="img/main.png" alt="主界面" width="400">
</p>

## 功能特性

- **双链组** - 同时创建自动选择 (Chain-Auto) 和手动选择 (Chain-Selector) 两个组
- **延迟测试** - Chain-Auto 使用 url-test 自动选择最快节点
- **智能检测** - 从 MATCH 规则自动检测主入口组
- **代理池** - 管理多个 SOCKS5 上游代理
- **文件监视** - Clash 配置被外部修改时自动重新应用
- **最近文件** - 快速访问最近使用的配置文件（支持删除）
- **跨平台** - 支持 Windows、macOS、Linux

## 工作原理

工具创建中继代理链：

1. **添加你的 SOCKS5 代理** 到代理池
2. **选择 Clash 配置** 文件
3. **点击 Apply** - 工具会：
   - 创建 `Local-Chain-Proxy` SOCKS5 节点
   - 为每个现有代理创建 `-Chain` 中继（如 `Tokyo-01-Chain`）
   - 创建 **Chain-Selector** 组（select 类型，用于手动选择）
   - 创建 **Chain-Auto** 组（url-test 类型，用于自动选最快）
   - 将两个组添加到主入口组（从 MATCH 规则自动检测）

流量走向：`VPN 节点 → 你的 SOCKS5 代理 → 互联网`

## 下载

从 [Releases](../../releases) 下载最新版本：

| 平台 | 文件 | 说明 |
|------|------|------|
| Windows | `*-setup.exe` | NSIS 安装包（推荐） |
| Windows | `*-portable.zip` | 便携版（解压即用） |
| macOS | `Clash-Chain-Patcher-macos.dmg` | 拖拽到应用程序 |
| macOS | `Clash-Chain-Patcher-macos.zip` | 包含 .app 包 |
| Linux | `clash-chain-patcher-linux` | 需要 `chmod +x` |

### macOS: 首次启动

由于应用未签名，macOS Gatekeeper 会阻止打开。

**解决方法：在终端运行**
```bash
xattr -cr /Applications/Clash\ Chain\ Patcher.app
```

### Linux: 首次启动
```bash
chmod +x clash-chain-patcher-linux
./clash-chain-patcher-linux
```

## 使用方法

### 第一步：添加 SOCKS5 代理

1. 填写 **Host**、**Port**、**User**、**Pass** 字段
2. 点击 **+ Add** 添加到代理池
3. 确保代理显示 ✓（已启用）

### 第二步：选择 Clash 配置

1. 点击 **Select** 选择 Clash YAML 配置文件
2. 最近使用的文件会被保存，点击 ▼ 可快速选择

### 第三步：应用

1. 点击 **Apply** 按钮
2. 等待完成提示
3. 在 Clash 中刷新配置

### 第四步：使用链式节点

应用后，Clash 侧边栏顶部会出现两个新组：

- **Chain-Selector** - 手动选择链式节点
- **Chain-Auto** - 自动选择最快链式节点（显示延迟）

可以直接选择这两个组，或从主代理组中选择它们。

### 文件监视（可选）

启用 **Watch** 可在 Clash 配置被修改时（如订阅更新）自动重新应用。

## 示例

原始配置：
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

应用后：
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

## 编译构建

### 前置要求
- Rust 1.70+

### 从源码构建

```bash
git clone https://github.com/user/clash-chain-patcher.git
cd clash-chain-patcher
cargo build --release
```

## 技术栈

- **GUI**: [Makepad](https://github.com/makepad/makepad) - Rust UI 框架
- **YAML**: serde_yaml
- **文件对话框**: rfd

## 免责声明

本软件仅供**学习交流和技术研究**使用。

- 本工具仅用于学习网络技术和个人研究目的
- 用户需自行确保使用行为符合当地法律法规
- 作者不对因使用本软件产生的任何滥用、损失或法律后果承担责任

**风险自负。**

## 许可证

MIT License
