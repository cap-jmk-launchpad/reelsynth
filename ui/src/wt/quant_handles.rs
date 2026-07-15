//! Quant-snapped waveform knob handles for slot-based sample editing.
//!
//! Knobs sit on the drawn waveform at each quant X. Vertical drag edits
//! amplitude at that control point (wave height), not the slot→frame morph map
//! (that stays on the Curve tool).

use egui::{CursorIcon, Pos2, Rect, Sense, Ui};
use reelsynth::WavetableBank;
use reelsynth_ui_theme::{ACCENT_UI, Tokens};

use super::slots::effective_quant_count;

const HANDLE_RADIUS: f32 = 6.0;
const WAVE_AMP: f32 = 0.42;

pub struct QuantHandleEditor<'a> {
    pub plot_rect: Rect,
    pub wave_quant: u8,
    pub bank: &'a mut WavetableBank,
    pub frame_idx: usize,
}

pub struct QuantHandleResponse {
    pub frame_edited: bool,
    pub hovered_slot: Option<usize>,
    pub dragged_slot: Option<usize>,
    pub status_label: Option<String>,
}

impl QuantHandleEditor<'_> {
    pub fn show(self, ui: &mut Ui) -> QuantHandleResponse {
        let tokens = Tokens::default();
        let accent_ui = ACCENT_UI;
        let slot_count = effective_quant_count(self.wave_quant).max(1);
        let show_all_knobs = self.wave_quant <= 64;

        if self.wave_quant == 0 || self.frame_idx >= self.bank.num_frames {
            return QuantHandleResponse {
                frame_edited: false,
                hovered_slot: None,
                dragged_slot: None,
                status_label: None,
            };
        }

        let drag_slot_id = ui.id().with("quant_drag_slot");
        let locked_slot: Option<usize> = ui.ctx().data(|d| d.get_temp(drag_slot_id));

        let sense = Sense::click_and_drag();
        let response = ui.allocate_rect(self.plot_rect, sense);

        let mut frame_edited = false;
        let mut hovered_slot = None;
        let mut dragged_slot = locked_slot;
        let mut status_label = None;

        let pointer = response
            .interact_pointer_pos()
            .filter(|p| self.plot_rect.contains(*p));

        if response.drag_started() {
            if let Some(pos) = pointer {
                let slot = snap_x_to_slot(pos.x, self.plot_rect, slot_count);
                ui.ctx().data_mut(|d| d.insert_temp(drag_slot_id, slot));
                dragged_slot = Some(slot);
            }
        }

        if !response.dragged() && response.drag_stopped() {
            ui.ctx().data_mut(|d| d.remove::<usize>(drag_slot_id));
            dragged_slot = None;
        }

        if let Some(pos) = pointer {
            if locked_slot.is_none() && !response.dragged() {
                hovered_slot = Some(nearest_slot(pos.x, self.plot_rect, slot_count));
            }
        }

        // Sample at the same phases as slot_x (i / (n−1)) so knobs sit on the wave.
        let points = quant_control_points(self.bank.frame(self.frame_idx), slot_count);

        if let Some(slot) = dragged_slot {
            if response.dragged() {
                if let Some(pos) = pointer {
                    let sample = y_to_sample(pos.y, self.plot_rect);
                    let prev = points.get(slot).copied().unwrap_or(0.0);
                    if (prev - sample).abs() > 1e-4 {
                        apply_quant_slot_amplitude(
                            self.bank.frame_mut(self.frame_idx),
                            slot,
                            slot_count,
                            sample,
                        );
                        frame_edited = true;
                        status_label = Some(format!(
                            "Slot {} → amp {:+.2}",
                            slot + 1,
                            sample
                        ));
                    }
                }
            }
        }

        let points = if frame_edited {
            quant_control_points(self.bank.frame(self.frame_idx), slot_count)
        } else {
            points
        };

        let painter = ui.painter_at(self.plot_rect);
        for i in 0..slot_count {
            let show = show_all_knobs
                || hovered_slot == Some(i)
                || dragged_slot == Some(i)
                || i == 0;
            if !show {
                continue;
            }
            let x = slot_x(i, slot_count, self.plot_rect);
            let sample = points.get(i).copied().unwrap_or(0.0);
            let y = sample_to_y(sample, self.plot_rect);
            let center = Pos2::new(x, y);
            let active = dragged_slot == Some(i) || hovered_slot == Some(i);
            let radius = if active {
                HANDLE_RADIUS * 1.25
            } else {
                HANDLE_RADIUS
            };
            let fill = if active {
                accent_ui.gamma_multiply(0.35)
            } else {
                tokens.surface2
            };
            painter.circle_filled(center, radius, fill);
            painter.circle_stroke(
                center,
                radius,
                egui::Stroke::new(if active { 2.0 } else { 1.0 }, accent_ui),
            );

            if active {
                let band_w = self.plot_rect.width() / slot_count as f32;
                let band = Rect::from_center_size(
                    Pos2::new(x, self.plot_rect.center().y),
                    egui::vec2(band_w, self.plot_rect.height()),
                );
                painter.rect_filled(band, 0.0, tokens.accent.gamma_multiply(0.12));
            }
        }

        if response.hovered() || response.dragged() {
            let cursor = if response.dragged() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Grab
            };
            ui.ctx().set_cursor_icon(cursor);
        }

        QuantHandleResponse {
            frame_edited,
            hovered_slot,
            dragged_slot,
            status_label,
        }
    }
}

