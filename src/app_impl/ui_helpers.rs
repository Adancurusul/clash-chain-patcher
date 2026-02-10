//! UI & Logging Helpers
//!
//! Methods for:
//! - Log management (add, clear, update display)
//! - Status bar updates
//! - Output saving

use makepad_widgets::*;
use crate::app::{App, MAX_LOG_LINES};

impl App {
    /// Add a log message and update display.
    /// Automatically trims old entries when exceeding MAX_LOG_LINES.
    pub(crate) fn add_log(&mut self, cx: &mut Cx, message: &str) {
        self.state.logs.push_back(message.to_string());

        // Trim old logs to prevent unbounded growth
        while self.state.logs.len() > MAX_LOG_LINES {
            self.state.logs.pop_front();
        }

        // Build display string from VecDeque
        let log_text: String = self.state.logs.iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        self.ui.label(id!(log_text)).set_text(cx, &log_text);
    }

    /// Clear all logs and update display
    pub(crate) fn clear_logs(&mut self, cx: &mut Cx) {
        self.state.logs.clear();
        self.ui.label(id!(log_text)).set_text(cx, "");
    }

    /// Update the log display from the logs buffer (for manual refresh)
    pub(crate) fn update_log_display(&mut self, cx: &mut Cx) {
        let log_text: String = self.state.logs.iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        self.ui.label(id!(log_text)).set_text(cx, &log_text);
    }

    /// Set status bar text
    pub(crate) fn set_status(&mut self, cx: &mut Cx, status: &str) {
        self.ui.label(id!(status_label)).set_text(cx, status);
    }

    /// Save output to file
    pub(crate) fn save_output(&mut self, cx: &mut Cx) {
        let content = match &self.state.output_content {
            Some(c) => c.clone(),
            None => {
                self.add_log(cx, "No output to save");
                self.update_log_display(cx);
                self.ui.redraw(cx);
                return;
            }
        };

        // Generate output filename
        let output_filename = if let Some(config_name) = &self.state.config_filename {
            let base = config_name.trim_end_matches(".yaml").trim_end_matches(".yml");
            format!("{}_patched.yaml", base)
        } else {
            "patched_config.yaml".to_string()
        };

        // Get download directory
        let download_dir = dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let output_path = download_dir.join(&output_filename);

        match std::fs::write(&output_path, content) {
            Ok(_) => {
                self.add_log(cx, &format!("Saved: {}", output_path.display()));
                self.set_status(cx, "Saved");
            }
            Err(e) => {
                self.add_log(cx, &format!("Save failed: {}", e));
                self.set_status(cx, "Save failed");
            }
        }
        self.update_log_display(cx);
        self.ui.redraw(cx);
    }
}
