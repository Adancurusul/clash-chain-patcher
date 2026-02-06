# 开发文档

## 开发环境配置

### 前置要求

| 工具 | 最低版本 | 推荐版本 | 用途 |
|------|----------|----------|------|
| Rust | 1.70 | 1.91+ | 核心开发语言 |
| Cargo | - | 最新 | 包管理和构建 |
| Python | 3.8 | 3.11+ | 图标生成脚本 |
| Git | 2.0 | 最新 | 版本控制 |

### 平台特定依赖

#### Windows
```powershell
# 安装Rust (通过rustup)
# 访问 https://rustup.rs/

# 安装Python
# 访问 https://www.python.org/downloads/

# 安装Pillow (图标生成)
pip install pillow
```

#### macOS
```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装Python (通过Homebrew)
brew install python

# 安装Pillow
pip3 install pillow

# 安装create-dmg (可选,用于生成DMG)
brew install create-dmg
```

#### Linux (Ubuntu/Debian)
```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装系统依赖
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libxcursor-dev \
    libx11-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libasound2-dev \
    libpulse-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libgl1-mesa-dev \
    libgtk-3-dev \
    python3 \
    python3-pip

# 安装Pillow
pip3 install pillow
```

### IDE配置

#### VS Code (推荐)

**必装插件**:
- `rust-analyzer`: Rust语言支持
- `CodeLLDB`: 调试支持
- `Even Better TOML`: TOML文件支持

**推荐插件**:
- `crates`: Cargo依赖管理
- `Error Lens`: 内联错误显示

**配置文件** (`.vscode/settings.json`):
```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true
}
```

#### IntelliJ IDEA / CLion

**插件**:
- IntelliJ Rust

---

## 运行指南

### 开发模式运行

```bash
# 编译并运行 (debug模式)
cargo run

# 启用日志
RUST_LOG=debug cargo run

# 仅检查编译错误
cargo check

# 运行单元测试
cargo test

# 运行单元测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_parse_proxy_string
```

### 发布模式构建

```bash
# 构建发布版本
cargo build --release

# 构建输出位置
# Windows: target/release/clash-chain-patcher.exe
# macOS/Linux: target/release/clash-chain-patcher
```

### 生成图标

```bash
# 从源图标生成各平台图标
python scripts/generate_icons.py

# 输出位置: logo/
```

### 跨平台打包

```bash
# 安装cargo-packager
cargo install cargo-packager --version 0.10.1

# Windows: 生成NSIS安装包
cargo packager --release --formats nsis

# macOS: 生成.app bundle
cargo packager --release --formats app

# Linux: 生成DEB包
cargo packager --release --formats deb

# 输出位置: dist/
```

---

## 代码规范

### Rust代码风格

遵循官方Rust代码风格指南:
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查常见问题

```bash
# 格式化所有代码
cargo fmt

# 运行Clippy
cargo clippy -- -D warnings

# 修复部分可自动修复的问题
cargo clippy --fix
```

### 命名约定

- **文件名**: snake_case (例: `patcher.rs`, `proxy_server.rs`)
- **结构体/枚举**: PascalCase (例: `ProxyServer`, `HealthStatus`)
- **函数/变量**: snake_case (例: `parse_proxy_string`, `upstream_manager`)
- **常量**: SCREAMING_SNAKE_CASE (例: `DEFAULT_PORT`, `MAX_CONNECTIONS`)
- **trait**: PascalCase (例: `ProxySelector`, `HealthChecker`)

### 文档注释

```rust
/// 解析代理字符串为SOCKS5配置
///
/// 支持两种格式:
/// 1. `user:pass@host:port`
/// 2. `host:port:user:pass`
///
/// # 参数
/// * `input` - 代理字符串
///
/// # 返回
/// * `Some(Socks5Proxy)` - 解析成功
/// * `None` - 解析失败
///
/// # 示例
/// ```
/// let proxy = parse_proxy_string("user:pass@host.com:1080");
/// assert!(proxy.is_some());
/// ```
pub fn parse_proxy_string(input: &str) -> Option<Socks5Proxy> {
    // ...
}
```

### 错误处理

```rust
// 使用Result返回错误
pub fn load_config(path: &Path) -> Result<Config, Error> {
    // ...
}

// 使用anyhow简化错误传播
use anyhow::{Result, Context};

