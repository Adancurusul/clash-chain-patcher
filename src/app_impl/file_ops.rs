//! File & Configuration Management
//!
//! Methods for:
//! - Selecting and loading Clash config files
//! - Managing recent files history
//! - File watching for external changes

use makepad_widgets::*;
use crate::app::App;

impl App {
    /// Select a Clash configuration file via file dialog
    pub(crate) fn select_config_file(&mut self, cx: &mut Cx) {
        use rfd::FileDialog;
        let file = FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            self.load_config_file(cx, path_str);
        }
    }

    /// Load a configuration file from the given path
    pub(crate) fn load_config_file(&mut self, cx: &mut Cx, path_str: String) {
        let path = std::path::Path::new(&path_str);

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let filename = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                self.state.config_content = Some(content);
                self.state.config_filename = Some(filename.clone());

                // Add to recent files (if not already there)
                self.add_to_recent_files(cx, path_str.clone());

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

    /// Clear the currently loaded configuration file
    pub(crate) fn clear_config_file(&mut self, cx: &mut Cx) {
        self.state.config_content = None;
        self.state.config_filename = None;

        // Clear Clash config path from ProxyState
        if let Some(_state) = &mut self.state.proxy_state {
            // Note: ProxyState doesn't have a clear method, but we can just not use it
            self.clear_logs(cx);
            self.add_log(cx, "✓ Cleared Clash config selection");
        }

        self.ui.label(id!(file_label)).set_text(cx, "No file");
        self.set_status(cx, "Ready");
        self.ui.redraw(cx);
    }

    /// Toggle file watching on/off
    pub(crate) fn toggle_watch(&mut self, cx: &mut Cx) {
        eprintln!("DEBUG: toggle_watch called, current watching = {}", self.state.watching);
        self.state.watching = !self.state.watching;

        let button_text = if self.state.watching {
            "Watch: ON"
        } else {
            "Watch: OFF"
        };

        eprintln!("DEBUG: Setting button text to: {}", button_text);
        self.ui.button(id!(watch_toggle_btn)).set_text(cx, button_text);

        self.clear_logs(cx);
        if self.state.watching {
            // Start file watcher - extract config_path first to avoid borrow conflict
            let config_path_opt = self.state.proxy_state
                .as_ref()
                .and_then(|state| state.clash_config_path())
                .map(|p| p.to_path_buf());

            if let Some(config_path) = config_path_opt {
                use clash_chain_patcher::bridge::WatcherBridge;

                match WatcherBridge::new(&config_path) {
                    Ok(mut bridge) => {
                        match bridge.start() {
                            Ok(rx) => {
                                self.state.watcher_rx = Some(rx);
                                self.state.watcher_bridge = Some(bridge);
                                self.add_log(cx, "✓ File watching enabled");
                                self.add_log(cx, &format!("  Monitoring: {}", config_path.display()));
                                self.add_log(cx, "  Will auto re-apply on external changes");
                                eprintln!("DEBUG: File watcher started successfully");
                            }
                            Err(e) => {
                                self.state.watching = false;
                                self.ui.button(id!(watch_toggle_btn)).set_text(cx, "Watch: OFF");
                                self.add_log(cx, &format!("✗ Failed to start watcher: {}", e));
                                eprintln!("ERROR: Failed to start watcher: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.state.watching = false;
                        self.ui.button(id!(watch_toggle_btn)).set_text(cx, "Watch: OFF");
                        self.add_log(cx, &format!("✗ Failed to create watcher: {}", e));
                        eprintln!("ERROR: Failed to create watcher: {}", e);
                    }
                }
            } else {
                self.state.watching = false;
                self.ui.button(id!(watch_toggle_btn)).set_text(cx, "Watch: OFF");
                self.add_log(cx, "✗ No Clash config file selected");
                self.add_log(cx, "  Select a file first, then enable Watch");
            }
        } else {
            // Stop file watcher
            if let Some(mut bridge) = self.state.watcher_bridge.take() {
                bridge.stop();
                self.state.watcher_rx = None;
                self.add_log(cx, "File watching disabled");
                eprintln!("DEBUG: File watcher stopped");
            }
        }

        self.ui.redraw(cx);
    }

    /// Toggle file history dropdown visibility
    pub(crate) fn toggle_file_history(&mut self, cx: &mut Cx) {
        eprintln!("DEBUG: toggle_file_history called, current state = {}", self.state.show_file_history);
        self.state.show_file_history = !self.state.show_file_history;

        // Toggle visibility
        self.ui.view(id!(file_history_view)).set_visible(cx, self.state.show_file_history);

        // Update button text
        let button_text = if self.state.show_file_history { "▲" } else { "▼" };
        self.ui.button(id!(toggle_history_btn)).set_text(cx, button_text);

        // Show feedback in logs
        if self.state.show_file_history {
            eprintln!("DEBUG: Showing file history, {} recent files", self.state.recent_files.len());
            if self.state.recent_files.is_empty() {
                self.clear_logs(cx);
                self.add_log(cx, "No recent files yet");
                self.add_log(cx, "Select a Clash config file to add it to history");
            }
        } else {
            eprintln!("DEBUG: Hiding file history");
        }

        self.ui.redraw(cx);
    }

    /// Add a file path to recent files list
    pub(crate) fn add_to_recent_files(&mut self, cx: &mut Cx, path: String) {
        // Save to persistent config through ProxyState
        if let Some(state) = &mut self.state.proxy_state {
            if let Err(e) = state.add_recent_file(path.clone()) {
                eprintln!("Failed to save recent file: {}", e);
            } else {
                // Update in-memory list from saved config
                self.state.recent_files = state.get_recent_files();
                eprintln!("DEBUG: Saved recent file, now have {} files", self.state.recent_files.len());
            }
        }

        // Refresh display
        self.refresh_file_history_display(cx);
    }

    /// Refresh the file history dropdown display
    pub(crate) fn refresh_file_history_display(&mut self, cx: &mut Cx) {
        // Update recent file buttons
        for i in 0..3 {
            let (btn_id, visible, text) = match i {
                0 => (id!(recent_file_1), !self.state.recent_files.is_empty(), self.state.recent_files.get(0)),
                1 => (id!(recent_file_2), self.state.recent_files.len() > 1, self.state.recent_files.get(1)),
                2 => (id!(recent_file_3), self.state.recent_files.len() > 2, self.state.recent_files.get(2)),
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

    /// Select a file from recent files list
    pub(crate) fn select_recent_file(&mut self, cx: &mut Cx, index: usize) {
        if let Some(path) = self.state.recent_files.get(index).cloned() {
            self.load_config_file(cx, path);
            // Hide history after selection
            self.state.show_file_history = false;
            self.ui.view(id!(file_history_view)).set_visible(cx, false);
            self.ui.button(id!(toggle_history_btn)).set_text(cx, "▼");
        }
    }

    /// Handle file watcher events
    pub(crate) fn handle_watcher_event(&mut self, cx: &mut Cx, event: clash_chain_patcher::watcher::WatcherEvent) {
        use clash_chain_patcher::watcher::WatcherEvent;

        match event {
            WatcherEvent::ConfigModified(path) | WatcherEvent::ConfigCreated(path) => {
                eprintln!("DEBUG: Config file modified: {}", path.display());
                self.add_log(cx, "⚠ Clash config file was modified externally");
                self.add_log(cx, &format!("  File: {}", path.display()));
                self.add_log(cx, "  Re-applying Local-Chain-Proxy...");

                // Re-apply the configuration
                if let Some(state) = &mut self.state.proxy_state {
                    match state.merge_to_clash() {
                        Ok(_) => {
                            self.add_log(cx, "✓ Auto re-applied successfully");
                            self.add_log(cx, "  Local-Chain-Proxy restored");
                        }
                        Err(e) => {
                            self.add_log(cx, &format!("✗ Auto re-apply failed: {}", e));
                        }
                    }
                } else {
                    self.add_log(cx, "✗ ProxyState not initialized");
                }

                self.ui.redraw(cx);
            }
            WatcherEvent::Error(error) => {
                eprintln!("ERROR: File watcher error: {}", error);
                self.add_log(cx, &format!("✗ Watcher error: {}", error));
                self.ui.redraw(cx);
            }
        }
    }
}
