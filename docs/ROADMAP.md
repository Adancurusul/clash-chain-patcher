# Memory Leak Fix Roadmap

**Branch:** `fix/memory-leak`
**Related:** [MEMORY_LEAK_ANALYSIS.md](./MEMORY_LEAK_ANALYSIS.md)

---

## Overview

This roadmap outlines the implementation plan to fix all identified memory leaks. Changes are organized into 4 phases, ordered by severity and dependency. Each phase can be independently tested and merged.

---

## Phase 1: Thread Cancellation Infrastructure

**Goal:** Make all background threads stoppable.
**Priority:** CRITICAL
**Estimated files changed:** 4

### Step 1.1: Add `AtomicBool` cancellation to Watcher thread

**File:** `src/watcher/clash_watcher.rs`

**Current code (line 139-141):**
```rust
loop {
    std::thread::sleep(Duration::from_secs(1));
}
```

**Fix:**
- Accept an `Arc<AtomicBool>` as a `stop_signal` parameter
- Replace infinite loop with: `while !stop_signal.load(Ordering::Relaxed) { sleep(100ms) }`
- Use shorter sleep intervals (100ms instead of 1s) for responsive shutdown
- Return the `stop_signal` from `start()` so the caller can trigger shutdown

```rust
// New signature
pub async fn start(self, stop_signal: Arc<AtomicBool>) -> Result<mpsc::Receiver<WatcherEvent>>

// New loop
loop {
    if stop_signal.load(Ordering::Relaxed) {
        info!("File watcher stopping...");
        break;
    }
    std::thread::sleep(Duration::from_millis(200));
}
```

**Also fix the debouncer task (line 145-211):**
- Pass `stop_signal` to the debouncer tokio task
- Add a check in the `loop` body: `if stop_signal.load() { break; }`

### Step 1.2: Update `WatcherBridge` to use stop signal

**File:** `src/bridge/watcher_bridge.rs`

**Changes:**
- Add field: `stop_signal: Option<Arc<AtomicBool>>`
- In `start()`: create `AtomicBool`, pass to `ClashConfigWatcher::start()`
- In `stop()`: set `stop_signal` to `true` before dropping sender
- In `Drop` impl: call `stop()` automatically for safety

```rust
pub struct WatcherBridge {
    runtime: Runtime,
    config_path: PathBuf,
    event_tx: Option<mpsc::UnboundedSender<WatcherEvent>>,
    stop_signal: Option<Arc<AtomicBool>>,  // NEW
}

pub fn stop(&mut self) {
    // Signal the thread to stop FIRST
    if let Some(signal) = &self.stop_signal {
        signal.store(true, Ordering::Relaxed);
    }
    // Then close the channel
    if let Some(tx) = self.event_tx.take() {
        drop(tx);
    }
    self.stop_signal = None;
}

impl Drop for WatcherBridge {
    fn drop(&mut self) {
        self.stop();
    }
}
```

### Step 1.3: Add cancellation to Health Check thread

**File:** `src/app_impl/health_ops.rs`

**Current code (line 156-188):**
```rust
let handle = std::thread::spawn(move || {
    loop {
        // ... check proxies ...
        thread::sleep(Duration::from_secs(interval_secs));
    }
});
drop(handle);
```

**Fix:**
- Add `stop_signal: Option<Arc<AtomicBool>>` to `AppState`
- Create the `AtomicBool` when starting auto check
- Replace `thread::sleep(300s)` with a loop of short sleeps that checks the flag:

```rust
// Replace: thread::sleep(Duration::from_secs(interval_secs));
// With:
let sleep_end = Instant::now() + Duration::from_secs(interval_secs);
while Instant::now() < sleep_end {
    if stop_signal.load(Ordering::Relaxed) {
        eprintln!("DEBUG: Auto check cancelled during sleep");
        return;
    }
    thread::sleep(Duration::from_millis(500));
}
```

- On toggle off: set `stop_signal` to true → thread exits within 500ms

**File:** `src/app.rs` (AppState)

```rust
pub struct AppState {
    // ... existing fields ...
    pub auto_check_stop: Option<Arc<AtomicBool>>,   // NEW
}
```

### Step 1.4: Test thread cancellation

**Verification:**
- Toggle Watch ON/OFF 20 times → check thread count stays constant
- Toggle Auto Check ON/OFF 10 times → check thread count
- macOS: `ps -M <pid> | wc -l` to count threads
- Or use Activity Monitor → sample process

---

## Phase 2: Shared Tokio Runtime

**Goal:** Use a single shared Tokio runtime instead of per-bridge runtimes.
**Priority:** MEDIUM
**Estimated files changed:** 5

