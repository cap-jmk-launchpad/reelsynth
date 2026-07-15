//! ADSR envelope graph — matches `.rs-adsr-graph` (80px tall).

use egui::{CursorIcon, FontId, Pos2, Shape, Ui, Vec2};
use reelsynth_ui_theme::{ACCENT_UI, Tokens};

pub const ADSR_GRAPH_HEIGHT: f32 = 64.0;

const HANDLE_RADIUS: f32 = 5.0;
const ATTACK_RANGE: std::ops::RangeInclusive<f32> = 0.001..=2.0;
const DECAY_RANGE: std::ops::RangeInclusive<f32> = 0.001..=2.0;
const RELEASE_RANGE: std::ops::RangeInclusive<f32> = 0.001..=3.0;

pub struct AdsrGraphResponse {
    pub response: egui::Response,
    pub changed: bool,
}

/// Draw an interactive ADSR envelope with draggable A / D / S / R control points.
///
/// `id_salt` must be unique per graph (e.g. `"filt_env"` / `"amp_env"`) so handles
/// do not clash when multiple envelopes are on screen.
pub fn adsr_graph(
    ui: &mut Ui,
    attack: &mut f32,
    decay: &mut f32,
    sustain: &mut f32,
    release: &mut f32,
    scale: f32,
    id_salt: impl std::hash::Hash,
) -> AdsrGraphResponse {
    let tokens = Tokens::default();
    let accent_ui = ACCENT_UI;
    let height = ADSR_GRAPH_HEIGHT * scale;
    ui.push_id(id_salt, |ui| {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::hover());

        let mut changed = false;

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 4.0, tokens.surface2);
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, tokens.border));

            let inner = rect.shrink(8.0);
            let layout = adsr_layout(inner, *attack, *decay, *sustain, *release);

            let points = vec![
                Pos2::new(inner.min.x, layout.bottom),
                Pos2::new(layout.ax, layout.top),
                Pos2::new(layout.dx, layout.sustain_y),
                Pos2::new(layout.sx, layout.sustain_y),
                Pos2::new(inner.max.x, layout.bottom),
            ];
            painter.add(Shape::line(
                points,
                egui::Stroke::new(2.0_f32, accent_ui),
            ));

            let handles = [
                (Pos2::new(layout.ax, layout.top), AdsrHandle::Attack),
                (
                    Pos2::new(layout.dx, layout.sustain_y),
                    AdsrHandle::DecaySustain,
                ),
                (
                    Pos2::new(layout.sx, layout.sustain_y),
                    AdsrHandle::SustainHold,
                ),
                (
                    Pos2::new(
                        egui::lerp(layout.sx..=inner.max.x, 0.72),
                        egui::lerp(layout.sustain_y..=layout.bottom, 0.72),
                    ),
                    AdsrHandle::Release,
                ),
            ];

            for (pt, _handle) in handles {
                painter.circle_filled(pt, HANDLE_RADIUS, tokens.accent);
                painter.circle_stroke(
                    pt,
                    HANDLE_RADIUS,
                    egui::Stroke::new(1.0_f32, tokens.accent_on),
                );
            }

            let hit = HANDLE_RADIUS + 4.0;
            for (pt, handle) in handles.into_iter().rev() {
                let handle_rect = egui::Rect::from_center_size(pt, Vec2::splat(hit * 2.0));
                let id = ui.id().with(("adsr_handle", handle as u8));
                let handle_resp = ui.interact(handle_rect, id, egui::Sense::drag());
                if handle_resp.dragged() {
                    if let Some(pos) = handle_resp.interact_pointer_pos() {
                        if apply_handle_drag(
                            handle,
                            inner,
                            pos,
                            attack,
                            decay,
                            sustain,
                            release,
                        ) {
                            changed = true;
                        }
                    }
                }
                if handle_resp.hovered() || handle_resp.dragged() {
                    ui.ctx().set_cursor_icon(CursorIcon::Grab);
                }
            }
        }

        AdsrGraphResponse { response, changed }
    })
    .inner
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdsrHandle {
    Attack = 0,
    DecaySustain = 1,
    SustainHold = 2,
    Release = 3,
}

struct AdsrLayout {
    ax: f32,
    dx: f32,
    sx: f32,
    top: f32,
    bottom: f32,
    sustain_y: f32,
}

fn visual_hold(attack: f32, decay: f32) -> f32 {
    (attack + decay).max(0.05)
}

fn adsr_layout(inner: egui::Rect, attack: f32, decay: f32, sustain: f32, release: f32) -> AdsrLayout {
    let a = attack.clamp(*ATTACK_RANGE.start(), *ATTACK_RANGE.end()).max(0.001);
    let d = decay.clamp(*DECAY_RANGE.start(), *DECAY_RANGE.end()).max(0.001);
    let r = release.clamp(*RELEASE_RANGE.start(), *RELEASE_RANGE.end()).max(0.001);
    let hold = visual_hold(a, d);
    let total = (a + d + hold + r).max(0.001);
    let top = inner.min.y + 8.0;
    let bottom = inner.max.y - 4.0;
    let sustain_level = sustain.clamp(0.0, 1.0);
    let sustain_y = bottom - (bottom - top) * sustain_level;

    AdsrLayout {
        ax: egui::lerp(inner.min.x..=inner.max.x, a / total),
        dx: egui::lerp(inner.min.x..=inner.max.x, (a + d) / total),
        sx: egui::lerp(inner.min.x..=inner.max.x, (a + d + hold) / total),
        top,
        bottom,
        sustain_y,
    }
}

