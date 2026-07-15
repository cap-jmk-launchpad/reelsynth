//! Allocate child UI at a fixed screen rect (egui 0.30+).

use egui::{Rect, Ui, UiBuilder};

/// Place a top-down child `Ui` in `rect`, clipped so content cannot paint into
/// neighboring bands (e.g. right rail into the piano/footer strip).
pub fn region<R>(ui: &mut Ui, rect: Rect, body: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
        let clip = rect.intersect(ui.clip_rect());
        if clip.is_positive() {
            ui.set_clip_rect(clip);
        }
        body(ui)
    })
    .inner
}
