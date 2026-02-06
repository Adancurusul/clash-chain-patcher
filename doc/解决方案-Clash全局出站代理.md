# ❌ [已废弃] Clash 全局出站代理配置方案

> **⚠️ 重要提示**: 本方案已废弃，不再推荐使用。
>
> **问题原因**: 此方案使用Clash的 `dialer-proxy` 功能会导致**流量顺序错误**：
> - ❌ 实际流程：Clash → 本地代理 → 上游SOCKS5 → 节点 → 互联网
> - ✅ 期望流程：Clash → 节点 → 本地代理 → 上游SOCKS5 → 互联网
>
> **新方案**: 请参考以下文档：
> - [方案优化-自动配置监控.md](方案优化-自动配置监控.md) - 推荐方案
> - [Windows代理链配置指南.md](Windows代理链配置指南.md) - Windows用户
>
> ---
>
> 以下为原文档内容（仅供参考）

---

## 需求

**让Clash的所有节点流量都经过本地SOCKS5代理链**

不修改具体节点配置，只需一次性配置，让Clash的所有出站流量都走我们的本地代理。

---

## 原理

```
用户应用
    ↓
Clash (TUN/系统代理)
    ↓
[所有Clash节点] → 使用dialer-proxy出站
    ↓
本地代理 (127.0.0.1:10808)
    ↓
上游SOCKS5 (64.32.179.160:60088)
    ↓
互联网
```

---

## 解决方案：使用 dialer-proxy

Clash支持为所有节点配置统一的出站代理（dialer-proxy）。

### 配置方法

在Clash配置文件**顶部**添加：

```yaml
# Clash配置文件 (config.yaml)

# 全局出站代理配置 - 所有节点都通过这个代理出站
dialer-proxy: "Local-Outbound"

# 定义出站代理节点
proxies:
  # 本地出站代理（所有Clash节点都会用这个）
  - name: "Local-Outbound"
    type: socks5
    server: 127.0.0.1
    port: 10808
    # 不需要认证（本地连接）

  # ... 你原有的其他代理节点 ...
  # 这些节点的流量都会自动通过 Local-Outbound
```

### 完整示例

```yaml
# ==============================
# 全局出站代理配置
# ==============================
dialer-proxy: "Local-Outbound"

# ==============================
# 代理节点
# ==============================
proxies:
  # 本地代理链节点（用于出站）
  - name: "Local-Outbound"
    type: socks5
    server: 127.0.0.1
    port: 10808

  # 你的原有节点
  - name: "节点1-香港"
    type: ss
    server: hk.example.com
    port: 8388
    cipher: aes-256-gcm
    password: "your-password"

  - name: "节点2-美国"
    type: vmess
    server: us.example.com
    port: 443
    uuid: your-uuid
    alterId: 0
    cipher: auto
    tls: true

# ==============================
# 代理组
# ==============================
proxy-groups:
  - name: "PROXY"
    type: select
    proxies:
      - "节点1-香港"
      - "节点2-美国"
      # 不需要添加 Local-Outbound 到代理组
      # 它只用于出站，不用于选择

  - name: "Auto"
    type: url-test
    proxies:
      - "节点1-香港"
      - "节点2-美国"
    url: 'http://www.gstatic.com/generate_204'
    interval: 300

# ==============================
# 规则
# ==============================
rules:
  - DOMAIN-SUFFIX,google.com,PROXY
  - DOMAIN-SUFFIX,github.com,PROXY
  - GEOIP,CN,DIRECT
  - MATCH,PROXY
```

---

## 工作流程

### 1. 流量路径

当你访问网站时：

```
浏览器
  ↓ (系统代理/TUN)
Clash接收请求
  ↓ (规则匹配)
选择代理节点 (例如"节点1-香港")
  ↓ (dialer-proxy生效)
通过 Local-Outbound (127.0.0.1:10808)
  ↓ (SOCKS5协议)
本地代理服务器
  ↓ (转发)
上游SOCKS5 (64.32.179.160:60088)
  ↓ (认证并连接)
节点1-香港 (hk.example.com:8388)
  ↓
目标网站
```