fn apply_handle_drag(
    handle: AdsrHandle,
    inner: egui::Rect,
    pos: Pos2,
    attack: &mut f32,
    decay: &mut f32,
    sustain: &mut f32,
    release: &mut f32,
) -> bool {
    let width = inner.width().max(1.0);
    let t = ((pos.x - inner.min.x) / width).clamp(0.02, 0.98);
    let top = inner.min.y + 8.0;
    let bottom = inner.max.y - 4.0;
    let level = (1.0 - ((pos.y - top) / (bottom - top).max(1.0)).clamp(0.0, 1.0)).clamp(0.0, 1.0);

    let a0 = *attack;
    let d0 = *decay;
    let s0 = *sustain;
    let r0 = *release;
    let hold0 = visual_hold(a0, d0);
    let total0 = (a0 + d0 + hold0 + r0).max(0.001);

    match handle {
        AdsrHandle::Attack => {
            let max_t = ((a0 + d0) / total0 - 0.04).clamp(0.02, 0.9);
            let t_peak = t.min(max_t);
            let others = d0 + hold0 + r0;
            *attack = (t_peak * others / (1.0 - t_peak).max(0.02))
                .clamp(*ATTACK_RANGE.start(), *ATTACK_RANGE.end());
        }
        AdsrHandle::DecaySustain => {
            let min_t = (a0 / total0 + 0.04).clamp(0.05, 0.95);
            let max_t = ((a0 + d0 + hold0) / total0 - 0.04).clamp(min_t, 0.98);
            let t_decay = t.clamp(min_t, max_t);
            *decay = (t_decay * total0 - a0).clamp(*DECAY_RANGE.start(), *DECAY_RANGE.end());
            *sustain = level;
        }
        AdsrHandle::SustainHold => {
            let min_t = ((a0 + d0) / total0 + 0.04).clamp(0.05, 0.95);
            let max_t = 0.98;
            let t_hold = t.clamp(min_t, max_t);
            let new_hold = (t_hold * total0 - a0 - d0).max(0.02);
            let new_total = a0 + d0 + new_hold + r0;
            let new_r = ((1.0 - t_hold) * new_total).max(0.001);
            *release = new_r.clamp(*RELEASE_RANGE.start(), *RELEASE_RANGE.end());
            *sustain = level;
        }
        AdsrHandle::Release => {
            let min_t = ((a0 + d0 + hold0) / total0 + 0.04).clamp(0.05, 0.98);
            let t_end = t.max(min_t);
            let body = a0 + d0 + hold0;
            *release = (t_end * total0 - body)
                .clamp(*RELEASE_RANGE.start(), *RELEASE_RANGE.end());
        }
    }

    a0 != *attack || d0 != *decay || s0 != *sustain || r0 != *release
}

pub fn format_env_time(seconds: f32) -> String {
    let ms = seconds * 1000.0;
    if ms < 1000.0 {
        format!("{:.0} ms", ms.max(1.0))
    } else {
        format!("{:.2} s", seconds)
    }
}

pub fn format_sustain(level: f32) -> String {
    format!("{:.0}%", level.clamp(0.0, 1.0) * 100.0)
}

pub fn format_lfo_rate(hz: f32) -> String {
    format!("{:.1} Hz", hz.max(0.0))
}

pub fn format_depth(depth: f32) -> String {
    format!("{:.0}%", depth.clamp(0.0, 1.0) * 100.0)
}

pub fn format_pan(pan: f32) -> String {
    if pan.abs() < 0.05 {
        "C".into()
    } else if pan < 0.0 {
        format!("L{:.0}", (-pan * 100.0).round())
    } else {
        format!("R{:.0}", (pan * 100.0).round())
    }
}

pub fn format_coarse(cents: f32) -> String {
    format!("{:.0} st", cents / 100.0)
}

pub fn format_unison(count: u32) -> String {
    if count <= 1 {
        "1 voice".into()
    } else {
        format!("{count} voices")
    }
}

pub fn knob_value_label(ui: &mut Ui, text: &str) {
    let tokens = Tokens::default();
    ui.label(
        egui::RichText::new(text)
            .font(FontId::monospace(11.0))
            .color(tokens.text),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adsr_layout_uses_release_time() {
        let inner = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 64.0));
        let short = adsr_layout(inner, 0.1, 0.1, 0.5, 0.1);
        let long = adsr_layout(inner, 0.1, 0.1, 0.5, 0.8);
        assert!(
            long.sx < short.sx,
            "longer release should compress the sustain-hold segment"
        );
    }

    #[test]
    fn attack_drag_increases_attack() {
        let inner = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 64.0));
        let mut a = 0.05;
        let mut d = 0.2;
        let mut s = 0.7;
        let mut r = 0.3;
        let layout = adsr_layout(inner, a, d, s, r);
        apply_handle_drag(
            AdsrHandle::Attack,
            inner,
            Pos2::new(layout.ax + 20.0, layout.top),
            &mut a,
            &mut d,
            &mut s,
            &mut r,
        );
        assert!(a > 0.05);
    }
}
