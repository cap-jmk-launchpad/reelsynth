use egui::{Rect, Ui};
use reelsynth::Patch;
use reelsynth_ui_theme::Tokens;

use super::*;
pub(super) fn draw_level_meter(ui: &mut Ui) {
    let tokens = Tokens::default();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 48.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let bar_w = 10.0;
        let gap = 6.0;
        let cx = rect.center().x;
        for (i, &level) in [0.62_f32, 0.48_f32].iter().enumerate() {
            let x = cx + (i as f32 - 0.5) * (bar_w + gap);
            let bar_h = rect.height() * level;
            let bar = egui::Rect::from_min_max(
                egui::pos2(x - bar_w * 0.5, rect.max.y - bar_h),
                egui::pos2(x + bar_w * 0.5, rect.max.y),
            );
            painter.rect_filled(bar, 2.0, tokens.accent.gamma_multiply(0.85));
        }
    }
}

pub(super) fn draw_piano_wrap(ui: &mut Ui, rect: Rect, state: &mut UiState, actions: &mut ShellActions) {
    ui.allocate_ui_at_rect(rect, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(SPACE_SM, GRID_UNIT))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let (_, piano) = PianoKeyboard::new(&state.keys_down).show(ui);
                    if let Some(n) = piano.note_on {
                        actions.note_on = Some(n);
                    }
                    if let Some(n) = piano.note_off {
                        actions.note_off = Some(n);
                    }
                });
            });
    });
}

pub(super) fn draw_footer(ui: &mut Ui, rect: Rect, state: &UiState) {
    let tokens = Tokens::default();
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.set_min_height(rect.height());
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(SPACE_SM, 0.0))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_min_height(rect.height());
                    ui.spacing_mut().item_spacing.x = GRID_UNIT;

                    ui.label(
                        egui::RichText::new("Performance")
                            .size(11.0)
                            .color(tokens.text_muted),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.set_width(ui.available_width());
                        let wt = state.wt_position.round() as i32;
                        ui.label(
                            egui::RichText::new(format!(
                                "WT {wt} · Cutoff {}",
                                format_cutoff(state.filter_cutoff)
                            ))
                            .font(FontId::monospace(11.0))
                            .color(tokens.text_muted),
                        );
                    });
                });
            });
    });
}

pub(super) fn draw_piano_toggle(ui: &mut Ui, on: bool) -> egui::Response {
    let tokens = Tokens::default();
    let label = "Piano";
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        FontId::proportional(11.0),
        if on { tokens.accent_on } else { tokens.text_muted },
    );
    let size = egui::vec2(galley.size().x + 20.0, galley.size().y + 8.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let fill = if on { tokens.accent } else { tokens.bg_muted };
        let stroke = if on {
            Color32::from_rgb(0x2a, 0x6b, 0x8a)
        } else {
            tokens.border
        };
        painter.rect_filled(rect, 6.0, fill);
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, stroke));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0),
            if on { tokens.accent_on } else { tokens.text_muted },
        );
    }
    response
}

pub(super) fn format_cutoff(hz: f32) -> String {
    if hz >= 1000.0 {
        format!("{:.1} kHz", hz / 1000.0)
    } else {
        format!("{:.0} Hz", hz)
    }
}
