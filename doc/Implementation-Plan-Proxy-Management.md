# Implementation Plan: Proxy Management in Original UI

**Date**: 2026-02-03
**Goal**: Add multi-proxy management to existing app.rs
**Approach**: Extend original UI, not replace

---

## Phase 1: Structure Analysis ✅

### Current app.rs Structure
- **Lines**: 554 total
- **Window Size**: 440x620
- **Main Sections**:
  1. Header with logo
  2. Config file selector
  3. SOCKS5 Proxy inputs (single proxy)
  4. Proxy string input with Fill button
  5. Filter input
  6. Action buttons (Preview, Apply, Save)
  7. Output/Log area
  8. Status bar with version

### Current State Management
```rust
pub struct AppState {
    config_content: Option<String>,
    config_filename: Option<String>,
    output_content: Option<String>,
    logs: Vec<String>,
}
```

---

## Phase 2: Integration Strategy

### Strategy: Hybrid Approach

Keep existing single-proxy UI for quick operations, add proxy management section below.

### New Window Size
- Current: 440x620
- New: 800x900 (wider for list, taller for proxy section)

### New UI Layout
```
┌─────────────────────────────────────────┐
│  Header (Logo + Title)                  │
├─────────────────────────────────────────┤
│  Config File Selector                   │
├─────────────────────────────────────────┤
│  Quick Proxy (Single - existing)        │
│  [Keep for quick single proxy ops]      │
├─────────────────────────────────────────┤
│  ┌─ Proxy Pool Management ─────────┐   │
│  │  [+ Add]  [Check All]           │   │
│  │  ┌───────────────────────────┐  │   │
│  │  │ □ HK Proxy  ✅ 120ms      │  │   │
│  │  │   [Edit] [Del] [Check]    │  │   │
│  │  ├───────────────────────────┤  │   │
│  │  │ □ SG Proxy  ❌ Timeout    │  │   │
│  │  │   [Edit] [Del] [Check]    │  │   │
│  │  └───────────────────────────┘  │   │
│  │  Stats: 2 proxies, 1 healthy    │   │
│  └──────────────────────────────────┘   │
├─────────────────────────────────────────┤
│  Action Buttons                         │
├─────────────────────────────────────────┤
│  Output/Logs                            │
├─────────────────────────────────────────┤
│  Status Bar                             │
└─────────────────────────────────────────┘
```

---

## Phase 3: Code Changes

### 3.1 Add Imports
```rust
use clash_chain_patcher::state::ProxyState;
use clash_chain_patcher::config::UpstreamProxy;
use clash_chain_patcher::proxy::config::UpstreamConfig;
```

### 3.2 Extend AppState
```rust
pub struct AppState {
    // Existing fields
    config_content: Option<String>,
    config_filename: Option<String>,
    output_content: Option<String>,
    logs: Vec<String>,

    // New fields
    proxy_state: Option<ProxyState>,
    checking: bool,
}
```

### 3.3 UI Additions (live_design!)

Add after existing proxy inputs, before action buttons:

```rust
// Proxy Pool Management Section
<View> {
    width: Fill,
    height: 300,  // Fixed height, scrollable
    padding: 8,
    flow: Down,
    spacing: 6,
    show_bg: true,
    draw_bg: {color: #333333}

    // Header
    <View> {
        width: Fill,
        height: Fit,
        flow: Right,
        spacing: 6,
        align: {y: 0.5},

        <Label> {
            text: "Proxy Pool"
            draw_text: {color: #ffffff, text_style: {font_size: 11.0}}
        }

        add_proxy_btn = <Button> {
            text: "+ Add"
            draw_text: {color: #ffffff}
        }

        check_all_proxies_btn = <Button> {
            text: "Check All"
            draw_text: {color: #ffffff}
        }

        <View> { width: Fill, height: Fit }

        pool_stats_label = <Label> {
            text: "0 proxies"
            draw_text: {color: #888888, text_style: {font_size: 9.0}}
        }
    }

    // Proxy List (ScrollView)
    proxy_list_scroll = <ScrollYView> {
        width: Fill,
        height: Fill,
        padding: 4,
        show_bg: true,
        draw_bg: {color: #222222}

        proxy_list_container = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: 4,

            // Proxy items will be added dynamically here
            // For now, add empty state
            proxy_empty_label = <Label> {
                width: Fill,
                height: Fit,
                padding: 20,
                text: "No proxies in pool. Click '+ Add' to create one."
                draw_text: {
                    color: #666666,
                    text_style: {font_size: 10.0}
                }
            }
        }
    }
}
```

### 3.4 Proxy Item Template

Since Makepad doesn't support dynamic lists easily without PortalList, we'll:
1. Store proxy data in AppState
2. Regenerate proxy items on update
3. Use fixed IDs with indices

