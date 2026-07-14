//! Header controls for key, scale, and layout mode.

use egui::Ui;
use reelsynth_ui_theme::Tokens;

use crate::widgets::{button_toggle, menu_selectable, reel_combo, select_value_text, styled_menu_body};
use crate::UiState;

use super::{CHORD_DEGREE_LABELS, LAYOUT_NAMES, ROOT_NAMES, SCALE_NAMES};

pub struct PerformanceHeaderActions {
    pub params_changed: bool,
    pub chord_degree_on: Option<usize>,
    pub chord_degree_off: Option<usize>,
}

impl Default for PerformanceHeaderActions {
    fn default() -> Self {
        Self {
            params_changed: false,
            chord_degree_on: None,
            chord_degree_off: None,
        }
    }
}

/// Key dropdown, scale dropdown, and Piano | Scale | Chords layout segment.
pub fn draw_performance_header(
    ui: &mut Ui,
    state: &mut UiState,
) -> PerformanceHeaderActions {
    let tokens = Tokens::default();
    let mut actions = PerformanceHeaderActions::default();
    let perf = &mut state.performance;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        ui.label(
            egui::RichText::new("Key")
                .size(10.0)
                .color(tokens.text_muted),
        );
        let root_label = ROOT_NAMES[perf.root.min(ROOT_NAMES.len().saturating_sub(1))];
        reel_combo(ui, "perf_root", select_value_text(root_label), 56.0, |ui| {
            styled_menu_body(ui, |ui| {
                for (idx, name) in ROOT_NAMES.iter().enumerate() {
                    if menu_selectable(ui, perf.root == idx, name).clicked() {
                        perf.root = idx;
                        actions.params_changed = true;
                    }
                }
            });
        });

        ui.label(
            egui::RichText::new("Scale")
                .size(10.0)
                .color(tokens.text_muted),
        );
        let scale_label = SCALE_NAMES[perf.scale.min(SCALE_NAMES.len().saturating_sub(1))];
        reel_combo(ui, "perf_scale", select_value_text(scale_label), 120.0, |ui| {
            styled_menu_body(ui, |ui| {
                for (idx, name) in SCALE_NAMES.iter().enumerate() {
                    if menu_selectable(ui, perf.scale == idx, name).clicked() {
                        perf.scale = idx;
                        actions.params_changed = true;
                    }
                }
            });
        });

        ui.add_space(4.0);
        for (idx, name) in LAYOUT_NAMES.iter().enumerate() {
            if button_toggle(ui, name, perf.layout == idx).clicked() {
                perf.layout = idx;
                actions.params_changed = true;
            }
        }

        if perf.layout == 2 {
            ui.add_space(4.0);
            for (deg, label) in CHORD_DEGREE_LABELS.iter().enumerate() {
                let active = state.active_chord_degree == Some(deg);
                if button_toggle(ui, label, active).clicked() {
                    if active {
                        actions.chord_degree_off = Some(deg);
                        state.active_chord_degree = None;
                    } else {
                        if let Some(prev) = state.active_chord_degree {
                            actions.chord_degree_off = Some(prev);
                        }
                        actions.chord_degree_on = Some(deg);
                        state.active_chord_degree = Some(deg);
                    }
                }
            }
        }
    });

    actions
}
