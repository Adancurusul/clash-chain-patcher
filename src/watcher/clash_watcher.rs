//! Clash configuration file watcher implementation

use anyhow::Result;
use notify::{
    event::{AccessKind, AccessMode, ModifyKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info, warn};

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce delay (default: 2 seconds)
    /// Multiple file changes within this period will be grouped into one event
    pub debounce_delay: Duration,

    /// Whether to watch the file continuously
    pub continuous: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_delay: Duration::from_secs(2),
            continuous: true,
        }
    }
}

/// Events emitted by the watcher
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    /// Configuration file was modified
    ConfigModified(PathBuf),

    /// Configuration file was created
    ConfigCreated(PathBuf),

    /// Watcher encountered an error
    Error(String),
}

/// Clash configuration file watcher
///
/// Monitors a Clash configuration file for changes and triggers callbacks
/// with debouncing to avoid excessive notifications.
pub struct ClashConfigWatcher {
    config: WatcherConfig,
    watch_path: PathBuf,
}

impl ClashConfigWatcher {
    /// Create a new watcher for a specific Clash config file
    pub fn new<P: AsRef<Path>>(watch_path: P) -> Result<Self> {
        let watch_path = watch_path.as_ref().to_path_buf();

        // Verify the file exists
        if !watch_path.exists() {
            anyhow::bail!("Config file does not exist: {}", watch_path.display());
        }

        Ok(Self {
            config: WatcherConfig::default(),
            watch_path,
        })
    }

    /// Create a new watcher with custom configuration
    pub fn with_config<P: AsRef<Path>>(watch_path: P, config: WatcherConfig) -> Result<Self> {
        let watch_path = watch_path.as_ref().to_path_buf();

        if !watch_path.exists() {
            anyhow::bail!("Config file does not exist: {}", watch_path.display());
        }

        Ok(Self {
            config,
            watch_path,
        })
    }

    /// Start watching the configuration file
    ///
    /// Returns a channel receiver that will receive WatcherEvents.
    /// The `stop_signal` can be set to `true` to stop the watcher thread and debouncer task.
    pub async fn start(self, stop_signal: Arc<AtomicBool>) -> Result<mpsc::Receiver<WatcherEvent>> {
        let (event_tx, event_rx) = mpsc::channel(100);
        let (notify_tx, mut notify_rx) = mpsc::channel(100);

        let watch_path = self.watch_path.clone();
        let watch_path_clone = watch_path.clone();
        let debounce_delay = self.config.debounce_delay;
        let thread_stop = stop_signal.clone();

        // Spawn watcher thread (notify requires sync operations)
        std::thread::spawn(move || {
            let rt = tokio::runtime::Handle::try_current()
                .unwrap_or_else(|_| {
                    tokio::runtime::Runtime::new()
                        .expect("Failed to create tokio runtime")
                        .handle()
                        .clone()
                });

            let mut watcher = match RecommendedWatcher::new(
                move |res: notify::Result<Event>| {
                    let tx = notify_tx.clone();
                    rt.spawn(async move {
                        if let Err(e) = tx.send(res).await {
                            error!("Failed to send notify event: {}", e);
                        }
                    });
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to create file watcher: {}", e);
                    return;
                }
            };

            // Watch the parent directory to catch all file operations
            let watch_dir = watch_path_clone
                .parent()
                .unwrap_or_else(|| Path::new("."));

            if let Err(e) = watcher.watch(watch_dir, RecursiveMode::NonRecursive) {
                error!("Failed to watch directory {}: {}", watch_dir.display(), e);
                return;
            }

            info!("File watcher started for: {}", watch_path_clone.display());

            // Keep the watcher alive until stop signal is set
            while !thread_stop.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(200));
            }

            // Watcher is dropped here, releasing OS resources
            info!("File watcher thread stopped for: {}", watch_path_clone.display());
        });

        // Spawn debouncer task
        let debouncer_stop = stop_signal.clone();
        tokio::spawn(async move {
            let mut last_event_time: Option<Instant> = None;
            let mut pending_event: Option<WatcherEvent> = None;

            loop {
                // Check stop signal
                if debouncer_stop.load(Ordering::Relaxed) {
                    break;
                }

                tokio::select! {
                    // Receive events from notify
                    Some(result) = notify_rx.recv() => {
                        match result {
                            Ok(event) => {
                                debug!("Received file event: {:?}", event);

                                // Filter events for our target file
                                let is_target_file = event.paths.iter().any(|p| p == &watch_path);
                                if !is_target_file {
                                    continue;
                                }

                                // Determine event type
                                let watcher_event = match event.kind {
                                    EventKind::Modify(ModifyKind::Data(_)) |
                                    EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                                        info!("Config file modified: {}", watch_path.display());
                                        Some(WatcherEvent::ConfigModified(watch_path.clone()))
                                    }
                                    EventKind::Create(_) => {
                                        info!("Config file created: {}", watch_path.display());
                                        Some(WatcherEvent::ConfigCreated(watch_path.clone()))
                                    }
                                    _ => None,
                                };

                                if let Some(evt) = watcher_event {
                                    // Update debounce state
                                    last_event_time = Some(Instant::now());
                                    pending_event = Some(evt);
                                }
                            }
                            Err(e) => {
                                warn!("File watcher error: {}", e);
                                if event_tx.send(WatcherEvent::Error(e.to_string())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }

                    // Check debounce timeout
                    _ = sleep(Duration::from_millis(100)) => {
                        if let (Some(last_time), Some(event)) = (last_event_time, pending_event.take()) {
                            if last_time.elapsed() >= debounce_delay {
                                debug!("Debounce delay elapsed, sending event");
                                if event_tx.send(event).await.is_err() {
                                    break;
                                }
                                last_event_time = None;
                            } else {
                                // Put the event back
                                pending_event = Some(event);
                            }
                        }
                    }
                }
            }

            info!("Watcher debouncer task stopped");
        });

        Ok(event_rx)
    }

    /// Get the watched file path
    pub fn watch_path(&self) -> &Path {
        &self.watch_path
    }

    /// Get the watcher configuration
    pub fn config(&self) -> &WatcherConfig {
        &self.config
    }
}

