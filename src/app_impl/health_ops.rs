//! Health Check Operations
//!
//! Methods for:
//! - Checking individual proxy health
//! - Checking all proxies health
//! - Toggle auto health check
//! - Background health check updates

use makepad_widgets::*;
use crate::app::App;

impl App {
    /// Check health of all enabled proxies
    pub(crate) fn check_all_proxies(&mut self, cx: &mut Cx) {
        if self.state.checking {
            self.add_log(cx, "Check in progress...");
            self.ui.redraw(cx);
            return;
        }

        // Get proxies to check
        let proxies_info: Vec<(String, String, u16, Option<String>, Option<String>)> = {
            if let Some(state) = &self.state.proxy_state {
                state.list_upstreams()
                    .into_iter()
                    .filter(|p| p.enabled)
                    .map(|p| (
                        p.id.clone(),
                        p.config.host.clone(),
                        p.config.port,
                        p.config.username.clone(),
                        p.config.password.clone(),
                    ))
                    .collect()
            } else {
                Vec::new()
            }
        };

        if proxies_info.is_empty() {
            self.clear_logs(cx);
            self.add_log(cx, "No enabled proxies to check");
            self.ui.redraw(cx);
            return;
        }

        self.state.checking = true;
        self.clear_logs(cx);
        self.add_log(cx, &format!("Checking {} proxies...", proxies_info.len()));
        self.add_log(cx, "Note: UI may freeze briefly (10s per proxy)");
        self.ui.redraw(cx);

        // Use enhanced validator (same as individual check)
        use clash_chain_patcher::health::ProxyValidator;
        let validator = ProxyValidator::new(10);

        for (i, (proxy_id, host, port, username, password)) in proxies_info.iter().enumerate() {
            self.add_log(cx, &format!("Checking {}/{}...", i + 1, proxies_info.len()));
            self.ui.redraw(cx);

            let result = validator.validate(
                host,
                *port,
                username.as_deref(),
                password.as_deref(),
            );

            // Update proxy health
            if let Some(state) = &mut self.state.proxy_state {
                if let Some(mut proxy) = state.get_upstream(proxy_id) {
                    if result.is_valid {
                        if let Some(latency) = result.latency_ms {
                            proxy.health.mark_healthy_with_details(
                                latency as u64,
                                result.exit_ip,
                                result.location.as_ref().map(|l| l.format_short()),
                                result.location.as_ref().map(|l| l.country_code.clone()),
                            );
                        }
                    } else if let Some(error) = result.error {
                        proxy.health.mark_unhealthy(error);
                    }
                    let _ = state.update_upstream(proxy);
                }
            }
        }

        self.state.checking = false;
        self.clear_logs(cx);
        self.add_log(cx, "✓ Health check completed");
        self.refresh_proxy_list_display(cx);
        self.ui.redraw(cx);
    }

    /// Toggle auto health check on/off
    pub(crate) fn toggle_auto_health_check(&mut self, cx: &mut Cx) {
        eprintln!("DEBUG: toggle_auto_health_check called, current state = {}", self.state.auto_checking);

        // Read interval from input
        let interval_str = self.ui.text_input(id!(interval_input)).text();
        let interval_minutes = interval_str.parse::<u64>().unwrap_or(5);
        self.state.auto_check_interval = interval_minutes;

        if self.state.auto_checking {
            // Stop auto checking
            self.state.auto_checking = false;
            self.state.health_check_rx = None;
            // Task will be dropped and cancelled

            self.ui.button(id!(auto_check_btn)).set_text(cx, "Auto: OFF");
            self.clear_logs(cx);
            self.add_log(cx, "Auto health check stopped");

            eprintln!("DEBUG: Auto check stopped");
        } else {
            // Start auto checking
            use std::sync::mpsc;

            let (tx, rx) = mpsc::channel();
            self.state.health_check_rx = Some(rx);

            // Get proxies info for background check
            if let Some(state) = &self.state.proxy_state {
                let proxies = state.list_upstreams();

                if proxies.is_empty() {
                    self.clear_logs(cx);
                    self.add_log(cx, "✗ No proxies to check");
                    self.add_log(cx, "  Add proxies first");
                    return;
                }

                // Create shared proxy list
                let proxy_list: Vec<_> = proxies.into_iter()
                    .filter(|p| p.enabled)
                    .map(|p| (
                        p.id.clone(),
                        p.config.host.clone(),
                        p.config.port,
                        p.config.username.clone(),
                        p.config.password.clone(),
                    ))
                    .collect();

                if proxy_list.is_empty() {
                    self.clear_logs(cx);
                    self.add_log(cx, "✗ No enabled proxies");
                    self.add_log(cx, "  Enable at least one proxy");
                    return;
                }

                let interval_secs = interval_minutes * 60;
                let proxy_count = proxy_list.len();

                // Spawn background task
                let handle = std::thread::spawn(move || {
                    use clash_chain_patcher::health::ProxyValidator;
                    use std::thread;
                    use std::time::Duration;

                    let validator = ProxyValidator::new(10);

                    eprintln!("DEBUG: Auto check background thread started, checking every {} minutes", interval_minutes);

                    loop {
                        eprintln!("DEBUG: Starting auto health check cycle");

                        for (proxy_id, host, port, username, password) in &proxy_list {
                            let result = validator.validate(
                                host,
                                *port,
                                username.as_deref(),
                                password.as_deref(),
                            );

                            // Send result to GUI thread
                            if tx.send((proxy_id.clone(), result)).is_err() {
                                eprintln!("DEBUG: Channel closed, stopping auto check");
                                return;
                            }
                        }

                        eprintln!("DEBUG: Auto check cycle completed, sleeping for {} seconds", interval_secs);

                        // Sleep until next check
                        thread::sleep(Duration::from_secs(interval_secs));
                    }
                });

                self.state.auto_checking = true;
                self.ui.button(id!(auto_check_btn)).set_text(cx, "Auto: ON");

                self.clear_logs(cx);
                self.add_log(cx, "✓ Auto health check started");
                self.add_log(cx, &format!("  Checking every {} minutes", interval_minutes));
                self.add_log(cx, &format!("  Monitoring {} enabled proxies", proxy_count));

                eprintln!("DEBUG: Auto check started with {} minute interval", interval_minutes);

                // Note: handle is dropped but thread continues running
                // It will stop when channel is closed
                drop(handle);
            } else {
                self.clear_logs(cx);
                self.add_log(cx, "✗ ProxyState not initialized");
            }
        }

        self.ui.redraw(cx);
    }

