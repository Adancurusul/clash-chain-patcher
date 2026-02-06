# Windows Clash出站代理链配置指南

## 需求

让Clash的所有节点在出站时自动走SOCKS5代理链：
```
用户 → Clash → 节点(SS/VMess) → SOCKS5代理 → 互联网
```

---

## 方案1：Proxifier（推荐）

### 特点
- ✅ 图形化界面，简单易用
- ✅ 稳定可靠
- ❌ 付费软件（$39.95，有30天试用）

### 下载安装

官网：https://www.proxifier.com/
下载：Proxifier Standard Edition for Windows

### 配置步骤

#### 1. 添加SOCKS5代理服务器

1. 打开Proxifier
2. 点击 `Profile` → `Proxy Servers...`
3. 点击 `Add`
4. 填写信息：
   - **Address**: `127.0.0.1`（你的本地代理地址）
   - **Port**: `10808`
   - **Protocol**: `SOCKS Version 5`
   - **Authentication**: 勾选并填写用户名密码（如果需要）
5. 点击 `OK`

#### 2. 配置代理规则

1. 点击 `Profile` → `Proxification Rules...`
2. 点击 `Add`
3. 配置规则：
   - **Name**: `Clash Chain`
   - **Applications**: 点击 `Browse...` 选择 `clash.exe`
   - **Target hosts**: `Any`
   - **Target ports**: `Any`
   - **Action**: 选择你刚添加的SOCKS5代理
4. 点击 `OK`

#### 3. 排除本地地址（重要！）

在Proxification Rules中添加排除规则：

1. 点击 `Add`
2. 配置：
   - **Name**: `Localhost`
   - **Target hosts**: `127.0.0.1; localhost; 10.*; 192.168.*`
   - **Action**: `Direct`
3. 将这条规则拖到最上面（优先级最高）

#### 4. 启动

1. 先启动你的本地SOCKS5代理服务器：
   ```powershell
   cd C:\path\to\clash-chain-patcher
   cargo run --example proxy_server -- --listen 127.0.0.1:10808 --upstream 64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup
   ```

2. 启动Clash

3. 在Proxifier中查看日志，应该能看到Clash的连接被代理

### 验证

打开Proxifier的日志窗口，你会看到类似：
```
clash.exe - 连接到 hk.example.com:8388 通过 127.0.0.1:10808 (SOCKS5)
```

---

## 方案2：Clash Verge + 自定义代理（免费）

### 特点
- ✅ 完全免费
- ✅ 使用Clash Verge的系统代理功能
- ⚠️ 配置稍复杂

### 实现原理

使用Clash Verge的"脚本"功能，自动为节点添加代理链

### 配置步骤

#### 1. 安装Clash Verge

下载：https://github.com/clash-verge-rev/clash-verge-rev/releases

#### 2. 创建配置脚本

在Clash Verge中添加配置预处理脚本：

1. 打开Clash Verge
2. 点击 `配置` → `脚本配置`
3. 添加以下脚本：

```javascript
// Clash Verge预处理脚本

function main(config) {
  // 定义本地代理链节点
  const chainProxy = {
    name: "Chain-Proxy",
    type: "socks5",
    server: "127.0.0.1",
    port: 10808
  };

  // 将本地代理添加到节点列表
  if (!config.proxies) {
    config.proxies = [];
  }
  config.proxies.unshift(chainProxy);

  // 为每个原有节点创建relay代理组
  const originalProxies = config.proxies.filter(p => p.name !== "Chain-Proxy");
  const chainedProxies = [];

  originalProxies.forEach(proxy => {
    const chainedProxy = {
      name: `${proxy.name}-Chain`,
      type: "relay",
      proxies: [proxy.name, "Chain-Proxy"]
    };
    chainedProxies.push(chainedProxy);
  });

  // 添加所有链式代理
  config.proxies.push(...chainedProxies);

  // 创建或更新代理组
  if (!config["proxy-groups"]) {
    config["proxy-groups"] = [];
  }

  // 添加链式代理到PROXY组
  const proxyGroup = config["proxy-groups"].find(g => g.name === "PROXY");
  if (proxyGroup) {
    chainedProxies.forEach(cp => {
      if (!proxyGroup.proxies.includes(cp.name)) {
        proxyGroup.proxies.push(cp.name);
      }
    });
  }

  return config;
}
```

#### 3. 启动

1. 先启动本地代理：
   ```powershell
   cargo run --example proxy_server -- --listen 127.0.0.1:10808 --upstream YOUR_UPSTREAM
   ```

2. 在Clash Verge中，你会看到每个节点都有一个 `节点名-Chain` 的版本

3. 选择带 `-Chain` 后缀的节点使用

---

## 方案3：SocksCap64（免费，但停止更新）

### 特点
- ✅ 完全免费
- ⚠️ 已停止更新（最后版本2016年）
- ⚠️ 兼容性可能有问题

