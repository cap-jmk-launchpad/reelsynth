//! Left oscillator column (S3) — Osc1–3 tabs, sub/noise + macro stubs.

use egui::Ui;
use reelsynth_ui_theme::Tokens;

use crate::layout::{GRID_UNIT, SPACE_SM};
use crate::widgets::{
    format_coarse, format_pan, format_unison, knob_value_label, tab_bar, Knob, KnobSize,
    KnobStyle, panel, panel_disabled,
};

const OSC_TABS: [&str; 3] = ["Osc 1", "Osc 2", "Osc 3"];
const OSC_TYPES: [&str; 5] = ["Wavetable", "Saw", "Square", "Triangle", "Pulse"];
const WARP_MODES: [&str; 3] = ["None", "Sync", "Bend"];

pub fn osc_type_index(ty: &str) -> usize {
    match ty.to_ascii_lowercase().as_str() {
        "saw" => 1,
        "square" => 2,
        "triangle" => 3,
        "pulse" => 4,
        _ => 0,
    }
}

pub fn osc_type_from_index(idx: usize) -> &'static str {
    match idx {
        1 => "saw",
        2 => "square",
        3 => "triangle",
        4 => "pulse",
        _ => "wavetable",
    }
}

pub fn warp_mode_index(mode: &str) -> usize {
    match mode.to_ascii_lowercase().as_str() {
        "sync" => 1,
        "bend" => 2,
        _ => 0,
    }
}

pub fn warp_mode_from_index(idx: usize) -> &'static str {
    match idx {
        1 => "sync",
        2 => "bend",
        _ => "none",
    }
}

pub struct OscColumnState<'a> {
    pub osc_tab: &'a mut usize,
    pub osc_type: &'a mut [usize; 3],
    pub osc_level: &'a mut [f32; 3],
    pub osc_pan: &'a mut [f32; 3],
    pub osc_coarse: &'a mut [f32; 3],
    pub osc_unison: &'a mut [u32; 3],
    pub osc_position: &'a mut [f32; 3],
    pub osc_pulse_width: &'a mut [f32; 3],
    pub osc_warp_mode: &'a mut [usize; 3],
    pub osc_warp_amount: &'a mut [f32; 3],
    pub unison_stereo_spread: &'a mut f32,
    pub sub_level: &'a mut f32,
    pub noise_level: &'a mut f32,
    pub macro_values: &'a mut [f32; 4],
}

pub struct OscColumnResult {
    pub changed: bool,
}

