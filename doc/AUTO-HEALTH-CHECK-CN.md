# 自动健康检测功能 - 使用指南

**版本**: 0.1.2
**日期**: 2026-02-03
**状态**: ✅ 已实现并可用

---

## ✨ 新功能

### 自动周期性健康检测

应用现在支持自动定时检测代理健康状态，无需手动点击 "Check All"。

**核心功能**:
- ✅ 后台自动检测所有启用的代理
- ✅ 可自定义检测间隔（分钟）
- ✅ 实时更新 GUI 显示
- ✅ 一键开启/关闭

---

## 🎨 UI 界面

### 新增控件

```
Proxy Pool [+ Add] [Check All] [Auto: OFF] Interval: [5] min [Clear All]
                                  ↑             ↑
                              自动检测按钮   间隔设置
```

**组件说明**:
1. **Auto: OFF/ON** - 自动检测开关按钮
   - `OFF` - 自动检测未启动
   - `ON` - 自动检测运行中

2. **Interval 输入框** - 设置检测间隔
   - 单位：分钟
   - 默认：5 分钟
   - 可输入任何正整数

---

## 📖 使用步骤

### 快速开始

```
1. 添加代理到代理池
   - 填写 Host, Port, Username, Password
   - 点击 "+ Add"

2. 启用代理
   - 确保至少有一个代理是启用状态（默认启用）

3. 设置检测间隔（可选）
   - 在 "Interval" 输入框输入数字（如 10 表示 10 分钟）
   - 默认 5 分钟

4. 开启自动检测
   - 点击 "Auto: OFF" 按钮
   - 按钮变为 "Auto: ON"
   - Output 显示：
     ✓ Auto health check started
       Checking every 5 minutes
       Monitoring 2 enabled proxies

5. 观察自动更新
   - 等待设置的时间（如 5 分钟）
   - 代理状态会自动更新
   - 槽位显示健康状态（✓/×）
```

---

## 🔧 工作原理

### 技术架构

```
┌─────────────────┐
│  GUI 主线程     │
│  (Makepad)      │
│                 │
│  - 渲染界面     │
│  - 处理点击     │
│  - 更新显示     │
└────────┬────────┘
         │
         │ mpsc channel
         │
┌────────▼────────┐
│  后台线程       │
│  (std::thread)  │
│                 │
│  - 周期性检测   │
│  - 每N分钟执行  │
│  - 发送结果     │
└─────────────────┘
```

### 检测流程

```
[启动] → [后台线程创建]
   ↓
[等待间隔时间]
   ↓
[检测循环开始]
   ↓
[遍历所有启用的代理]
   ↓
[使用 ProxyValidator 验证]
   - SOCKS5 握手
   - 获取出口 IP
   - 测量延迟
   - 获取地理位置
   ↓
[发送结果到 GUI] (通过 channel)
   ↓
[GUI 主线程接收]
   ↓
[更新代理健康状态]
   ↓
[刷新界面显示]
   ↓
[等待下一个间隔] → [重复]
```

---

## 📊 输出示例

### 启动时

```
Output:
✓ Auto health check started
  Checking every 5 minutes
  Monitoring 2 enabled proxies
```

### 检测中（终端）

```
DEBUG: Auto check background thread started, checking every 5 minutes
DEBUG: Starting auto health check cycle
DEBUG: Auto check cycle completed, sleeping for 300 seconds
```

### 结果更新（终端）

```
DEBUG: Received health check result for proxy xxx: valid=true
DEBUG: Received health check result for proxy yyy: valid=false
```

### GUI 更新

```
Proxy Pool:
  ✓ [Proxy-1] 64.32.179.160:60088 120ms | US, Illinois, Chicago
  × [Proxy-2] 192.168.1.1:1080
    Error: Connection timeout
```

---

## ⚙️ 配置选项

### 检测间隔

**可配置范围**: 任何正整数（分钟）

**建议值**:
- `1` 分钟 - 快速检测，适合调试
- `5` 分钟 - **默认**，平衡性能和及时性
- `10` 分钟 - 低频检测，节省资源
- `30` 分钟 - 长期监控
- `60` 分钟 - 极低频

**CPU/网络占用**:
- 检测时：中等（进行 SOCKS5 连接和 HTTP 请求）
- 等待时：几乎为 0

---

## 🛑 停止自动检测

### 方法 1: 点击按钮

```
点击 "Auto: ON" → 变为 "Auto: OFF"

Output:
Auto health check stopped
```

### 方法 2: 关闭应用

- 应用关闭时，后台线程自动终止
- 下次启动需要重新开启

---

## ❓ 常见问题

### Q1: 自动检测会检测哪些代理？

**A**: 只检测**启用的**代理（enabled = true）

- 禁用的代理不会被检测
- 如果所有代理都禁用，会显示错误提示

---

### Q2: 间隔时间什么时候生效？

**A**: 在**开启自动检测时**读取

- 修改间隔后，需要**停止并重新开启**自动检测
- 或者：关闭后再开启

**示例**:
```
1. Interval = 5, 点击 Auto: OFF → ON  (使用 5 分钟)
2. 修改 Interval = 10
3. 当前仍然使用 5 分钟（未生效）
4. 点击 Auto: ON → OFF → ON  (现在使用 10 分钟)
```

---

### Q3: 自动检测和手动 Check All 有什么区别？

**对比**:

