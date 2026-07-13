//! Branded select fields and popup menus — `.rs-select` in mockups.

use egui::{
    Align2, FontId, PopupCloseBehavior, Rect, Response, RichText, Sense, Stroke, Ui, WidgetText,
};
use reelsynth_ui_theme::{ACCENT_UI, Tokens};

use crate::layout::{BUTTON_FONT_SIZE, RADIUS_SM};

pub const SELECT_HEIGHT: f32 = 26.0;
pub const SELECT_MENU_MAX_HEIGHT: f32 = 168.0;

/// Branded combo trigger + styled popup menu.
pub fn reel_combo<R>(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    selected_text: impl Into<WidgetText>,
    width: f32,
    menu: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_text)
        .width(width)
        .height(SELECT_MENU_MAX_HEIGHT)
        .truncate()
        .icon(chevron_icon)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show_ui(ui, |ui| styled_popup_body(ui, menu))
}

/// Label on the left, select field aligned right (Type, Warp, Shape, etc.).
pub fn labeled_select(
    ui: &mut Ui,
    field_label: &str,
    options: &[&str],
    selected: &mut usize,
) -> bool {
    let tokens = Tokens::default();
    let mut changed = false;
    let prev = *selected;
    let id = ui.make_persistent_id(field_label);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(field_label)
                .size(10.0)
                .color(tokens.text_muted),
        );
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let w = ui.available_width().max(108.0);
                let label = options[(*selected).min(options.len().saturating_sub(1))];
                reel_combo(
                    ui,
                    id,
                    select_value_text(label),
                    w,
                    |ui| {
                        for (idx, option) in options.iter().enumerate() {
                            if menu_selectable(ui, idx == *selected, option).clicked() {
                                *selected = idx;
                            }
                        }
                    },
                );
            },
        );
    });

    if *selected != prev {
        changed = true;
    }
    changed
}

/// Styled popup wrapper for [`egui::Ui::menu_button`] bodies.
pub fn styled_menu_body<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    styled_popup_body(ui, body)
}

/// Menu row — accent highlight when selected or hovered.
pub fn menu_selectable(ui: &mut Ui, selected: bool, label: &str) -> Response {
    let tokens = Tokens::default();
    let height = 22.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());

    if ui.is_rect_visible(rect) {
        if selected {
            ui.painter()
                .rect_filled(rect, 4.0, tokens.accent.gamma_multiply(0.45));
        } else if response.hovered() {
            ui.painter()
                .rect_filled(rect, 4.0, tokens.accent_muted);
        }
        ui.painter().text(
            rect.left_center() + egui::vec2(8.0, 0.0),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(11.0),
            if selected {
                tokens.accent_on
            } else {
                tokens.text
            },
        );
    }

    response
}

/// Plain menu action row (header menus, context menus).
pub fn menu_action(ui: &mut Ui, label: &str) -> Response {
    menu_selectable(ui, false, label)
}

/// Muted section label inside a menu.
pub fn menu_section_label(ui: &mut Ui, label: &str) {
    let tokens = Tokens::default();
    ui.label(
        RichText::new(label)
            .size(10.0)
            .color(tokens.text_muted),
    );
}

pub fn menu_divider(ui: &mut Ui) {
    let tokens = Tokens::default();
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 7.0), Sense::hover());
    if ui.is_rect_visible(rect) {
        let y = rect.center().y;
        ui.painter().hline(
            rect.min.x..=rect.max.x,
            y,
            Stroke::new(1.0_f32, tokens.border),
        );
    }
}

pub fn select_value_text(label: &str) -> WidgetText {
    RichText::new(label)
        .font(FontId::monospace(BUTTON_FONT_SIZE))
        .size(BUTTON_FONT_SIZE)
        .into()
}

fn styled_popup_body<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    let tokens = Tokens::default();
    ui.set_min_width(120.0);
    egui::Frame::popup(ui.style())
        .fill(tokens.bg_muted)
        .stroke(Stroke::new(1.0_f32, tokens.border))
        .rounding(egui::Rounding::same(RADIUS_SM))
        .inner_margin(egui::Margin::symmetric(6.0, 4.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            body(ui)
        })
        .inner
}

fn chevron_icon(
    ui: &Ui,
    rect: Rect,
    visuals: &egui::style::WidgetVisuals,
    is_open: bool,
    above_or_below: egui::AboveOrBelow,
) {
    let tokens = Tokens::default();
    let size = egui::vec2(8.0, 5.0);
    let center = rect.center();
    let chevron = Rect::from_center_size(center, size);
    let color = if is_open {
        ACCENT_UI
    } else {
        visuals.fg_stroke.color
    };
    let points = match above_or_below {
        egui::AboveOrBelow::Below => {
            vec![chevron.left_top(), chevron.right_top(), chevron.center_bottom()]
        }
        egui::AboveOrBelow::Above => {
            vec![
                chevron.left_bottom(),
                chevron.right_bottom(),
                chevron.center_top(),
            ]
        }
    };
    let _ = tokens;
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        Stroke::NONE,
    ));
}
