//! Custom Rules Operations
//!
//! Methods for:
//! - Managing custom domain rules (add/toggle/delete)
//! - Cycling match type and target group
//! - Preset save/load/delete
//! - Building custom rules for the apply flow

use makepad_widgets::*;
use clash_chain_patcher::patcher::{CustomRule, CustomRuleSet};
use crate::app::{App, MAX_CUSTOM_RULE_SLOTS, MAX_PRESET_SLOTS};

impl App {
    /// Toggle the custom rules panel visibility
    pub(crate) fn toggle_custom_rules_panel(&mut self, cx: &mut Cx) {
        self.state.show_custom_rules_panel = !self.state.show_custom_rules_panel;
        self.ui.view(id!(custom_rules_panel)).set_visible(cx, self.state.show_custom_rules_panel);
        let btn_text = if self.state.show_custom_rules_panel { "▲" } else { "▼" };
        self.ui.button(id!(toggle_custom_rules_btn)).set_text(cx, btn_text);
        self.ui.redraw(cx);
    }

    /// Refresh available target groups from loaded config
    pub(crate) fn refresh_available_targets(&mut self, _cx: &mut Cx) {
        let mut targets = vec!["DIRECT".to_string()];
        // Add groups from loaded config rules
        for group in &self.state.rule_groups {
            if group.name != "DIRECT" && group.name != "REJECT"
                && !targets.contains(&group.name)
            {
                targets.push(group.name.clone());
            }
        }
        // Always include chain targets
        if !targets.iter().any(|t| t == "Chain-Selector") {
            targets.push("Chain-Selector".to_string());
        }
        if !targets.iter().any(|t| t == "Chain-Auto") {
            targets.push("Chain-Auto".to_string());
        }
        self.state.available_targets = targets;
        // Reset target index if out of bounds
        if self.state.custom_rule_target_index >= self.state.available_targets.len() {
            self.state.custom_rule_target_index = 0;
        }
    }

    /// Cycle the match type button
    pub(crate) fn cycle_custom_rule_type(&mut self, cx: &mut Cx) {
        self.state.custom_rule_match_type = self.state.custom_rule_match_type.next();
        self.ui.button(id!(custom_rule_type_btn))
            .set_text(cx, self.state.custom_rule_match_type.label());
        self.ui.redraw(cx);
    }

    /// Cycle the target group button
    pub(crate) fn cycle_custom_rule_target(&mut self, cx: &mut Cx) {
        if self.state.available_targets.is_empty() {
            return;
        }
        self.state.custom_rule_target_index =
            (self.state.custom_rule_target_index + 1) % self.state.available_targets.len();
        let target = &self.state.available_targets[self.state.custom_rule_target_index];
        self.ui.button(id!(custom_rule_target_btn)).set_text(cx, target);
        self.ui.redraw(cx);
    }

