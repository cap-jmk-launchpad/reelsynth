//! FX rack section (S5/S6) — reorderable slot cards per COMPONENT_SPEC.

use egui::{Color32, FontId, Rect, Ui};
use reelsynth::{EffectSlot, EffectType};
use reelsynth_ui_theme::Tokens;

use crate::layout::{UiScale, GRID_UNIT, RADIUS_SM, sidebar_fx_slot_height, sidebar_panel_chrome_height};
use crate::region::region;
use crate::widgets::{
    button_icon, button_toggle, card_stroke, collapsible_panel, menu_selectable, reel_combo,
    select_value_text, sidebar_panel,
};

const FX_FOOTER_HEIGHT: f32 = 18.0;
const FX_TITLE_HEIGHT: f32 = 14.0;
const FX_PARAM_ROW_HEIGHT: f32 = 18.0;

pub const FX_SLOT_WIDTH: f32 = 148.0;
pub const FX_SECTION_HEADER: f32 = 24.0;
const CPU_WARN_ACTIVE_SLOTS: usize = 4;

#[derive(Debug, Clone, Copy)]
struct FxMetrics {
    slot_width: f32,
    card_height: f32,
    controls_height: f32,
    column_height: f32,
    add_width: f32,
    header_h: f32,
}

impl FxMetrics {
    fn from_scale(scale: UiScale, body_h: f32) -> Self {
        let s = scale.ui();
        let header_h = FX_SECTION_HEADER * s;
        let controls_h = FX_FOOTER_HEIGHT * s;
        let gap = 2.0 * s;
        let card_body = (FX_TITLE_HEIGHT + FX_PARAM_ROW_HEIGHT + 6.0) * s;
        let body = (body_h - header_h).max(40.0 * s);
        let card_h = (body - controls_h - gap)
            .max(card_body)
            .clamp(card_body, 72.0 * s);
        let column_h = card_h + gap + controls_h;
        Self {
            slot_width: FX_SLOT_WIDTH * s,
            card_height: card_h,
            controls_height: controls_h,
            column_height: column_h,
            add_width: 40.0 * s,
            header_h,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectSlotUi {
    pub effect_type: EffectType,
    pub bypassed: bool,
    pub mix: f32,
    pub rate: f32,
    pub depth: f32,
    pub time_ms: f32,
    pub feedback: f32,
    pub size: f32,
    pub damping: f32,
    pub drive: f32,
    pub tone: f32,
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
}

impl EffectSlotUi {
    pub fn from_slot(slot: &EffectSlot) -> Self {
        Self {
            effect_type: slot.effect_type.clone(),
            bypassed: slot.bypassed,
            mix: slot.mix,
            rate: slot.rate,
            depth: slot.depth,
            time_ms: slot.time_ms,
            feedback: slot.feedback,
            size: slot.size,
            damping: slot.damping,
            drive: slot.drive,
            tone: slot.tone,
            threshold: slot.threshold,
            ratio: slot.ratio,
            attack: slot.attack,
            release: slot.release,
        }
    }

    pub fn to_slot(&self) -> EffectSlot {
        let mut slot = EffectSlot::for_type(self.effect_type.clone());
        slot.bypassed = self.bypassed;
        slot.mix = self.mix;
        slot.rate = self.rate;
        slot.depth = self.depth;
        slot.time_ms = self.time_ms;
        slot.feedback = self.feedback;
        slot.size = self.size;
        slot.damping = self.damping;
        slot.drive = self.drive;
        slot.tone = self.tone;
        slot.threshold = self.threshold;
        slot.ratio = self.ratio;
        slot.attack = self.attack;
        slot.release = self.release;
        slot
    }

    pub fn detail(&self) -> String {
        if self.bypassed {
            return "Bypassed".into();
        }
        match self.effect_type {
            EffectType::Chorus => format!("Mix {:.0}% · {:.1} Hz", self.mix * 100.0, self.rate),
            EffectType::Delay => format!("{:.0} ms · FB {:.0}%", self.time_ms, self.feedback * 100.0),
            EffectType::Reverb => format!("Size {:.0}% · Mix {:.0}%", self.size * 100.0, self.mix * 100.0),
            EffectType::Distortion => format!("Drive {:.0}% · Mix {:.0}%", self.drive * 100.0, self.mix * 100.0),
            EffectType::Compressor => format!("{:.0} dB · {:.1}:1", self.threshold, self.ratio),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.bypassed && self.mix > 0.001
    }
}

pub fn default_effect_slots() -> Vec<EffectSlotUi> {
    effect_slots_from_patch(&reelsynth::default_effects())
}

pub fn effect_slots_from_patch(effects: &[EffectSlot]) -> Vec<EffectSlotUi> {
    if effects.is_empty() {
        return default_effect_slots();
    }
    effects.iter().map(EffectSlotUi::from_slot).collect()
}

pub fn effect_slots_to_patch(slots: &[EffectSlotUi]) -> Vec<EffectSlot> {
    slots.iter().map(EffectSlotUi::to_slot).collect()
}

pub fn effect_slots_to_bypass(slots: &[EffectSlotUi]) -> reelsynth::FxBypass {
    let mut bypass = reelsynth::FxBypass::default();
    for slot in slots {
        match slot.effect_type {
            EffectType::Chorus => bypass.chorus_bypassed = slot.bypassed,
            EffectType::Delay => bypass.delay_bypassed = slot.bypassed,
            EffectType::Reverb => bypass.reverb_bypassed = slot.bypassed,
            _ => {}
        }
    }
    bypass
}

pub fn effect_slots_from_bypass(bypass: &reelsynth::FxBypass) -> Vec<EffectSlotUi> {
    effect_slots_from_patch(&reelsynth::effects_from_bypass(bypass))
}

pub struct EffectRackState<'a> {
    pub open: &'a mut bool,
    pub slots: &'a mut Vec<EffectSlotUi>,
}

pub struct FxRackResult {
    pub changed: bool,
}

pub fn draw_effect_rack(
    ui: &mut Ui,
    rect: Rect,
    mut state: EffectRackState<'_>,
    scale: UiScale,
) -> FxRackResult {
    draw_effect_rack_inner(
        ui,
        rect,
        &mut state,
        scale,
        RackLayout::Horizontal,
        RackChrome::Collapsible,
    )
}

/// Narrow-column layout: vertical scrollable FX chain (left osc column).
pub fn draw_effect_rack_sidebar(
    ui: &mut Ui,
    rect: Rect,
    mut state: EffectRackState<'_>,
    scale: UiScale,
) -> FxRackResult {
    draw_effect_rack_inner(
        ui,
        rect,
        &mut state,
        scale,
        RackLayout::VerticalChain,
        RackChrome::NativePanel,
    )
}

#[derive(Clone, Copy)]
enum RackLayout {
    Horizontal,
    VerticalChain,
}

#[derive(Clone, Copy)]
enum RackChrome {
    Collapsible,
    NativePanel,
}

fn draw_effect_rack_inner(
    ui: &mut Ui,
    rect: Rect,
    state: &mut EffectRackState<'_>,
    scale: UiScale,
    layout: RackLayout,
    chrome: RackChrome,
) -> FxRackResult {
    let mut changed = false;
    let metrics = FxMetrics::from_scale(scale, rect.height());

    let EffectRackState { open, slots } = state;

    region(ui, rect, |ui| {
        ui.set_clip_rect(rect);
        ui.set_min_height(rect.height());
        ui.set_max_height(rect.height());
        let active = slots.iter().filter(|s| s.is_active()).count();
        let mut meta = format!("{active} active");
        if active > CPU_WARN_ACTIVE_SLOTS {
            meta.push_str(" · CPU ⚠");
        }

        let chrome_h = sidebar_panel_chrome_height(scale.ui(), true);
        let body_h = (rect.height() - chrome_h).max(0.0);

        let body = |ui: &mut Ui| {
            if active > CPU_WARN_ACTIVE_SLOTS {
                ui.label(
                    egui::RichText::new(format!(
                        "⚠ {active} active FX slots — may increase CPU usage"
                    ))
                    .size(10.0)
                    .color(Color32::from_rgb(0xe8, 0xa8, 0x40)),
                );
                ui.add_space(4.0);
            }

            ui.set_max_height(body_h);
            match layout {
                RackLayout::Horizontal => {
                    draw_effect_rack_horizontal(ui, slots, scale, metrics, &mut changed);
                }
                RackLayout::VerticalChain => {
                    draw_effect_rack_chain(ui, slots, scale, &mut changed, body_h);
                }
            }
        };

        match chrome {
            RackChrome::Collapsible => {
                collapsible_panel(ui, "Effects", &meta, open, body);
            }
            RackChrome::NativePanel => {
                sidebar_panel(ui, "Effects", &meta, body);
            }
        }
    });

    FxRackResult { changed }
}

fn draw_effect_rack_horizontal(
    ui: &mut Ui,
    slots: &mut Vec<EffectSlotUi>,
    scale: UiScale,
    metrics: FxMetrics,
    changed: &mut bool,
) {
    let s = scale.ui();
    let gap = GRID_UNIT * s;
    let slot_count = slots.len().max(1);
    let add_w = metrics.add_width;
    let gaps = gap * slot_count as f32;
    let avail = ui.available_width();
    let flex_slot_w = ((avail - add_w - gaps) / slot_count as f32)
        .clamp(96.0 * s, FX_SLOT_WIDTH * s * 1.35);
    let mut flex_metrics = metrics;
    flex_metrics.slot_width = flex_slot_w;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        ui.set_min_height(flex_metrics.column_height);
        for idx in 0..slots.len() {
            if draw_fx_slot_column(ui, slots, idx, flex_metrics).changed {
                *changed = true;
            }
        }
        if draw_add_slot(ui, flex_metrics).clicked() {
            slots.push(EffectSlotUi::from_slot(&EffectSlot::chorus()));
            *changed = true;
        }
    });
}

fn draw_effect_rack_chain(
    ui: &mut Ui,
    slots: &mut Vec<EffectSlotUi>,
    scale: UiScale,
    changed: &mut bool,
    body_h: f32,
) {
    let s = scale.ui();
    let gap = GRID_UNIT * s * 0.5;
    let slot_h = sidebar_fx_slot_height(s);
    let card_body = (FX_TITLE_HEIGHT + FX_PARAM_ROW_HEIGHT + 6.0) * s;

    egui::ScrollArea::vertical()
        .id_salt("fx_sidebar_chain_scroll")
        .max_height(body_h)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for idx in 0..slots.len() {
                let slot_w = ui.available_width();
                let metrics = FxMetrics {
                    slot_width: slot_w,
                    card_height: card_body,
                    controls_height: FX_FOOTER_HEIGHT * s,
                    column_height: slot_h,
                    add_width: slot_w,
                    header_h: 0.0,
                };
                if draw_fx_slot_column(ui, slots, idx, metrics).changed {
                    *changed = true;
                }
                if idx + 1 < slots.len() {
                    ui.add_space(gap);
                }
            }
            if !slots.is_empty() {
                ui.add_space(gap);
            }
            if draw_add_slot_row(ui, ui.available_width(), s).clicked() {
                slots.push(EffectSlotUi::from_slot(&EffectSlot::chorus()));
                *changed = true;
            }
        });
}