| 功能 | 手动 Check All | 自动检测 |
|------|---------------|----------|
| 触发方式 | 点击按钮 | 后台定时 |
| UI 冻结 | ✅ 会冻结（同步执行） | ❌ 不冻结（异步执行） |
| 检测频率 | 按需 | 周期性 |
| 资源占用 | 高（检测时） | 低（分散检测） |
| 适用场景 | 立即查看状态 | 长期监控 |

---

### Q4: 自动检测会影响应用性能吗？

**A**: 影响很小

**内存**:
- 后台线程：~2MB
- 消息通道：~几KB

**CPU**:
- 等待时：0%
- 检测时：取决于代理数量
  - 1-5 个代理：~1-5%
  - 10 个代理：~5-10%
  - 持续时间：每个代理 ~1-3 秒

**网络**:
- 每次检测每个代理：~10-50KB
- 间隔 5 分钟，10 个代理：~2KB/秒

---

### Q5: 关闭应用后再打开，需要重新设置吗？

**A**: 部分需要

**保留**:
- ✅ 间隔值（保存在输入框，下次打开仍然是上次的值）
- ✅ 代理列表及健康状态

**不保留**:
- ❌ 自动检测状态（需要重新开启）

**TODO**: 将自动检测状态保存到配置文件

---

### Q6: 可以同时运行手动检测和自动检测吗？

**A**: 可以，但不推荐

- 两者独立运行
- 可能同时检测同一个代理
- 结果会相互覆盖（最后更新的为准）

**建议**: 使用自动检测时，避免手动 Check All

---

## 🐛 故障排除

### 问题 1: 点击 "Auto: OFF" 后没有反应

**检查**:
1. 是否有启用的代理？
   - Output 显示：`✗ No enabled proxies`
   - 解决：启用至少一个代理

2. 是否有添加代理？
   - Output 显示：`✗ No proxies to check`
   - 解决：先添加代理

**调试**:
- 查看终端输出
- 应该有 `DEBUG: toggle_auto_health_check called`

---

### 问题 2: 自动检测开启后，状态没有更新

**检查**:
1. 是否等待了足够时间？
   - 如果间隔是 5 分钟，需要等 5 分钟

2. 查看终端调试输出：
   ```
   DEBUG: Auto check background thread started, checking every 5 minutes
   DEBUG: Starting auto health check cycle
   DEBUG: Auto check cycle completed, sleeping for 300 seconds
   ```

3. 是否收到结果？
   ```
   DEBUG: Received health check result for proxy xxx: valid=true
   ```

**可能原因**:
- 后台线程卡住
- 代理检测超时（默认 10 秒）
- 网络问题

---

### 问题 3: GUI 显示结果不正确

**现象**: 终端显示收到结果，但 GUI 未更新

**原因**: 消息通道或 UI 刷新问题

**解决**:
1. 尝试手动点击 "Check All" 触发刷新
2. 重启应用
3. 查看终端是否有错误

---

## 🔬 调试模式

### 启用详细日志

应用已内置调试日志，运行时会自动输出到终端：

```bash
# 运行应用
cargo run --release

# 查看调试输出
DEBUG: toggle_auto_health_check called, current state = false
DEBUG: Auto check started with 5 minute interval
DEBUG: Auto check background thread started, checking every 5 minutes
DEBUG: Starting auto health check cycle
DEBUG: Auto check cycle completed, sleeping for 300 seconds
DEBUG: Received health check result for proxy xxx: valid=true
```

---

## 💡 使用建议

### 推荐配置

**日常使用**:
```
间隔：5 分钟
适用：桌面应用长期运行
```

**快速调试**:
```
间隔：1 分钟
适用：测试代理、调试问题
```

**服务器监控**:
```
间隔：10-30 分钟
适用：后台服务，节省资源
```

### 最佳实践

1. **启动顺序**:
   ```
   添加代理 → 手动 Check All（验证） → 开启自动检测
   ```

2. **间隔设置**:
   - 根据代理稳定性调整
   - 稳定代理：10-30 分钟
   - 不稳定代理：1-5 分钟

3. **资源管理**:
   - 不用时关闭自动检测
   - 只启用需要监控的代理

---

## 📝 实现细节

### 代码位置

**UI 定义**: `src/app.rs:358-391`
```rust
auto_check_btn = <Button> {
    text: "Auto: OFF"
    ...
}

interval_input = <TextInput> {
    width: 50,
    text: "5"
    ...
}
```

**状态字段**: `src/app.rs:694-700`
```rust
auto_checking: bool,
auto_check_interval: u64,
health_check_rx: Option<std::sync::mpsc::Receiver<...>>,
```

**主要方法**:
- `toggle_auto_health_check()` - 开启/关闭自动检测
- `update_proxy_health_from_background()` - 处理后台结果
- `handle_event()` - 检查消息通道

---

## ✅ 总结

### 已实现功能

1. ✅ 后台自动检测
2. ✅ 可配置间隔
3. ✅ 实时 GUI 更新
4. ✅ 一键开关
5. ✅ 调试日志

### 待实现功能

1. ⏳ 保存自动检测状态到配置文件
2. ⏳ 显示下次检测倒计时
3. ⏳ 检测历史记录
4. ⏳ 失败重试机制

---

## 🎉 开始使用！

```bash
# 1. 编译
cargo build --release

# 2. 运行
./target/release/clash-chain-patcher

# 3. 添加代理

# 4. 点击 "Auto: OFF"

# 5. 享受自动检测！
```

**享受自动化的代理健康监控！** 🚀