pub fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;

    let config: Config = serde_yaml::from_str(&content)
        .context("Failed to parse config")?;

    Ok(config)
}
```

---

## 调试技巧

### 日志输出

```rust
// 使用tracing crate记录日志
use tracing::{debug, info, warn, error};

info!("Server started on {}", addr);
debug!("Processing request: {:?}", request);
warn!("Upstream proxy unhealthy: {}", proxy_id);
error!("Failed to connect: {}", err);
```

```bash
# 设置日志级别
RUST_LOG=trace cargo run  # 最详细
RUST_LOG=debug cargo run
RUST_LOG=info cargo run   # 默认
RUST_LOG=warn cargo run
RUST_LOG=error cargo run  # 仅错误

# 针对特定模块
RUST_LOG=clash_chain_patcher::proxy=debug cargo run
```

### 调试器使用

#### VS Code + CodeLLDB

在 `.vscode/launch.json` 添加配置:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable",
            "cargo": {
                "args": [
                    "build",
                    "--bin=clash-chain-patcher",
                    "--package=clash-chain-patcher"
                ],
                "filter": {
                    "name": "clash-chain-patcher",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### 性能分析

```bash
# 使用cargo-flamegraph生成火焰图
cargo install flamegraph

# Linux
cargo flamegraph

# macOS (需要sudo)
sudo cargo flamegraph

# Windows (使用perf)
# 参考: https://github.com/flamegraph-rs/flamegraph
```

---

## 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proxy_string() {
        let result = parse_proxy_string("user:pass@host:1080");
        assert!(result.is_some());

        let proxy = result.unwrap();
        assert_eq!(proxy.host, "host");
        assert_eq!(proxy.port, 1080);
    }

    #[test]
    #[should_panic(expected = "Invalid port")]
    fn test_invalid_port() {
        parse_proxy_string("host:99999").unwrap();
    }
}
```

### 集成测试

在 `tests/` 目录创建集成测试:

```rust
// tests/proxy_server_test.rs
use clash_chain_patcher::proxy::ProxyServer;

#[tokio::test]
async fn test_server_lifecycle() {
    let server = ProxyServer::new(config).await.unwrap();
    server.start().await.unwrap();
    // ...
    server.stop().await.unwrap();
}
```

### 运行测试

```bash
# 所有测试
cargo test

# 单元测试
cargo test --lib

# 集成测试
cargo test --test '*'

# 带输出
cargo test -- --nocapture

# 单线程运行(避免并发问题)
cargo test -- --test-threads=1
```

---

## 常见问题

### Q: 编译错误 "linker not found"
**Windows**: 安装 Visual Studio Build Tools
**Linux**: `sudo apt-get install build-essential`
**macOS**: `xcode-select --install`

### Q: Makepad相关编译错误
确保安装了所有平台依赖,参考"开发环境配置"章节。

### Q: cargo-packager打包失败
检查 `dist/resources/` 目录是否存在,运行:
```bash
cargo run --manifest-path packaging/before-packaging-command/Cargo.toml -- before-packaging
```

### Q: macOS上应用无法打开
```bash
xattr -cr "Clash Chain Patcher.app"
```

---

## 开发记录

### 2024-12-17 - v0.1.2发布
- 修复Windows标题栏显示"Makepad"问题
  - 解决方案: 隐藏Makepad内置标题栏
- 修复Windows独立exe无法运行
  - 原因: 缺少Makepad资源文件
  - 解决方案: 发布便携版ZIP,包含资源目录
- 更新README,添加主界面截图
- CI/CD: 移除独立exe,保留NSIS和便携版ZIP

### 2024-12-17 - Makepad资源打包方案
- 问题: robius-packaging-commands无法发现中文字体crate
- 根本原因: 中文字体crate不生成.path文件
- 解决方案: 自定义打包命令
  - 使用cargo-metadata发现所有Makepad依赖
  - 直接复制资源到dist/resources/
  - 在Cargo.toml中显式列出所有资源目录

### 2024-12-16 - 项目初始化
- 使用Makepad框架实现跨平台GUI
- 实现核心YAML配置修改功能
- 支持两种代理字符串格式
- 添加关键词过滤功能
- 配置CI/CD自动发布流程

---

最后更新: 2026-02-02