fn draw_add_slot_row(ui: &mut Ui, width: f32, scale: f32) -> egui::Response {
    let tokens = Tokens::default();
    let h = 28.0 * scale;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, h), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let stroke = card_stroke(false, response.hovered(), &tokens);
        let fill = if response.hovered() {
            tokens.surface2
        } else {
            tokens.bg
        };
        painter.rect_filled(rect, RADIUS_SM, fill);
        painter.rect_stroke(rect, RADIUS_SM, egui::Stroke::new(1.0_f32, stroke));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "+ Add effect",
            FontId::proportional(11.0),
            if response.hovered() {
                tokens.text
            } else {
                tokens.text_secondary
            },
        );
    }
    response
}

struct FxSlotResult {
    changed: bool,
}

fn fx_drag_pct(ui: &mut Ui, label: &str, value: &mut f32, max: f32) -> bool {
    let tokens = Tokens::default();
    let mut pct = (*value * 100.0).clamp(0.0, max * 100.0);
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.label(
            egui::RichText::new(label)
                .size(9.0)
                .color(tokens.text_muted),
        );
        if ui
            .add(
                egui::DragValue::new(&mut pct)
                    .speed(0.4)
                    .range(0.0..=(max * 100.0))
                    .suffix("%"),
            )
            .changed()
        {
            changed = true;
        }
    });
    if changed {
        *value = (pct / 100.0).clamp(0.0, max);
    }
    changed
}