### 下载

搜索 "SocksCap64" 下载最后一个版本

### 配置步骤

1. 打开SocksCap64
2. 添加SOCKS5代理（设置 → 代理）
3. 添加Clash程序（点击 `+` → 浏览到 `clash.exe`）
4. 右键Clash → `Run via SocksCap64`

### 注意

- 可能与某些Clash版本不兼容
- 已停止维护，不推荐用于生产环境

---

## 方案4：PowerShell + WSL（高级方案）

### 原理

在WSL中运行Linux的iptables+redsocks方案

### 步骤

1. **安装WSL2**：
   ```powershell
   wsl --install -d Ubuntu
   ```

2. **在WSL中配置redsocks**（参考Linux脚本）

3. **在WSL中运行Clash**：
   ```bash
   cd /mnt/c/path/to/clash
   sudo -u youruser ./clash -f config.yaml
   ```

### 缺点
- 配置复杂
- 需要在WSL环境运行Clash

---

## 推荐方案对比

| 方案 | 难度 | 成本 | 推荐度 | 说明 |
|-----|------|-----|-------|------|
| Proxifier | ⭐ | $39.95 | ⭐⭐⭐⭐⭐ | 最简单稳定，付费 |
| Clash Verge脚本 | ⭐⭐⭐ | 免费 | ⭐⭐⭐ | 免费但需要写脚本 |
| SocksCap64 | ⭐⭐ | 免费 | ⭐⭐ | 免费但停止更新 |
| WSL方案 | ⭐⭐⭐⭐⭐ | 免费 | ⭐ | 最复杂，仅geek适用 |

---

## 最终推荐

### 如果你不介意花钱
→ **使用Proxifier**，简单稳定可靠

### 如果你想免费方案
→ **使用Clash Verge + 脚本**，功能完整

### 临时测试
→ **试用Proxifier 30天**，到期后决定是否购买

---

## 完整使用流程（Proxifier为例）

### 1. 准备

```powershell
# 克隆项目
cd C:\path\to\
git clone https://github.com/yourname/clash-chain-patcher

# 编译
cd clash-chain-patcher
cargo build --release
```

### 2. 启动本地代理

```powershell
# PowerShell
$env:RUST_LOG="info"
.\target\release\examples\proxy_server.exe --listen 127.0.0.1:10808 --upstream "64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup"
```

### 3. 配置Proxifier

按照上面的步骤配置

### 4. 启动Clash

正常启动Clash，Proxifier会自动劫持其流量

### 5. 验证

打开 https://api.ip.sb/ip
- 应该看到你的节点IP（不是SOCKS5代理IP）
- 但流量确实经过了SOCKS5代理

在Proxifier日志中你会看到：
```
clash.exe - hk.example.com:8388 → 127.0.0.1:10808 (SOCKS5)
```

在本地代理日志中你会看到：
```
INFO Connected 127.0.0.1:xxxxx -> hk.example.com:8388 via upstream
```

---

## 疑难解答

### Q1: Proxifier说"无法连接到代理服务器"

**A**: 检查本地代理是否启动：
```powershell
netstat -ano | findstr 10808
```

应该能看到监听在10808端口的进程

### Q2: Clash连接失败

**A**: 在Proxifier中排除localhost：
```
Profile → Proxification Rules → 添加规则
Target hosts: 127.0.0.1;localhost
Action: Direct
优先级: 最高
```

### Q3: 性能影响大吗？

**A**: 本地转发延迟 < 1ms，几乎无感知

### Q4: 可以用免费的proxifier替代品吗？

**A**: 可以尝试：
- **Proxycap**（类似Proxifier，也收费）
- **SSTap**（停止更新，可能不稳定）
- **NekoRay**（支持进程代理，免费）

---

## 脚本自动化（PowerShell）

创建一个启动脚本 `start-chain.ps1`：

```powershell
# start-chain.ps1
# 一键启动Clash代理链

$UPSTREAM = "64.32.179.160:60088:ZUvGbvjcI52P:0UxQRzGfZoup"
$LISTEN = "127.0.0.1:10808"

# 启动本地代理（后台）
Start-Process -FilePath ".\target\release\examples\proxy_server.exe" `
  -ArgumentList "--listen $LISTEN --upstream $UPSTREAM" `
  -NoNewWindow `
  -PassThru

Write-Host "✅ 本地代理已启动 ($LISTEN)"
Write-Host "✅ 请在Proxifier中启动Clash"
Write-Host ""
Write-Host "按任意键停止本地代理..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# 停止本地代理
Get-Process proxy_server -ErrorAction SilentlyContinue | Stop-Process
Write-Host "✅ 本地代理已停止"
```

使用：
```powershell
.\start-chain.ps1
```

---

最后更新：2026-02-02