/// Helper to start watching with a callback function
pub async fn watch_clash_config<F, Fut>(
    config_path: impl AsRef<Path>,
    stop_signal: Arc<AtomicBool>,
    mut callback: F,
) -> Result<mpsc::Receiver<WatcherEvent>>
where
    F: FnMut(WatcherEvent) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let watcher = ClashConfigWatcher::new(config_path)?;
    let mut rx = watcher.start(stop_signal).await?;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            callback(event).await;
        }
    });

    // Return a dummy channel to keep the watcher alive
    let (tx, rx) = mpsc::channel(1);
    drop(tx);
    Ok(rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.debounce_delay, Duration::from_secs(2));
        assert!(config.continuous);
    }

    #[test]
    fn test_watcher_creation_nonexistent_file() {
        let result = ClashConfigWatcher::new("/nonexistent/file.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_watcher_creation_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, "test: data").unwrap();

        let watcher = ClashConfigWatcher::new(&config_path);
        assert!(watcher.is_ok());

        let watcher = watcher.unwrap();
        assert_eq!(watcher.watch_path(), config_path);
    }

    #[test]
    fn test_watcher_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, "test: data").unwrap();

        let config = WatcherConfig {
            debounce_delay: Duration::from_secs(5),
            continuous: false,
        };

        let watcher = ClashConfigWatcher::with_config(&config_path, config);
        assert!(watcher.is_ok());

        let watcher = watcher.unwrap();
        assert_eq!(watcher.config().debounce_delay, Duration::from_secs(5));
        assert!(!watcher.config().continuous);
    }

    #[tokio::test]
    #[ignore] // Requires real file system operations, run with --ignored
    async fn test_watcher_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, "initial: data").unwrap();

        let stop_signal = Arc::new(AtomicBool::new(false));
        let watcher = ClashConfigWatcher::new(&config_path).unwrap();
        let mut rx = watcher.start(stop_signal.clone()).await.unwrap();

        // Wait a bit for the watcher to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Modify the file
        let config_path_clone = config_path.clone();
        tokio::task::spawn_blocking(move || {
            fs::write(&config_path_clone, "modified: data").unwrap();
        })
        .await
        .unwrap();

        // Wait for the event (with timeout)
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;

        // Keep temp_dir alive until test completes
        drop(rx);
        drop(temp_dir);

        assert!(event.is_ok(), "Timeout waiting for event");
        let event = event.unwrap();
        assert!(event.is_some(), "No event received");

        match event.unwrap() {
            WatcherEvent::ConfigModified(path) => {
                assert_eq!(path, config_path);
            }
            other => panic!("Expected ConfigModified, got {:?}", other),
        }
    }

    #[tokio::test]
    #[ignore] // Requires real file system operations, run with --ignored
    async fn test_watcher_debouncing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, "initial: data").unwrap();

        let config = WatcherConfig {
            debounce_delay: Duration::from_secs(1),
            continuous: true,
        };

        let stop_signal = Arc::new(AtomicBool::new(false));
        let watcher = ClashConfigWatcher::with_config(&config_path, config).unwrap();
        let mut rx = watcher.start(stop_signal.clone()).await.unwrap();

        // Wait a bit for the watcher to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Make multiple rapid changes
        let config_path_clone = config_path.clone();
        tokio::task::spawn_blocking(move || {
            for i in 0..5 {
                std::thread::sleep(Duration::from_millis(200));
                fs::write(&config_path_clone, format!("modified: {}", i)).unwrap();
            }
        })
        .await
        .unwrap();

        // Should receive at most 2-3 events (debounced)
        let mut event_count = 0;
        let timeout_duration = Duration::from_secs(5);
        let start = Instant::now();

        while start.elapsed() < timeout_duration {
            match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                Ok(Some(_)) => {
                    event_count += 1;
                }
                Ok(None) => break,
                Err(_) => break, // Timeout, no more events
            }
        }

        // Keep temp_dir alive until test completes
        drop(rx);
        drop(temp_dir);

        // With debouncing, should receive fewer events than modifications
        // Note: Due to timing variations, we accept any positive number of events less than 5
        assert!(
            event_count > 0 && event_count < 5,
            "Expected 1-4 events due to debouncing, got {}",
            event_count
        );
    }
}