fn fx_drag_f32(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f32,
    suffix: &str,
) -> bool {
    let tokens = Tokens::default();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.label(
            egui::RichText::new(label)
                .size(9.0)
                .color(tokens.text_muted),
        );
        if ui
            .add(
                egui::DragValue::new(value)
                    .speed(speed)
                    .range(range)
                    .suffix(suffix),
            )
            .changed()
        {
            changed = true;
        }
    });
    changed
}

fn draw_fx_slot_params(ui: &mut Ui, slot: &mut EffectSlotUi) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        match slot.effect_type {
            EffectType::Chorus => {
                if fx_drag_pct(ui, "Mix", &mut slot.mix, 1.0) {
                    changed = true;
                }
                if fx_drag_f32(ui, "Rate", &mut slot.rate, 0.05..=8.0, 0.02, "Hz") {
                    changed = true;
                }
                if fx_drag_pct(ui, "Depth", &mut slot.depth, 1.0) {
                    changed = true;
                }
            }
            EffectType::Delay => {
                if fx_drag_f32(ui, "Time", &mut slot.time_ms, 1.0..=2000.0, 1.0, "ms") {
                    changed = true;
                }
                if fx_drag_pct(ui, "FB", &mut slot.feedback, 0.95) {
                    changed = true;
                }
                if fx_drag_pct(ui, "Mix", &mut slot.mix, 1.0) {
                    changed = true;
                }
            }
            EffectType::Reverb => {
                if fx_drag_pct(ui, "Size", &mut slot.size, 1.0) {
                    changed = true;
                }
                if fx_drag_pct(ui, "Mix", &mut slot.mix, 1.0) {
                    changed = true;
                }
                if fx_drag_pct(ui, "Damp", &mut slot.damping, 1.0) {
                    changed = true;
                }
            }
            EffectType::Distortion => {
                if fx_drag_pct(ui, "Drive", &mut slot.drive, 1.0) {
                    changed = true;
                }
                if fx_drag_pct(ui, "Mix", &mut slot.mix, 1.0) {
                    changed = true;
                }
                if fx_drag_pct(ui, "Tone", &mut slot.tone, 1.0) {
                    changed = true;
                }
            }
            EffectType::Compressor => {
                if fx_drag_f32(
                    ui,
                    "Thr",
                    &mut slot.threshold,
                    -60.0..=0.0,
                    0.25,
                    "dB",
                ) {
                    changed = true;
                }
                if fx_drag_f32(ui, "Ratio", &mut slot.ratio, 1.0..=20.0, 0.05, ":1") {
                    changed = true;
                }
                if fx_drag_pct(ui, "Mix", &mut slot.mix, 1.0) {
                    changed = true;
                }
            }
        }
    });
    changed
}

