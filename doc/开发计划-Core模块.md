# Core模块开发计划

## 总体策略

采用 **Core-First** 开发策略:
1. 先实现完整的后端功能 (独立于GUI)
2. 通过命令行充分测试
3. 确保Core稳定后再集成GUI

---

## 阶段1: 最小可用Core (MVP)

**目标**: 实现一个可以工作的本地SOCKS5服务器,能够转发流量到单个上游代理

**时间估计**: 3-5天

### 任务分解

#### Task 1.1: 项目结构搭建
- [ ] 创建 `src/proxy/` 模块目录
- [ ] 创建模块文件骨架
  - `src/proxy/mod.rs`
  - `src/proxy/server.rs`
  - `src/proxy/upstream.rs`
  - `src/proxy/relay.rs`
  - `src/proxy/config.rs`
- [ ] 添加依赖到 `Cargo.toml`
- [ ] 创建 `examples/proxy_server.rs` (命令行测试程序)

**依赖清单**:
```toml
[dependencies]
# 异步运行时
tokio = { version = "1.43", features = ["full"] }

# SOCKS5协议
fast-socks5 = "0.9"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 配置管理
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"
```

#### Task 1.2: 配置结构定义
**文件**: `src/proxy/config.rs`

```rust
// 定义核心配置结构
pub struct ProxyConfig {
    pub listen_addr: String,      // 本地监听地址
    pub upstream: UpstreamConfig,  // 上游配置
}

pub struct UpstreamConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}
```

#### Task 1.3: SOCKS5服务器实现
**文件**: `src/proxy/server.rs`

**核心功能**:
- [ ] 监听本地端口 (默认: 127.0.0.1:10808)
- [ ] 接受客户端连接
- [ ] SOCKS5握手处理
- [ ] 获取目标地址
- [ ] 调用relay模块转发流量

**实现要点**:
```rust
pub struct ProxyServer {
    config: ProxyConfig,
    upstream: Arc<UpstreamProxy>,
}

impl ProxyServer {
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.listen_addr).await?;
        tracing::info!("Proxy server listening on {}", self.config.listen_addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let upstream = self.upstream.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_client(stream, peer_addr, upstream).await {
                    tracing::error!("Client error: {}", e);
                }
            });
        }
    }
}
```

#### Task 1.4: 上游代理连接
**文件**: `src/proxy/upstream.rs`

**核心功能**:
- [ ] 存储上游配置
- [ ] 建立到上游SOCKS5的连接
- [ ] 通过上游连接目标地址

**实现要点**:
```rust
pub struct UpstreamProxy {
    config: UpstreamConfig,
}

impl UpstreamProxy {
    pub async fn connect(&self, target_addr: &str, target_port: u16) -> Result<TcpStream> {
        // 1. 连接到上游SOCKS5
        let stream = TcpStream::connect((self.config.host.as_str(), self.config.port)).await?;

        // 2. 使用fast-socks5建立SOCKS5连接
        let stream = Socks5Stream::connect(
            stream,
            target_addr,
            target_port,
            self.config.username.clone(),
            self.config.password.clone(),
        ).await?;

        Ok(stream.into_inner())
    }
}
```

#### Task 1.5: 流量转发实现
**文件**: `src/proxy/relay.rs`

**核心功能**:
- [ ] 双向TCP流量转发
- [ ] 基础流量统计
- [ ] 错误处理

**实现要点**:
```rust
pub async fn relay_traffic(
    mut client: TcpStream,
    mut upstream: TcpStream,
) -> Result<(u64, u64)> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();

    let client_to_upstream = async {
        tokio::io::copy(&mut client_read, &mut upstream_write).await
    };

    let upstream_to_client = async {
        tokio::io::copy(&mut upstream_read, &mut client_write).await
    };

    let (sent, recv) = tokio::try_join!(client_to_upstream, upstream_to_client)?;
    Ok((sent, recv))
}
```

#### Task 1.6: 命令行测试程序
**文件**: `examples/proxy_server.rs`

**功能**:
- [ ] 解析命令行参数
- [ ] 启动代理服务器
- [ ] 显示运行状态
- [ ] 优雅关闭

