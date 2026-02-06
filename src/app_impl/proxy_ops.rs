//! Proxy Management Operations
//!
//! Methods for:
//! - Adding/removing proxies from the pool
//! - Loading proxy info to form
//! - Parsing proxy strings
//! - Refreshing proxy list display

use makepad_widgets::*;
use clash_chain_patcher::patcher::{self, Socks5Proxy};
use clash_chain_patcher::config::UpstreamProxy;
use clash_chain_patcher::proxy::config::UpstreamConfig;
use clash_chain_patcher::state::ProxyState;
use crate::app::App;

impl App {
    /// Fill proxy form fields from proxy string input
    pub(crate) fn fill_proxy_fields(&mut self, cx: &mut Cx) {
        let proxy_string = self.ui.text_input(id!(proxy_string_input)).text();
        if proxy_string.is_empty() {
            self.add_log(cx, "Enter proxy string first");
            self.ui.redraw(cx);
            return;
        }

        if let Some(proxy) = patcher::parse_proxy_string(&proxy_string) {
            self.ui.text_input(id!(host_input)).set_text(cx, &proxy.host);
            self.ui.text_input(id!(port_input)).set_text(cx, &proxy.port.to_string());
            if let Some(u) = &proxy.username {
                self.ui.text_input(id!(username_input)).set_text(cx, u);
            }
            if let Some(p) = &proxy.password {
                self.ui.text_input(id!(password_input)).set_text(cx, p);
            }
            self.add_log(cx, "Parsed OK");
            self.ui.redraw(cx);
        } else {
            self.add_log(cx, "Invalid format");
            self.ui.redraw(cx);
        }
    }

    /// Get proxy info from form fields
    pub(crate) fn get_proxy_from_form(&self) -> Option<Socks5Proxy> {
        let host = self.ui.text_input(id!(host_input)).text();
        let port_str = self.ui.text_input(id!(port_input)).text();
        let username = self.ui.text_input(id!(username_input)).text();
        let password = self.ui.text_input(id!(password_input)).text();

        if host.is_empty() || port_str.is_empty() { return None; }
        let port = port_str.parse::<u16>().ok()?;

        Some(Socks5Proxy::new(
            host, port,
            if username.is_empty() { None } else { Some(username) },
            if password.is_empty() { None } else { Some(password) },
        ))
    }

    /// Initialize proxy state on startup
    pub(crate) fn init_proxy_state(&mut self, cx: &mut Cx) {
        let mut state = ProxyState::new();
        if let Err(e) = state.initialize() {
            self.add_log(cx, &format!("ProxyState init error: {}", e));
            return;
        }

        // Debug: Check how many proxies were loaded
        let proxy_count = state.list_upstreams().len();
        eprintln!("DEBUG: Loaded {} proxies from config", proxy_count);

        self.state.proxy_state = Some(state);

        // Load recent files from config
        if let Some(state) = &self.state.proxy_state {
            self.state.recent_files = state.get_recent_files();
            eprintln!("DEBUG: Loaded {} recent files from config", self.state.recent_files.len());
            self.refresh_file_history_display(cx);
        }

        // Refresh display will show proxy list (and clear logs to show current state)
        self.refresh_proxy_list_display(cx);

        // Add initialization message if no proxies exist
        if let Some(state) = &self.state.proxy_state {
            let proxies = state.list_upstreams();
            eprintln!("DEBUG: After setting state, proxy count = {}", proxies.len());
            if proxies.is_empty() {
                self.add_log(cx, "✓ ProxyState initialized - No proxies configured yet");
            }
        }
    }

    /// Add proxy from form to pool
    pub(crate) fn add_proxy_to_pool(&mut self, cx: &mut Cx) {
        // Get proxy from form
        let proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => {
                self.clear_logs(cx);
                self.add_log(cx, "✗ Please fill proxy info first");
                self.ui.redraw(cx);
                return;
            }
        };

        // Check for duplicates (same host:port)
        if let Some(state) = &self.state.proxy_state {
            let exists = state.list_upstreams()
                .iter()
                .any(|p| p.config.host == proxy.host && p.config.port == proxy.port);

            if exists {
                self.clear_logs(cx);
                self.add_log(cx, &format!("✗ Proxy {}:{} already exists!", proxy.host, proxy.port));
                self.refresh_proxy_list_display(cx);
                self.ui.redraw(cx);
                return;
            }
        }

        // Get a name - use host:port format
        let name = format!("{}:{}", proxy.host, proxy.port);

        // Convert to UpstreamProxy
        let upstream = UpstreamProxy {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            config: UpstreamConfig {
                host: proxy.host.clone(),
                port: proxy.port,
                username: proxy.username.clone(),
                password: proxy.password.clone(),
            },
            enabled: true,
            health: Default::default(),
        };

