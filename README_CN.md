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

## 功能特性

- **添加代理链** - 在现有 Clash 代理前添加 SOCKS5 代理
- **过滤代理** - 通过关键词筛选特定代理
- **两种输入格式** - 支持 `user:pass@host:port` 和 `ip:port:user:pass`
- **预览更改** - 应用前查看将要修改的内容
- **跨平台** - 支持 Windows、macOS、Linux

## 下载

从 [Releases](../../releases) 下载最新版本：

| 平台 | 文件 | 说明 |
|------|------|------|
| Windows | `clash-chain-patcher-windows.exe` | 单文件可执行 |
| macOS | `Clash-Chain-Patcher-macos.dmg` | 拖拽到应用程序 |
| macOS | `Clash-Chain-Patcher-macos.zip` | 包含 .app 包 |
| Linux | `clash-chain-patcher-linux` | 需要 `chmod +x` 添加执行权限 |

## 使用方法

### 1. 选择配置文件
点击 **Select** 选择你的 Clash YAML 配置文件。

### 2. 输入 SOCKS5 代理
填写代理信息：
- **Host**: 代理服务器主机名或 IP
- **Port**: 代理端口（如 1080）
- **User/Pass**: 认证凭据（可选）

或者粘贴代理字符串后点击 **Fill**：
```
user:pass@host:port
# 或
ip:port:user:pass
```

### 3. 过滤（可选）
输入关键词（逗号分隔）来只修改匹配的代理。
留空则修改所有代理。

### 4. 预览和应用
- **Preview** - 查看将创建哪些代理链
- **Apply** - 生成修改后的配置
- **Save** - 保存结果到新文件

## 工作原理

工具通过以下方式创建"中继"代理链：

1. 读取你的 Clash 配置
2. 为你的代理服务器创建一个 SOCKS5 代理条目
3. 为每个原始代理创建一个新的 "relay" 类型代理，通过你的 SOCKS5 进行链接

示例：
```yaml
# 原始代理
- name: "Tokyo-01"
  type: vmess
  server: example.com
  ...

# 生成的链
- name: "Tokyo-01-chain"
  type: relay
  proxies:
    - "SOCKS5-Proxy"
    - "Tokyo-01"
```

## 编译构建

### 前置要求
- Rust 1.70+
- Python 3.8+（用于图标生成）

### 从源码构建

```bash
# 克隆
git clone https://github.com/user/clash-chain-patcher.git
cd clash-chain-patcher

# 生成图标（可选，用于自定义 logo）
pip install pillow
python scripts/generate_icons.py

# 构建
cargo build --release

# macOS: 创建 .app 包
./scripts/bundle_macos.sh
```

### 构建输出
- **Windows**: `target/release/clash-chain-patcher.exe`（图标已嵌入）
- **macOS**: `target/bundle/Clash Chain Patcher.app`
- **Linux**: `target/release/clash-chain-patcher`

## 开发

### 项目结构
```
clash-chain-patcher-rust/
├── src/
│   ├── main.rs          # 入口点
│   ├── app.rs           # GUI 应用
│   └── patcher.rs       # 核心修改逻辑
├── logo/
│   ├── clash-chain-patcher.png  # 源 logo
│   ├── AppIcon.icns     # macOS 图标
│   └── app.ico          # Windows 图标
├── scripts/
│   ├── generate_icons.py    # 图标转换器
│   └── bundle_macos.sh      # macOS 打包器
└── .github/workflows/
    └── release.yml      # CI/CD
```

### 技术栈
- **GUI**: [Makepad](https://github.com/makepad/makepad) - Rust UI 框架
- **YAML**: serde_yaml
- **文件对话框**: rfd

## 免责声明

本软件仅供**学习交流和技术研究**使用。

- 本工具仅用于学习网络技术和个人研究目的
- 用户需自行确保使用行为符合当地法律法规
- 作者不对因使用本软件产生的任何滥用、损失或法律后果承担责任
- 使用本软件即表示您已理解并接受上述条款

**风险自负。**

## 许可证

MIT License

## 贡献

欢迎提交 Issues 和 Pull Requests！