**使用示例**:
```bash
# 启动代理服务器
cargo run --example proxy_server -- \
  --listen 127.0.0.1:10808 \
  --upstream host:port:user:pass

# 测试连接 (使用curl)
curl --proxy socks5://127.0.0.1:10808 https://ifconfig.me
```

#### Task 1.7: 基础测试
- [ ] 测试本地服务器启动
- [ ] 测试SOCKS5握手
- [ ] 测试HTTP流量转发
- [ ] 测试HTTPS流量转发
- [ ] 测试连接关闭处理

---

## 阶段2: Core功能扩展

**目标**: 添加多上游、健康检查、故障切换

**时间估计**: 1-2周

### Task 2.1: 多上游管理
**文件**: `src/proxy/upstream.rs` (扩展)

- [ ] 实现 `UpstreamManager`
- [ ] 支持添加/删除上游
- [ ] 实现选择策略接口 `ProxySelector`
- [ ] 实现轮询选择器
- [ ] 实现随机选择器
- [ ] 实现优先级选择器

### Task 2.2: 健康检查
**文件**: `src/proxy/health.rs`

- [ ] 实现 `HealthChecker`
- [ ] 定期检查上游可用性
- [ ] 更新代理状态
- [ ] 触发状态变更通知

### Task 2.3: 自动故障切换
**文件**: `src/proxy/upstream.rs` (扩展)

- [ ] 检测连接失败
- [ ] 自动切换到健康的上游
- [ ] 记录切换日志

### Task 2.4: 监控统计
**文件**: `src/proxy/monitor.rs`

- [ ] 收集全局统计
- [ ] 收集每个上游的统计
- [ ] 提供查询接口

### Task 2.5: 配置持久化
**文件**: `src/proxy/config.rs` (扩展)

- [ ] 支持从YAML加载配置
- [ ] 支持保存配置
- [ ] 支持热重载

---

## 阶段3: GUI集成

**目标**: 将Core集成到Makepad GUI

**时间估计**: 1-2周

### Task 3.1: GUI扩展
- [ ] 添加代理服务Tab
- [ ] 实现启动/停止按钮
- [ ] 显示运行状态

### Task 3.2: 状态同步
- [ ] 使用tokio::sync::watch同步状态
- [ ] 实时更新GUI显示

### Task 3.3: 监控面板
- [ ] 显示连接数
- [ ] 显示流量统计
- [ ] 显示上游状态

### Task 3.4: 日志显示
- [ ] 显示最近日志
- [ ] 支持日志过滤

---

## 开发规范

### 代码风格
- 使用 `cargo fmt` 格式化
- 使用 `cargo clippy` 检查
- 添加文档注释
- 添加单元测试

### 错误处理
```rust
use anyhow::{Result, Context};

pub fn foo() -> Result<()> {
    bar().context("Failed to do bar")?;
    Ok(())
}
```

### 日志记录
```rust
use tracing::{debug, info, warn, error};

info!("Server started on {}", addr);
debug!("Processing request: {:?}", req);
```

### 测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_start() {
        // ...
    }
}
```

---

## 验收标准

### 阶段1验收
- [ ] 代理服务器可以启动
- [ ] 可以转发HTTP流量
- [ ] 可以转发HTTPS流量
- [ ] curl测试成功
- [ ] Clash可以通过代理连接

### 阶段2验收
- [ ] 支持多个上游
- [ ] 健康检查正常工作
- [ ] 故障切换<3秒
- [ ] 统计数据准确

### 阶段3验收
- [ ] GUI可以启动/停止服务
- [ ] 状态实时更新
- [ ] 监控数据显示正确
- [ ] 日志显示正常

---

## 风险和注意事项

### 风险1: SOCKS5协议兼容性
**缓解**: 使用fast-socks5成熟库,充分测试

### 风险2: 异步编程复杂度
**缓解**: 参考Tokio官方示例,从简单开始

### 风险3: 错误处理不完善
**缓解**: 使用Result和anyhow,记录详细日志

---

## 下一步行动

### 立即开始 (今天)
1. 创建 `src/proxy/` 目录结构
2. 添加依赖到 Cargo.toml
3. 实现基础的配置结构

### 短期 (本周)
1. 实现SOCKS5服务器
2. 实现单上游转发
3. 创建命令行测试程序

### 中期 (1-2周)
1. 添加多上游支持
2. 实现健康检查
3. 充分测试

---

最后更新: 2026-02-02
