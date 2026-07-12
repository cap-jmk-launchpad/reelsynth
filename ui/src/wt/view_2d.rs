use egui::{Color32, Pos2, Rect, Sense, Shape, Ui, Vec2};
use reelsynth::WavetableBank;
use reelsynth_ui_theme::Tokens;

use crate::layout::{RADIUS_SM, WT_VIEW_MIN_HEIGHT};

use super::waveform::{frame_index, peak_point, waveform_points};

pub struct WtView2d<'a> {
    pub position: f32,
    pub bank: Option<&'a WavetableBank>,
    pub bank_name: Option<&'a str>,
}

impl WtView2d<'_> {
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

        let frame_idx = self
            .bank
            .map(|b| frame_index(self.position, b.num_frames))
            .unwrap_or(0);
        let label = if let Some(name) = self.bank_name {
            format!("2D Waveform · {name} · frame {frame_idx}")
        } else {
            format!("2D Waveform · frame {frame_idx}")
        };
        painter.text(
            Pos2::new(rect.min.x + 8.0, rect.min.y + 6.0),
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(10.0),
            tokens.text_muted,
        );

        let inner = rect.shrink2(egui::vec2(8.0, 20.0));
        let mid_y = inner.center().y;
        painter.line_segment(
            [Pos2::new(inner.min.x, mid_y), Pos2::new(inner.max.x, mid_y)],
            egui::Stroke::new(1.0_f32, tokens.border),
        );

        let wave = if let Some(bank) = self.bank {
            let frame = bank.frame(frame_idx);
            waveform_points(frame, inner, 256, 0.42)
        } else {
            placeholder_wave(inner, mid_y)
        };

        if wave.len() >= 2 {
            let mut fill = wave.clone();
            fill.push(Pos2::new(inner.max.x, mid_y));
            fill.push(Pos2::new(inner.min.x, mid_y));
            painter.add(Shape::convex_polygon(
                fill,
                tokens.accent.gamma_multiply(0.35),
                egui::Stroke::NONE,
            ));
            painter.add(Shape::line(
                wave.clone(),
                egui::Stroke::new(2.0_f32, accent_ui),
            ));

            if let Some(peak) = peak_point(&wave) {
                painter.circle_filled(peak, 4.0, tokens.accent);
                painter.circle_stroke(peak, 4.0, egui::Stroke::new(1.0_f32, tokens.accent_on));
            }
        }

        rect
    }
}

fn placeholder_wave(inner: Rect, mid_y: f32) -> Vec<Pos2> {
    (0..=128)
        .map(|i| {
            let t = i as f32 / 128.0;
            let x = egui::lerp(inner.min.x..=inner.max.x, t);
            let y = mid_y + (t * std::f32::consts::TAU * 2.0).sin() * inner.height() * 0.35;
            Pos2::new(x, y)
        })
        .collect()
}
