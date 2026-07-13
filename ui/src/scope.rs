//! Four-tap signal-chain scope strip (Osc → Filter → FX → Out).

use egui::{Color32, Pos2, Rect, Shape, Ui};
use reelsynth::{render_scope_previews, Patch, WavetableBank};
use reelsynth_ui_theme::Tokens;

use crate::layout::{GRID_UNIT, RADIUS_SM, SPACE_SM};
use crate::wt::waveform_points;

pub const SCOPE_STRIP_HEIGHT: f32 = 88.0;
const SCOPE_SAMPLE_COUNT: usize = 48;

const STAGE_LABELS: [&str; 4] = ["Osc", "Filter", "FX", "Out"];
const STAGE_COLORS: [Color32; 4] = [
    Color32::from_rgb(0x5b, 0xc0, 0xde),
    Color32::from_rgb(0x9b, 0x7e, 0xde),
    Color32::from_rgb(0xde, 0x9b, 0x7e),
    Color32::from_rgb(0x4a, 0xde, 0x80),
];

pub fn draw_scope_strip(
    ui: &mut Ui,
    rect: Rect,
    patch: &Patch,
    banks: &[WavetableBank],
    bank_for_osc: impl Fn(usize) -> usize + Copy,
) {
    let tokens = Tokens::default();
    let previews = render_scope_previews(banks, bank_for_osc, patch, SCOPE_SAMPLE_COUNT);
    let inner = rect.shrink(SPACE_SM);

    ui.allocate_ui_at_rect(inner, |ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Signal chain")
                    .size(10.0)
                    .color(tokens.text_muted),
            );
            ui.add_space(4.0);

            let gap = GRID_UNIT;
            let cell_w = ((inner.width() - gap * 3.0) / 4.0).max(40.0);
            let cell_h = (inner.height() - 18.0).max(48.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                draw_scope_cell(ui, &previews.osc, STAGE_LABELS[0], STAGE_COLORS[0], cell_w, cell_h);
                draw_arrow(ui, cell_h);
                draw_scope_cell(ui, &previews.filter, STAGE_LABELS[1], STAGE_COLORS[1], cell_w, cell_h);
                draw_arrow(ui, cell_h);
                draw_scope_cell(ui, &previews.fx, STAGE_LABELS[2], STAGE_COLORS[2], cell_w, cell_h);
                draw_arrow(ui, cell_h);
                draw_scope_cell(ui, &previews.out, STAGE_LABELS[3], STAGE_COLORS[3], cell_w, cell_h);
            });
        });
    });
}

fn draw_scope_cell(
    ui: &mut Ui,
    tap: &reelsynth::ScopeTap,
    label: &str,
    accent: Color32,
    width: f32,
    height: f32,
) {
    let tokens = Tokens::default();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, RADIUS_SM, tokens.surface2);
    painter.rect_stroke(rect, RADIUS_SM, egui::Stroke::new(1.0_f32, tokens.border));

    painter.text(
        egui::pos2(rect.min.x + 6.0, rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::monospace(9.0),
        accent,
    );

    let wave_rect = rect.shrink2(egui::vec2(6.0, 16.0));
    let points = waveform_points(&tap.samples, wave_rect, SCOPE_SAMPLE_COUNT.min(tap.samples.len()), 0.42);
    if points.len() >= 2 {
        painter.add(Shape::line(points, egui::Stroke::new(1.25_f32, accent.gamma_multiply(0.9))));
        let mid = wave_rect.center().y;
        painter.line_segment(
            [Pos2::new(wave_rect.min.x, mid), Pos2::new(wave_rect.max.x, mid)],
            egui::Stroke::new(0.5_f32, tokens.border),
        );
    }
}

fn draw_arrow(ui: &mut Ui, height: f32) {
    let tokens = Tokens::default();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, height), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let c = rect.center();
        ui.painter_at(rect).line_segment(
            [
                Pos2::new(rect.min.x + 2.0, c.y),
                Pos2::new(rect.max.x - 2.0, c.y),
            ],
            egui::Stroke::new(1.0_f32, tokens.text_muted),
        );
    }
}
