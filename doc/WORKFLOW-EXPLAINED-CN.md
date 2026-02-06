# Clash Chain Patcher - 工作流程详解

**版本**: 0.1.2
**日期**: 2026-02-03

---

## 📖 核心概念

### 1. Proxy Pool (代理池)
- 管理你的上游 SOCKS5 代理
- 支持健康检查、启用/禁用
- 配置保存在：`~/Library/Application Support/clash-chain-patcher/config.json`

### 2. Local Chain Proxy (本地链式代理)
- 在 `127.0.0.1:10808` 运行
- 自动链式连接所有**启用的**代理
- 相当于一个"超级代理"，流量经过所有代理链

### 3. Clash Config (Clash 配置)
- 你的 Clash 配置文件（如 `config.yaml`）
- Apply 后会添加 `Local-Chain-Proxy` 节点
- 自动插入到所有 `select` 类型的代理组

---

## 🔄 完整工作流程

### 阶段 1: 管理代理池

```
1. 填写代理信息（Host, Port, Username, Password）
   ↓
2. 点击 "+ Add" 添加到代理池
   ↓
3. 重复 1-2 添加多个代理
   ↓
4. 点击 "Check" 或 "Check All" 验证健康
```

**此时状态**:
- ✅ 代理池已配置（保存在 `config.json`）
- ❌ 本地链式代理**未运行**
- ❌ Clash 配置**未修改**

---

### 阶段 2: 选择 Clash 配置

```
1. 点击 "Select" 按钮
   ↓
2. 选择你的 Clash 配置文件（如 `~/.config/clash/config.yaml`）
   ↓
3. 文件名显示在 Config 行
```

**此时状态**:
- ✅ 代理池已配置
- ✅ Clash 文件已选择
- ❌ 本地链式代理**未运行**
- ❌ Clash 配置**未修改**

---

### 阶段 3: Apply（应用配置）

```
点击 "Apply" 按钮
```

**Apply 做了什么**:
1. ✅ **启动本地链式代理** (127.0.0.1:10808)
   - 链接所有**启用的**上游代理
   - 流量路径：你的应用 → 本地代理 → 代理1 → 代理2 → ... → 目标网站

2. ✅ **修改 Clash 配置文件**
   - 添加 `Local-Chain-Proxy` 节点到 `proxies` 列表
   - 插入到所有 `select` 类型代理组的**开头**

**修改示例**:

**修改前的 Clash 配置**:
```yaml
proxies:
  - name: "US-Proxy"
    type: ss
    server: example.com
    port: 8388

proxy-groups:
  - name: "Auto"
    type: select
    proxies:
      - US-Proxy
      - DIRECT
```

**修改后的 Clash 配置**:
```yaml
proxies:
  - name: "US-Proxy"
    type: ss
    server: example.com
    port: 8388

  # ✨ 新添加的节点
  - name: "Local-Chain-Proxy"
    type: socks5
    server: 127.0.0.1
    port: 10808

proxy-groups:
  - name: "Auto"
    type: select
    proxies:
      - Local-Chain-Proxy  # ✨ 插入到开头
      - US-Proxy
      - DIRECT
```

**此时状态**:
- ✅ 代理池已配置
- ✅ Clash 文件已选择
- ✅ 本地链式代理**正在运行**
- ✅ Clash 配置**已修改**（自动备份到 `.yaml.backup.YYYYMMDD_HHMMSS`）

---

### 阶段 4: 使用 Clash

```
在 Clash 中：
1. 刷新配置或重启 Clash
   ↓
2. 在代理组中看到 "Local-Chain-Proxy"
   ↓
3. 选择 "Local-Chain-Proxy"
   ↓
4. 流量现在经过你的代理链！
```

---

## ❓ 常见问题解答

### Q1: Watch 是否真实实现了监控？

**A**: **目前没有**。Watch 按钮只是 UI 状态切换。

**当前代码**:
```rust
// src/app.rs:770
// TODO: Start file watcher using WatcherBridge
```

**要实现真正的监控，需要**:
1. 集成 `WatcherBridge`（代码已存在）
2. 监听 Clash 配置文件变化
3. 检测到变化时自动重新 Apply

**如何判断是否实现**:
- ✅ 手动修改 Clash 配置文件
- ✅ 如果应用自动检测并重新添加 `Local-Chain-Proxy`，则实现了
- ❌ 如果没有反应，则未实现（**当前状态**）

---

### Q2: 如何触发 Watch？需要 Apply 吗？

**A**:

**当前逻辑**:
1. Select 文件后，点击 "Watch: OFF" → "Watch: ON"
2. **但实际没有启动监控**（见 TODO 注释）

**应该的逻辑**（待实现）:
1. Select 文件
2. Apply（首次添加 `Local-Chain-Proxy`）
3. 开启 Watch
4. 之后如果 Clash 配置被其他程序修改（如 Clash 订阅更新），自动重新 Apply

**Watch 的意义**:
- Clash 订阅更新会覆盖整个配置
- `Local-Chain-Proxy` 会被删除
- Watch 监控到变化后自动重新添加

---

### Q3: 如何判断是否有去重（避免重复添加链）？

**A**: ✅ **已实现去重**

**代码位置**: `src/merger/clash_merger.rs:179-187`

```rust
// Check if proxy already exists
for proxy in proxies_seq.iter() {
    if let Some(name) = proxy.get("name").and_then(|v| v.as_str()) {
        if name == self.config.proxy_name {  // "Local-Chain-Proxy"
            debug!("Proxy '{}' already exists", self.config.proxy_name);
            return Ok(false);  // ✅ 不会重复添加
        }
    }
}
```

**测试方法**:

1. **首次 Apply**:
   ```bash
   # 查看配置文件
   grep "Local-Chain-Proxy" ~/.config/clash/config.yaml | wc -l
   # 应该显示: 2-3 行（proxies 节点 + proxy-groups 引用）
   ```

2. **再次 Apply**:
   ```bash
   # 再查看
   grep "Local-Chain-Proxy" ~/.config/clash/config.yaml | wc -l
   # 应该还是: 2-3 行（没有重复）
   ```

**Output 区域提示**:
- 首次: `✓ Added proxy node: Local-Chain-Proxy`
- 再次: `ℹ Proxy 'Local-Chain-Proxy' already exists`（不会显示 Added）

---

### Q4: 如何判断是否被更新了而没有添加链？

**A**: 通过 **Apply 后的 Output 信息**判断。

**情况 1 - 配置已有链，Apply 没有修改**:
```
Output:
ℹ Proxy 'Local-Chain-Proxy' already exists
ℹ Proxy already in 0 groups
✓ Merge completed
  - Added proxies: 0
  - Updated groups: 0
```

**情况 2 - 配置被外部修改，链被删除，Apply 重新添加**:
```
Output:
✓ Added proxy node: Local-Chain-Proxy
✓ Added proxy to group: Auto
✓ Added proxy to group: Manual
✓ Merge completed
  - Added proxies: 1
  - Updated groups: 2
  - Backup: /path/to/config.yaml.backup.20260203_180430
```

**手动验证方法**:

1. **检查 proxies 列表**:
   ```bash
   yq '.proxies[] | select(.name == "Local-Chain-Proxy")' ~/.config/clash/config.yaml
   ```
   应该显示:
   ```yaml
   name: Local-Chain-Proxy
   type: socks5
   server: 127.0.0.1
   port: 10808
   ```

2. **检查 proxy-groups**:
   ```bash
   yq '.proxy-groups[] | select(.type == "select") | .proxies' ~/.config/clash/config.yaml
   ```
   每个 select 组的第一个应该是 `Local-Chain-Proxy`

---

### Q5: Save 按钮有什么用？

**A**: **Save 按钮保存代理池配置**（不是 Clash 配置）。

**Save 保存什么**:
- 代理池中的所有代理
- 健康状态
- 启用/禁用状态

**保存位置**:
```
~/Library/Application Support/clash-chain-patcher/config.json
```

**何时需要 Save**:
- ✅ 添加/删除代理后（自动保存）
- ✅ 修改代理信息后（自动保存）
- ✅ 健康检查后（自动保存）

**实际上**: 大部分操作都会**自动保存**，Save 按钮可能是手动触发保存的备用选项。

---

## 🎯 最佳实践工作流程

### 初次设置

```
1. 添加代理到代理池
   - 填写表单 → "+ Add"
   - 重复添加多个代理

2. 检查健康
   - 点击 "Check All"
   - 查看健康状态（✓ 绿色 = 健康）

3. 选择 Clash 配置
   - 点击 "Select"
   - 选择 ~/.config/clash/config.yaml

4. Apply
   - 点击 "Apply"
   - 查看 Output：应该显示 "Added proxy node"

5. 重启 Clash
   - 刷新配置或重启 Clash
   - 在代理组中选择 "Local-Chain-Proxy"

6. 测试
   - 访问 https://ip-api.com
   - 应该看到最后一个代理的 IP
```

### 日常使用

```
# 修改代理池
1. 添加/删除代理
2. Check 健康
3. 如果修改了代理，需要重新 Apply

# Clash 订阅更新后
1. 订阅更新会删除 Local-Chain-Proxy
2. 重新点击 "Apply"（自动去重）
3. 刷新 Clash

# 开启 Watch（待实现）
1. Apply 后开启 Watch
2. 订阅更新后自动重新 Apply
3. 无需手动操作
```

---

## 🔧 技术细节

### 去重逻辑的实现位置

**代码文件**: `src/merger/clash_merger.rs`

**关键函数**:
1. `add_proxy_node()` - 添加代理节点
   - 第 179-187 行: 检查是否已存在
   - 第 208-211 行: 添加节点

2. `add_to_proxy_groups()` - 添加到代理组
   - 第 272-279 行: 检查组中是否已存在
   - 第 282-287 行: 添加到组

**检查逻辑**:
```rust
// 1. 检查 proxies 列表
for proxy in proxies_seq.iter() {
    if name == "Local-Chain-Proxy" {
        return Ok(false);  // 已存在，不添加
    }
}

// 2. 检查 proxy-groups
let already_exists = group_proxies_seq.iter().any(|p| {
    p.as_str() == Some("Local-Chain-Proxy")
});
if already_exists {
    continue;  // 已在组中，跳过
}
```

---

## 📊 状态流转图

```
┌─────────────────┐
│  启动应用        │
│  加载 config.json│
└────────┬────────┘
         │
         v
┌─────────────────┐
│  管理代理池      │ ← 添加/删除/检查
│  (未运行)       │
└────────┬────────┘
         │ Select 文件
         v
┌─────────────────┐
│  已选择 Clash   │
│  配置文件       │
└────────┬────────┘
         │ Apply
         v
┌─────────────────┐
│  ✓ 本地代理运行 │
│  ✓ Clash 已修改 │ ← 重新 Apply（自动去重）
└────────┬────────┘
         │ Watch (待实现)
         v
┌─────────────────┐
│  自动监控       │
│  配置变化       │
└─────────────────┘
```

---

## ✅ 总结

1. **代理池**: 管理上游 SOCKS5 代理
2. **Apply**: 启动本地链式代理 + 修改 Clash 配置
3. **去重**: ✅ 已实现，不会重复添加
4. **Watch**: ❌ 未实现，只是 UI 状态
5. **Save**: 自动保存代理池配置

**下一步**: 实现 Watch 监控 + YAML 文件历史下拉菜单
