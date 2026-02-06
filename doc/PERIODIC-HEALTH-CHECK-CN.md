# 长时间周期性健康检测 - 功能说明

**版本**: 0.1.2
**日期**: 2026-02-03

---

## ❓ 用户问题

> 现在是已经会长时间检测吗?

---

## ✅ 答案：代码已实现，但 **GUI 中未启用**

### 当前状态

| 功能 | 后端代码 | GUI 集成 | 状态 |
|------|---------|---------|------|
| 周期性健康检测 | ✅ 已实现 | ❌ 未启用 | 可用但未使用 |
| 手动健康检测 | ✅ 已实现 | ✅ 已启用 | 正常工作 |
| 配置保存 | ✅ 已实现 | ✅ 已启用 | 正常工作 |

---

## 🔍 技术细节

### 1. 后端代码已完整实现

#### 配置文件支持

**位置**: `~/Library/Application Support/clash-chain-patcher/config.json`

```json
{
  "health_check": {
    "enabled": true,              // 是否启用周期性检测
    "interval_seconds": 300,      // 检测间隔（5分钟）
    "test_url": "http://www.gstatic.com/generate_204",
    "timeout_seconds": 10,        // 超时时间
    "failure_threshold": 3        // 失败阈值
  }
}
```

**代码**: `src/config/manager.rs:110-137`

```rust
pub struct HealthCheckConfig {
    /// Whether health checks are enabled
    pub enabled: bool,

    /// Check interval (seconds)
    pub interval_seconds: u64,  // 默认 300 秒 = 5 分钟

    /// Test URL
    pub test_url: String,

    /// Timeout duration (seconds)
    pub timeout_seconds: u64,

    /// Number of failures before marking as unhealthy
    pub failure_threshold: u32,
}
```

---

#### 周期性检测实现

**代码**: `src/health/checker.rs:225-255`

```rust
pub fn start_periodic_check<F>(
    self: Arc<Self>,
    proxies: Arc<RwLock<Vec<UpstreamProxy>>>,
    mut callback: F,
) -> tokio::task::JoinHandle<()>
where
    F: FnMut(String, HealthCheckResult) + Send + 'static,
{
    let check_interval = self.config.check_interval;

    tokio::spawn(async move {
        let mut interval = interval(check_interval);
        interval.tick().await; // Skip the first immediate tick

        loop {
            interval.tick().await;  // ← 周期性触发

            info!("Starting periodic health check");

            // 获取所有启用的代理
            let proxies_snapshot = {
                let proxies_guard = proxies.read().await;
                proxies_guard.clone()
            };

            // 检测每个启用的代理
            for proxy in proxies_snapshot.iter().filter(|p| p.enabled) {
                let result = self.check_proxy(proxy).await;
                callback(proxy.id.clone(), result);  // 回调通知结果
            }

            info!("Periodic health check completed");
        }
    })
}
```

**工作原理**:
1. 启动一个异步任务
2. 每隔 `check_interval`（默认 5 分钟）触发一次
3. 检测所有**启用的**代理
4. 通过回调函数通知检测结果
5. 无限循环，直到任务被停止

---

#### Bridge 层支持

**代码**: `src/bridge/health_bridge.rs:93`

```rust
pub fn start_background_check(
    &self,
    callback: impl FnMut(String, HealthCheckResult) + Send + 'static,
) -> tokio::task::JoinHandle<()> {
    let proxies = self.get_proxies_arc();
    let checker = Arc::clone(&self.checker);

    let handle = checker.start_periodic_check(proxies, callback);  // ← 调用周期性检测

    // Convert JoinHandle<Result<(), JoinError>> to JoinHandle<()>
    self.runtime.spawn(async move {
        let _ = handle.await;
    })
}
```

---

### 2. GUI 中未启用

#### 当前 GUI 只使用手动检测

**代码**: `src/app.rs`

**手动检测**:
- 点击 "Check" 按钮 → 检测单个代理
- 点击 "Check All" 按钮 → 检测所有代理
- **一次性检测，不会持续运行**