```rust
// Will generate items like:
proxy_item_0 = <View> {
    width: Fill,
    height: Fit,
    padding: 6,
    flow: Down,
    spacing: 4,
    show_bg: true,
    draw_bg: {color: #2a2a2a}

    <View> {
        width: Fill,
        height: Fit,
        flow: Right,
        spacing: 6,
        align: {y: 0.5},

        proxy_0_name = <Label> {
            width: Fill,
            text: "Proxy Name"
            draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
        }

        proxy_0_status = <Label> {
            text: "⚫ Unknown"
            draw_text: {color: #888888, text_style: {font_size: 9.0}}
        }

        proxy_0_latency = <Label> {
            text: ""
            draw_text: {color: #00ff00, text_style: {font_size: 9.0}}
        }
    }

    <View> {
        width: Fill,
        height: Fit,
        flow: Right,
        spacing: 4,
        padding: {left: 10},

        proxy_0_edit_btn = <Button> {
            text: "Edit"
            draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
        }

        proxy_0_delete_btn = <Button> {
            text: "Delete"
            draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
        }

        proxy_0_check_btn = <Button> {
            text: "Check"
            draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
        }

        proxy_0_toggle_btn = <Button> {
            text: "Enable"
            draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
        }
    }
}
```

---

## Phase 4: Implementation Steps

### Step 1: Extend AppState ✓
- Add proxy_state field
- Add checking flag
- Update Default impl

### Step 2: Add UI Elements ✓
- Increase window size
- Add proxy pool section
- Add buttons and labels

### Step 3: Initialize ProxyState ✓
- Create ProxyState on app start
- Load existing proxies from config
- Add test data if empty (for dev)

### Step 4: Implement Add Proxy ✓
- Button handler
- For now: use existing proxy string input
- Add proxy from quick form to pool
- Refresh display

### Step 5: Implement Proxy Display ✓
- Read proxies from ProxyState
- Update labels with proxy info
- Show health status with colors

### Step 6: Implement Health Check ✓
- Individual check button handler
- Check All button handler
- Update UI with results
- Handle async operations

### Step 7: Implement Edit/Delete ✓
- Edit: populate quick form
- Delete: remove from pool
- Refresh display

### Step 8: Integration with Apply ✓
- When Apply is clicked
- Use enabled proxies from pool
- Merge into Clash config
- Show results

### Step 9: Fix Warnings ✓
- Remove unused imports
- Mark intentionally unused items with #[allow(dead_code)]
- Fix any clippy warnings

### Step 10: Testing ✓
- Test add/edit/delete
- Test health checks
- Test apply with multiple proxies
- Test save/load

---

## Phase 5: Dynamic Display Strategy

Since Makepad's PortalList has issues, we'll use a simpler approach:

### Option A: Fixed Maximum (Recommended)
- Support up to 10 proxies in UI
- Pre-define proxy_item_0 through proxy_item_9
- Show/hide based on actual proxy count
- Simple, no dynamic memory

### Option B: Text-Based List
- Show proxies as formatted text in a single Label
- Buttons at bottom for selected proxy
- Selection via index input
- Very simple but less visual

### Decision: Use Option A for better UX

---

## Phase 6: File Structure

### Modified Files
- `src/app.rs` - Main changes

### No New Files
- Keep everything in app.rs for simplicity
- Use existing bridge layer from state

---

## Phase 7: Error Handling

### Principles
1. All proxy operations return Result
2. Display errors in status bar
3. Log detailed errors in output
4. Never panic on user input

### Error Display
- Red color for errors in status
- Detailed messages in log area
- Clear user-actionable messages

---

## Phase 8: Testing Checklist

- [ ] App starts without errors
- [ ] ProxyState initializes correctly
- [ ] Add proxy from quick form works
- [ ] Proxy list displays correctly
- [ ] Individual health check works
- [ ] Check all works
- [ ] Edit loads proxy to form
- [ ] Delete removes proxy
- [ ] Enable/disable toggle works
- [ ] Apply uses enabled proxies only
- [ ] Save persists proxy pool
- [ ] Load restores proxy pool
- [ ] No compiler warnings
- [ ] No runtime panics

---

## Phase 9: Timeline

- **Step 1-2**: 30 min (Structure setup)
- **Step 3-4**: 45 min (State + Add)
- **Step 5**: 30 min (Display)
- **Step 6**: 45 min (Health checks)
- **Step 7**: 30 min (Edit/Delete)
- **Step 8**: 30 min (Integration)
- **Step 9**: 30 min (Fix warnings)
- **Step 10**: 45 min (Testing)

**Total**: ~5 hours of careful implementation

---

## Phase 10: Success Criteria

✅ Compiles with 0 errors
✅ 0 warnings (or all marked as intentional)
✅ All features working
✅ Original functionality preserved
✅ New proxy pool working
✅ Health checks functional
✅ No runtime crashes

---

**Ready to begin implementation**