### Step 2.1: Create shared runtime in ProxyState

**File:** `src/state/proxy_state.rs`

**Changes:**
- Add a shared `Arc<Runtime>` field
- Create one runtime in `initialize()`, pass it to all bridges

```rust
pub struct ProxyState {
    runtime: Option<Arc<Runtime>>,  // NEW: shared runtime
    config_bridge: Option<ConfigBridge>,
    health_bridge: Option<HealthBridge>,
    merger_bridge: Option<MergerBridge>,
    // ...
}

pub fn initialize(&mut self) -> Result<(), String> {
    let runtime = Arc::new(Runtime::new()
        .map_err(|e| format!("Failed to create runtime: {}", e))?);

    self.runtime = Some(runtime.clone());

    self.config_bridge = Some(
        ConfigBridge::with_runtime(runtime.clone())?
    );
    self.health_bridge = Some(
        HealthBridge::with_runtime(runtime.clone())?
    );
    // ...
}
```

### Step 2.2: Update bridges to accept shared runtime

**Files:** `src/bridge/config_bridge.rs`, `src/bridge/health_bridge.rs`, `src/bridge/watcher_bridge.rs`

**Changes per bridge:**
- Add `with_runtime(runtime: Arc<Runtime>)` constructor
- Keep `new()` for backward compatibility (tests)
- Use the shared runtime instead of creating a new one

```rust
impl ConfigBridge {
    pub fn with_runtime(runtime: Arc<Runtime>) -> BridgeResult<Self> {
        let manager = ConfigManager::new()
            .map_err(|e| BridgeError::Config(format!("{}", e)))?;
        Ok(Self {
            runtime,
            manager: Arc::new(RwLock::new(manager)),
        })
    }
}
```

### Step 2.3: WatcherBridge uses shared runtime

**File:** `src/bridge/watcher_bridge.rs`

- Accept `Arc<Runtime>` instead of creating a new one
- This is the most impactful change: no more runtime creation on each Watch toggle

```rust
pub fn with_runtime(config_path: impl AsRef<Path>, runtime: Arc<Runtime>) -> BridgeResult<Self> {
    Ok(Self {
        runtime,
        config_path: config_path.as_ref().to_path_buf(),
        event_tx: None,
        stop_signal: None,
    })
}
```

**File:** `src/app_impl/file_ops.rs`

- Pass the shared runtime from `ProxyState` when creating `WatcherBridge`

### Step 2.4: Update WatcherBridge runtime field type

Since the runtime is now shared (`Arc<Runtime>`), `WatcherBridge` should store `Arc<Runtime>` instead of owning `Runtime`. This prevents the runtime from being dropped when the bridge is dropped.

**Impact:**
- Startup: 1 runtime instead of 3 → saves ~16 threads on 8-core Mac
- Watch toggle: 0 new runtimes → no runtime leak

---

## Phase 3: Bounded Data Structures

**Goal:** Cap all growable data structures and add backpressure.
**Priority:** MEDIUM
**Estimated files changed:** 3

### Step 3.1: Cap logs at 200 lines

**File:** `src/app_impl/ui_helpers.rs`

```rust
const MAX_LOG_LINES: usize = 200;

pub(crate) fn add_log(&mut self, cx: &mut Cx, message: &str) {
    self.state.logs.push(message.to_string());

    // Trim old logs if over limit
    if self.state.logs.len() > MAX_LOG_LINES {
        let drain_count = self.state.logs.len() - MAX_LOG_LINES;
        self.state.logs.drain(..drain_count);
    }

    let log_text = self.state.logs.join("\n");
    self.ui.label(id!(log_text)).set_text(cx, &log_text);
}
```

**Alternative (more efficient):** Use `VecDeque<String>` instead of `Vec<String>` so draining from the front is O(1):

```rust
// In app.rs AppState:
pub logs: VecDeque<String>,

// In ui_helpers.rs:
pub(crate) fn add_log(&mut self, cx: &mut Cx, message: &str) {
    self.state.logs.push_back(message.to_string());
    while self.state.logs.len() > MAX_LOG_LINES {
        self.state.logs.pop_front();
    }
    // ...
}
```

### Step 3.2: Use bounded channel for watcher events

**File:** `src/bridge/watcher_bridge.rs`

```rust
// Change from:
let (tx, rx) = mpsc::unbounded_channel();

// Change to:
let (tx, rx) = mpsc::channel(64);  // Max 64 pending events
```

The sender becomes `mpsc::Sender` (bounded), which will exert backpressure when full. The async forwarding task will naturally slow down when the buffer is full.

### Step 3.3: Reuse Vec in handle_event

**File:** `src/app.rs`

