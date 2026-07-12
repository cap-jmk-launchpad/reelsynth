use std::collections::HashSet;

use egui::{FontId, Pos2, Rect, Shape, Ui};
use reelsynth::WavetableBank;
use reelsynth_ui_theme::Tokens;

use crate::layout::{self, S1Layout, GRID_UNIT, SPACE_SM, WT_STRIP_HEIGHT};
use crate::widgets::{Knob, KnobSize, KnobStyle, PianoKeyboard, panel, panel_disabled};
use crate::wt::WtStrip;

#[derive(Default)]
pub struct S1Actions {
    pub params_changed: bool,
    pub note_on: Option<u8>,
    pub note_off: Option<u8>,
    pub open_preset: bool,
    pub save_preset: bool,
}

pub struct S1State {
    pub wt_position: f32,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub keys_down: HashSet<u8>,
    pub piano_visible: bool,
    pub preset_name: String,
    pub preset_category: String,
    pub status: String,
    pub midi_device: String,
}

impl Default for S1State {
    fn default() -> Self {
        Self {
            wt_position: 108.0,
            filter_cutoff: 1200.0,
            filter_resonance: 0.3,
            keys_down: HashSet::new(),
            piano_visible: true,
            preset_name: "Factory Lead".into(),
            preset_category: "Bass · Wavetable · Saw Morph".into(),
            status: "Audio OK — click keys or use QWERTY row (Z–M)".into(),
            midi_device: "Default".into(),
        }
    }
}

pub fn draw_s1(
    ui: &mut Ui,
    screen: Rect,
    state: &mut S1State,
    bank: Option<&WavetableBank>,
) -> S1Actions {
    let layout = S1Layout::compute(screen, state.piano_visible);
    let tokens = Tokens::default();
    let mut actions = S1Actions::default();

    let painter = ui.painter_at(screen);
    let border = egui::Stroke::new(1.0_f32, tokens.border);
    painter.rect_filled(layout.header, 0.0, tokens.surface2);
    painter.line_segment(
        [layout.header.left_bottom(), layout.header.right_bottom()],
        border,
    );
    painter.rect_filled(layout.main, 0.0, tokens.bg);
    painter.rect_filled(layout.rail, 0.0, tokens.bg);
    painter.line_segment(
        [layout.rail.left_top(), layout.rail.left_bottom()],
        border,
    );
    if state.piano_visible && layout.piano_wrap.is_positive() {
        painter.rect_filled(layout.piano_wrap, 0.0, tokens.surface2);
        painter.line_segment(
            [layout.piano_wrap.left_top(), layout.piano_wrap.right_top()],
            border,
        );
    }
    painter.rect_filled(layout.footer, 0.0, tokens.surface2);
    painter.line_segment(
        [layout.footer.left_top(), layout.footer.right_top()],
        border,
    );

    draw_header(ui, layout.header, state, &mut actions);
    draw_center(ui, layout.center, state, bank, &mut actions);
    draw_rail(ui, layout.rail, state, &mut actions);

    if state.piano_visible && layout.piano_wrap.is_positive() {
        draw_piano_wrap(ui, layout.piano_wrap, state, &mut actions);
    }

    draw_footer(ui, layout.footer, state);

    actions
}

fn draw_header(ui: &mut Ui, rect: Rect, state: &mut S1State, actions: &mut S1Actions) {
    let tokens = Tokens::default();
    ui.allocate_ui_at_rect(rect, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(SPACE_SM, 0.0))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = GRID_UNIT;
                    ui.label(
                        egui::RichText::new("REELSYNTH")
                            .font(FontId::proportional(15.0))
                            .strong()
                            .color(tokens.text),
                    );
                    ui.add_space(GRID_UNIT);
                    if ui.button("Open").clicked() {
                        actions.open_preset = true;
                    }
                    if ui.button("Save").clicked() {
                        actions.save_preset = true;
                    }
                    ui.separator();
                    ui.label("MIDI:");
                    egui::ComboBox::from_id_source("midi")
                        .selected_text(&state.midi_device)
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.midi_device, "Default".into(), "Default");
                            ui.selectable_value(&mut state.midi_device, "None".into(), "None (stub)");
                        });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            egui::RichText::new("CPU — · Poly 0/8")
                                .font(FontId::monospace(11.0))
                                .color(tokens.text_muted),
                        );
                    });
                });
            });
    });
}

fn draw_center(
    ui: &mut Ui,
    rect: Rect,
    state: &mut S1State,
    bank: Option<&WavetableBank>,
    actions: &mut S1Actions,
) {
    let inner = rect.shrink(SPACE_SM);
    let strip_rect = Rect::from_min_max(
        egui::pos2(inner.min.x, inner.max.y - WT_STRIP_HEIGHT),
        inner.max,
    );
    let hero_rect = Rect::from_min_max(inner.min, egui::pos2(inner.max.x, strip_rect.min.y - GRID_UNIT));

    draw_spectrum_hero(ui, hero_rect, state);

    ui.allocate_ui_at_rect(strip_rect, |ui| {
        let strip = WtStrip {
            position: &mut state.wt_position,
            bank,
            visible_frames: 16,
        };
        if strip.show(ui).changed {
            actions.params_changed = true;
        }
    });
}

