use egui::{Rect, Ui};
use reelsynth::Patch;
use reelsynth_ui_theme::Tokens;

use super::*;
use super::header::{sync_morph_from_active_tab, sync_osc_position_from_wt};
pub(super) fn draw_center(
    ui: &mut Ui,
    rect: Rect,
    state: &mut UiState,
    mut bank: Option<&mut WavetableBank>,
    preview_patch: &Patch,
    config: &ShellConfig,
    scope: Option<ScopeStripContext<'_>>,
    actions: &mut ShellActions,
) {
    let inner = rect.shrink(SPACE_SM);
    let morph_h = if config.show_wt_editor {
        WT_MORPH_HEIGHT + GRID_UNIT
    } else {
        0.0
    };
    let views_h = if config.show_wt_editor {
        WT_VIEW_MIN_HEIGHT + GRID_UNIT
    } else {
        0.0
    };

    let scope_rect = Rect::from_min_max(
        inner.min,
        egui::pos2(inner.max.x, inner.min.y + SCOPE_STRIP_HEIGHT),
    );
    let content_top = scope_rect.max.y + GRID_UNIT;

    let (strip_rect, morph_rect, views_rect) = if config.show_osc_column {
        let strip_rect = Rect::from_min_max(
            egui::pos2(inner.min.x, content_top),
            egui::pos2(inner.max.x, content_top + WT_STRIP_HEIGHT),
        );
        let morph_rect = if config.show_wt_editor {
            Rect::from_min_max(
                egui::pos2(inner.min.x, strip_rect.max.y + GRID_UNIT),
                egui::pos2(inner.max.x, strip_rect.max.y + GRID_UNIT + WT_MORPH_HEIGHT),
            )
        } else {
            Rect::NOTHING
        };
        let views_top = if config.show_wt_editor {
            morph_rect.max.y + GRID_UNIT
        } else {
            strip_rect.max.y + GRID_UNIT
        };
        let views_rect = if config.show_wt_editor {
            Rect::from_min_max(
                egui::pos2(inner.min.x, views_top),
                inner.max,
            )
        } else {
            Rect::NOTHING
        };
        (strip_rect, morph_rect, views_rect)
    } else {
        let views_rect = if config.show_wt_editor {
            Rect::from_min_max(
                egui::pos2(inner.min.x, inner.max.y - views_h),
                inner.max,
            )
        } else {
            Rect::NOTHING
        };
        let morph_rect = if config.show_wt_editor {
            Rect::from_min_max(
                egui::pos2(inner.min.x, views_rect.min.y - morph_h),
                egui::pos2(inner.max.x, views_rect.min.y - GRID_UNIT),
            )
        } else {
            Rect::NOTHING
        };
        let strip_bottom = if config.show_wt_editor {
            morph_rect.min.y - GRID_UNIT
        } else {
            inner.max.y
        };
        let strip_rect = Rect::from_min_max(
            egui::pos2(inner.min.x, strip_bottom - WT_STRIP_HEIGHT),
            egui::pos2(inner.max.x, strip_bottom),
        );
        (strip_rect, morph_rect, views_rect)
    };

    let bank_name = state.wt_bank_name.clone();

    if scope_rect.is_positive() {
        ui.allocate_ui_at_rect(scope_rect, |ui| {
            if let Some(ctx) = scope {
                draw_scope_strip(
                    ui,
                    scope_rect,
                    ScopeStripInput {
                        patch: preview_patch,
                        banks: ctx.banks,
                        bank_for_osc: ctx.bank_for_osc,
                        live: ctx.live,
                        is_playing: ctx.is_playing,
                        now_secs: ctx.now_secs,
                        state: ctx.state,
                    },
                );
            } else if let Some(b) = bank.as_deref() {
                let bank_for_osc: &dyn Fn(usize) -> usize = &|_| 0;
                let mut strip_state = ScopeStripState::default();
                draw_scope_strip(
                    ui,
                    scope_rect,
                    ScopeStripInput {
                        patch: preview_patch,
                        banks: std::slice::from_ref(b),
                        bank_for_osc: &bank_for_osc,
                        live: None,
                        is_playing: false,
                        now_secs: ui.input(|i| i.time),
                        state: &mut strip_state,
                    },
                );
            }
        });
    }

    if config.show_wt_editor && morph_rect.is_positive() {
        ui.allocate_ui_at_rect(morph_rect, |ui| {
            let morph = WtMorph {
                frame_a: &mut state.wt_morph_a,
                frame_b: &mut state.wt_morph_b,
                amount: &mut state.wt_morph_amount,
                position: &mut state.wt_position,
            };
            if morph.show(ui).changed {
                sync_osc_position_from_wt(state);
                sync_morph_from_active_tab(state);
                actions.params_changed = true;
            }
        });
    }

    if config.show_wt_editor && views_rect.is_positive() {
        ui.allocate_ui_at_rect(views_rect, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = GRID_UNIT;
                let half_w = (ui.available_width() - GRID_UNIT) * 0.5;
                ui.allocate_ui_with_layout(
                    egui::vec2(half_w, WT_VIEW_MIN_HEIGHT),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let view = WtView2d {
                            position: state.wt_position,
                            bank: bank.as_deref_mut(),
                            bank_name: Some(bank_name.as_str()),
                            tool: &mut state.wt_edit_tool,
                        };
                        if view.show(ui).frame_edited {
                            actions.frame_edited = true;
                        }
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(half_w, WT_VIEW_MIN_HEIGHT),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        WtView3d {
                            position: state.wt_position,
                            bank: bank.as_deref(),
                        }
                        .show(ui);
                    },
                );
            });
        });
    }

    ui.allocate_ui_at_rect(strip_rect, |ui| {
        let strip = WtStrip {
            position: &mut state.wt_position,
            bank: bank.as_deref(),
            bank_name: Some(bank_name.as_str()),
            visible_frames: 16,
        };
        if strip.show(ui).changed {
            sync_osc_position_from_wt(state);
            state.wt_morph_amount =
                morph_amount_for_position(state.wt_morph_a, state.wt_morph_b, state.wt_position);
            sync_morph_from_active_tab(state);
            actions.params_changed = true;
        }
    });
}

