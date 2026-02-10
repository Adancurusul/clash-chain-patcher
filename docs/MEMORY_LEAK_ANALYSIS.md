# Memory Leak Analysis Report

**Date:** 2026-02-10
**Branch:** `fix/memory-leak`
**Platform:** macOS (Darwin 25.3.0)

---

## Executive Summary

The application suffers from memory leaks caused primarily by **unkillable background threads** and **unbounded data structures**. The root cause pattern is consistent: spawning `std::thread` with infinite `loop { sleep() }` without any cancellation mechanism. Each toggle of Watch or Auto Health Check leaks OS threads and Tokio runtimes that never get reclaimed.

---

## Issue #1: File Watcher Thread Never Stops (CRITICAL)

**Files:**
- `src/watcher/clash_watcher.rs:139-141`
- `src/bridge/watcher_bridge.rs:14-78`
- `src/app_impl/file_ops.rs:80-147`

**Problem:**

`ClashConfigWatcher::start()` spawns an OS thread with an infinite sleep loop to keep the `notify::Watcher` alive:

```rust
// clash_watcher.rs:99-142
std::thread::spawn(move || {
    // ... creates RecommendedWatcher ...
    // Keep the watcher alive
    loop {
        std::thread::sleep(Duration::from_secs(1));  // NEVER EXITS
    }
});
```

When the user clicks "Watch: OFF", `WatcherBridge::stop()` only drops the channel sender:

```rust
// watcher_bridge.rs:63-67
pub fn stop(&mut self) {
    if let Some(tx) = self.event_tx.take() {
        drop(tx); // Closes the sender, but the thread doesn't know
    }
}
```

The OS thread has **no way to observe** that the sender was dropped. It will loop forever.

Additionally, `ClashConfigWatcher::start()` also spawns a Tokio debouncer task (`tokio::spawn` at line 145) that runs inside the `WatcherBridge`'s runtime. When the runtime is dropped (with the bridge), the debouncer may or may not be cleaned up depending on runtime shutdown behavior.

**Leak per toggle cycle:**
- 1 OS thread (never exits)
- 1 Tokio Runtime (inside WatcherBridge, dropped but thread persists)
- 1 `notify::RecommendedWatcher` instance (held alive by the OS thread)
- 2 async tasks (debouncer + forwarding)
- 2 `mpsc::channel` instances

**Reproduction:**
1. Select a config file
2. Click "Watch: ON"
3. Click "Watch: OFF"
4. Repeat 10 times
5. Observe: 10 zombie threads consuming memory

---

## Issue #2: Auto Health Check Thread Never Stops (CRITICAL)

**Files:**
- `src/app_impl/health_ops.rs:156-202`

**Problem:**

The auto health check spawns a background thread with an infinite loop:

```rust
// health_ops.rs:156-188
let handle = std::thread::spawn(move || {
    let validator = ProxyValidator::new(10);
    loop {
        for (proxy_id, host, port, ...) in &proxy_list {
            let result = validator.validate(host, *port, ...);
            if tx.send((proxy_id.clone(), result)).is_err() {
                return;  // Only exit: when channel is closed
            }
        }
        thread::sleep(Duration::from_secs(interval_secs)); // 5 min default!
    }
});
drop(handle);  // JoinHandle dropped, thread detached
```

The thread can only exit when `tx.send()` returns `Err`, which happens after the receiver is dropped. However, if the thread is in `thread::sleep(300 seconds)`, it won't attempt `send()` until the sleep finishes.

When toggling off (`health_ops.rs:104-108`):
```rust
self.state.auto_checking = false;
self.state.health_check_rx = None;  // Drops receiver
```

Dropping the receiver closes the channel, but **the thread won't check until after sleep(300s)**.

**Race condition:** If user toggles Auto Check ON → OFF → ON within the sleep window, two threads will run concurrently, both sending to different channels.

**Leak per toggle cycle:**
- 1 OS thread (stuck in sleep for up to 5 minutes)
- 1 `ProxyValidator` instance with TCP connections
- 1 `mpsc::Sender` (held by the thread)
- Cloned proxy list data

---

## Issue #3: Multiple Tokio Runtimes (MEDIUM)

**Files:**
- `src/bridge/config_bridge.rs:22-23`
- `src/bridge/health_bridge.rs:24`
- `src/bridge/watcher_bridge.rs:23`
- `src/state/proxy_state.rs:55-71`

**Problem:**

Each Bridge creates its own Tokio runtime:

```rust
// config_bridge.rs
let runtime = Runtime::new()?;    // Runtime #1

// health_bridge.rs
let runtime = Runtime::new()?;    // Runtime #2

// watcher_bridge.rs (on each Watch toggle)
let runtime = Runtime::new()?;    // Runtime #3, #4, #5...
```

On `ProxyState::initialize()`, 3 runtimes are created immediately. Each Tokio runtime spawns worker threads (default = number of CPU cores). On an 8-core Mac, that's **24 threads** just for the runtime pools.