/// Write amplitude across the Voronoi phase band of a quant slot so vertical
/// drag clearly reshapes the wave around that knob.
pub fn apply_quant_slot_amplitude(
    frame: &mut [f32],
    slot: usize,
    slot_count: usize,
    sample: f32,
) {
    if frame.is_empty() || slot_count == 0 {
        return;
    }
    let sample = sample.clamp(-1.0, 1.0);
    let len = frame.len() as f32;
    let denom = (slot_count - 1).max(1) as f32;
    let left = if slot == 0 {
        0.0
    } else {
        (slot as f32 - 0.5) / denom
    };
    let right = if slot + 1 >= slot_count {
        1.0
    } else {
        (slot as f32 + 0.5) / denom
    };
    let i0 = (left * len).floor() as usize;
    let i1 = ((right * len).ceil() as usize).min(frame.len()).max(i0 + 1);
    for s in &mut frame[i0..i1] {
        *s = sample;
    }
}

/// Control-point amplitudes at each quant slot phase (aligned with [`slot_x`]).
pub fn quant_control_points(frame: &[f32], slot_count: usize) -> Vec<f32> {
    let n = slot_count.max(1);
    (0..n)
        .map(|i| sample_at_quant_phase(frame, i, n))
        .collect()
}

pub fn sample_at_quant_phase(frame: &[f32], slot: usize, slot_count: usize) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let phase = if slot_count <= 1 {
        0.0
    } else {
        slot as f32 / (slot_count - 1) as f32
    };
    let idx = ((phase * frame.len() as f32).round() as usize).min(frame.len() - 1);
    frame[idx]
}

pub fn snap_x_to_slot(x: f32, plot: Rect, slot_count: usize) -> usize {
    nearest_slot(x, plot, slot_count)
}

pub fn nearest_slot(x: f32, plot: Rect, slot_count: usize) -> usize {
    if slot_count <= 1 {
        return 0;
    }
    let t = ((x - plot.min.x) / plot.width()).clamp(0.0, 1.0);
    (t * (slot_count - 1) as f32).round() as usize
}

pub fn slot_x(slot: usize, slot_count: usize, plot: Rect) -> f32 {
    if slot_count <= 1 {
        return plot.center().x;
    }
    let t = slot as f32 / (slot_count - 1) as f32;
    egui::lerp(plot.min.x..=plot.max.x, t)
}

