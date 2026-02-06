//! File watching module for Clash configuration monitoring

pub mod clash_watcher;

pub use clash_watcher::{ClashConfigWatcher, WatcherConfig, WatcherEvent};