Every Watch toggle creates an additional runtime. These runtimes are never shared.

**Impact:**
- Base: 3 runtimes × 8 worker threads = 24 threads at startup
- Each Watch toggle: +1 runtime × 8 worker threads = +8 threads
- Memory overhead: ~2-4 MB per runtime

---

## Issue #4: Logs Vec Unbounded Growth + Quadratic Join (MEDIUM)

**Files:**
- `src/app.rs:717` (definition)
- `src/app_impl/ui_helpers.rs:13-17` (add_log)

**Problem:**

```rust
// ui_helpers.rs:13-17
pub(crate) fn add_log(&mut self, cx: &mut Cx, message: &str) {
    self.state.logs.push(message.to_string());  // Unbounded push
    let log_text = self.state.logs.join("\n");   // Full join every time!
    self.ui.label(id!(log_text)).set_text(cx, &log_text);
}
```

Two issues:
1. `logs: Vec<String>` has no maximum size. With Auto Check and Watch active, logs accumulate indefinitely.
2. Every `add_log` call does a full `.join("\n")` — this is **O(n)** where n is total log content size. As logs grow, each add becomes more expensive and allocates a larger temporary string.

While `clear_logs()` is called in many places, during continuous monitoring (Watch + Auto Check), logs can grow between clears.

**Worst case scenario:**
- Auto Check runs every 5 minutes, checking 10 proxies
- Each check adds ~3 log lines
- Watch triggers on every file change, adding ~4 log lines
- After 24 hours without manual interaction: thousands of log entries

---

## Issue #5: Unbounded Channel for Watcher Events (MEDIUM)

**Files:**
- `src/bridge/watcher_bridge.rs:40`

**Problem:**

```rust
let (tx, rx) = mpsc::unbounded_channel();
```

The watcher event channel is unbounded. If the file system generates rapid events (e.g., editor auto-save, rapid writes), events queue up without backpressure.

The consumer in `app.rs:912-917` drains all events per frame:
```rust
while let Ok(event) = rx.try_recv() {
    watcher_events.push(event);
}
```

However, each event triggers `handle_watcher_event` which calls `merge_to_clash()` — a potentially slow file I/O + YAML parsing operation. During that time, more events queue up.

---

## Issue #6: Temporary Vec Allocation in Hot Path (LOW)

**Files:**
- `src/app.rs:899-916`

**Problem:**

`handle_event` is called **every frame** by Makepad. Each invocation allocates two temporary Vecs:

```rust
let mut results = Vec::new();        // Allocated every frame
let mut watcher_events = Vec::new(); // Allocated every frame
```

At 60fps, that's 120 small heap allocations per second. While individually trivial, this adds unnecessary GC pressure.

---

## Memory Leak Flow Diagram

```
User clicks "Watch: ON"
  └─> WatcherBridge::new() → creates Tokio Runtime #N
  └─> WatcherBridge::start()
        └─> ClashConfigWatcher::start()
              └─> std::thread::spawn(infinite_loop)  ← LEAKED
              └─> tokio::spawn(debouncer_task)
        └─> runtime.spawn(forwarding_task)

User clicks "Watch: OFF"
  └─> WatcherBridge::stop() → drops tx sender
  └─> WatcherBridge dropped → Runtime dropped
  └─> BUT: OS thread still running loop { sleep(1s) }
  └─> BUT: notify::Watcher still alive inside thread

User clicks "Watch: ON" again
  └─> Creates ANOTHER WatcherBridge + Runtime + Thread
  └─> Previous thread STILL running ← ACCUMULATING
```

```
User clicks "Auto: ON"
  └─> std::thread::spawn(health_check_loop)
        └─> loop { check_all(); sleep(300s); }

User clicks "Auto: OFF"
  └─> Drops health_check_rx (receiver)
  └─> Thread in sleep(300s) ← CAN'T EXIT FOR UP TO 5 MINUTES
  └─> After 5 min: tx.send() fails → thread exits (finally)

User clicks "Auto: ON" before old thread exits
  └─> NOW TWO health check threads running
  └─> Old one will exit eventually, new one keeps going
  └─> Memory: 2× proxy list + 2× validator instances
```

---

## Summary Table

| # | Issue | Severity | Root Cause | Leak Rate |
|---|-------|----------|------------|-----------|
| 1 | Watcher thread never stops | CRITICAL | Infinite loop, no cancellation | 1 thread + 1 runtime per toggle |
| 2 | Health check thread never stops | CRITICAL | sleep() blocks cancellation | 1 thread per toggle (delayed cleanup) |
| 3 | Multiple Tokio runtimes | MEDIUM | No shared runtime | 24+ threads at startup |
| 4 | Unbounded logs + O(n) join | MEDIUM | No size limit | Linear growth |
| 5 | Unbounded watcher channel | MEDIUM | No backpressure | Burst growth |
| 6 | Per-frame Vec allocations | LOW | Hot path allocation | Constant churn |