**未使用的功能**:
- ❌ `start_periodic_check()` - 未调用
- ❌ `start_background_check()` - 未调用
- ❌ 配置中的 `health_check.enabled` - 未读取

---

## 🎯 如何启用长时间检测

### 方案 1: 在 Apply 时启动（推荐）

**逻辑**: Apply 后自动启动后台健康检测

```rust
// src/app.rs

fn apply_with_pool(&mut self, cx: &mut Cx, _config: &str) {
    // ... 现有的 Apply 逻辑 ...

    // ✨ 新增：启动周期性健康检测
    if let Some(state) = &mut self.state.proxy_state {
        state.start_periodic_health_check(|proxy_id, result| {
            // 回调：更新代理健康状态
            // 注意：这个回调在后台线程运行
            // 需要通过某种机制通知 GUI 更新（如 channel）
        });

        self.add_log(cx, "✓ Background health check started");
        self.add_log(cx, "  Checking every 5 minutes");
    }
}
```

**优点**:
- Apply 后自动启动
- 用户无需额外操作
- 符合预期（Apply = 启动所有功能）

**缺点**:
- 需要处理回调与 GUI 的同步问题
- Makepad 可能不支持从后台线程更新 UI

---

### 方案 2: 添加 "Start Auto Check" 按钮

**UI 设计**:
```
Proxy Pool [+ Add] [Check All] [Clear All] [🔄 Auto Check: OFF]
                                              ↑ 新增按钮
```

**功能**:
- 点击切换自动检测开/关
- 开启后每 5 分钟检测一次
- 关闭后停止检测

**实现**:
```rust
fn toggle_auto_health_check(&mut self, cx: &mut Cx) {
    if let Some(state) = &mut self.state.proxy_state {
        if state.is_auto_check_running() {
            // 停止自动检测
            state.stop_periodic_health_check();
            self.add_log(cx, "Auto health check stopped");
        } else {
            // 启动自动检测
            state.start_periodic_health_check(|proxy_id, result| {
                // TODO: 更新 GUI
            });
            self.add_log(cx, "Auto health check started (every 5 min)");
        }
    }
}
```

**优点**:
- 用户可控
- 明确的开关状态

**缺点**:
- 需要额外 UI 空间
- 用户需要手动开启

---

### 方案 3: 读取配置文件自动启动

**逻辑**: 如果配置文件中 `health_check.enabled = true`，应用启动时自动启动

```rust
// src/app.rs

fn init_proxy_state(&mut self, cx: &mut Cx) {
    // ... 现有初始化代码 ...

    // ✨ 读取配置并启动自动检测
    if let Some(state) = &mut self.state.proxy_state {
        if state.is_health_check_enabled() {  // 读取配置
            state.start_periodic_health_check(|proxy_id, result| {
                // TODO: 更新 GUI
            });
            self.add_log(cx, "✓ Auto health check enabled");
        }
    }
}
```

**优点**:
- 遵循配置文件设置
- 无需修改 UI

**缺点**:
- 用户不知道自动检测是否在运行
- 缺少 UI 反馈

---

## 🚧 当前障碍

### 主要问题：后台线程无法直接更新 GUI

**Makepad 限制**: GUI 更新必须在主线程进行

**周期性检测的回调**:
```rust
callback(proxy_id, result);  // ← 这个在后台线程运行！
```

**无法直接调用**:
```rust
// ❌ 错误！不在主线程
self.ui.label(id!(proxy_status_1)).set_text(cx, "✓");
```

---

### 解决方案：使用消息通道

#### 1. 创建消息通道

```rust
use std::sync::mpsc;

struct AppState {
    // ... 现有字段 ...

    /// 健康检测结果通道
    health_check_rx: Option<mpsc::Receiver<(String, HealthCheckResult)>>,
}
```

#### 2. 启动检测时创建通道

