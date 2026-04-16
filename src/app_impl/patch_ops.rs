//! Patch Operations (Preview & Apply)
//!
//! Methods for:
//! - Preview patch changes
//! - Apply patch to Clash config
//! - Handle apply results

use makepad_widgets::*;
use clash_chain_patcher::patcher::{self, PatchOptions};
use clash_chain_patcher::merger::MergerConfig;
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
            None => { self.add_log(cx, "Select file first"); self.update_log_display(cx); self.ui.redraw(cx); return; }
        };
        let proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => { self.add_log(cx, "Fill proxy info"); self.update_log_display(cx); self.ui.redraw(cx); return; }
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
        self.update_log_display(cx);
        self.ui.redraw(cx);
    }

    /// Apply patch to Clash config (async, non-blocking)
    pub(crate) fn apply_patch(&mut self, cx: &mut Cx) {
        // Check if already applying
        if self.state.is_applying {
            self.add_log(cx, "⚠ Apply is already in progress");
            self.update_log_display(cx);
            self.ui.redraw(cx);
            return;
        }

        // Check if config file is selected
        if self.state.config_filename.is_none() {
            self.clear_logs(cx);
            self.add_log(cx, "✗ Select Clash config file first");
            self.set_status(cx, "No config");
            self.update_log_display(cx);
            self.ui.redraw(cx);
            return;
        }

        // Check ProxyState
        let Some(state) = &self.state.proxy_state else {
            self.clear_logs(cx);
            self.add_log(cx, "✗ ProxyState not initialized");
            self.set_status(cx, "Error");
            self.update_log_display(cx);
            self.ui.redraw(cx);
            return;
        };

        // Use the proxy currently shown in the form (user's selection)
        let form_proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => {
                self.clear_logs(cx);
                self.add_log(cx, "✗ Fill proxy info in the form first");
                self.set_status(cx, "No proxy");
                self.update_log_display(cx);
                self.ui.redraw(cx);
                return;
            }
        };

        let proxy_host = form_proxy.host.clone();
        let proxy_port = form_proxy.port;
        let proxy_username = form_proxy.username.clone();
        let proxy_password = form_proxy.password.clone();

        // Extract config path
        let config_path = state.clash_config_path()
            .map(|p| p.to_path_buf());

        let Some(config_path) = config_path else {
            self.clear_logs(cx);
            self.add_log(cx, "✗ Clash config path not set");
            self.set_status(cx, "Error");
            self.update_log_display(cx);
            self.ui.redraw(cx);
            return;
        };

        // Create channel for result
        let (tx, rx) = std::sync::mpsc::channel();
        self.state.apply_result_rx = Some(rx);
        self.state.is_applying = true;

        // Display progress
        self.clear_logs(cx);
        self.add_log(cx, "⏳ Applying configuration...");
        self.add_log(cx, &format!("  Using proxy: {}:{}", proxy_host, proxy_port));
        self.add_log(cx, &format!("  Proxy from form: {}:{}", proxy_host, proxy_port));
        self.set_status(cx, "Applying...");
        self.update_log_display(cx);

        // Build rule replacements from UI state
        let rule_replacements = self.build_rule_replacements();
        let has_rule_rewrites = !rule_replacements.is_empty();

        // Build custom rules from UI state
        let custom_rules = self.build_custom_rules();
        let has_custom_rules = !custom_rules.is_empty();

        // Clone data for thread
        let proxy_host_clone = proxy_host.clone();
        let proxy_username_clone = proxy_username.clone().filter(|s| !s.is_empty());
        let proxy_password_clone = proxy_password.clone().filter(|s| !s.is_empty());

        if has_rule_rewrites {
            self.add_log(cx, &format!("  Rules rewrite: {} groups to replace", rule_replacements.len()));
            self.update_log_display(cx);
        }
        if has_custom_rules {
            self.add_log(cx, &format!("  Custom rules: {} to inject", custom_rules.len()));
            self.update_log_display(cx);
        }

        // Spawn background thread
        std::thread::spawn(move || {
            use clash_chain_patcher::bridge::MergerBridge;

            let result = (|| -> Result<ApplyResult, String> {
                // Create MergerConfig with actual proxy from pool
                let merger_config = MergerConfig {
                    proxy_name: "Local-Chain-Proxy".to_string(),
                    proxy_host: proxy_host_clone.clone(),
                    proxy_port,
                    proxy_username: proxy_username_clone.clone(),
                    proxy_password: proxy_password_clone.clone(),
                    create_backup: true,
                    insert_at_beginning: true,
                    chain_suffix: "-Chain".to_string(),
                };

                // Create MergerBridge with custom config
                let merger = MergerBridge::with_config(merger_config);

                // Execute merge
                match merger.merge(&config_path) {
                    Ok(merge_result) => {
                        let mut details = Vec::new();
                        details.push(format!("SOCKS5 Proxy: {}:{}", proxy_host_clone, proxy_port));
                        if let Some(ref username) = proxy_username_clone {
                            details.push(format!("  User: {}", username));
                        }
                        details.push("".to_string());
                        details.push(format!("Local proxy added: {}", merge_result.proxy_added));
                        details.push(format!("Chain relays created: {}", merge_result.chains_created));
                        details.push(format!("Groups updated: {}", merge_result.groups_updated));

                        if let Some(backup_path) = merge_result.backup_path {
                            details.push(format!("Backup: {}", backup_path.display()));
                        }

                        // Apply rules rewrite if any (text-based, format-preserving)
                        if !rule_replacements.is_empty() {
                            match std::fs::read_to_string(&config_path) {
                                Ok(content) => {
                                    let (output, count) = clash_chain_patcher::patcher::rewrite_rules_text(&content, &rule_replacements);
                                    if count > 0 {
                                        if let Err(e) = std::fs::write(&config_path, output) {
                                            details.push(format!("✗ Rules rewrite write failed: {}", e));
                                        } else {
                                            details.push(format!("Rules rewritten: {} rules updated", count));
                                        }
                                    }
                                }
                                Err(e) => {
                                    details.push(format!("✗ Rules rewrite read failed: {}", e));
                                }
                            }
                        }

                        // Inject custom rules if any (Step 3)
                        if !custom_rules.is_empty() {
                            // Log each custom rule
                            for r in &custom_rules {
                                if r.enabled {
                                    details.push(format!("  + {},{},{}", r.match_type.clash_prefix(), r.domain, r.target_group));
                                }
                            }
                            match std::fs::read_to_string(&config_path) {
                                Ok(content) => {
                                    let (output, count) = clash_chain_patcher::patcher::inject_custom_rules_text(&content, &custom_rules);
                                    if count > 0 {
                                        if let Err(e) = std::fs::write(&config_path, output) {
                                            details.push(format!("✗ Custom rules write failed: {}", e));
                                        } else {
                                            details.push(format!("Custom rules injected: {} rules", count));
                                        }
                                    }
                                }
                                Err(e) => {
                                    details.push(format!("✗ Custom rules read failed: {}", e));
                                }
                            }
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
            self.add_log(cx, "Next steps:");
            self.add_log(cx, "1. Refresh Clash configuration");
            self.add_log(cx, "2. Select a '-Chain' node (e.g., 香港 01-Chain)");
            self.add_log(cx, "3. Traffic will flow: VPN Node → Your SOCKS5 Proxy");
            self.set_status(cx, "Done");
        } else {
            self.set_status(cx, "Failed");
        }

        self.update_log_display(cx);
        self.ui.redraw(cx);
    }
}