    /// Update proxy health from background check result
    pub(crate) fn update_proxy_health_from_background(
        &mut self,
        cx: &mut Cx,
        proxy_id: String,
        result: clash_chain_patcher::health::ProxyValidationResult,
    ) {
        eprintln!("DEBUG: Received health check result for proxy {}: valid={}", proxy_id, result.is_valid);

        if let Some(state) = &mut self.state.proxy_state {
            if let Some(mut proxy) = state.get_upstream(&proxy_id) {
                if result.is_valid {
                    if let Some(latency) = result.latency_ms {
                        proxy.health.mark_healthy_with_details(
                            latency as u64,
                            result.exit_ip,
                            result.location.as_ref().map(|l| l.format_short()),
                            result.location.as_ref().map(|l| l.country_code.clone()),
                        );
                    }
                } else if let Some(error) = result.error {
                    proxy.health.mark_unhealthy(error);
                }

                let _ = state.update_upstream(proxy);
            }
        }

        // Refresh display
        self.refresh_proxy_list_display(cx);
        self.ui.redraw(cx);
    }

    /// Check health of proxy in a specific slot
    pub(crate) fn check_proxy_by_slot(&mut self, cx: &mut Cx, slot_index: usize) {
        // First, get proxy info from state
        let (proxy_id, proxy_name, host, port, username, password) = {
            if let Some(state) = &self.state.proxy_state {
                let proxies = state.list_upstreams();
                if let Some(proxy) = proxies.get(slot_index) {
                    (
                        proxy.id.clone(),
                        proxy.name.clone(),
                        proxy.config.host.clone(),
                        proxy.config.port,
                        proxy.config.username.clone(),
                        proxy.config.password.clone(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        // Now we can use self mutably
        self.clear_logs(cx);
        self.add_log(cx, &format!("Checking {}...", proxy_name));
        self.ui.redraw(cx);

        // Use enhanced validator
        use clash_chain_patcher::health::ProxyValidator;
        let validator = ProxyValidator::new(10);
        let result = validator.validate(
            &host,
            port,
            username.as_deref(),
            password.as_deref(),
        );

        // Update proxy health
        if let Some(state) = &mut self.state.proxy_state {
            if let Some(mut updated_proxy) = state.get_upstream(&proxy_id) {
                if result.is_valid {
                    if let Some(latency) = result.latency_ms {
                        updated_proxy.health.mark_healthy_with_details(
                            latency as u64,
                            result.exit_ip,
                            result.location.as_ref().map(|l| l.format_short()),
                            result.location.as_ref().map(|l| l.country_code.clone()),
                        );
                    }
                } else if let Some(error) = result.error {
                    updated_proxy.health.mark_unhealthy(error);
                }
                let _ = state.update_upstream(updated_proxy);
            }
        }

        self.refresh_proxy_list_display(cx);
        self.ui.redraw(cx);
    }
}