fn draw_spectrum_hero(ui: &mut Ui, area: Rect, state: &S1State) {
    let tokens = Tokens::default();
    ui.allocate_ui_at_rect(area, |ui| {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = SPACE_SM;
            ui.label(
                egui::RichText::new(&state.preset_name)
                    .size(28.0)
                    .strong()
                    .color(tokens.text),
            );
            ui.label(
                egui::RichText::new(&state.preset_category)
                    .size(12.0)
                    .color(tokens.text_muted),
            );

            let viz_w = (area.width() - layout::SPACE_MD * 2.0).min(520.0);
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(viz_w, 180.0),
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 16.0, tokens.surface2);
            painter.rect_stroke(
                rect,
                16.0,
                egui::Stroke::new(1.0_f32, tokens.border),
            );

        let bar_heights: [f32; 32] = [
            58.0, 76.0, 100.0, 120.0, 130.0, 136.0, 140.0, 142.0, 138.0, 132.0, 124.0, 114.0,
            104.0, 96.0, 90.0, 84.0, 78.0, 72.0, 66.0, 60.0, 56.0, 52.0, 48.0, 44.0, 40.0, 36.0,
            32.0, 28.0, 24.0, 20.0, 16.0, 12.0,
        ];
            let inner = rect.shrink(GRID_UNIT);
            let bar_w = 8.0;
            let gap = 4.0;
            for (i, h) in bar_heights.iter().enumerate() {
                let x = inner.min.x + i as f32 * (bar_w + gap);
                let bar_h = h * (inner.height() / 160.0);
                let bar_rect = Rect::from_min_max(
                    Pos2::new(x, inner.max.y - bar_h),
                    Pos2::new(x + bar_w, inner.max.y),
                );
                painter.rect_filled(bar_rect, 1.0, tokens.accent.gamma_multiply(0.85));
            }

            let wave: Vec<Pos2> = (0..=64)
                .map(|i| {
                    let t = i as f32 / 64.0;
                    let x = egui::lerp(inner.min.x..=inner.max.x, t);
                    let y = inner.center().y
                        - (t * std::f32::consts::TAU * 2.0).sin() * inner.height() * 0.25;
                    Pos2::new(x, y)
                })
                .collect();
            painter.add(Shape::line(
                wave,
                egui::Stroke::new(1.5_f32, tokens.accent_on.gamma_multiply(0.6)),
            ));
        });
    });
}

fn draw_rail(ui: &mut Ui, rect: Rect, state: &mut S1State, actions: &mut S1Actions) {
    ui.allocate_ui_at_rect(rect, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::same(SPACE_SM))
            .show(ui, |ui| {
                ui.set_width(rect.width() - SPACE_SM * 2.0);
                ui.spacing_mut().item_spacing.y = SPACE_SM;

                panel(ui, "Performance", |ui| {
                    ui.horizontal_centered(|ui| {
                        let wt_frame = state.wt_position.round() as i32;
                        let r = Knob::new(&mut state.wt_position, 0.0..=255.0, "WT Position")
                            .size(KnobSize::Lg)
                            .style(KnobStyle::Wired)
                            .value_text(format!("{wt_frame}"))
                            .show(ui);
                        if r.changed {
                            actions.params_changed = true;
                        }
                    });
                });

                panel(ui, "Filter", |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_SM;
                        let cutoff_text = format_cutoff(state.filter_cutoff);
                        let r1 = Knob::new(&mut state.filter_cutoff, 40.0..=12000.0, "Cutoff")
                            .size(KnobSize::Lg)
                            .style(KnobStyle::Wired)
                            .logarithmic(true)
                            .value_text(cutoff_text)
                            .show(ui);
                        let res_text = format!("{:.2}", state.filter_resonance);
                        let r2 = Knob::new(&mut state.filter_resonance, 0.0..=0.95, "Resonance")
                            .size(KnobSize::Lg)
                            .style(KnobStyle::Wired)
                            .value_text(res_text)
                            .show(ui);
                        if r1.changed || r2.changed {
                            actions.params_changed = true;
                        }
                    });
                });

                panel_disabled(ui, "Amp Envelope", |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_SM;
                        for label in ["A", "D", "S", "R"] {
                            let mut v = 0.0_f32;
                            Knob::new(&mut v, 0.0..=1.0, label)
                                .size(KnobSize::Sm)
                                .style(KnobStyle::Disabled)
                                .value_text("—")
                                .show(ui);
                        }
                    });
                });

                panel_disabled(ui, "LFO", |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = SPACE_SM;
                        for label in ["Rate", "Depth"] {
                            let mut v = 0.0_f32;
                            Knob::new(&mut v, 0.0..=1.0, label)
                                .size(KnobSize::Sm)
                                .style(KnobStyle::Disabled)
                                .value_text("—")
                                .show(ui);
                        }
                    });
                });
            });
    });
}

fn draw_piano_wrap(ui: &mut Ui, rect: Rect, state: &mut S1State, actions: &mut S1Actions) {
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

fn draw_footer(ui: &mut Ui, rect: Rect, state: &mut S1State) {
    let tokens = Tokens::default();
    ui.allocate_ui_at_rect(rect, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(SPACE_SM, 0.0))
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = GRID_UNIT;
                    let label = if state.piano_visible {
                        "Piano ✓"
                    } else {
                        "Piano"
                    };
                    if ui.selectable_label(state.piano_visible, label).clicked() {
                        state.piano_visible = !state.piano_visible;
                    }
                    ui.label(
                        egui::RichText::new(format!("● {}", state.status))
                            .font(FontId::monospace(11.0))
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

fn format_cutoff(hz: f32) -> String {
    if hz >= 1000.0 {
        format!("{:.1} kHz", hz / 1000.0)
    } else {
        format!("{:.0} Hz", hz)
    }
}
