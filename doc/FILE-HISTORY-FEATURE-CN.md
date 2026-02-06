# YAML 文件历史功能 - 实现说明

**版本**: 0.1.2
**日期**: 2026-02-03
**方案**: A - 下拉菜单（简化版）

---

## ✨ 新功能

### YAML 文件历史列表

在 Config 行右侧添加了一个 **"▼" 按钮**，点击后展开/折叠最近使用的 Clash 配置文件列表。

---

## 🎨 UI 设计

### Config 行布局

```
┌─────────────────────────────────────────────────────────────┐
│ Config [Select] clash-config.yaml [×] [Watch: OFF] [▼]     │
└─────────────────────────────────────────────────────────────┘
```

**组件说明**:
- `Config` - 标签
- `[Select]` - 选择新文件按钮
- `clash-config.yaml` - 当前文件名
- `[×]` - 清除按钮（红色）
- `[Watch: OFF]` - 监控切换按钮
- `[▼]` - 历史展开/折叠按钮（**新增**）

### 展开后的历史列表

```
┌─────────────────────────────────────────────────────────────┐
│ Config [Select] clash-config.yaml [×] [Watch: OFF] [▲]     │
├─────────────────────────────────────────────────────────────┤
│   Recent Files:                                              │
│   [clash-config.yaml]  ← 点击加载                           │
│   [backup-config.yaml]                                       │
│   [old-config.yaml]                                          │
└─────────────────────────────────────────────────────────────┘
```

**交互**:
- 点击 `[▼]` → 变成 `[▲]` + 显示历史列表
- 点击 `[▲]` → 变成 `[▼]` + 隐藏列表
- 点击任何历史文件 → 加载该文件 + 自动隐藏列表

---

## 🔧 技术实现

### 1. 数据结构

**AppState 新增字段**:
```rust
pub struct AppState {
    // ... 原有字段 ...

    /// Recent Clash config file paths (max 5)
    recent_files: Vec<String>,

    /// Whether file history dropdown is shown
    show_file_history: bool,
}
```

**存储策略**:
- 最多保存 **5** 个最近文件
- 新文件插入到列表开头
- 如果文件已存在，移除旧位置，插入到开头
- **当前**: 只在内存中（应用重启后丢失）
- **待实现**: 持久化到 `config.json`

---

### 2. UI 组件

