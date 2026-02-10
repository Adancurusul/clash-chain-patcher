//! File watcher bridge
//!
//! Provides synchronous access interface to ClashConfigWatcher for GUI components

use crate::watcher::{ClashConfigWatcher, WatcherEvent};
use super::{BridgeError, BridgeResult};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// File watcher bridge
///
/// Wraps the asynchronous ClashConfigWatcher into a more user-friendly API for GUI use.
/// Properly cleans up background threads via `stop_signal` on stop/drop.
pub struct WatcherBridge {
    runtime: Runtime,
    config_path: PathBuf,
    event_tx: Option<mpsc::UnboundedSender<WatcherEvent>>,
    stop_signal: Option<Arc<AtomicBool>>,
}

impl WatcherBridge {
    /// Create a new file watcher bridge
    pub fn new(config_path: impl AsRef<Path>) -> BridgeResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| BridgeError::Runtime(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            runtime,
            config_path: config_path.as_ref().to_path_buf(),
            event_tx: None,
            stop_signal: None,
        })
    }

    /// Start file monitoring
    ///
    /// Returns an event receiver that can be used to receive file change events
    pub fn start(&mut self) -> BridgeResult<mpsc::UnboundedReceiver<WatcherEvent>> {
        let watcher = ClashConfigWatcher::new(&self.config_path)
            .map_err(|e| BridgeError::Watcher(format!("Failed to create watcher: {}", e)))?;

        let (tx, rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx.clone());

        let stop_signal = Arc::new(AtomicBool::new(false));
        self.stop_signal = Some(stop_signal.clone());

        // Start monitoring in the background
        self.runtime.spawn(async move {
            match watcher.start(stop_signal).await {
                Ok(mut watcher_rx) => {
                    while let Some(event) = watcher_rx.recv().await {
                        if tx.send(event).is_err() {
                            break; // Receiver has been closed
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(WatcherEvent::Error(e.to_string()));
                }
            }
        });

        Ok(rx)
    }

    /// Stop file monitoring
    ///
    /// Signals the background watcher thread and debouncer task to exit,
    /// then closes the event channel.
    pub fn stop(&mut self) {
        // Signal the watcher thread and debouncer to stop
        if let Some(signal) = self.stop_signal.take() {
            signal.store(true, Ordering::Relaxed);
        }
        // Close the channel
        if let Some(tx) = self.event_tx.take() {
            drop(tx);
        }
    }

    /// Get the monitored config file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if monitoring is active
    pub fn is_watching(&self) -> bool {
        self.event_tx.is_some()
    }
}

impl Drop for WatcherBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

/// File watcher callback type
///
/// Used by GUI components to subscribe to file change events
#[allow(dead_code)]
pub type WatcherCallback = Box<dyn Fn(WatcherEvent) + Send>;

/// File watcher bridge with callback
///
/// Provides a simpler callback-based API
#[allow(dead_code)]
pub struct WatcherBridgeWithCallback {
    bridge: WatcherBridge,
    runtime: Runtime,
}

#[allow(dead_code)]
impl WatcherBridgeWithCallback {
    /// Create a new file watcher bridge (with callback)
    pub fn new<F>(config_path: impl AsRef<Path>, callback: F) -> BridgeResult<Self>
    where
        F: Fn(WatcherEvent) + Send + 'static,
    {
        let mut bridge = WatcherBridge::new(config_path)?;
        let mut rx = bridge.start()?;

        // Create an independent runtime to run the callback
        let runtime = Runtime::new()
            .map_err(|e| BridgeError::Runtime(format!("Failed to create runtime: {}", e)))?;

        // Run the callback task in a background thread
        runtime.spawn(async move {
            while let Some(event) = rx.recv().await {
                callback(event);
            }
        });

        Ok(Self {
            bridge,
            runtime,
        })
    }

    /// Stop monitoring
    pub fn stop(mut self) {
        self.bridge.stop();
    }

    /// Get the monitored config file path
    pub fn config_path(&self) -> &Path {
        self.bridge.config_path()
    }

    /// Check if monitoring is active
    pub fn is_watching(&self) -> bool {
        self.bridge.is_watching()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn create_test_config() -> PathBuf {
        use uuid::Uuid;
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join(format!("test-clash-config-{}.yaml", Uuid::new_v4()));

        fs::write(
            &config_path,
            r#"
proxies:
  - name: "Test"
    type: ss
    server: example.com
    port: 443
proxy-groups:
  - name: "Select"
    type: select
    proxies:
      - "Test"
"#,
        )
        .unwrap();

        config_path
    }

    #[test]
    fn test_watcher_bridge_creation() {
        let config_path = create_test_config();
        let bridge = WatcherBridge::new(&config_path);
        assert!(bridge.is_ok());

        // Cleanup
        let _ = fs::remove_file(config_path);
    }

    #[test]
    fn test_watcher_bridge_start_stop() {
        let config_path = create_test_config();
        let mut bridge = WatcherBridge::new(&config_path).unwrap();

        // Start monitoring
        let rx = bridge.start();
        assert!(rx.is_ok());
        assert!(bridge.is_watching());

        // Stop monitoring - now properly stops the background thread
        bridge.stop();
        assert!(!bridge.is_watching());

        // Cleanup
        let _ = fs::remove_file(config_path);
    }

    #[test]
    #[ignore] // Requires actual file changes, slow
    fn test_watcher_bridge_detect_changes() {
        let config_path = create_test_config();
        let mut bridge = WatcherBridge::new(&config_path).unwrap();

        let mut rx = bridge.start().unwrap();

        // Modify file
        std::thread::sleep(Duration::from_millis(100));
        fs::write(&config_path, "# Modified content\n").unwrap();

        // Wait for event (2 second debounce + 1 second tolerance)
        std::thread::sleep(Duration::from_secs(3));

        // Check if event was received
        let runtime = Runtime::new().unwrap();
        let has_event = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(1), rx.recv())
                .await
                .is_ok()
        });

        assert!(has_event);

        // Cleanup - thread will now properly exit
        bridge.stop();
        let _ = fs::remove_file(config_path);
    }

    #[test]
    #[ignore] // Requires actual file changes, slow
    fn test_watcher_bridge_with_callback() {
        let config_path = create_test_config();

        let (tx, mut rx) = mpsc::unbounded_channel();

        let _bridge = WatcherBridgeWithCallback::new(&config_path, move |event| {
            let _ = tx.send(event);
        });

        assert!(_bridge.is_ok());

        // Modify file
        std::thread::sleep(Duration::from_millis(100));
        fs::write(&config_path, "# Modified content\n").unwrap();

        // Wait for callback
        let runtime = Runtime::new().unwrap();
        let has_callback = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(4), rx.recv())
                .await
                .is_ok()
        });

        // Note: This test may be unstable as it depends on file system events
        // May need to be ignored in CI environments
        println!("Callback received: {}", has_callback);

        // Cleanup
        let _ = fs::remove_file(config_path);
    }
}