Move the temporary Vecs into `AppState` so they're allocated once:

```rust
pub struct AppState {
    // ... existing fields ...
    // Reusable buffers (avoids per-frame allocation)
    pub health_results_buf: Vec<(String, clash_chain_patcher::health::ProxyValidationResult)>,
    pub watcher_events_buf: Vec<clash_chain_patcher::watcher::WatcherEvent>,
}
```

In `handle_event`:
```rust
self.state.health_results_buf.clear();
if let Some(rx) = &self.state.health_check_rx {
    while let Ok(result) = rx.try_recv() {
        self.state.health_results_buf.push(result);
    }
}
// process from buf...
```

---

## Phase 4: Cleanup & Safety Nets

**Goal:** Add Drop impls and defensive code to prevent future regressions.
**Priority:** LOW
**Estimated files changed:** 3

### Step 4.1: Implement Drop for App/AppState

Ensure all background resources are cleaned up when the app exits:

**File:** `src/app.rs`

```rust
impl Drop for AppState {
    fn drop(&mut self) {
        // Stop auto health check
        if let Some(signal) = &self.auto_check_stop {
            signal.store(true, Ordering::Relaxed);
        }

        // Stop file watcher
        if let Some(mut bridge) = self.watcher_bridge.take() {
            bridge.stop();
        }

        eprintln!("DEBUG: AppState cleaned up");
    }
}
```

### Step 4.2: Add thread count monitoring (debug only)

For development builds, add a periodic log of active thread count:

```rust
#[cfg(debug_assertions)]
fn log_thread_count() {
    // Use sysinfo or /proc/self/status to report thread count
    eprintln!("DEBUG: Active threads: {}", /* count */);
}
```

### Step 4.3: Document threading model

Add a doc comment in `src/lib.rs` or `src/bridge/mod.rs` explaining:
- Which threads exist and why
- How to properly shut them down
- The shared runtime pattern

---

## Implementation Order & Dependencies

```
Phase 1 (CRITICAL - do first)
  ├── Step 1.1: Watcher thread cancellation
  ├── Step 1.2: WatcherBridge stop signal
  ├── Step 1.3: Health check thread cancellation
  └── Step 1.4: Test

Phase 2 (MEDIUM - do after Phase 1)
  ├── Step 2.1: Shared runtime in ProxyState
  ├── Step 2.2: Update bridge constructors
  ├── Step 2.3: WatcherBridge shared runtime
  └── Step 2.4: Update runtime field type

Phase 3 (MEDIUM - independent of Phase 2)
  ├── Step 3.1: Cap logs
  ├── Step 3.2: Bounded watcher channel
  └── Step 3.3: Reuse Vec buffers

Phase 4 (LOW - do last)
  ├── Step 4.1: Drop impls
  ├── Step 4.2: Debug monitoring
  └── Step 4.3: Documentation
```

**Phase 1 and Phase 3 are independent** and can be worked on in parallel.
**Phase 2 depends on Phase 1** (stop signal infrastructure needed before restructuring runtimes).
**Phase 4 depends on all previous phases.**

---

## Testing Strategy

### Manual Testing
1. **Thread leak test:** Toggle Watch ON/OFF 20 times, observe thread count in Activity Monitor
2. **Memory growth test:** Leave Auto Check running for 1 hour, observe RSS in Activity Monitor
3. **Rapid toggle test:** Rapidly toggle Auto Check and Watch to expose race conditions
4. **Long running test:** Run app for 24 hours with Watch + Auto Check active

### Automated Testing
1. Unit tests for `AtomicBool` cancellation pattern
2. Integration test: start watcher, stop watcher, verify thread exits within 1s
3. Integration test: start auto check, stop auto check, verify thread exits within 1s
4. Test that logs never exceed MAX_LOG_LINES

### macOS-Specific Monitoring
```bash
# Count threads for the process
ps -M $(pgrep clash-chain-patcher) | wc -l

# Monitor memory over time
while true; do ps -o rss,vsz -p $(pgrep clash-chain-patcher); sleep 5; done

# Profile with Instruments
xcrun xctrace record --template 'Allocations' --launch clash-chain-patcher
```

---

## Expected Results After Fix

| Metric | Before | After |
|--------|--------|-------|
| Threads at startup | ~30+ (3 runtimes) | ~10 (1 runtime) |
| Threads after 20 Watch toggles | ~50+ (leaked) | ~10 (stable) |
| Memory after 1 hour | Growing | Stable |
| Watch stop latency | Never (leaked) | < 500ms |
| Auto Check stop latency | Up to 5 minutes | < 1s |
| Max log entries | Unbounded | 200 |
