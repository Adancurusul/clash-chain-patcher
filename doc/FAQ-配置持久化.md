# FAQ - 配置持久化问题

## 问题：Clash订阅更新后，手动添加的代理节点丢失

### 问题描述

每次Clash订阅更新后，手动添加到配置文件中的本地代理节点（Local-Chain-Proxy）和relay代理组都会被覆盖掉。

### 根本原因

Clash的订阅更新机制会**完全替换**现有配置文件，不会保留手动修改的内容。这是Clash的设计，无法直接避免。

---

## 解决方案

### 🎯 方案1：自动配置合并脚本（推荐）

每次订阅更新后，运行脚本自动添加本地代理节点。

#### 使用步骤

1. **首次设置**
   ```bash
   # 给脚本添加执行权限
   chmod +x scripts/merge-clash-config.sh
   ```

2. **每次订阅更新后运行**
   ```bash
   # 自动检测Clash配置路径
   ./scripts/merge-clash-config.sh

   # 或指定配置文件路径
   ./scripts/merge-clash-config.sh ~/.config/clash/config.yaml
   ```

3. **重启Clash**
   - 配置已自动合并
   - 本地代理节点已添加到所有select代理组

#### 脚本功能

- ✅ 自动备份原配置
- ✅ 检测并添加本地代理节点
- ✅ 自动添加到所有select类型的代理组
- ✅ 不会重复添加（幂等性）
- ✅ 安全的YAML解析（使用Python）

#### 自动化

可以创建一个钩子，在Clash更新后自动运行：

```bash
# 创建一个监控脚本
cat > ~/watch-clash-config.sh << 'EOF'
#!/bin/bash
CLASH_CONFIG="$HOME/.config/clash/config.yaml"
SCRIPT_PATH="/path/to/clash-chain-patcher/scripts/merge-clash-config.sh"

while true; do
    # 监控配置文件变化
    fswatch -1 "$CLASH_CONFIG"
    echo "Clash config updated, merging local proxy..."
    sleep 2  # 等待写入完成
    "$SCRIPT_PATH" "$CLASH_CONFIG"
done
EOF

chmod +x ~/watch-clash-config.sh

# 后台运行监控（可选）
nohup ~/watch-clash-config.sh > /tmp/clash-merger.log 2>&1 &
```

---

### 🎯 方案2：使用Clash Preprocessor（推荐 - 如果使用Clash Premium）

Clash Premium支持配置预处理器，可以在加载配置时自动添加内容。

#### 配置方法

在Clash配置文件顶部添加preprocessor：

```yaml
# 在订阅URL中使用预处理器
proxy-providers:
  my-subscription:
    type: http
    url: "你的订阅URL"
    interval: 3600
    path: ./profiles/subscription.yaml
    health-check:
      enable: true
      url: http://www.gstatic.com/generate_204
      interval: 300

# 使用mixin合并自定义配置
mixin:
  proxies:
    - name: "Local-Chain-Proxy"
      type: socks5
      server: 127.0.0.1
      port: 10808

  proxy-groups:
    - name: "PROXY"
      type: select
      use:
        - my-subscription
      proxies:
        - Local-Chain-Proxy
```

**优点**：Clash自动处理，不需要手动运行脚本

**缺点**：仅Clash Premium支持

---

### 🎯 方案3：使用单独的配置文件

不修改订阅配置，使用include功能（部分Clash版本支持）。

#### 目录结构

```
~/.config/clash/
├── config.yaml           # 主配置（订阅）
├── local-proxy.yaml      # 本地代理配置
└── rules.yaml            # 自定义规则
```

#### local-proxy.yaml

```yaml
proxies:
  - name: "Local-Chain-Proxy"
    type: socks5
    server: 127.0.0.1
    port: 10808

proxy-groups:
  - name: "Local-Chain"
    type: select
    proxies:
      - Local-Chain-Proxy
```

#### 在主配置中引用

```yaml
# config.yaml
# ... 订阅配置 ...

# 引入本地配置
include:
  - local-proxy.yaml
```

**注意**：不是所有Clash版本都支持include

---

### 🎯 方案4：使用Clash的配置覆盖功能

某些Clash客户端（如Clash Verge）支持配置覆盖（Override）。

#### 设置方法

1. 在Clash客户端中找到"配置覆盖"或"Profile Override"
2. 添加以下内容：

```yaml
proxies:
  - name: "Local-Chain-Proxy"
    type: socks5
    server: 127.0.0.1
    port: 10808

prepend-proxy-groups:
  - name: "Local-Chain"
    type: select
    proxies:
      - Local-Chain-Proxy
```

3. 每次订阅更新后，覆盖配置会自动应用

**优点**：图形化配置，方便

