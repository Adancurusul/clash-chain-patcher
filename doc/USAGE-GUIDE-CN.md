# Clash Chain Patcher - 完整使用指南

**版本**: 0.1.2
**更新日期**: 2026-02-03

---

## 📖 目录

1. [快速开始](#快速开始)
2. [核心功能](#核心功能)
3. [详细使用步骤](#详细使用步骤)
4. [自动健康检测](#自动健康检测)
5. [配置文件管理](#配置文件管理)
6. [常见问题](#常见问题)
7. [故障排除](#故障排除)

---

## 🚀 快速开始

### 安装运行

```bash
# 1. 克隆仓库
git clone https://github.com/your-username/clash-chain-patcher
cd clash-chain-patcher

# 2. 编译（Release 模式）
cargo build --release

# 3. 运行
./target/release/clash-chain-patcher
```

---

## 🎯 核心功能

### 功能概览

| 功能 | 说明 | 状态 |
|------|------|------|
| **代理池管理** | 管理多个 SOCKS5 上游代理 | ✅ 可用 |
| **健康检测** | 手动或自动检测代理健康状态 | ✅ 可用 |
| **链式代理** | 所有代理串联成链 | ✅ 可用 |
| **Clash 集成** | 自动添加链式代理到 Clash 配置 | ✅ 可用 |
| **配置文件监控** | 监控 Clash 配置变化 | ✅ UI 完成 |
| **历史文件** | 最近使用的配置文件列表 | ✅ 可用 |

---

## 📋 详细使用步骤

### 第一步：添加代理到代理池

#### 方法 1: 手动填写（推荐）

```
1. 填写表单：
   Host: 64.32.179.160
   Port: 60088
   User: myusername
   Pass: mypassword

2. 点击 "+ Add" 按钮

3. 代理出现在 Proxy Pool 列表中
```

#### 方法 2: 使用代理字符串

```
1. 在 "user:pass@host:port" 输入框输入：
   myuser:mypass@64.32.179.160:60088
   或
   64.32.179.160:60088:myuser:mypass

2. 点击 "Fill" 按钮自动填充表单

3. 点击 "+ Add" 添加
```

**支持的格式**:
- `user:pass@host:port`
- `ip:port:user:pass`

---

### 第二步：检测代理健康

#### 手动检测（立即查看）

```
1. 点击 "Check All" 按钮

2. 等待检测完成（可能需要 10-30 秒）

3. 查看结果：
   ✓ [Proxy-1] 64.32.179.160:60088 120ms | US, Illinois, Chicago
   × [Proxy-2] 192.168.1.1:1080
     Error: Connection timeout
```

**状态说明**:
- `✓` (绿色) - 健康，可用
- `×` (红色) - 不健康，有错误
- `○` (灰色) - 未检测

#### 自动检测（长期监控）

```
1. 设置检测间隔：
   - 在 "Interval" 输入框输入分钟数（如 5）

2. 开启自动检测：
   - 点击 "Auto: OFF" 按钮
   - 按钮变为 "Auto: ON"

3. 观察自动更新：
   - Output 显示：
     ✓ Auto health check started
       Checking every 5 minutes
       Monitoring 2 enabled proxies

   - 每 5 分钟自动检测一次
   - 状态实时更新，无需手动点击
```

**推荐间隔**:
- 快速调试：1 分钟
- 日常使用：5 分钟（默认）
- 长期监控：10-30 分钟

---

### 第三步：选择 Clash 配置文件

```
1. 点击 "Select" 按钮

2. 选择你的 Clash 配置文件
   例如：~/.config/clash/config.yaml

3. 文件名显示在 Config 行

4. 可选：开启监控
   - 点击 "Watch: OFF" → "Watch: ON"
   - 监控配置文件变化（TODO: 自动重新 Apply）
```

**历史文件快速选择**:
```
1. 点击右侧的 "▼" 按钮

2. 看到最近使用的文件：
   Recent Files:
   [config.yaml]
   [backup-config.yaml]

3. 点击文件名快速加载
```

---

### 第四步：Apply 应用配置

```
1. 确认：
   - ✅ 至少有 1 个启用的代理
   - ✅ 已选择 Clash 配置文件

2. 点击 "Apply" 按钮

3. 查看 Output：
   Using proxy pool mode
   Enabled proxies: 2
     - Proxy-64.32.179.160
     - Proxy-64.32.179.253

   Note: Pool mode uses all enabled proxies as chain
   Local proxy: 127.0.0.1:10808
```

**Apply 做了什么**:
1. 启动本地链式代理 (127.0.0.1:10808)
2. 修改 Clash 配置文件，添加 `Local-Chain-Proxy` 节点
3. 自动备份原配置（.yaml.backup.YYYYMMDD_HHMMSS）

---

### 第五步：在 Clash 中使用

```
1. 刷新 Clash 配置或重启 Clash

2. 在 Clash 的代理组中找到 "Local-Chain-Proxy"

3. 选择使用它

4. 现在你的流量会经过：
   你的应用 → Clash → Local-Chain-Proxy → 代理1 → 代理2 → ... → 目标网站
```

---

## 🔄 自动健康检测

### 详细说明

**功能**: 后台自动定时检测所有启用的代理

**优势**:
- ✅ 不阻塞 GUI（异步执行）
- ✅ 实时更新状态
- ✅ 无需手动操作
- ✅ 适合长期运行

### 使用步骤

```
┌─────────────────────────────────────┐
│ 1. 添加代理                         │
│    - 至少 1 个代理                  │
│    - 确保已启用                     │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│ 2. 设置检测间隔                     │
│    Interval: [5] min                │
│    - 默认 5 分钟                    │
│    - 可输入任何正整数               │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│ 3. 开启自动检测                     │
│    点击 "Auto: OFF" → "Auto: ON"    │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│ 4. 观察自动更新                     │
│    - 等待设置的间隔时间             │
│    - 代理状态自动刷新               │
│    - 延迟和位置信息更新             │
└─────────────────────────────────────┘
```

### 工作原理

```
后台线程                    GUI 主线程
   │                           │
   ├─ 启动                     │
   │                           │
   ├─ 等待 5 分钟              │
   │                           │
   ├─ 检测代理 1 ──[结果]───→  ├─ 接收结果
   ├─ 检测代理 2 ──[结果]───→  ├─ 更新状态
   ├─ 检测代理 3 ──[结果]───→  ├─ 刷新界面
   │                           │
   ├─ 等待 5 分钟              │
   │                           │
   └─ (重复)                   │
```

---

## 📁 配置文件管理

### 配置文件位置

```
~/Library/Application Support/clash-chain-patcher/config.json
```

**内容**:
```json
{
  "upstream_proxies": [
    {
      "id": "xxx",
      "name": "Proxy-1",
      "enabled": true,
      "config": {
        "host": "64.32.179.160",
        "port": 60088,
        "username": "user",
        "password": "pass"
      },
      "health": {
        "status": "healthy",
        "latency_ms": 120,
        "location": "US, Illinois, Chicago"
      }
    }
  ],
  "health_check": {
    "enabled": true,
    "interval_seconds": 300
  }
}
```

### Watch 配置文件监控

**功能**: 监控 Clash 配置文件变化

```
1. 选择 Clash 配置文件

2. 点击 "Watch: OFF" → "Watch: ON"

3. Output 显示：
   ✓ File watching enabled
     Will monitor Clash config for changes

4. 当 Clash 配置被其他程序修改时：
   - TODO: 自动检测并重新 Apply
   - 确保 Local-Chain-Proxy 始终存在
```

**使用场景**:
- Clash 订阅更新会覆盖配置
- Watch 监控到变化后自动重新添加链式代理

---

## ❓ 常见问题

### Q1: 代理池显示 "0 proxies"，但我已经添加了

**A**: 检查以下几点：

1. 是否点击了 "+ Add" 按钮？
2. 查看 Output 是否显示错误
3. 检查配置文件：
   ```bash
   cat ~/Library/Application\ Support/clash-chain-patcher/config.json
   ```

---

### Q2: Apply 后 Clash 配置没有变化

**A**: 检查：

1. Output 是否显示成功？
2. 查看备份文件：
   ```bash
   ls -la ~/.config/clash/*.backup.*
   ```
3. 手动查看配置：
   ```bash
   grep "Local-Chain-Proxy" ~/.config/clash/config.yaml
   ```

---

### Q3: 自动检测开启后没有反应

**A**:

1. 是否有启用的代理？
   - Output 显示：`✗ No enabled proxies`

2. 是否等待了足够时间？
   - 间隔 5 分钟 = 需要等 5 分钟

3. 查看终端调试输出：
   ```
   DEBUG: Auto check background thread started
   DEBUG: Starting auto health check cycle
   ```

---

### Q4: 如何删除代理？

**A**: 当前版本：

1. 找到对应的槽位
2. 点击槽位右侧的 "×" 按钮
3. 代理被删除

---

### Q5: Watch 按钮点击没有反应

**A**:

1. 查看终端是否有调试输出：
   ```
   DEBUG: toggle_watch called
   ```

2. Output 是否显示：
   ```
   ✓ File watching enabled
   ```

3. 如果都没有，尝试重启应用

---

## 🔧 故障排除

### 编译问题

```bash
# 清理并重新编译
cargo clean
cargo build --release

# 检查 Rust 版本
rustc --version
# 需要 Rust 1.70+
```

---

### 代理检测失败

**现象**: 所有代理显示 "×"

**检查**:
1. 代理信息是否正确？
2. 网络是否可达？
   ```bash
   nc -zv 64.32.179.160 60088
   ```
3. 防火墙是否阻止？

---

### Apply 失败

**现象**: Output 显示错误

**常见错误**:

1. "Select file first"
   - 解决：先选择 Clash 配置文件

2. "ProxyState not initialized"
   - 解决：重启应用

3. "Failed to merge"
   - 检查 Clash 配置文件格式
   - 查看备份文件是否正常

---

## 📊 性能说明

### 资源占用

| 项目 | 占用 |
|------|------|
| 内存 | ~50-100 MB |
| CPU（空闲） | ~0-1% |
| CPU（检测时） | ~5-10%（取决于代理数量） |
| 网络（每次检测） | ~10-50 KB/代理 |

### 检测耗时

| 代理数量 | 手动 Check All | 自动检测 |
|---------|----------------|----------|
| 1-3 个 | 3-10 秒 | 3-10 秒/周期 |
| 5-10 个 | 10-30 秒 | 10-30 秒/周期 |

---

## 🎓 高级用法

### 自定义检测间隔

```
场景 1: 快速调试
  Interval: 1 分钟
  适用：测试新添加的代理

场景 2: 日常使用
  Interval: 5 分钟
  适用：桌面应用长期运行

场景 3: 服务器监控
  Interval: 30 分钟
  适用：后台服务，节省资源
```

### 代理链优化

**技巧**:
1. 只启用健康的代理
2. 按延迟排序（手动调整顺序）
3. 定期清理不可用的代理

---

## 📚 相关文档

- [自动健康检测详细说明](AUTO-HEALTH-CHECK-CN.md)
- [工作流程详解](WORKFLOW-EXPLAINED-CN.md)
- [文件历史功能](FILE-HISTORY-FEATURE-CN.md)
- [周期性检测说明](PERIODIC-HEALTH-CHECK-CN.md)

---

## 💡 最佳实践

### 日常工作流

```
1. 启动应用
   ↓
2. 自动加载代理池（从配置文件）
   ↓
3. 开启自动检测（Interval: 5 min, Auto: ON）
   ↓
4. 选择 Clash 配置文件（或从历史选择）
   ↓
5. 开启 Watch 监控
   ↓
6. Apply 应用配置
   ↓
7. 在 Clash 中使用 Local-Chain-Proxy
   ↓
8. 后台自动维护，无需手动操作
```

---

## 🆘 获取帮助

**遇到问题？**

1. 查看终端调试输出
2. 查看 Output 区域错误信息
3. 检查配置文件
4. 提交 Issue：https://github.com/your-username/clash-chain-patcher/issues

**提供信息**:
- 操作系统版本
- 应用版本（左下角显示）
- 错误截图
- 终端输出
- 配置文件（去掉敏感信息）

---

**享受自动化的代理链管理！** 🎉