fn draw_fx_slot_column(
    ui: &mut Ui,
    slots: &mut Vec<EffectSlotUi>,
    idx: usize,
    metrics: FxMetrics,
) -> FxSlotResult {
    let tokens = Tokens::default();
    let mut changed = false;

    let column = ui.vertical(|ui| {
        ui.set_width(metrics.slot_width);
        ui.set_min_height(metrics.column_height);

        let active = slots[idx].is_active();
        let bypassed = slots[idx].bypassed;
        let stroke_color = card_stroke(active, false, &tokens);

        egui::Frame {
            fill: if active {
                tokens.accent_muted.gamma_multiply(0.55)
            } else if bypassed {
                tokens.bg
            } else {
                tokens.surface2.gamma_multiply(0.85)
            },
            stroke: egui::Stroke::new(1.0_f32, stroke_color),
            rounding: egui::Rounding::same(RADIUS_SM),
            inner_margin: egui::Margin::symmetric(GRID_UNIT * 0.5, 3.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_height(metrics.card_height - 6.0);
            ui.horizontal(|ui| {
                let title_color = if bypassed {
                    tokens.text_secondary
                } else {
                    tokens.text
                };
                ui.label(
                    egui::RichText::new(slots[idx].effect_type.label())
                        .size(11.0)
                        .color(title_color),
                );
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.set_width(ui.available_width());
                        let on_label = if slots[idx].bypassed { "Off" } else { "On" };
                        if button_toggle(ui, on_label, !slots[idx].bypassed).clicked() {
                            slots[idx].bypassed = !slots[idx].bypassed;
                            changed = true;
                        }
                    },
                );
            });
            if draw_fx_slot_params(ui, &mut slots[idx]) {
                changed = true;
            }
        });

        ui.allocate_ui_with_layout(
            egui::vec2(metrics.slot_width, metrics.controls_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                if idx > 0 && button_icon(ui, "◀").clicked() {
                    slots.swap(idx, idx - 1);
                    changed = true;
                }
                if idx + 1 < slots.len() && button_icon(ui, "▶").clicked() {
                    slots.swap(idx, idx + 1);
                    changed = true;
                }
                if slots.len() > 1 && button_icon(ui, "✕").clicked() {
                    slots.remove(idx);
                    changed = true;
                    return;
                }
                let combo_w = (ui.available_width() - 56.0).max(48.0);
                reel_combo(
                    ui,
                    format!("fx_type_{idx}"),
                    select_value_text(slots[idx].effect_type.label()),
                    combo_w,
                    |ui| {
                        for ty in EffectType::ALL {
                            if menu_selectable(
                                ui,
                                slots[idx].effect_type == ty,
                                ty.label(),
                            )
                            .clicked()
                            {
                                let bypassed = slots[idx].bypassed;
                                let mix = slots[idx].mix;
                                slots[idx] = EffectSlotUi::from_slot(&EffectSlot::for_type(ty.clone()));
                                slots[idx].bypassed = bypassed;
                                slots[idx].mix = mix;
                                changed = true;
                            }
                        }
                    },
                );
            },
        );
    });

    let _ = column;
    FxSlotResult { changed }
}

