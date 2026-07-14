//! Morph A→B position controls — slot-aware when quantization is enabled.

use egui::{FontId, Sense, Ui, Vec2};
use reelsynth::patch::WaveSlot;
use reelsynth_ui_theme::Tokens;

use crate::layout::WT_MORPH_HEIGHT;
use crate::region::region;

use super::slots::{frame_to_slot_coord, slot_coord_to_frame};

pub struct WtMorph<'a> {
    pub frame_a: &'a mut f32,
    pub frame_b: &'a mut f32,
    pub amount: &'a mut f32,
    pub position: &'a mut f32,
    pub wave_quant: u8,
    pub wave_slots: &'a [WaveSlot],
}

pub struct WtMorphResponse {
    pub changed: bool,
}

impl<'a> WtMorph<'a> {
    pub fn show(self, ui: &mut Ui) -> WtMorphResponse {
        let tokens = Tokens::default();
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), WT_MORPH_HEIGHT),
            Sense::hover(),
        );

        let mut changed = false;
        let mut endpoints_changed = false;
        let slot_mode = self.wave_quant > 0;
        let max_slot = self.wave_quant.saturating_sub(1) as f32;

        region(ui, rect, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                ui.label(
                    egui::RichText::new("Morph")
                        .size(10.0)
                        .color(tokens.text_muted),
                );

                if slot_mode {
                    let mut slot_a = frame_to_slot_coord(self.wave_slots, *self.frame_a);
                    let mut slot_b = frame_to_slot_coord(self.wave_slots, *self.frame_b);
                    endpoints_changed |= morph_slot_field(ui, "A", &mut slot_a, max_slot);
                    endpoints_changed |= morph_slot_field(ui, "B", &mut slot_b, max_slot);
                    if endpoints_changed {
                        *self.frame_a = slot_coord_to_frame(self.wave_slots, slot_a);
                        *self.frame_b = slot_coord_to_frame(self.wave_slots, slot_b);
                    }
                } else {
                    endpoints_changed |= morph_frame_field(ui, "A", self.frame_a);
                    endpoints_changed |= morph_frame_field(ui, "B", self.frame_b);
                    let max_frame = 255.0_f32;
                    *self.frame_a = self.frame_a.clamp(0.0, max_frame);
                    *self.frame_b = self.frame_b.clamp(0.0, max_frame);
                }

                if endpoints_changed {
                    *self.position = morph_position(*self.frame_a, *self.frame_b, *self.amount);
                    changed = true;
                }

                let slider_label = format!("{:.0}%", *self.amount * 100.0_f32);
                let mut amount = *self.amount;
                let slider = egui::Slider::new(&mut amount, 0.0..=1.0)
                    .show_value(false)
                    .text(slider_label);
                if ui.add(slider).changed() {
                    *self.amount = amount;
                    *self.position = morph_position(*self.frame_a, *self.frame_b, amount);
                    changed = true;
                }

                ui.label(
                    egui::RichText::new(format!("pos {:.0}", *self.position))
                        .font(FontId::monospace(10.0))
                        .color(tokens.text_muted),
                );
            });
        });

        WtMorphResponse { changed }
    }
}

fn morph_frame_field(ui: &mut Ui, label: &str, value: &mut f32) -> bool {
    let tokens = Tokens::default();
    let mut frame = value.round();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(10.0)
                .color(tokens.text_muted),
        );
        let drag = egui::DragValue::new(&mut frame)
            .range(0.0..=255.0)
            .speed(1.0);
        if ui.add(drag).changed() {
            *value = frame;
            changed = true;
        }
    });
    changed
}

fn morph_slot_field(ui: &mut Ui, label: &str, value: &mut f32, max_slot: f32) -> bool {
    let tokens = Tokens::default();
    let mut slot = value.round();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(10.0)
                .color(tokens.text_muted),
        );
        let drag = egui::DragValue::new(&mut slot)
            .range(0.0..=max_slot)
            .speed(1.0);
        if ui.add(drag).changed() {
            *value = slot;
            changed = true;
        }
    });
    changed
}

/// Linear interpolation between frame indices A and B.
pub fn morph_position(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

/// Normalized morph amount for a position between A and B.
pub fn morph_amount_for_position(a: f32, b: f32, position: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        ((position - a) / (b - a)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reelsynth::patch::WaveSlot;

    #[test]
    fn morph_position_endpoints() {
        assert!((morph_position(0.0, 255.0, 0.0) - 0.0).abs() < 1e-5);
        assert!((morph_position(0.0, 255.0, 1.0) - 255.0).abs() < 1e-5);
        assert!((morph_position(100.0, 200.0, 0.5) - 150.0).abs() < 1e-5);
    }

    #[test]
    fn morph_amount_for_position_roundtrip() {
        let a = 20.0;
        let b = 180.0;
        let amount = 0.35;
        let pos = morph_position(a, b, amount);
        let back = morph_amount_for_position(a, b, pos);
        assert!((back - amount).abs() < 1e-4);
    }

    #[test]
    fn slot_coord_roundtrip() {
        let slots: Vec<WaveSlot> = (0..16)
            .map(|i| WaveSlot {
                frame: i as f32 * 17.0,
                label: String::new(),
            })
            .collect();
        let frame = slot_coord_to_frame(&slots, 7.5);
        let back = frame_to_slot_coord(&slots, frame);
        assert!((back - 7.5).abs() < 0.6);
    }
}
