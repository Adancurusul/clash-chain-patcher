# Clash API 自动化调研

## 目标

实现"打开软件就让所有流量走代理链"，需要自动化以下步骤：
1. 修改Clash配置文件（已实现）
2. 重载Clash配置
3. 自动选择我们的代理节点

## Clash API 功能

### API 端点

Clash提供RESTful API（默认端口9090）：

```bash
# 基础URL
http://127.0.0.1:9090

# 常用端点
GET  /proxies              # 获取所有代理
GET  /proxies/:name        # 获取特定代理信息
PUT  /proxies/:group       # 切换代理组的选择
POST /configs              # 重载配置
GET  /traffic              # 获取流量统计
```

### 关键功能：自动选择节点

```bash
# 1. 获取代理组
curl http://127.0.0.1:9090/proxies

# 返回示例
{
  "proxies": {
    "PROXY": {
      "name": "PROXY",
      "type": "Selector",
      "now": "节点1",  # 当前选择
      "all": ["节点1", "节点2", "Local-Chain-Proxy"]
    }
  }
}

# 2. 切换到我们的节点
curl -X PUT http://127.0.0.1:9090/proxies/PROXY \
  -H "Content-Type: application/json" \
  -d '{"name": "Local-Chain-Proxy"}'

# 3. 重载配置
curl -X PUT http://127.0.0.1:9090/configs \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/config.yaml"}'
```

## 完整自动化流程

```rust
// 伪代码
async fn activate_proxy_chain() -> Result<()> {
    // 1. 启动本地代理服务器
    let server = start_local_proxy("127.0.0.1:10808", upstream).await?;

    // 2. 修改Clash配置，添加本地代理节点
    add_local_proxy_to_clash_config(clash_config_path)?;

    // 3. 通知Clash重载配置
    reload_clash_config().await?;

    // 4. 自动选择我们的节点
    for group in ["PROXY", "Fallback", "Auto"] {
        switch_proxy_group(group, "Local-Chain-Proxy").await?;
    }

    println!("✓ 所有流量已自动走代理链！");
    Ok(())
}
```

## 实现方案

### 方案1：完全自动化（推荐）

**GUI功能**：
- 输入上游代理信息
- 点击"启动"按钮
- 自动完成所有配置

**后台操作**：
1. 启动本地代理服务器（Core模块）
2. 修改Clash配置文件（已有功能）
3. 调用Clash API重载配置
4. 调用Clash API切换到Local-Chain-Proxy
5. 显示状态：✅ 代理链已激活

**优点**：
- ✅ 用户体验最佳
- ✅ 真正的"一键启动"
- ✅ 自动处理所有细节

**缺点**：
- ⚠️ 依赖Clash API（需要开启External Controller）
- ⚠️ 不同Clash版本API可能略有差异

### 方案2：半自动化（当前）

**GUI功能**：
- 输入上游代理信息
- 点击"启动"按钮

**后台操作**：
1. 启动本地代理服务器
2. 修改Clash配置文件
3. 提示用户："请在Clash中选择 Local-Chain-Proxy 节点"

**优点**：
- ✅ 简单可靠
- ✅ 不依赖Clash API
- ✅ 适用所有Clash版本

**缺点**：
- ❌ 用户还需要手动点一下Clash

### 方案3：系统级代理（最彻底）

不通过Clash，直接设置系统代理：

```rust
// 伪代码
fn set_system_proxy() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // macOS: 使用networksetup
        Command::new("networksetup")
            .args(&["-setsocksfirewallproxy", "Wi-Fi", "127.0.0.1", "10808"])
            .output()?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 修改注册表
        // HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings
    }

    Ok(())
}
```

**优点**：
- ✅ 不依赖Clash
- ✅ 真正的"所有流量"

**缺点**：
- ❌ 失去Clash的规则分流功能
- ❌ 不支持TUN模式的高级功能
- ❌ 需要系统权限

## 推荐方案对比

| 方案 | 自动化程度 | 用户操作 | 依赖 | 推荐度 |
|-----|----------|---------|-----|-------|
| 方案1 | 100% | 点击"启动" | Clash API | ⭐⭐⭐⭐⭐ |
| 方案2 | 80% | 点击"启动" + 选节点 | 无 | ⭐⭐⭐⭐ |
| 方案3 | 100% | 点击"启动" | 系统权限 | ⭐⭐⭐ |

## 实现计划

### Phase 1（当前）：Core模块 ✅
- ✅ 本地代理服务器
- ✅ 单上游代理

