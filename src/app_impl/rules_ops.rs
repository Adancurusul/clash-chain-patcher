//! Rules Rewrite Operations
//!
//! Methods for:
//! - Parsing rule groups from loaded config
//! - Toggling rule group checkboxes and targets
//! - Building replacement map for apply

use makepad_widgets::*;
use clash_chain_patcher::patcher;
use crate::app::{App, MAX_RULE_SLOTS, RuleReplaceTarget};
use std::collections::HashMap;

impl App {
    /// Toggle the rules panel visibility
    pub(crate) fn toggle_rules_panel(&mut self, cx: &mut Cx) {
        self.state.show_rules_panel = !self.state.show_rules_panel;

        self.ui.view(id!(rules_panel)).set_visible(cx, self.state.show_rules_panel);

        let btn_text = if self.state.show_rules_panel { "▲" } else { "▼" };
        self.ui.button(id!(toggle_rules_btn)).set_text(cx, btn_text);

        self.ui.redraw(cx);
    }

    /// Parse rule groups from the currently loaded config and populate UI
    pub(crate) fn refresh_rule_groups(&mut self, cx: &mut Cx) {
        // Reset state
        self.state.rule_groups.clear();
        self.state.rule_checked = vec![false; MAX_RULE_SLOTS];
        self.state.rule_targets = vec![RuleReplaceTarget::Keep; MAX_RULE_SLOTS];

        if let Some(content) = &self.state.config_content {
            self.state.rule_groups = patcher::extract_rule_groups(content);
        }

        let total_groups = self.state.rule_groups.len();
        let total_rules: usize = self.state.rule_groups.iter().map(|g| g.count).sum();

        // All unchecked by default - user clicks to enable
        // (rule_checked and rule_targets already reset to false/Keep above)

        // Update stats label
        let stats = if total_groups > 0 {
            format!("{} groups, {} rules", total_groups, total_rules)
        } else {
            "No rules".to_string()
        };
        self.ui.label(id!(rules_stats_label)).set_text(cx, &stats);

        // Log rule groups to output
        if total_groups > 0 {
            self.add_log(cx, &format!("Rules: {} groups, {} total rules", total_groups, total_rules));
            // Collect log lines first to avoid borrow conflict
            let log_lines: Vec<String> = self.state.rule_groups.iter().enumerate()
                .take(MAX_RULE_SLOTS)
                .map(|(i, group)| {
                    let status = if self.state.rule_checked[i] {
                        format!("-> {}", self.state.rule_targets[i].label())
                    } else {
                        "keep".to_string()
                    };
                    format!("  {} ({} rules) {}", group.name, group.count, status)
                })
                .collect();
            for line in &log_lines {
                self.add_log(cx, line);
            }
            self.update_log_display(cx);
        }

        // Update slots
        self.refresh_rule_slots_display(cx);
    }