    /// Add a custom rule from the input fields
    pub(crate) fn add_custom_rule(&mut self, cx: &mut Cx) {
        if self.state.custom_rules.len() >= MAX_CUSTOM_RULE_SLOTS {
            self.add_log(cx, "Max custom rules reached (15)");
            self.update_log_display(cx);
            return;
        }

        let domain_str = self.ui.text_input(id!(custom_rule_domain_input)).text();
        let domain_str = domain_str.trim();
        if domain_str.is_empty() {
            return;
        }

        let target = self.state.available_targets
            .get(self.state.custom_rule_target_index)
            .cloned()
            .unwrap_or_else(|| "DIRECT".to_string());
        let match_type = self.state.custom_rule_match_type;

        // Support comma-separated domains in one input
        let domains: Vec<&str> = domain_str.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for domain in domains {
            if self.state.custom_rules.len() >= MAX_CUSTOM_RULE_SLOTS {
                break;
            }
            // Skip duplicates
            let already_exists = self.state.custom_rules.iter().any(|r|
                r.domain == domain && r.target_group == target && r.match_type == match_type
            );
            if already_exists {
                continue;
            }
            self.state.custom_rules.push(CustomRule {
                match_type,
                domain: domain.to_string(),
                target_group: target.clone(),
                enabled: true,
            });
        }

        // Clear input
        self.ui.text_input(id!(custom_rule_domain_input)).set_text(cx, "");
        self.refresh_custom_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Toggle a custom rule's enabled state
    pub(crate) fn toggle_custom_rule_enabled(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.state.custom_rules.len() {
            return;
        }
        self.state.custom_rules[index].enabled = !self.state.custom_rules[index].enabled;
        self.refresh_custom_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Delete a custom rule
    pub(crate) fn delete_custom_rule(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.state.custom_rules.len() {
            return;
        }
        self.state.custom_rules.remove(index);
        self.refresh_custom_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Clear all custom rules
    pub(crate) fn clear_custom_rules(&mut self, cx: &mut Cx) {
        self.state.custom_rules.clear();
        self.refresh_custom_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Refresh the custom rule slots display
    pub(crate) fn refresh_custom_rule_slots_display(&mut self, cx: &mut Cx) {
        // Update stats
        let active_count = self.state.custom_rules.iter().filter(|r| r.enabled).count();
        let stats = format!("{} rules", active_count);
        self.ui.label(id!(custom_rules_stats_label)).set_text(cx, &stats);

        for slot in 0..MAX_CUSTOM_RULE_SLOTS {
            let visible = slot < self.state.custom_rules.len();

            macro_rules! update_cr_slot {
                ($slot_id:ident, $check_id:ident, $type_id:ident, $domain_id:ident, $target_id:ident, $delete_id:ident, $idx:expr) => {
                    if visible {
                        let rule = &self.state.custom_rules[$idx];
                        self.ui.view(id!($slot_id)).set_visible(cx, true);
                        let check_text = if rule.enabled { "✓" } else { "·" };
                        self.ui.button(id!($check_id)).set_text(cx, check_text);
                        self.ui.label(id!($type_id)).set_text(cx, rule.match_type.label());
                        self.ui.label(id!($domain_id)).set_text(cx, &rule.domain);
                        self.ui.label(id!($target_id)).set_text(cx, &rule.target_group);
                    } else {
                        self.ui.view(id!($slot_id)).set_visible(cx, false);
                    }
                };
            }

            match slot {
                0 => update_cr_slot!(cr_slot_1, cr_check_1, cr_type_1, cr_domain_1, cr_target_1, cr_delete_1, 0),
                1 => update_cr_slot!(cr_slot_2, cr_check_2, cr_type_2, cr_domain_2, cr_target_2, cr_delete_2, 1),
                2 => update_cr_slot!(cr_slot_3, cr_check_3, cr_type_3, cr_domain_3, cr_target_3, cr_delete_3, 2),
                3 => update_cr_slot!(cr_slot_4, cr_check_4, cr_type_4, cr_domain_4, cr_target_4, cr_delete_4, 3),
                4 => update_cr_slot!(cr_slot_5, cr_check_5, cr_type_5, cr_domain_5, cr_target_5, cr_delete_5, 4),
                5 => update_cr_slot!(cr_slot_6, cr_check_6, cr_type_6, cr_domain_6, cr_target_6, cr_delete_6, 5),
                6 => update_cr_slot!(cr_slot_7, cr_check_7, cr_type_7, cr_domain_7, cr_target_7, cr_delete_7, 6),
                7 => update_cr_slot!(cr_slot_8, cr_check_8, cr_type_8, cr_domain_8, cr_target_8, cr_delete_8, 7),
                8 => update_cr_slot!(cr_slot_9, cr_check_9, cr_type_9, cr_domain_9, cr_target_9, cr_delete_9, 8),
                9 => update_cr_slot!(cr_slot_10, cr_check_10, cr_type_10, cr_domain_10, cr_target_10, cr_delete_10, 9),
                10 => update_cr_slot!(cr_slot_11, cr_check_11, cr_type_11, cr_domain_11, cr_target_11, cr_delete_11, 10),
                11 => update_cr_slot!(cr_slot_12, cr_check_12, cr_type_12, cr_domain_12, cr_target_12, cr_delete_12, 11),
                12 => update_cr_slot!(cr_slot_13, cr_check_13, cr_type_13, cr_domain_13, cr_target_13, cr_delete_13, 12),
                13 => update_cr_slot!(cr_slot_14, cr_check_14, cr_type_14, cr_domain_14, cr_target_14, cr_delete_14, 13),
                14 => update_cr_slot!(cr_slot_15, cr_check_15, cr_type_15, cr_domain_15, cr_target_15, cr_delete_15, 14),
                _ => {}
            }
        }
    }

    /// Save current custom rules as a named preset
    pub(crate) fn save_custom_rule_preset(&mut self, cx: &mut Cx) {
        if self.state.custom_rules.is_empty() {
            self.add_log(cx, "No custom rules to save");
            self.update_log_display(cx);
            return;
        }

        let input_name = self.ui.text_input(id!(custom_rule_preset_input)).text();
        let name = if input_name.trim().is_empty() {
            // Auto-generate name from first rule's domain + target
            let first = &self.state.custom_rules[0];
            format!("{} {}", first.domain, first.target_group)
        } else {
            input_name.trim().to_string()
        };

        let preset = CustomRuleSet {
            name: name.clone(),
            rules: self.state.custom_rules.clone(),
        };

        // Update in-memory list (overwrite if same name)
        self.state.custom_rule_presets.retain(|p| p.name != name);
        self.state.custom_rule_presets.push(preset);

        // Persist via bridge
        if let Some(state) = &self.state.proxy_state {
            if let Err(e) = state.save_custom_rule_presets(self.state.custom_rule_presets.clone()) {
                self.add_log(cx, &format!("Failed to save preset: {}", e));
                self.update_log_display(cx);
                self.ui.redraw(cx);
                return;
            }
        }

        self.add_log(cx, &format!("Preset '{}' saved ({} rules)", name, self.state.custom_rules.len()));
        self.update_log_display(cx);
        self.refresh_preset_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Toggle the preset list visibility
    pub(crate) fn toggle_preset_list(&mut self, cx: &mut Cx) {
        self.state.show_preset_list = !self.state.show_preset_list;
        self.ui.view(id!(custom_rule_preset_panel)).set_visible(cx, self.state.show_preset_list);
        if self.state.show_preset_list {
            self.refresh_preset_slots_display(cx);
        }
        let btn_text = if self.state.show_preset_list { "Load ▲" } else { "Load ▼" };
        self.ui.button(id!(custom_rule_load_btn)).set_text(cx, btn_text);
        self.ui.redraw(cx);
    }

    /// Load a preset into the working custom rules
    pub(crate) fn load_preset(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.state.custom_rule_presets.len() {
            return;
        }
        let preset = self.state.custom_rule_presets[index].clone();
        self.state.custom_rules = preset.rules;
        // Truncate to max slots
        self.state.custom_rules.truncate(MAX_CUSTOM_RULE_SLOTS);

        self.add_log(cx, &format!("Loaded preset '{}'", preset.name));
        self.update_log_display(cx);

        // Hide preset list
        self.state.show_preset_list = false;
        self.ui.view(id!(custom_rule_preset_panel)).set_visible(cx, false);
        self.ui.button(id!(custom_rule_load_btn)).set_text(cx, "Load ▼");

        self.refresh_custom_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Delete a preset
    pub(crate) fn delete_preset(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.state.custom_rule_presets.len() {
            return;
        }
        let name = self.state.custom_rule_presets[index].name.clone();
        self.state.custom_rule_presets.remove(index);

        // Persist
        if let Some(state) = &self.state.proxy_state {
            if let Err(e) = state.save_custom_rule_presets(self.state.custom_rule_presets.clone()) {
                self.add_log(cx, &format!("Failed to persist preset deletion: {}", e));
            }
        }

        self.add_log(cx, &format!("Preset '{}' deleted", name));
        self.update_log_display(cx);
        self.refresh_preset_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Refresh the preset dropdown slots
    pub(crate) fn refresh_preset_slots_display(&mut self, cx: &mut Cx) {
        for slot in 0..MAX_PRESET_SLOTS {
            let visible = slot < self.state.custom_rule_presets.len();

            macro_rules! update_preset_slot {
                ($slot_id:ident, $btn_id:ident, $del_id:ident, $idx:expr) => {
                    if visible {
                        let preset = &self.state.custom_rule_presets[$idx];
                        self.ui.view(id!($slot_id)).set_visible(cx, true);
                        self.ui.button(id!($btn_id)).set_text(cx,
                            &format!("{} ({} rules)", preset.name, preset.rules.len()));
                    } else {
                        self.ui.view(id!($slot_id)).set_visible(cx, false);
                    }
                };
            }

            match slot {
                0 => update_preset_slot!(cr_preset_slot_1, cr_preset_btn_1, cr_preset_del_1, 0),
                1 => update_preset_slot!(cr_preset_slot_2, cr_preset_btn_2, cr_preset_del_2, 1),
                2 => update_preset_slot!(cr_preset_slot_3, cr_preset_btn_3, cr_preset_del_3, 2),
                3 => update_preset_slot!(cr_preset_slot_4, cr_preset_btn_4, cr_preset_del_4, 3),
                4 => update_preset_slot!(cr_preset_slot_5, cr_preset_btn_5, cr_preset_del_5, 4),
                _ => {}
            }
        }
    }

    /// Build the list of enabled custom rules for the apply flow
    pub(crate) fn build_custom_rules(&self) -> Vec<CustomRule> {
        self.state.custom_rules.iter()
            .filter(|r| r.enabled)
            .cloned()
            .collect()
    }

    /// Load presets from config on startup, seeding built-in templates if empty
    pub(crate) fn load_presets_from_config(&mut self) {
        if let Some(state) = &self.state.proxy_state {
            self.state.custom_rule_presets = state.get_custom_rule_presets();
        }

        // Seed built-in templates on first use only (not after user deletes them)
        let seeded = self.state.proxy_state.as_ref().map(|s| s.presets_seeded()).unwrap_or(true);
        if !seeded {
            self.state.custom_rule_presets = Self::builtin_presets();
            if let Some(state) = &self.state.proxy_state {
                let _ = state.save_custom_rule_presets(self.state.custom_rule_presets.clone());
                let _ = state.set_presets_seeded();
            }
        }
    }

    /// Built-in preset templates
    fn builtin_presets() -> Vec<CustomRuleSet> {
        use clash_chain_patcher::patcher::RuleMatchType;

        vec![
            CustomRuleSet {
                name: "Lark Direct".to_string(),
                rules: vec![
                    CustomRule { match_type: RuleMatchType::DomainKeyword, domain: "lark".to_string(), target_group: "DIRECT".to_string(), enabled: true },
                    CustomRule { match_type: RuleMatchType::DomainKeyword, domain: "feishu".to_string(), target_group: "DIRECT".to_string(), enabled: true },
                    CustomRule { match_type: RuleMatchType::DomainSuffix, domain: "larksuite.com".to_string(), target_group: "DIRECT".to_string(), enabled: true },
                ],
            },
            CustomRuleSet {
                name: "SSH Direct".to_string(),
                rules: vec![
                    CustomRule { match_type: RuleMatchType::DstPort, domain: "22".to_string(), target_group: "DIRECT".to_string(), enabled: true },
                ],
            },
        ]
    }
}