        // Add to pool
        if let Some(state) = &mut self.state.proxy_state {
            match state.add_upstream(upstream) {
                Ok(_) => {
                    self.clear_logs(cx);
                    self.add_log(cx, &format!("✓ Added proxy: {}", name));
                    self.refresh_proxy_list_display(cx);
                }
                Err(e) => {
                    self.clear_logs(cx);
                    self.add_log(cx, &format!("✗ Add error: {}", e));
                }
            }
        } else {
            self.clear_logs(cx);
            self.add_log(cx, "✗ ProxyState not initialized!");
        }
        self.ui.redraw(cx);
    }

    /// Clear all proxies from pool
    pub(crate) fn clear_all_proxies(&mut self, cx: &mut Cx) {
        if let Some(state) = &mut self.state.proxy_state {
            let proxies = state.list_upstreams();
            let count = proxies.len();

            if count == 0 {
                self.clear_logs(cx);
                self.add_log(cx, "No proxies to clear");
                self.ui.redraw(cx);
                return;
            }

            // Remove all proxies
            for proxy in proxies {
                let _ = state.remove_upstream(&proxy.id);
            }

            // Refresh will clear logs and show empty state
            self.refresh_proxy_list_display(cx);
        }
        self.ui.redraw(cx);
    }

    /// Load proxy from slot to form (for editing)
    pub(crate) fn load_proxy_to_form(&mut self, cx: &mut Cx, slot_index: usize) {
        if let Some(state) = &self.state.proxy_state {
            let proxies = state.list_upstreams();
            if let Some(proxy) = proxies.get(slot_index) {
                // Load proxy info to form
                self.ui.text_input(id!(host_input)).set_text(cx, &proxy.config.host);
                self.ui.text_input(id!(port_input)).set_text(cx, &proxy.config.port.to_string());

                if let Some(username) = &proxy.config.username {
                    self.ui.text_input(id!(username_input)).set_text(cx, username);
                } else {
                    self.ui.text_input(id!(username_input)).set_text(cx, "");
                }

                if let Some(password) = &proxy.config.password {
                    self.ui.text_input(id!(password_input)).set_text(cx, password);
                } else {
                    self.ui.text_input(id!(password_input)).set_text(cx, "");
                }

                self.clear_logs(cx);
                self.add_log(cx, &format!("✓ Loaded {} to form", proxy.name));
                if proxy.config.username.is_some() {
                    self.add_log(cx, "   (Credentials loaded)");
                }
                self.ui.redraw(cx);
            } else {
                self.clear_logs(cx);
                self.add_log(cx, &format!("✗ Slot {} not found", slot_index + 1));
                self.ui.redraw(cx);
            }
        }
    }

    /// Delete proxy from a specific slot
    pub(crate) fn delete_proxy_by_slot(&mut self, cx: &mut Cx, slot_index: usize) {
        if let Some(state) = &mut self.state.proxy_state {
            let proxies = state.list_upstreams();
            if let Some(proxy) = proxies.get(slot_index) {
                let proxy_id = proxy.id.clone();
                let proxy_name = proxy.name.clone();

                match state.remove_upstream(&proxy_id) {
                    Ok(_) => {
                        self.clear_logs(cx);
                        self.add_log(cx, &format!("✓ Deleted {}", proxy_name));
                        self.refresh_proxy_list_display(cx);
                    }
                    Err(e) => {
                        self.add_log(cx, &format!("✗ Delete error: {}", e));
                    }
                }
                self.ui.redraw(cx);
            }
        }
    }

    /// Refresh the proxy list display in UI
    pub(crate) fn refresh_proxy_list_display(&mut self, cx: &mut Cx) {
        if let Some(state) = &self.state.proxy_state {
            let proxies = state.list_upstreams();
            eprintln!("DEBUG: refresh_proxy_list_display called with {} proxies", proxies.len());

            let enabled_count = proxies.iter().filter(|p| p.enabled).count();
            let healthy_count = proxies.iter()
                .filter(|p| p.enabled && p.health.is_healthy())
                .count();

            // Update stats
            let stats_text = format!(
                "{} proxies, {} enabled, {} healthy",
                proxies.len(),
                enabled_count,
                healthy_count
            );
            eprintln!("DEBUG: Setting stats_text = {}", stats_text);
            self.ui.label(id!(pool_stats_label)).set_text(cx, &stats_text);

            // Update slots (max 10)
            for slot in 0..10 {
                let slot_view_id = match slot {
                    0 => id!(proxy_slot_1), 1 => id!(proxy_slot_2), 2 => id!(proxy_slot_3),
                    3 => id!(proxy_slot_4), 4 => id!(proxy_slot_5), 5 => id!(proxy_slot_6),
                    6 => id!(proxy_slot_7), 7 => id!(proxy_slot_8), 8 => id!(proxy_slot_9),
                    9 => id!(proxy_slot_10), _ => continue,
                };
                let status_id = match slot {
                    0 => id!(proxy_status_1), 1 => id!(proxy_status_2), 2 => id!(proxy_status_3),
                    3 => id!(proxy_status_4), 4 => id!(proxy_status_5), 5 => id!(proxy_status_6),
                    6 => id!(proxy_status_7), 7 => id!(proxy_status_8), 8 => id!(proxy_status_9),
                    9 => id!(proxy_status_10), _ => continue,
                };
                let name_id = match slot {
                    0 => id!(load_btn_1), 1 => id!(load_btn_2), 2 => id!(load_btn_3),
                    3 => id!(load_btn_4), 4 => id!(load_btn_5), 5 => id!(load_btn_6),
                    6 => id!(load_btn_7), 7 => id!(load_btn_8), 8 => id!(load_btn_9),
                    9 => id!(load_btn_10), _ => continue,
                };
                let info_id = match slot {
                    0 => id!(proxy_info_1), 1 => id!(proxy_info_2), 2 => id!(proxy_info_3),
                    3 => id!(proxy_info_4), 4 => id!(proxy_info_5), 5 => id!(proxy_info_6),
                    6 => id!(proxy_info_7), 7 => id!(proxy_info_8), 8 => id!(proxy_info_9),
                    9 => id!(proxy_info_10), _ => continue,
                };

                if let Some(proxy) = proxies.get(slot) {
                    // Show slot
                    self.ui.view(slot_view_id).set_visible(cx, true);

                    // Update status icon (text instead of emoji)
                    let status_icon = if proxy.health.is_healthy() {
                        "✓"  // Green check mark (text)
                    } else if proxy.health.error.is_some() {
                        "×"  // Red X mark (text)
                    } else {
                        "○"  // Gray circle (text)
                    };
                    self.ui.label(status_id).set_text(cx, status_icon);

                    // Update name (as button for loading to form)
                    self.ui.button(name_id).set_text(cx, &proxy.name);

                    // Update info (host:port, latency, location)
                    let mut info_parts = vec![format!("{}:{}", proxy.config.host, proxy.config.port)];

                    if let Some(latency) = proxy.health.latency_ms {
                        info_parts.push(format!("{}ms", latency));
                    }

                    if let Some(location) = &proxy.health.location {
                        info_parts.push(location.clone());
                    } else if let Some(exit_ip) = &proxy.health.exit_ip {
                        info_parts.push(exit_ip.clone());
                    }

                    let info_text = info_parts.join(" | ");
                    self.ui.label(info_id).set_text(cx, &info_text);
                } else {
                    // Hide slot
                    self.ui.view(slot_view_id).set_visible(cx, false);
                }
            }

            // Show/hide empty message
            let empty_visible = proxies.is_empty();
            self.ui.view(id!(proxy_empty_msg)).set_visible(cx, empty_visible);

            // Clear old logs and show current proxy pool state
            self.clear_logs(cx);

            if !proxies.is_empty() {
                self.add_log(cx, "=== Proxy Pool ===");
                for (i, proxy) in proxies.iter().enumerate() {
                    let status_icon = if proxy.health.is_healthy() {
                        "✓"  // Green check mark
                    } else if proxy.health.error.is_some() {
                        "×"  // Red X mark
                    } else {
                        "○"  // Gray circle
                    };
                    let latency_str = if let Some(latency) = proxy.health.latency_ms {
                        format!(" {}ms", latency)
                    } else {
                        String::new()
                    };
                    let enabled_str = if proxy.enabled { "[ON]" } else { "[OFF]" };
                    let mut log_line = format!(
                        "{}. {} {} {}{}",
                        i + 1,
                        status_icon,
                        enabled_str,
                        proxy.name,  // Already contains host:port
                        latency_str
                    );

                    // Add location if available
                    if let Some(location) = &proxy.health.location {
                        log_line.push_str(&format!(" [{}]", location));
                    } else if let Some(exit_ip) = &proxy.health.exit_ip {
                        log_line.push_str(&format!(" [IP: {}]", exit_ip));
                    }

                    self.add_log(cx, &log_line);

                    // Also show error if present
                    if let Some(err) = &proxy.health.error {
                        self.add_log(cx, &format!("   Error: {}", err));
                    }
                }
            } else {
                self.add_log(cx, "Ready");
            }
        }
    }
}