**缺点**：取决于Clash客户端是否支持

---

### 🎯 方案5：使用本项目的GUI功能（静态方式）

使用Clash Chain Patcher的静态配置修改功能。

#### 使用步骤

1. **订阅更新后，打开GUI**
   ```bash
   cargo run --release
   ```

2. **加载更新后的配置**
   - Load Config → 选择Clash配置文件

3. **重新添加代理链**
   - 配置SOCKS5代理信息
   - 选择要添加链的节点
   - Apply保存

**优点**：图形化操作，灵活

**缺点**：每次订阅更新都要手动操作

---

## 推荐的工作流程

### 场景1：使用Clash Premium
**推荐**：方案2（Preprocessor + Mixin）
- 自动化，无需干预
- Clash原生支持

### 场景2：使用开源Clash / Clash Verge
**推荐**：方案4（配置覆盖）或 方案1（自动脚本）
- 方案4更方便（如果客户端支持）
- 方案1通用性强，适用所有Clash

### 场景3：不想依赖外部脚本
**推荐**：方案5（使用GUI）
- 可视化操作
- 完全手动控制

---

## 实际使用示例

### 示例：使用自动合并脚本

```bash
# 1. Clash订阅更新
# （在Clash客户端中点击更新订阅）

# 2. 运行合并脚本
cd /path/to/clash-chain-patcher
./scripts/merge-clash-config.sh

# 输出：
# Merging local proxy configuration into Clash config...
# Config file: /Users/xxx/.config/clash/config.yaml
# Backup created: /Users/xxx/.config/clash/config.yaml.backup-20260202-123456
# ✓ Added local proxy node
# ✓ Added local proxy to group: PROXY
# ✓ Added local proxy to group: Fallback
# ✓ Configuration merged successfully!

# 3. 重启Clash
# （在Clash客户端中重启）

# 4. 启动本地代理
RUST_LOG=info cargo run --example proxy_server -- \
  --listen 127.0.0.1:10808 \
  --upstream 64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup

# 5. 在Clash中选择 "Local-Chain-Proxy" 节点
# 完成！
```

### 示例：配置自动监控

```bash
# 安装fswatch（macOS）
brew install fswatch

# 创建监控脚本
cat > ~/auto-merge-clash.sh << 'EOF'
#!/bin/bash
CLASH_CONFIG="$HOME/.config/clash/config.yaml"
MERGE_SCRIPT="$HOME/codes/githubCode/crawlers/clash-chain-patcher/scripts/merge-clash-config.sh"

echo "Monitoring Clash config for changes..."
fswatch -0 "$CLASH_CONFIG" | while read -d "" event; do
    echo "[$(date)] Config changed, merging..."
    sleep 2  # 等待写入完成
    "$MERGE_SCRIPT" "$CLASH_CONFIG"
    echo "Done!"
done
EOF

chmod +x ~/auto-merge-clash.sh

# 启动监控（在后台运行）
nohup ~/auto-merge-clash.sh > ~/clash-auto-merge.log 2>&1 &

# 查看日志
tail -f ~/clash-auto-merge.log
```

---

## 注意事项

### ⚠️ 备份重要性

- 合并脚本会自动创建备份
- 备份文件命名格式：`config.yaml.backup-日期时间`
- 如果合并失败，可以手动恢复：
  ```bash
  cp ~/.config/clash/config.yaml.backup-20260202-123456 ~/.config/clash/config.yaml
  ```

### ⚠️ Python依赖

合并脚本需要Python 3和PyYAML：

```bash
# 安装PyYAML
pip3 install pyyaml

# 或使用系统包管理器
# macOS
brew install python-yq

# Ubuntu/Debian
apt install python3-yaml
```

### ⚠️ 配置文件路径

不同系统和Clash版本的配置文件路径可能不同：

- **macOS**: `~/.config/clash/config.yaml`
- **Linux**: `~/.config/clash/config.yaml`
- **Windows**: `%USERPROFILE%\.config\clash\config.yaml`
- **Clash Verge**: 可能在 `~/Library/Application Support/io.github.clash-verge-rev/config.yaml`

使用脚本时需要指定正确的路径。

---

## 未来改进

在未来版本中，我们将：

1. **GUI集成**：在GUI中添加"配置监控"功能
2. **自动检测**：自动检测Clash配置路径和客户端类型
3. **一键修复**：检测到订阅更新后，一键重新添加配置
4. **配置模板**：提供不同Clash版本的配置模板

---

## 相关文档

- [使用指南-代理链配置.md](使用指南-代理链配置.md) - 详细使用说明
- [技术方案-动态代理.md](技术方案-动态代理.md) - 技术架构说明

---

最后更新：2026-02-02
