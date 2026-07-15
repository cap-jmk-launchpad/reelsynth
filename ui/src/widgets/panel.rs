use egui::{Color32, FontId, Frame, Margin, Ui};
use reelsynth_ui_theme::{heading_font, ACCENT_UI, Tokens};

use crate::audit_registry::{record_region, AuditId};
use crate::layout::{RADIUS_SM, SPACE_SM};

/// Branded section frame matching `.rs-panel`.
pub fn panel<R>(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    panel_audit(ui, title, None, add_contents)
}

/// Like [`panel`] but records the panel rect under `audit_id` when provided.
pub fn panel_audit<R>(
    ui: &mut Ui,
    title: &str,
    audit_id: Option<AuditId>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let tokens = Tokens::default();
    // Namespace all child widgets under the title so Filter Env / Amp Env /
    // repeated FX slots never collide on egui Id (avoids 🔥 debug flames).
    let inner = ui
        .push_id(title, |ui| {
            Frame {
                fill: tokens.bg_muted,
                stroke: egui::Stroke::new(1.0_f32, tokens.border),
                rounding: egui::Rounding::same(RADIUS_SM),
                inner_margin: Margin::same(6.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let display = if let Some(base) = title.strip_suffix(" (locked)") {
                    format!("{} (locked)", base.to_uppercase())
                } else {
                    title.to_uppercase()
                };
                ui.label(
                    egui::RichText::new(display)
                        .font(heading_font(10.0))
                        .color(tokens.text_muted),
                );
                ui.add_space(6.0);
                add_contents(ui)
            })
        })
        .inner;
    if let Some(id) = audit_id {
        let r = inner.response.rect;
        record_region(ui.ctx(), id, r, r);
    }
    inner.inner
}

/// Disabled panel wrapper matching `.rs-group--disabled`.
pub fn panel_disabled<R>(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let locked_title = format!("{} (locked)", title.to_uppercase());
    ui.add_enabled_ui(false, |ui| panel(ui, &locked_title, add_contents))
        .inner
}

/// Native sidebar section — same chrome as [`panel`], with optional meta line (Effects, Mod Matrix).
pub fn sidebar_panel<R>(
    ui: &mut Ui,
    title: &str,
    meta: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    sidebar_panel_audit(ui, title, meta, None, add_contents)
}

/// Like [`sidebar_panel`] but records the panel rect under `audit_id` when provided.
///
/// Title + meta use a reserved header band (like the right-rail collapsible chrome)
/// so "EFFECTS" never paints under the first slot card.
pub fn sidebar_panel_audit<R>(
    ui: &mut Ui,
    title: &str,
    meta: &str,
    audit_id: Option<AuditId>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let tokens = Tokens::default();
    let inner = ui
        .push_id(title, |ui| {
            Frame {
                fill: tokens.bg_muted,
                stroke: egui::Stroke::new(1.0_f32, tokens.border),
                rounding: egui::Rounding::same(RADIUS_SM),
                inner_margin: Margin::ZERO,
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let header_h = 22.0;
                let (header_rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), header_h),
                    egui::Sense::hover(),
                );
                if ui.is_rect_visible(header_rect) {
                    let painter = ui.painter_at(header_rect);
                    painter.rect_filled(
                        header_rect,
                        egui::Rounding {
                            nw: RADIUS_SM,
                            ne: RADIUS_SM,
                            sw: 0.0,
                            se: 0.0,
                        },
                        tokens.surface2,
                    );
                    painter.line_segment(
                        [header_rect.left_bottom(), header_rect.right_bottom()],
                        egui::Stroke::new(1.0_f32, tokens.border),
                    );
                    painter.text(
                        egui::pos2(header_rect.min.x + SPACE_SM, header_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        title.to_uppercase(),
                        heading_font(10.0),
                        tokens.text_muted,
                    );
                    if !meta.is_empty() {
                        let meta_color = if meta.contains('⚠') {
                            Color32::from_rgb(0xe8, 0xa8, 0x40)
                        } else {
                            tokens.text_muted
                        };
                        painter.text(
                            egui::pos2(header_rect.max.x - SPACE_SM, header_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            meta,
                            FontId::monospace(10.0),
                            meta_color,
                        );
                    }
                }
                egui::Frame::none()
                    .inner_margin(Margin::symmetric(SPACE_SM, 6.0))
                    .show(ui, |ui| add_contents(ui))
                    .inner
            })
        })
        .inner;
    if let Some(id) = audit_id {
        let r = inner.response.rect;
        record_region(ui.ctx(), id, r, r);
    }
    inner.inner
}

/// Collapsible sidebar section — bottom-strip layout when FX/mod are not embedded.
pub fn collapsible_panel(
    ui: &mut Ui,
    title: &str,
    meta: &str,
    open: &mut bool,
    add_contents: impl FnOnce(&mut Ui),
) {
    let tokens = Tokens::default();
    let mut is_open = *open;
    ui.push_id(title, |ui| {
        Frame {
            fill: tokens.bg_muted,
            stroke: egui::Stroke::new(1.0_f32, tokens.border),
            rounding: egui::Rounding::same(RADIUS_SM),
            inner_margin: Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let header_h = 26.0;
            let (header_rect, header_resp) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), header_h),
                egui::Sense::click(),
            );
            if ui.is_rect_visible(header_rect) {
                let painter = ui.painter_at(header_rect);
                painter.rect_filled(
                    header_rect,
                    egui::Rounding {
                        nw: RADIUS_SM,
                        ne: RADIUS_SM,
                        sw: if is_open { 0.0 } else { RADIUS_SM },
                        se: if is_open { 0.0 } else { RADIUS_SM },
                    },
                    tokens.surface2,
                );
                painter.line_segment(
                    [header_rect.left_bottom(), header_rect.right_bottom()],
                    egui::Stroke::new(1.0_f32, tokens.border),
                );
                let chevron = if is_open { "▼" } else { "▶" };
                painter.text(
                    egui::pos2(header_rect.min.x + SPACE_SM, header_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    chevron,
                    FontId::proportional(10.0),
                    tokens.text_secondary,
                );
                painter.text(
                    egui::pos2(header_rect.min.x + SPACE_SM + 16.0, header_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    title.to_uppercase(),
                    heading_font(11.0),
                    tokens.text,
                );
                let meta_color = if meta.contains('⚠') {
                    Color32::from_rgb(0xe8, 0xa8, 0x40)
                } else {
                    tokens.text_secondary
                };
                painter.text(
                    egui::pos2(header_rect.max.x - SPACE_SM, header_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    meta,
                    FontId::monospace(10.0),
                    meta_color,
                );
            }
            if header_resp.clicked() {
                is_open = !is_open;
            }

            if is_open {
                egui::Frame::none()
                    .inner_margin(Margin::symmetric(SPACE_SM, 6.0))
                    .show(ui, |ui| add_contents(ui));
            }
        });
    });
    *open = is_open;
}

/// Highlight stroke for active / hovered cards in sidebar grids.
pub fn card_stroke(active: bool, hovered: bool, tokens: &Tokens) -> Color32 {
    if active {
        ACCENT_UI
    } else if hovered {
        tokens.border_strong
    } else {
        tokens.border
    }
}
