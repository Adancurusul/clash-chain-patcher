//! Patch Operations (Preview & Apply)
//!
//! Methods for:
//! - Preview patch changes
//! - Apply patch to Clash config
//! - Handle apply results

use makepad_widgets::*;
use clash_chain_patcher::patcher::{self, PatchOptions};
use crate::app::{App, ApplyResult};

impl App {
    /// Get patch options from form
    pub(crate) fn get_options_from_form(&self) -> PatchOptions {
        let filter_str = self.ui.text_input(id!(filter_input)).text();
        let filter_keywords: Vec<String> = if filter_str.is_empty() {
            vec![]
        } else {
            filter_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        };
        PatchOptions { filter_keywords }
    }

    /// Preview the patch without applying
    pub(crate) fn preview_patch(&mut self, cx: &mut Cx) {
        self.clear_logs(cx);
        let config = match &self.state.config_content {
            Some(c) => c.clone(),
            None => { self.add_log(cx, "Select file first"); self.ui.redraw(cx); return; }
        };
        let proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => { self.add_log(cx, "Fill proxy info"); self.ui.redraw(cx); return; }
        };
        let opts = self.get_options_from_form();
        let result = patcher::preview_patch(&config, &proxy, &opts);
        for log in &result.logs { self.add_log(cx, log); }
        if result.success {
            self.add_log(cx, "");
            for name in &result.relay_names {
                self.add_log(cx, &format!("  {}", name));
            }
            self.set_status(cx, "Preview OK");
        } else { self.set_status(cx, "Failed"); }
        self.ui.redraw(cx);
    }

    /// Apply patch to Clash config (async, non-blocking)
    pub(crate) fn apply_patch(&mut self, cx: &mut Cx) {
        // Check if already applying
        if self.state.is_applying {
            self.add_log(cx, "⚠ Apply is already in progress");
            return;
        }

        // Check if config file is selected
        if self.state.config_filename.is_none() {
            self.add_log(cx, "✗ Select file first");
            return;
        }

        // Check ProxyState
        let Some(state) = &self.state.proxy_state else {
            self.add_log(cx, "✗ ProxyState not initialized");
            return;
        };

        // Get enabled proxies
        let enabled_proxies: Vec<_> = state.list_upstreams()
            .into_iter()
            .filter(|p| p.enabled)
            .collect();

        if enabled_proxies.is_empty() {
            self.add_log(cx, "✗ No enabled proxies");
            self.add_log(cx, "  Add and enable at least 1 proxy");
            return;
        }

        // Extract config path
        let config_path = state.clash_config_path()
            .map(|p| p.to_path_buf());

        let Some(config_path) = config_path else {
            self.add_log(cx, "✗ Clash config path not set");
            return;
        };

        // Create channel for result
        let (tx, rx) = std::sync::mpsc::channel();
        self.state.apply_result_rx = Some(rx);
        self.state.is_applying = true;

        // Display progress
        self.clear_logs(cx);
        self.add_log(cx, "⏳ Applying configuration...");
        self.add_log(cx, &format!("  {} enabled proxies", enabled_proxies.len()));
        self.add_log(cx, "  (Non-blocking, UI remains responsive)");
        self.set_status(cx, "Applying...");

        // Spawn background thread
        std::thread::spawn(move || {
            use clash_chain_patcher::bridge::MergerBridge;

            let result = (|| -> Result<ApplyResult, String> {
                // Create MergerBridge
                let merger = MergerBridge::new();

                // Execute merge
                match merger.merge(&config_path) {
                    Ok(merge_result) => {
                        let mut details = Vec::new();
                        details.push("Using proxy pool mode".to_string());
                        details.push(format!("Enabled proxies: {}", enabled_proxies.len()));

                        for proxy in &enabled_proxies {
                            details.push(format!("  - {}", proxy.name));
                        }

                        details.push("".to_string());
                        details.push(format!("Proxy added: {}", merge_result.proxy_added));
                        details.push(format!("Groups updated: {}", merge_result.groups_updated));

                        if let Some(backup_path) = merge_result.backup_path {
                            details.push(format!("Backup: {}", backup_path.display()));
                        }

                        Ok(ApplyResult {
                            success: true,
                            message: "✓ Configuration applied successfully".to_string(),
                            details,
                        })
                    }
                    Err(e) => {
                        Ok(ApplyResult {
                            success: false,
                            message: format!("✗ Apply failed: {}", e),
                            details: vec![],
                        })
                    }
                }
            })();

            let apply_result = result.unwrap_or_else(|e| ApplyResult {
                success: false,
                message: format!("✗ Error: {}", e),
                details: vec![],
            });

            let _ = tx.send(apply_result);
        });

        self.ui.redraw(cx);
    }

    /// Handle apply result from background thread
    pub(crate) fn handle_apply_result(&mut self, cx: &mut Cx, result: ApplyResult) {
        eprintln!("DEBUG: Apply completed: success={}", result.success);

        self.clear_logs(cx);
        self.add_log(cx, &result.message);

        for detail in result.details {
            self.add_log(cx, &detail);
        }

        if result.success {
            self.add_log(cx, "");
            self.add_log(cx, "Local proxy: 127.0.0.1:10808");
            self.add_log(cx, "");
            self.add_log(cx, "Next steps:");
            self.add_log(cx, "1. Refresh Clash configuration");
            self.add_log(cx, "2. Select 'Local-Chain-Proxy' in Clash");
            self.add_log(cx, "3. Enable Watch to protect against subscription updates");
            self.set_status(cx, "Done");
        } else {
            self.set_status(cx, "Failed");
        }

        self.ui.redraw(cx);
    }
}
