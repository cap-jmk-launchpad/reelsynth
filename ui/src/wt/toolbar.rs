//! Draw/edit tool strip above the 2D waveform view.

use egui::{Color32, FontId, Sense, Ui};
use reelsynth_ui_theme::Tokens;

use crate::layout::{RADIUS_SM, WT_TOOLBAR_HEIGHT};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WtEditTool {
    #[default]
    Select,
    Pencil,
    Line,
    Smooth,
}

impl WtEditTool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Pencil => "Pencil",
            Self::Line => "Line",
            Self::Smooth => "Smooth",
        }
    }

    fn enabled(self) -> bool {
        matches!(self, Self::Select | Self::Pencil)
    }
}

pub struct WtToolbar;

impl WtToolbar {
    pub fn show(ui: &mut Ui, tool: &mut WtEditTool) -> bool {
        let tokens = Tokens::default();
        let accent_ui = Color32::from_rgb(0x2a, 0x6b, 0x8a);
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), WT_TOOLBAR_HEIGHT),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return false;
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, RADIUS_SM, tokens.surface2);
        painter.rect_stroke(
            rect,
            RADIUS_SM,
            egui::Stroke::new(1.0_f32, tokens.border),
        );

        let mut changed = false;
        ui.allocate_ui_at_rect(rect.shrink2(egui::vec2(6.0, 4.0)), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for candidate in [
                    WtEditTool::Select,
                    WtEditTool::Pencil,
                    WtEditTool::Line,
                    WtEditTool::Smooth,
                ] {
                    if tool_button(ui, candidate, *tool == candidate, candidate.enabled()).clicked()
                    {
                        if candidate.enabled() {
                            *tool = candidate;
                            changed = true;
                        }
                    }
                }
                ui.add_space(8.0);
                let hint = match *tool {
                    WtEditTool::Pencil => "Drag on waveform to sculpt frame",
                    WtEditTool::Select => "Click strip or knob to change position",
                    _ => "",
                };
                if !hint.is_empty() {
                    ui.label(
                        egui::RichText::new(hint)
                            .size(10.0)
                            .color(tokens.text_muted),
                    );
                }
            });
        });

        let _ = accent_ui;
        changed
    }
}

fn tool_button(ui: &mut Ui, tool: WtEditTool, active: bool, enabled: bool) -> egui::Response {
    let tokens = Tokens::default();
    let accent_ui = Color32::from_rgb(0x2a, 0x6b, 0x8a);
    let label = tool.label();
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        FontId::proportional(10.0),
        if enabled {
            if active {
                tokens.accent_on
            } else {
                tokens.text
            }
        } else {
            tokens.text_muted.gamma_multiply(0.5)
        },
    );
    let size = egui::vec2(galley.size().x + 14.0, galley.size().y + 6.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let fill = if active && enabled {
            tokens.accent
        } else if response.hovered() && enabled {
            tokens.bg_muted
        } else {
            Color32::TRANSPARENT
        };
        let stroke = if active && enabled {
            accent_ui
        } else {
            tokens.border
        };
        painter.rect_filled(rect, 6.0, fill);
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, stroke));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            FontId::proportional(10.0),
            if enabled {
                if active {
                    tokens.accent_on
                } else {
                    tokens.text
                }
            } else {
                tokens.text_muted.gamma_multiply(0.5)
            },
        );
    }
    if !enabled {
        return response;
    }
    response
}