pub fn draw_osc_column(ui: &mut Ui, state: OscColumnState<'_>) -> OscColumnResult {
    let mut changed = false;

    egui::Frame::none()
        .inner_margin(egui::Margin::same(SPACE_SM))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = SPACE_SM;

            panel(ui, "Oscillators", |ui| {
                tab_bar(ui, &OSC_TABS, state.osc_tab);
                ui.add_space(GRID_UNIT);

                let idx = (*state.osc_tab).min(2);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Type")
                            .size(10.0)
                            .color(Tokens::default().text_muted),
                    );
                    let ty_label = OSC_TYPES[state.osc_type[idx].min(OSC_TYPES.len() - 1)];
                    if ui.button(ty_label).clicked() {
                        state.osc_type[idx] = (state.osc_type[idx] + 1) % OSC_TYPES.len();
                        changed = true;
                    }
                });

                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                    let level_text = format!("{:.2}", state.osc_level[idx]);
                    let r1 = Knob::new(&mut state.osc_level[idx], 0.0..=1.0, "Level")
                        .size(KnobSize::Md)
                        .style(KnobStyle::Wired)
                        .value_text(level_text)
                        .show(ui);
                    let pan_text = format_pan(state.osc_pan[idx]);
                    let r2 = Knob::new(&mut state.osc_pan[idx], -1.0..=1.0, "Pan")
                        .size(KnobSize::Md)
                        .style(KnobStyle::Normal)
                        .value_text(pan_text)
                        .show(ui);
                    let coarse_text = format_coarse(state.osc_coarse[idx]);
                    let r3 = Knob::new(&mut state.osc_coarse[idx], -2400.0..=2400.0, "Coarse")
                        .size(KnobSize::Md)
                        .style(KnobStyle::Wired)
                        .value_text(coarse_text)
                        .show(ui);
                    if r1.changed || r2.changed || r3.changed {
                        changed = true;
                    }
                });

                ui.add_space(GRID_UNIT);
                let pos = &mut state.osc_position[idx];
                let is_wt = state.osc_type[idx] == 0;
                if is_wt {
                    if param_slider(
                        ui,
                        "WT Position",
                        pos,
                        0.0..=255.0,
                        &format!("{:.0} / 255", pos.round()),
                    ) {
                        changed = true;
                    }

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Warp")
                                .size(10.0)
                                .color(Tokens::default().text_muted),
                        );
                        let warp_label = WARP_MODES[state.osc_warp_mode[idx].min(2)];
                        if ui.button(warp_label).clicked() {
                            state.osc_warp_mode[idx] =
                                (state.osc_warp_mode[idx] + 1) % WARP_MODES.len();
                            changed = true;
                        }
                    });
                    if param_slider(
                        ui,
                        "Warp Amt",
                        &mut state.osc_warp_amount[idx],
                        0.0..=1.0,
                        &format!("{:.0}%", state.osc_warp_amount[idx] * 100.0),
                    ) {
                        changed = true;
                    }
                }

                let is_pulse = matches!(state.osc_type[idx], 2 | 4);
                if is_pulse {
                    if param_slider(
                        ui,
                        "Pulse W",
                        &mut state.osc_pulse_width[idx],
                        0.05..=0.95,
                        &format!("{:.0}%", state.osc_pulse_width[idx] * 100.0),
                    ) {
                        changed = true;
                    }
                }

                let unison_f = &mut (state.osc_unison[idx] as f32);
                let unison_label = format_unison(state.osc_unison[idx]);
                if param_slider(ui, "Unison", unison_f, 1.0..=8.0, &unison_label) {
                    state.osc_unison[idx] = unison_f.round().clamp(1.0, 8.0) as u32;
                    changed = true;
                }

                if param_slider(
                    ui,
                    "Spread",
                    state.unison_stereo_spread,
                    0.0..=1.0,
                    &format!("{:.0}%", *state.unison_stereo_spread * 100.0),
                ) {
                    changed = true;
                }
            });

            panel(ui, "Sub / Noise", |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                    let sub_text = format!("{:.2}", state.sub_level);
                    let r1 = Knob::new(state.sub_level, 0.0..=1.0, "Sub")
                        .size(KnobSize::Sm)
                        .style(KnobStyle::Wired)
                        .value_text(sub_text)
                        .show(ui);
                    let noise_text = format!("{:.2}", state.noise_level);
                    let r2 = Knob::new(state.noise_level, 0.0..=1.0, "Noise")
                        .size(KnobSize::Sm)
                        .style(KnobStyle::Wired)
                        .value_text(noise_text)
                        .show(ui);
                    if r1.changed || r2.changed {
                        changed = true;
                    }
                });
            });

            panel_disabled(ui, "Macros", |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = SPACE_SM;
                    for (i, label) in ["M1", "M2", "M3", "M4"].iter().enumerate() {
                        let mut v = state.macro_values[i];
                        Knob::new(&mut v, 0.0..=1.0, label)
                            .size(KnobSize::Sm)
                            .style(KnobStyle::Disabled)
                            .value_text("—")
                            .show(ui);
                    }
                });
                ui.add_space(4.0);
                let tokens = Tokens::default();
                ui.label(
                    egui::RichText::new("Macro routing ships in S6")
                        .size(10.0)
                        .color(tokens.text_muted),
                );
            });
        });

    OscColumnResult { changed }
}

fn param_slider(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    value_label: &str,
) -> bool {
    let tokens = Tokens::default();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(10.0)
                .color(tokens.text_muted),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            knob_value_label(ui, value_label);
            let slider_w = ui.available_width().max(80.0);
            let norm = ((*value - *range.start()) / (*range.end() - *range.start())).clamp(0.0, 1.0);
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(slider_w, 16.0),
                egui::Sense::click_and_drag(),
            );
            if resp.dragged() || resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    let t = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                    *value = *range.start() + t * (*range.end() - *range.start());
                    changed = true;
                }
            }
            if ui.is_rect_visible(rect) {
                let painter = ui.painter_at(rect);
                let track = rect.shrink2(egui::vec2(0.0, 5.0));
                painter.rect_filled(track, 3.0, tokens.surface2);
                let fill_w = track.width() * norm;
                let fill = egui::Rect::from_min_size(track.min, egui::vec2(fill_w, track.height()));
                painter.rect_filled(fill, 3.0, tokens.accent);
                let thumb_x = track.min.x + track.width() * norm;
                painter.circle_filled(
                    egui::pos2(thumb_x, track.center().y),
                    6.0,
                    tokens.accent_on,
                );
            }
        });
    });
    changed
}