```rust
fn start_auto_health_check(&mut self, cx: &mut Cx) {
    let (tx, rx) = mpsc::channel();
    self.state.health_check_rx = Some(rx);

    if let Some(state) = &mut self.state.proxy_state {
        state.start_periodic_health_check(move |proxy_id, result| {
            // 发送到通道（后台线程安全）
            let _ = tx.send((proxy_id, result));
        });
    }
}
```

#### 3. 在事件循环中检查消息

```rust
fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
    // ... 现有事件处理 ...

    // 检查健康检测结果
    if let Some(rx) = &self.state.health_check_rx {
        while let Ok((proxy_id, result)) = rx.try_recv() {
            // 在主线程更新 GUI
            self.update_proxy_health(cx, proxy_id, result);
        }
    }
}
```

---

## 📊 当前配置文件内容

```bash
$ cat ~/Library/Application\ Support/clash-chain-patcher/config.json
```

```json
{
  "upstream_proxies": [...],
  "clash": {...},
  "local_proxy": {...},
  "health_check": {
    "enabled": true,              // ← 已配置但未使用
    "interval_seconds": 300,      // ← 5 分钟
    "test_url": "http://www.gstatic.com/generate_204",
    "timeout_seconds": 10,
    "failure_threshold": 3
  }
}
```

**结论**: 配置已经存在，只需要在代码中读取并使用。

---

## ✅ 总结

### 当前状态

| 问题 | 答案 |
|------|------|
| **是否会长时间检测?** | ❌ 不会 - GUI 未启用 |
| **后端代码是否支持?** | ✅ 是 - 完整实现 |
| **配置文件是否支持?** | ✅ 是 - 已有配置 |
| **为什么未启用?** | GUI 集成未完成 |

---

### 手动检测 vs 自动检测

| 功能 | 手动检测 (当前) | 自动检测 (未启用) |
|------|----------------|-----------------|
| 触发方式 | 点击按钮 | 定时自动 |
| 检测间隔 | 手动控制 | 5 分钟（可配置） |
| GUI 更新 | 立即 | 需要消息通道 |
| CPU 占用 | 按需 | 持续后台运行 |
| 实现状态 | ✅ 已完成 | ⏳ 部分完成 |

---

### 需要的工作

**要启用长时间周期性检测，需要**:

1. ✅ 后端代码 - **已完成**
2. ✅ 配置支持 - **已完成**
3. ⏳ 消息通道 - **待实现**
4. ⏳ GUI 集成 - **待实现**
5. ⏳ 用户控制 - **待实现**（开/关按钮）

**预估工作量**: 2-3 小时

---

## 🔜 建议

### 如果你需要长时间检测

**选项 1 - 使用手动检测**:
- 当前完全可用
- 点击 "Check All" 按钮
- 需要时重复点击

**选项 2 - 实现自动检测**:
- 需要开发工作
- 更符合长期运行场景
- 建议使用方案 2（添加开关按钮）

---

## 💡 临时解决方案

### 使用外部脚本定时触发

虽然 GUI 未集成，但可以通过配置文件 + 外部脚本实现：

```bash
# check_proxies.sh
#!/bin/bash

# 读取配置
CONFIG=~/Library/Application\ Support/clash-chain-patcher/config.json

# 提取代理列表
# 调用 clash-chain-patcher 的健康检测 API（如果有）
# 或者直接使用 Python/Rust 脚本检测

# 每 5 分钟运行一次（cron）
*/5 * * * * /path/to/check_proxies.sh
```

**缺点**:
- GUI 不会自动更新
- 需要额外脚本维护

---

## 📚 相关代码位置

| 功能 | 文件 | 行号 |
|------|------|------|
| 周期性检测实现 | `src/health/checker.rs` | 225-255 |
| Bridge 启动方法 | `src/bridge/health_bridge.rs` | 93 |
| 配置定义 | `src/config/manager.rs` | 110-137 |
| 配置默认值 | `src/config/manager.rs` | 131 (300秒) |
| GUI 手动检测 | `src/app.rs` | 1124-1187 |

---

**总结**: 代码已实现，但 GUI 未集成。需要添加消息通道机制才能在 GUI 中启用长时间周期性检测。