### 2. 实际例子

假设你在Clash中选择了"节点1-香港"：

```
1. 用户访问 google.com
2. Clash匹配规则 → 使用 PROXY 组 → 选中"节点1-香港"
3. Clash尝试连接 hk.example.com:8388
4. ✨ dialer-proxy生效：
   - 不直接连接 hk.example.com
   - 而是通过 127.0.0.1:10808 连接
5. 本地代理接收到：
   - 目标：hk.example.com:8388
   - 转发给上游：64.32.179.160:60088
6. 上游SOCKS5连接到 hk.example.com:8388
7. 最终建立连接链：
   用户 → Clash → 本地代理 → 上游SOCKS5 → 香港节点 → Google
```

---

## 使用步骤

### 1. 一次性配置Clash

编辑Clash配置文件，在**最顶部**添加：

```yaml
dialer-proxy: "Local-Outbound"

proxies:
  - name: "Local-Outbound"
    type: socks5
    server: 127.0.0.1
    port: 10808

  # ... 你原有的节点 ...
```

**重要**：
- ✅ 只需要配置一次
- ✅ 订阅更新不会影响这个配置（在顶部）
- ✅ 所有节点自动生效

### 2. 启动本地代理服务器

```bash
RUST_LOG=info cargo run --example proxy_server -- \
  --listen 127.0.0.1:10808 \
  --upstream 64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup
```

### 3. 重启Clash

让配置生效

### 4. 正常使用Clash

- 在Clash中随意选择任何节点
- 所有节点的流量都会自动走代理链
- 不需要选择特定节点

---

## 验证

### 测试1：检查IP

```bash
# 使用Clash (选择任意节点)
curl http://ping0.cc

# 应该看到：64.32.179.160
# 这是上游SOCKS5的IP
```

### 测试2：查看日志

启动本地代理后，你会看到大量连接日志：

```
INFO Connected 127.0.0.1:xxxxx -> hk.example.com:8388 via upstream
INFO Connected 127.0.0.1:xxxxx -> us.example.com:443 via upstream
INFO Connected 127.0.0.1:xxxxx -> jp.example.com:8080 via upstream
```

这说明Clash的所有节点都在通过本地代理连接！

---

## 自动化配置脚本

创建一个脚本自动添加dialer-proxy配置：

```bash
#!/bin/bash
# add-dialer-proxy.sh

CLASH_CONFIG="$HOME/.config/clash/config.yaml"
BACKUP="${CLASH_CONFIG}.backup-$(date +%Y%m%d-%H%M%S)"

# 备份
cp "$CLASH_CONFIG" "$BACKUP"
echo "Backup created: $BACKUP"

# 使用Python添加配置
python3 << 'EOF'
import yaml
import sys

config_file = "$HOME/.config/clash/config.yaml"

with open(config_file.replace('$HOME', os.path.expanduser('~')), 'r', encoding='utf-8') as f:
    config = yaml.safe_load(f)

# 添加dialer-proxy
config['dialer-proxy'] = 'Local-Outbound'

# 确保proxies存在
if 'proxies' not in config:
    config['proxies'] = []

# 添加本地出站代理节点
local_proxy = {
    'name': 'Local-Outbound',
    'type': 'socks5',
    'server': '127.0.0.1',
    'port': 10808
}

# 检查是否已存在
proxy_names = [p['name'] for p in config['proxies']]
if 'Local-Outbound' not in proxy_names:
    config['proxies'].insert(0, local_proxy)
    print('✓ Added Local-Outbound proxy')
else:
    print('✓ Local-Outbound already exists')

# 保存
with open(config_file.replace('$HOME', os.path.expanduser('~')), 'w', encoding='utf-8') as f:
    yaml.dump(config, f, allow_unicode=True, default_flow_style=False, sort_keys=False)

print('✓ dialer-proxy configured successfully')
EOF

echo "Done! Please restart Clash."
```

---

## 常见问题

