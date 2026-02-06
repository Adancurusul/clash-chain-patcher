# 新 UI 测试指南

**创建日期**: 2026-02-02
**UI 版本**: v2 (代理管理版本)

---

## 运行新 UI

### 方法1: 修改 main.rs

编辑 `src/main.rs`，将：
```rust
mod app;
use app::*;
```

改为：
```rust
mod app_v2;
use app_v2::*;
```

然后运行：
```bash
cargo run --release
```

### 方法2: 创建独立二进制

在 `Cargo.toml` 中添加：
```toml
[[bin]]
name = "clash-patcher-v2"
path = "src/main_v2.rs"
```

创建 `src/main_v2.rs`：
```rust
mod app_v2;
use app_v2::*;
```

运行：
```bash
cargo run --release --bin clash-patcher-v2
```

---

## 功能测试清单

### 1. 启动测试
- [ ] 应用正常启动
- [ ] 窗口大小正确 (700x800)
- [ ] 界面完整显示
- [ ] 状态栏显示 "初始化完成"

### 2. 代理列表测试
- [ ] 空状态提示显示正确
- [ ] 统计信息显示 "共 0 个代理"

### 3. 添加代理测试（待实现）
- [ ] 点击 "+ 添加代理" 按钮
- [ ] 对话框打开
- [ ] 填写代理信息
- [ ] 保存成功
- [ ] 列表刷新显示新代理

### 4. 代理操作测试（待实现）
- [ ] 启用/禁用代理
- [ ] 编辑代理信息
- [ ] 删除代理
- [ ] 单个代理健康检查

### 5. 批量健康检查测试（待实现）
- [ ] 点击 "检查所有" 按钮
- [ ] 状态栏显示进度
- [ ] 健康状态更新
- [ ] 延迟信息显示
- [ ] 错误信息显示

### 6. Clash 配置测试（待实现）
- [ ] 选择 Clash 配置文件
- [ ] 路径显示正确
- [ ] 点击 "合并到 Clash"
- [ ] 合并成功提示
- [ ] 配置文件更新

### 7. 监控测试（待实现）
- [ ] 启动监控
- [ ] 监控状态显示
- [ ] 自动检测配置变化
- [ ] 自动合并功能

---

## 已知问题

### 当前版本限制

1. **文件对话框未实现**
   - 需要集成系统文件选择器
   - 临时方案: 硬编码测试路径

2. **添加代理对话框未实现**
   - 需要创建自定义对话框组件
   - 临时方案: 通过代码添加测试数据

3. **PortalList 集成未完成**
   - 代理列表暂时无法动态更新
   - 需要完善 PortalList 数据绑定

4. **实时状态更新**
   - 健康检查是同步的，会阻塞 UI
   - 需要实现异步更新机制

---

## 临时测试方案

### 通过代码添加测试数据

在 `app_v2.rs` 的初始化代码中添加：

```rust
// 初始化完成后添加测试数据
if state.list_upstreams().is_empty() {
    // 添加测试代理1
    let proxy1 = UpstreamProxy::from_proxy_string(
        "Hong Kong Proxy".to_string(),
        "64.32.179.160:60088:user:pass"
    ).unwrap();
    let _ = state.add_upstream(proxy1);

    // 添加测试代理2
    let proxy2 = UpstreamProxy::from_proxy_string(
        "Singapore Proxy".to_string(),
        "45.76.123.45:1080:user:pass"
    ).unwrap();
    let _ = state.add_upstream(proxy2);

    println!("添加了 {} 个测试代理", state.list_upstreams().len());
}
```

### 设置测试 Clash 配置路径

```rust
// 设置 Clash 配置路径
use std::path::PathBuf;
let clash_path = PathBuf::from("/Users/xxx/.config/clash/config.yaml");
if clash_path.exists() {
    state.set_clash_config_path(clash_path.clone());
    println!("Clash 配置路径: {}", clash_path.display());
}
```

---

## 下一步开发

### 高优先级
1. ✅ 基础 UI 框架
2. ⏳ 代理列表动态更新
3. ⏳ 添加/编辑代理对话框
4. ⏳ 健康检查功能集成

### 中优先级
5. ⏳ 文件对话框集成
6. ⏳ Clash 配置合并
7. ⏳ 监控功能

### 低优先级
8. ⏳ 异步状态更新
9. ⏳ 进度指示器
10. ⏳ 动画效果

---

## 性能指标

### 目标性能
- 启动时间: < 2s
- UI 响应: < 100ms
- 健康检查: < 5s (10个代理)
- 配置合并: < 1s
- 内存占用: < 100MB

---

## 截图位置

测试截图保存在: `doc/screenshots/v2/`

- `main-interface.png` - 主界面
- `proxy-list-empty.png` - 空列表
- `proxy-list-full.png` - 有数据的列表
- `health-check.png` - 健康检查状态
- `clash-config.png` - Clash 配置区域

---

**测试人员**: [你的名字]
**测试日期**: 2026-02-02
**测试版本**: v0.2.0-dev