### Phase 2（下一步）：多上游管理
- ⏳ 多个上游代理
- ⏳ 健康检查
- ⏳ 自动故障切换

### Phase 3（最终目标）：GUI自动化
- ⏳ GUI启动代理服务器
- ⏳ GUI修改Clash配置
- ⏳ GUI调用Clash API
- ⏳ 一键启动代理链

## Clash API 示例代码

```rust
use reqwest;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct ClashApi {
    base_url: String,
    secret: Option<String>,
}

impl ClashApi {
    pub fn new(host: &str, port: u16, secret: Option<String>) -> Self {
        Self {
            base_url: format!("http://{}:{}", host, port),
            secret,
        }
    }

    /// 重载配置
    pub async fn reload_config(&self, config_path: &str) -> Result<()> {
        let url = format!("{}/configs", self.base_url);
        let client = reqwest::Client::new();

        let mut req = client.put(&url)
            .json(&json!({
                "path": config_path,
                "payload": ""
            }));

        if let Some(secret) = &self.secret {
            req = req.header("Authorization", format!("Bearer {}", secret));
        }

        req.send().await?;
        Ok(())
    }

    /// 切换代理组选择
    pub async fn switch_proxy(&self, group: &str, proxy: &str) -> Result<()> {
        let url = format!("{}/proxies/{}", self.base_url, group);
        let client = reqwest::Client::new();

        let mut req = client.put(&url)
            .json(&json!({
                "name": proxy
            }));

        if let Some(secret) = &self.secret {
            req = req.header("Authorization", format!("Bearer {}", secret));
        }

        req.send().await?;
        Ok(())
    }

    /// 获取所有代理信息
    pub async fn get_proxies(&self) -> Result<serde_json::Value> {
        let url = format!("{}/proxies", self.base_url);
        let client = reqwest::Client::new();

        let mut req = client.get(&url);

        if let Some(secret) = &self.secret {
            req = req.header("Authorization", format!("Bearer {}", secret));
        }

        let resp = req.send().await?;
        Ok(resp.json().await?)
    }
}

/// 完整的自动化流程
pub async fn activate_proxy_chain(
    upstream: &str,
    clash_config: &str,
    clash_api: &ClashApi,
) -> Result<()> {
    println!("🚀 启动代理链...");

    // 1. 启动本地代理服务器
    println!("1/4 启动本地代理服务器...");
    let server = ProxyServer::new(ProxyConfig {
        listen_addr: "127.0.0.1:10808".to_string(),
        upstream: UpstreamConfig::from_proxy_string(upstream)?,
    });
    tokio::spawn(async move {
        server.start().await
    });

    // 2. 修改Clash配置
    println!("2/4 修改Clash配置...");
    add_local_proxy_to_config(clash_config)?;

    // 3. 重载Clash配置
    println!("3/4 重载Clash配置...");
    clash_api.reload_config(clash_config).await?;

    // 4. 自动切换到本地代理
    println!("4/4 切换到本地代理节点...");

    // 获取所有代理组
    let proxies = clash_api.get_proxies().await?;

    // 对所有Selector类型的组切换到本地代理
    if let Some(proxy_map) = proxies["proxies"].as_object() {
        for (group_name, group_info) in proxy_map {
            if group_info["type"] == "Selector" {
                if let Some(all) = group_info["all"].as_array() {
                    // 检查是否包含我们的节点
                    if all.iter().any(|n| n == "Local-Chain-Proxy") {
                        clash_api.switch_proxy(group_name, "Local-Chain-Proxy").await?;
                        println!("   ✓ {} → Local-Chain-Proxy", group_name);
                    }
                }
            }
        }
    }

    println!("✅ 代理链已激活！所有流量现在走代理链。");
    Ok(())
}
```

## Clash配置要求

需要在Clash配置中启用External Controller：

```yaml
# Clash配置文件
external-controller: 127.0.0.1:9090
secret: ""  # 可选，建议生产环境设置

# 或者
external-controller: 0.0.0.0:9090
secret: "your-secret-key"
```

## 结论

**推荐实现方案1（完全自动化）**：

1. 在GUI中添加Clash API配置
2. 实现Clash API客户端
3. 实现自动化流程
4. 用户体验：点击"启动" → 所有搞定

**实现优先级**：
1. ✅ Core模块（已完成）
2. ⏳ Clash API集成（Phase 3）
3. ⏳ GUI自动化（Phase 3）

---

最后更新：2026-02-02