/// Map sample amplitude (−1..1) to plot Y (matches [`super::waveform::waveform_points`]).
pub fn sample_to_y(sample: f32, plot: Rect) -> f32 {
    plot.center().y - sample.clamp(-1.0, 1.0) * plot.height() * WAVE_AMP
}

pub fn y_to_sample(y: f32, plot: Rect) -> f32 {
    ((plot.center().y - y) / (plot.height() * WAVE_AMP)).clamp(-1.0, 1.0)
}

/// Legacy alias used by curve morph overlays (slot frame index → Y).
pub fn frame_to_y(frame: f32, plot: Rect) -> f32 {
    let norm = (frame / 255.0).clamp(-1.0, 1.0);
    plot.center().y - norm * plot.height() * WAVE_AMP
}

pub fn y_to_frame(y: f32, plot: Rect) -> f32 {
    let norm = (plot.center().y - y) / (plot.height() * WAVE_AMP);
    (norm * 255.0).clamp(-255.0, 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Rect;

    #[test]
    fn snap_x_to_slot_boundaries() {
        let plot = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(160.0, 80.0));
        assert_eq!(snap_x_to_slot(plot.min.x, plot, 16), 0);
        assert_eq!(snap_x_to_slot(plot.max.x, plot, 16), 15);
        assert_eq!(snap_x_to_slot(plot.center().x, plot, 16), 8);
    }

    #[test]
    fn sample_y_roundtrip() {
        let plot = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 100.0));
        for sample in [-1.0_f32, -0.5, 0.0, 0.42, 1.0] {
            let y = sample_to_y(sample, plot);
            let back = y_to_sample(y, plot);
            assert!(
                (back - sample).abs() < 1e-4,
                "roundtrip {sample} -> {back}"
            );
        }
    }

    #[test]
    fn knob_y_matches_waveform_intersection() {
        let plot = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 100.0));
        let frame: Vec<f32> = (0..2048)
            .map(|i| (i as f32 / 2048.0 * std::f32::consts::TAU).sin())
            .collect();
        let slot_count = 16;
        let points = quant_control_points(&frame, slot_count);
        let wave = crate::wt::waveform::waveform_points(&frame, plot, 256, WAVE_AMP);
        assert!(wave.len() >= 2);

        for i in 0..slot_count {
            let x = slot_x(i, slot_count, plot);
            let knob_y = sample_to_y(points[i], plot);
            let nearest = wave
                .windows(2)
                .filter_map(|seg| {
                    let (a, b) = (seg[0], seg[1]);
                    if (a.x - x).abs() < 1e-3 {
                        return Some(a.y);
                    }
                    if (a.x - x) * (b.x - x) <= 0.0 && (b.x - a.x).abs() > 1e-6 {
                        let t = (x - a.x) / (b.x - a.x);
                        return Some(a.y + (b.y - a.y) * t);
                    }
                    None
                })
                .next()
                .unwrap_or(knob_y);
            assert!(
                (nearest - knob_y).abs() < plot.height() * 0.08,
                "slot {i}: knob_y={knob_y} wave_y={nearest}"
            );
        }
    }

    #[test]
    fn vertical_edit_changes_frame_amplitude() {
        let mut bank = WavetableBank::factory_sine();
        let slot_count = 8;
        let before = sample_at_quant_phase(bank.frame(0), 3, slot_count);
        apply_quant_slot_amplitude(bank.frame_mut(0), 3, slot_count, 0.85);
        let after = sample_at_quant_phase(bank.frame(0), 3, slot_count);
        assert!((after - before).abs() > 0.2);
        assert!((after - 0.85).abs() < 1e-3);
    }

    #[test]
    fn frame_y_roundtrip() {
        let plot = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 100.0));
        let frame = 128.0;
        let y = frame_to_y(frame, plot);
        let back = y_to_frame(y, plot);
        assert!((back - frame).abs() < 2.0, "roundtrip {frame} -> {back}");
    }
}