**历史按钮**: [src/app.rs:124-134](src/app.rs#L124-L134)
```rust
toggle_history_btn = <Button> {
    width: Fit,
    height: Fit,
    padding: {left: 6, right: 6, top: 2, bottom: 2},
    text: "▼"
    draw_text: {color: #888888, text_style: {font_size: 10.0}}
    draw_bg: {
        fn pixel(self) -> vec4 {
            return mix(#333333, #555555, self.hover);
        }
    }
}
```

**历史列表**: [src/app.rs:137-198](src/app.rs#L137-L198)
```rust
file_history_view = <View> {
    visible: false,  // 默认隐藏
    width: Fill,
    height: Fit,
    padding: {left: 16, right: 8, top: 4, bottom: 4},
    flow: Down,
    spacing: 2,
    show_bg: true,
    draw_bg: {color: #2a2a2a}

    <Label> {
        text: "Recent Files:"
        draw_text: {color: #888888, text_style: {font_size: 9.0}}
    }

    recent_file_1 = <Button> { ... }  // 最近文件 1
    recent_file_2 = <Button> { ... }  // 最近文件 2
    recent_file_3 = <Button> { ... }  // 最近文件 3
}
```

**显示逻辑**:
- 只显示前 3 个最近文件（虽然内存保存 5 个）
- 如果少于 3 个，只显示实际数量
- 文件名自动提取（不显示完整路径）

---

### 3. 核心方法

#### toggle_file_history() - 展开/折叠

[src/app.rs:883-893](src/app.rs#L883-L893)

```rust
fn toggle_file_history(&mut self, cx: &mut Cx) {
    self.state.show_file_history = !self.state.show_file_history;

    // Toggle visibility
    self.ui.view(id!(file_history_view))
        .set_visible(cx, self.state.show_file_history);

    // Update button text
    let button_text = if self.state.show_file_history { "▲" } else { "▼" };
    self.ui.button(id!(toggle_history_btn)).set_text(cx, button_text);

    self.ui.redraw(cx);
}
```

**功能**:
- 切换 `show_file_history` 状态
- 显示/隐藏 `file_history_view`
- 更新按钮文字（▼ ↔ ▲）

---

#### add_to_recent_files() - 添加到历史

[src/app.rs:895-906](src/app.rs#L895-L906)

```rust
fn add_to_recent_files(&mut self, path: String) {
    // Remove if already exists
    self.state.recent_files.retain(|p| p != &path);

    // Add to front
    self.state.recent_files.insert(0, path);

    // Keep only last 5
    if self.state.recent_files.len() > 5 {
        self.state.recent_files.truncate(5);
    }

    // TODO: Save to persistent config
}
```

**逻辑**:
1. 如果文件已存在，先删除
2. 插入到列表开头（最新）
3. 如果超过 5 个，截断到 5 个
4. **待实现**: 保存到配置文件

---

#### refresh_file_history_display() - 刷新显示

[src/app.rs:908-932](src/app.rs#L908-L932)

```rust
fn refresh_file_history_display(&mut self, cx: &mut Cx) {
    // Update recent file buttons
    for i in 0..3 {
        let (btn_id, visible, text) = match i {
            0 => (id!(recent_file_1), self.state.recent_files.len() > 0,
                  self.state.recent_files.get(0)),
            1 => (id!(recent_file_2), self.state.recent_files.len() > 1,
                  self.state.recent_files.get(1)),
            2 => (id!(recent_file_3), self.state.recent_files.len() > 2,
                  self.state.recent_files.get(2)),
            _ => continue,
        };

        if visible {
            if let Some(path) = text {
                // Show filename only (not full path)
                let filename = std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());

                self.ui.button(btn_id).set_text(cx, &filename);
                self.ui.button(btn_id).set_visible(cx, true);
            }
        } else {
            self.ui.button(btn_id).set_visible(cx, false);
        }
    }
}
```

**功能**:
- 遍历前 3 个最近文件
- 提取文件名（去掉路径）
- 设置按钮文字和可见性

---

#### select_recent_file() - 选择历史文件

[src/app.rs:934-943](src/app.rs#L934-L943)

```rust
fn select_recent_file(&mut self, cx: &mut Cx, index: usize) {
    if let Some(path) = self.state.recent_files.get(index).cloned() {
        self.load_config_file(cx, path);

        // Hide history after selection
        self.state.show_file_history = false;
        self.ui.view(id!(file_history_view)).set_visible(cx, false);
        self.ui.button(id!(toggle_history_btn)).set_text(cx, "▼");
    }
}
```

**功能**:
- 获取指定索引的文件路径
- 调用 `load_config_file()` 加载
- 自动隐藏历史列表

---

#### load_config_file() - 加载配置文件

[src/app.rs:802-836](src/app.rs#L802-L836)

```rust
fn load_config_file(&mut self, cx: &mut Cx, path_str: String) {
    let path = std::path::Path::new(&path_str);

    match std::fs::read_to_string(path) {
        Ok(content) => {
            let filename = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            self.state.config_content = Some(content);
            self.state.config_filename = Some(filename.clone());

            // Add to recent files (if not already there)
            self.add_to_recent_files(path_str.clone());

            // Set Clash config path for proxy pool merging
            if let Some(state) = &mut self.state.proxy_state {
                state.set_clash_config_path(path.to_path_buf());
                self.add_log(cx, &format!("✓ Loaded: {}", filename));
                self.add_log(cx, "  Clash config path set for proxy pool");
            } else {
                self.add_log(cx, &format!("Loaded: {}", filename));
            }

            self.ui.label(id!(file_label)).set_text(cx, &filename);
            self.set_status(cx, "Loaded");
            self.refresh_file_history_display(cx);
            self.ui.redraw(cx);
        }
        Err(e) => {
            self.add_log(cx, &format!("Error: {}", e));
            self.set_status(cx, "Error");
            self.ui.redraw(cx);
        }
    }
}
```

**功能**:
- 读取文件内容
- 更新应用状态
- **添加到历史记录**（第 816 行）
- 刷新历史列表显示（第 826 行）

---

### 4. 事件处理

[src/app.rs:693-707](src/app.rs#L693-L707)

```rust
// Toggle history button
if self.ui.button(id!(toggle_history_btn)).clicked(actions) {
    self.toggle_file_history(cx);
}

// Recent file buttons
if self.ui.button(id!(recent_file_1)).clicked(actions) {
    self.select_recent_file(cx, 0);
}
if self.ui.button(id!(recent_file_2)).clicked(actions) {
    self.select_recent_file(cx, 1);
}
if self.ui.button(id!(recent_file_3)).clicked(actions) {
    self.select_recent_file(cx, 2);
}
```

---

## 📊 工作流程

### 流程图

```
用户选择文件 (Select)
    ↓
load_config_file() 被调用
    ↓
add_to_recent_files() 添加到历史
    ↓
refresh_file_history_display() 刷新 UI
    ↓
最近文件列表更新（但默认隐藏）
    ↓
用户点击 "▼" 按钮
    ↓
toggle_file_history() 展开列表
    ↓
用户点击某个历史文件
    ↓
select_recent_file() 加载该文件
    ↓
历史列表自动隐藏
```

---

## 🧪 使用示例

### 场景 1: 首次使用

```
1. 点击 "Select" 选择 /Users/me/.config/clash/config.yaml
   → 文件名显示：config.yaml
   → 历史列表添加该文件（但隐藏）

2. 点击 "▼" 展开历史
   → 显示:
     Recent Files:
     [config.yaml]

3. 关闭应用（历史丢失，因为未持久化）
```

### 场景 2: 多文件切换

```
1. 选择 config-1.yaml
2. 选择 config-2.yaml
3. 选择 config-3.yaml

4. 点击 "▼" 展开历史
   → 显示:
     Recent Files:
     [config-3.yaml]  ← 最新
     [config-2.yaml]
     [config-1.yaml]

5. 点击 [config-1.yaml]
   → 加载 config-1.yaml
   → 列表自动隐藏
   → 历史列表变为:
     [config-1.yaml]  ← 移到最前
     [config-3.yaml]
     [config-2.yaml]
```

### 场景 3: 超过 5 个文件

```
1. 依次选择 6 个文件

2. 点击 "▼"
   → 只显示最近 3 个:
     [file-6.yaml]
     [file-5.yaml]
     [file-4.yaml]

   但内存中保存 5 个:
   recent_files = [file-6, file-5, file-4, file-3, file-2]
   （file-1 被截断丢弃）
```

---

## 🔜 待实现功能

### 1. 持久化存储 (高优先级)

**问题**: 当前历史只在内存中，应用重启后丢失

**解决方案**: 保存到 `config.json`

```json
{
  "recent_clash_files": [
    "/Users/me/.config/clash/config.yaml",
    "/Users/me/backup-config.yaml",
    "/Users/me/old-config.yaml"
  ],
  "upstream_proxies": [...],
  ...
}
```

**实现位置**:
- `add_to_recent_files()` 中添加保存逻辑
- `init_proxy_state()` 中加载历史

---

### 2. 文件验证

**功能**: 检查历史文件是否仍然存在

```rust
fn refresh_file_history_display(&mut self, cx: &mut Cx) {
    for path in &self.state.recent_files {
        // 检查文件是否存在
        if !std::path::Path::new(path).exists() {
            // 标记为不可用或从列表移除
        }
    }
}
```

---

### 3. 右键菜单

**功能**: 右键点击历史文件显示菜单

- "Open" - 加载文件
- "Show in Finder" - 在文件管理器中显示
- "Remove from History" - 从历史中删除
- "Copy Path" - 复制完整路径

**挑战**: Makepad 可能不支持右键菜单

---

### 4. 完整路径提示

**功能**: 鼠标悬停时显示完整路径

```
[config.yaml]  ← 鼠标悬停
  ↓
显示 tooltip: /Users/me/.config/clash/config.yaml
```

**挑战**: Makepad 可能不支持 tooltip

---

### 5. 显示更多信息

**功能**: 除了文件名，还显示修改时间

```
Recent Files:
[config.yaml] (2 hours ago)
[backup.yaml] (yesterday)
[old.yaml] (3 days ago)
```

---

## 📝 修改的文件

### src/app.rs

**UI 定义** (第 124-198 行):
- 添加 `toggle_history_btn` 按钮
- 添加 `file_history_view` 视图
- 添加 `recent_file_1/2/3` 按钮

**数据结构** (第 585-587 行):
- `recent_files: Vec<String>`
- `show_file_history: bool`

**事件处理** (第 693-707 行):
- `toggle_history_btn` 点击事件
- `recent_file_1/2/3` 点击事件

**方法实现**:
- `load_config_file()` - 第 802-836 行
- `toggle_file_history()` - 第 883-893 行
- `add_to_recent_files()` - 第 895-906 行
- `refresh_file_history_display()` - 第 908-932 行
- `select_recent_file()` - 第 934-943 行

---

## ✅ 编译状态

```bash
$ cargo build --release
   Compiling clash-chain-patcher v0.1.2
    Finished `release` profile [optimized] target(s) in 1m 45s
```

**结果**:
- ✅ 0 errors
- ✅ 0 warnings
- ✅ 编译成功

---

## 🎯 总结

### 已实现

1. ✅ "▼/▲" 展开/折叠按钮
2. ✅ 显示最近 3 个文件
3. ✅ 点击加载历史文件
4. ✅ 自动添加新文件到历史
5. ✅ 最多保存 5 个文件（内存）
6. ✅ 自动提取文件名显示

### 待实现

1. ⏳ 持久化到 `config.json`
2. ⏳ 文件存在性验证
3. ⏳ 右键菜单（删除、复制路径）
4. ⏳ 完整路径 tooltip
5. ⏳ 显示修改时间

### 使用建议

1. Select 文件后，点击 "▼" 查看历史
2. 可以快速切换最近使用的 3 个文件
3. **注意**: 应用重启后历史丢失（待持久化）

---

## 📚 相关文档

- [工作流程详解](WORKFLOW-EXPLAINED-CN.md) - 完整工作流程说明
- [修复总结](FIXES-2026-02-03-2.md) - 今日其他修复
- [10-槽位实现](10-SLOT-IMPLEMENTATION.md) - 代理池实现