    /// Refresh the rule slots UI from state
    fn refresh_rule_slots_display(&mut self, cx: &mut Cx) {
        for slot in 0..MAX_RULE_SLOTS {
            let visible = slot < self.state.rule_groups.len();

            macro_rules! update_slot {
                ($slot_id:ident, $check_id:ident, $name_id:ident, $count_id:ident, $target_id:ident, $idx:expr) => {
                    if visible {
                        let group = &self.state.rule_groups[$idx];
                        self.ui.view(id!($slot_id)).set_visible(cx, true);
                        self.ui.label(id!($name_id)).set_text(cx, &group.name);
                        self.ui.label(id!($count_id)).set_text(cx, &format!("{} rules", group.count));
                        let check_text = if self.state.rule_checked[$idx] { "✓" } else { "·" };
                        self.ui.button(id!($check_id)).set_text(cx, check_text);
                        self.ui.button(id!($target_id)).set_text(cx, self.state.rule_targets[$idx].label());
                    } else {
                        self.ui.view(id!($slot_id)).set_visible(cx, false);
                    }
                };
            }

            match slot {
                0 => update_slot!(rule_slot_1, rule_check_1, rule_name_1, rule_count_1, rule_target_1, 0),
                1 => update_slot!(rule_slot_2, rule_check_2, rule_name_2, rule_count_2, rule_target_2, 1),
                2 => update_slot!(rule_slot_3, rule_check_3, rule_name_3, rule_count_3, rule_target_3, 2),
                3 => update_slot!(rule_slot_4, rule_check_4, rule_name_4, rule_count_4, rule_target_4, 3),
                4 => update_slot!(rule_slot_5, rule_check_5, rule_name_5, rule_count_5, rule_target_5, 4),
                5 => update_slot!(rule_slot_6, rule_check_6, rule_name_6, rule_count_6, rule_target_6, 5),
                6 => update_slot!(rule_slot_7, rule_check_7, rule_name_7, rule_count_7, rule_target_7, 6),
                7 => update_slot!(rule_slot_8, rule_check_8, rule_name_8, rule_count_8, rule_target_8, 7),
                8 => update_slot!(rule_slot_9, rule_check_9, rule_name_9, rule_count_9, rule_target_9, 8),
                9 => update_slot!(rule_slot_10, rule_check_10, rule_name_10, rule_count_10, rule_target_10, 9),
                10 => update_slot!(rule_slot_11, rule_check_11, rule_name_11, rule_count_11, rule_target_11, 10),
                11 => update_slot!(rule_slot_12, rule_check_12, rule_name_12, rule_count_12, rule_target_12, 11),
                12 => update_slot!(rule_slot_13, rule_check_13, rule_name_13, rule_count_13, rule_target_13, 12),
                13 => update_slot!(rule_slot_14, rule_check_14, rule_name_14, rule_count_14, rule_target_14, 13),
                14 => update_slot!(rule_slot_15, rule_check_15, rule_name_15, rule_count_15, rule_target_15, 14),
                15 => update_slot!(rule_slot_16, rule_check_16, rule_name_16, rule_count_16, rule_target_16, 15),
                _ => {}
            }
        }
    }

    /// Toggle a rule group checkbox
    pub(crate) fn toggle_rule_check(&mut self, cx: &mut Cx, index: usize) {
        if index >= MAX_RULE_SLOTS || index >= self.state.rule_groups.len() {
            return;
        }

        self.state.rule_checked[index] = !self.state.rule_checked[index];

        // If checking on and target is Keep, auto-set to Chain-Selector
        if self.state.rule_checked[index] && self.state.rule_targets[index] == RuleReplaceTarget::Keep {
            self.state.rule_targets[index] = RuleReplaceTarget::ChainSelector;
        }

        self.refresh_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Cycle the replace target for a rule group
    pub(crate) fn cycle_rule_target(&mut self, cx: &mut Cx, index: usize) {
        if index >= MAX_RULE_SLOTS || index >= self.state.rule_groups.len() {
            return;
        }

        self.state.rule_targets[index] = self.state.rule_targets[index].next();

        // If target changed from Keep, auto-check
        if self.state.rule_targets[index] != RuleReplaceTarget::Keep {
            self.state.rule_checked[index] = true;
        }

        self.refresh_rule_slots_display(cx);
        self.ui.redraw(cx);
    }

    /// Build the replacement map from current UI state
    /// Returns HashMap<original_group_name, new_group_name>
    pub(crate) fn build_rule_replacements(&self) -> HashMap<String, String> {
        let mut replacements = HashMap::new();

        for (i, group) in self.state.rule_groups.iter().enumerate() {
            if i >= MAX_RULE_SLOTS { break; }
            if !self.state.rule_checked[i] { continue; }

            match &self.state.rule_targets[i] {
                RuleReplaceTarget::Keep => {}
                RuleReplaceTarget::ChainSelector => {
                    replacements.insert(group.name.clone(), "Chain-Selector".to_string());
                }
                RuleReplaceTarget::ChainAuto => {
                    replacements.insert(group.name.clone(), "Chain-Auto".to_string());
                }
            }
        }

        replacements
    }
}