fn draw_add_slot(ui: &mut Ui, metrics: FxMetrics) -> egui::Response {
    let tokens = Tokens::default();
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(metrics.add_width, metrics.card_height),
        egui::Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let stroke = card_stroke(false, response.hovered(), &tokens);
        let fill = if response.hovered() {
            tokens.surface2
        } else {
            tokens.bg
        };
        painter.rect_filled(rect, RADIUS_SM, fill);
        painter.rect_stroke(rect, RADIUS_SM, egui::Stroke::new(1.0_f32, stroke));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            FontId::proportional(18.0),
            if response.hovered() {
                tokens.text
            } else {
                tokens.text_secondary
            },
        );
    }
    response
}

#[cfg(test)]
mod bridge_tests {
    use super::*;
    use reelsynth::fx::{EffectSlot, EffectType, FxBypass};

    #[test]
    fn fx_slot_ui_roundtrip() {
        let slot = EffectSlot::delay();
        let ui = EffectSlotUi::from_slot(&slot);
        let restored = ui.to_slot();
        assert_eq!(restored.effect_type, slot.effect_type);
        assert_eq!(restored.time_ms, slot.time_ms);
        assert!((restored.mix - slot.mix).abs() < 1e-5);
    }

    #[test]
    fn bypass_migration_roundtrip() {
        let bypass = FxBypass {
            chorus_bypassed: true,
            delay_bypassed: false,
            reverb_bypassed: true,
        };
        let slots = effect_slots_from_bypass(&bypass);
        assert_eq!(slots.len(), 3);
        assert!(slots[0].bypassed);
        assert!(!slots[1].bypassed);
        let back = effect_slots_to_bypass(&slots);
        assert_eq!(back.chorus_bypassed, bypass.chorus_bypassed);
        assert_eq!(back.delay_bypassed, bypass.delay_bypassed);
    }

    #[test]
    fn effect_type_labels() {
        let mut slot = EffectSlot::for_type(EffectType::Distortion);
        slot.bypassed = false;
        let ui = EffectSlotUi::from_slot(&slot);
        assert!(ui.detail().contains("Drive"));
    }

    #[test]
    fn delay_params_roundtrip_to_engine() {
        let mut ui_slot = EffectSlotUi::from_slot(&EffectSlot::delay());
        ui_slot.time_ms = 420.0;
        ui_slot.feedback = 0.55;
        ui_slot.mix = 0.6;
        let engine = ui_slot.to_slot();
        assert!((engine.time_ms - 420.0).abs() < 1e-3);
        assert!((engine.feedback - 0.55).abs() < 1e-4);
        assert!((engine.mix - 0.6).abs() < 1e-4);
    }
}
