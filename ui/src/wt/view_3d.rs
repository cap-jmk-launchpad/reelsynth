use egui::{Color32, Pos2, Rect, Sense, Shape, Ui, Vec2};
use reelsynth::WavetableBank;
use reelsynth_ui_theme::Tokens;

use crate::layout::{RADIUS_SM, WT_VIEW_MIN_HEIGHT};

pub struct WtView3d<'a> {
    pub position: f32,
    pub bank: Option<&'a WavetableBank>,
}

impl WtView3d<'_> {
    pub fn show(self, ui: &mut Ui) -> Rect {
        let tokens = Tokens::default();
        let accent_ui = Color32::from_rgb(0x2a, 0x6b, 0x8a);
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), WT_VIEW_MIN_HEIGHT),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return rect;
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, RADIUS_SM, tokens.bg);
        painter.rect_stroke(
            rect,
            RADIUS_SM,
            egui::Stroke::new(1.0_f32, tokens.border),
        );

        painter.text(
            Pos2::new(rect.min.x + 8.0, rect.min.y + 6.0),
            egui::Align2::LEFT_TOP,
            "3D Mesh",
            egui::FontId::proportional(10.0),
            tokens.text_muted,
        );

        let inner = rect.shrink2(egui::vec2(8.0, 20.0));
        paint_grid(&painter, inner, tokens.border);

        if let Some(bank) = self.bank {
            paint_mesh_from_bank(&painter, inner, bank, self.position, accent_ui, tokens.accent);
        } else {
            paint_placeholder_mesh(&painter, inner, accent_ui);
        }

        rect
    }
}

fn paint_grid(painter: &egui::Painter, rect: Rect, border: Color32) {
    let step = 24.0;
    let mut x = rect.min.x;
    while x <= rect.max.x {
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            egui::Stroke::new(0.5_f32, border.gamma_multiply(0.5)),
        );
        x += step;
    }
    let mut y = rect.min.y;
    while y <= rect.max.y {
        painter.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
            egui::Stroke::new(0.5_f32, border.gamma_multiply(0.5)),
        );
        y += step;
    }
}

fn paint_mesh_from_bank(
    painter: &egui::Painter,
    rect: Rect,
    bank: &WavetableBank,
    position: f32,
    accent_ui: Color32,
    accent: Color32,
) {
    let num_slices = 12usize;
    let max_frame = bank.num_frames.saturating_sub(1).max(1);
    let center = (position / max_frame as f32).clamp(0.0, 1.0);
    let half = (num_slices as f32 * 0.45) as i32;

    for s in 0..num_slices {
        let fi = ((center * max_frame as f32) as i32 + s as i32 - half)
            .clamp(0, max_frame as i32) as usize;
        let depth = (s as f32 / num_slices as f32 - 0.5).abs();
        let alpha = (1.0 - depth * 1.6).clamp(0.15, 1.0);
        let z_offset = (s as f32 / num_slices as f32 - 0.5) * rect.width() * 0.35;
        let y_offset = depth * rect.height() * 0.25;

        let frame = bank.frame(fi);
        let step = (frame.len() / 48).max(1);
        let mid_y = rect.center().y + y_offset;
        let left = rect.min.x + rect.width() * 0.1 + z_offset;
        let width = rect.width() * 0.75;

        let points: Vec<Pos2> = frame
            .iter()
            .step_by(step)
            .take(49)
            .enumerate()
            .map(|(i, sample)| {
                let t = i as f32 / 48.0;
                Pos2::new(
                    left + t * width,
                    mid_y - sample * rect.height() * 0.28,
                )
            })
            .collect();

        if points.len() >= 2 {
            let is_active = s == num_slices / 2;
            let color = if is_active {
                accent
            } else {
                accent_ui.gamma_multiply(alpha)
            };
            let width_stroke = if is_active { 2.0_f32 } else { 1.0_f32 };
            painter.add(Shape::line(points, egui::Stroke::new(width_stroke, color)));
        }
    }
}

fn paint_placeholder_mesh(painter: &egui::Painter, rect: Rect, accent_ui: Color32) {
    for i in 0..8 {
        let t = i as f32 / 7.0;
        let y_off = t * rect.height() * 0.35;
        let x_off = (t - 0.5) * rect.width() * 0.2;
        let points: Vec<Pos2> = (0..=32)
            .map(|j| {
                let u = j as f32 / 32.0;
                let x = rect.min.x + x_off + u * rect.width() * 0.8;
                let y = rect.center().y + y_off
                    + (u * std::f32::consts::TAU * 2.0 + t * 2.0).sin() * rect.height() * 0.2;
                Pos2::new(x, y)
            })
            .collect();
        painter.add(Shape::line(
            points,
            egui::Stroke::new(1.0_f32, accent_ui.gamma_multiply(0.4 + t * 0.5)),
        ));
    }
}
