# Phase 3 Implementation - Issue Report

**Date**: 2026-02-03
**Status**: ❌ BLOCKED - Runtime Panic

---

## Summary

Successfully created the UI code for Phase 3 (app_v2.rs), but encountering a Makepad runtime panic that prevents the application from launching.

## Current Status

### ✅ Completed
1. Fixed all compilation errors
2. Translated all code and comments to English
3. Restructured borrowing to avoid conflicts
4. Created comprehensive UI design
5. Build completes successfully (0 errors, 11 warnings - expected)

### ❌ Blocking Issue

**Error**: Runtime panic in Makepad's live_registry
```
thread 'main' panicked at makepad-live-compiler-1.0.0/src/live_registry.rs:965:75:
called `Option::unwrap()` on a `None` value
```

**Location**: `LiveRegistry::expand_all_documents`

## Investigation

### Findings

1. **Original App Also Crashes**
   - When run directly (`./target/release/clash-chain-patcher`): crashes with "Bundled file open failed"
   - This is a resource loading issue, expected behavior

2. **Running with cargo run**
   - Original app:  runs successfully with `cargo run --bin clash-chain-patcher`
   - V2 app: PANICS with `cargo run --bin clash-patcher-v2`
   - V3 minimal app: PANICS with `cargo run --bin clash-patcher-v3`

3. **Widgets Investigation**
   - Working app uses: Root, Window, View, Label, Button, TextInput, ScrollYView, Image
   - V2 app was using: CheckBox (not in working app) - **REMOVED**
   - Still panicking after CheckBox removal

4. **Attempted Fixes**
   - ✅ Removed ScrollXYView (changed to View)
   - ✅ Added proper caption_bar structure
   - ✅ Removed CheckBox widget
   - ✅ Simplified to minimal test app (v3)
   - ❌ Still panicking

### Code Structure Comparison

**Working App (app.rs)**:
```rust
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                // ... working structure
            }
        }
    }
}
```

**V2 App (app_v2.rs)** - Same structure, but panics
**V3 Minimal** - Copied exact structure from working app, but panics

## Hypotheses

### 1. Widget Name Conflict ❓
- The `AppV2` name might conflict with something
- **Test**: Rename to different name

### 2. Missing Widget Definition ❓
- Some widget we're using doesn't exist in `link::widgets::*`
- **Evidence**: CheckBox was one such widget
- **Action**: Need to audit all widgets used

### 3. Makepad Version/Build Issue ❓
- Something about how the binary is built
- **Evidence**: Minimal v3 app with same structure also fails
- **Action**: Check if deps are correctly linked

### 4. Live Design Macro Syntax Error ❓
- Some subtle syntax error in live_design! macro
- **Action**: Copy working app line-by-line

## Files Involved

- [src/app_v2.rs](src/app_v2.rs) - Main v2 implementation (~600 lines)
- [src/main_v2.rs](src/main_v2.rs) - Entry point
- [src/app_v3_minimal.rs](src/app_v3_minimal.rs) - Minimal test version
- [src/main_v3.rs](src/main_v3.rs) - V3 entry point

## Build Output

```bash
$ cargo build --bin clash-patcher-v2
   Compiling clash-chain-patcher v0.1.2
warning: `clash-chain-patcher` (lib) generated 11 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.64s

$ cargo run --bin clash-patcher-v2
     Running `target/debug/clash-patcher-v2`

thread 'main' panicked at makepad-live-compiler-1.0.0/src/live_registry.rs:965:75:
called `Option::unwrap()` on a `None` value
```

## Next Steps

### Immediate Actions
1. **Copy exact working app structure** - Make V3 identical to app.rs, just change content
2. **Widget audit** - List all widgets used in v2, verify each exists
3. **Binary name test** - Try using original binary name
4. **Check Makepad examples** - See if there are similar multi-binary setups

### Alternative Approaches
1. **Modify app.rs directly** - Instead of new file, add features to existing app
2. **Feature flags** - Use Cargo features to switch between UIs
3. **Ask for help** - Post issue on Makepad GitHub/Discord

## Compiler Warnings

All warnings are expected (unused bridge code for future features):
- 11 warnings in bridge layer
- All `dead_code` warnings
- Will be resolved as features are implemented

---

**Priority**: 🔴 HIGH - Blocking all Phase 3 progress
**Impact**: Cannot test or develop UI until resolved
**Estimated Time**: Unknown - needs debugging help or different approach