### Q1: 订阅更新会不会覆盖配置？

**A**: 取决于你的订阅方式：

**方法1（推荐）**：使用Clash的配置混合（mixin）
```yaml
# 在Clash客户端中配置mixin
mixin:
  dialer-proxy: "Local-Outbound"
  proxies:
    - name: "Local-Outbound"
      type: socks5
      server: 127.0.0.1
      port: 10808
```

**方法2**：订阅更新后手动运行脚本
```bash
./add-dialer-proxy.sh
```

### Q2: 会不会影响速度？

**A**: 影响很小：
- 本地转发延迟 < 1ms
- 主要延迟在上游SOCKS5和最终节点
- 实测几乎无感知差异

### Q3: 可以只对部分节点生效吗？

**A**: 可以，但需要手动配置每个节点：

```yaml
proxies:
  - name: "节点1"
    type: ss
    server: example.com
    port: 8388
    cipher: aes-256-gcm
    password: "pass"
    dialer-proxy: "Local-Outbound"  # 只对这个节点生效

  - name: "节点2"
    type: ss
    server: example2.com
    port: 8388
    cipher: aes-256-gcm
    password: "pass"
    # 这个节点不走代理链
```

不过这样就需要每次订阅更新后都修改，不推荐。

### Q4: 本地代理挂了怎么办？

**A**: Clash会报错连接失败。

**解决方案**：
- 临时：注释掉 `dialer-proxy: "Local-Outbound"` 这行
- 长期：确保本地代理稳定运行（未来会做成系统服务）

### Q5: 如何临时禁用代理链？

**方法1**：停止本地代理服务器
- Clash会报错，但仍能使用（直连节点）

**方法2**：修改配置
```yaml
# 注释掉这行
# dialer-proxy: "Local-Outbound"
```

**方法3**：使用环境变量（未来功能）

---

## 与之前方案的对比

### 方案1：添加relay代理组（原方案）
```yaml
proxies:
  - name: "Local-Chain"
    type: socks5
    server: 127.0.0.1
    port: 10808

  - name: "节点1-香港"
    type: ss
    server: hk.example.com
    port: 8388

proxy-groups:
  - name: "节点1-链"
    type: relay
    proxies:
      - "节点1-香港"
      - "Local-Chain"
```

**缺点**：
- ❌ 需要为每个节点创建relay
- ❌ 订阅更新后丢失
- ❌ 配置复杂

### 方案2：使用dialer-proxy（新方案）✅
```yaml
dialer-proxy: "Local-Outbound"

proxies:
  - name: "Local-Outbound"
    type: socks5
    server: 127.0.0.1
    port: 10808

  - name: "节点1-香港"
    type: ss
    server: hk.example.com
    port: 8388
```

**优点**：
- ✅ 只需配置一次
- ✅ 所有节点自动生效
- ✅ 配置简单
- ✅ 可以放在配置顶部，不受订阅影响

---

## 完整使用流程

### 初始设置（只做一次）

```bash
# 1. 修改Clash配置
vi ~/.config/clash/config.yaml

# 在文件开头添加：
# dialer-proxy: "Local-Outbound"
#
# proxies:
#   - name: "Local-Outbound"
#     type: socks5
#     server: 127.0.0.1
#     port: 10808

# 2. 重启Clash
```

### 日常使用

```bash
# 1. 启动本地代理（每次开机）
RUST_LOG=info cargo run --example proxy_server -- \
  --listen 127.0.0.1:10808 \
  --upstream 64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup

# 2. 正常使用Clash
# - 随意选择任何节点
# - 所有流量自动走代理链

# 3. 验证
curl http://ping0.cc
# 输出：64.32.179.160
```

---

## 下一步：GUI自动化

未来在GUI中实现：

1. **自动配置**：点击按钮自动添加dialer-proxy
2. **自动启动**：GUI启动时自动启动本地代理
3. **状态监控**：实时显示代理链状态
4. **一键切换**：快速启用/禁用代理链

---

最后更新：2026-02-02
