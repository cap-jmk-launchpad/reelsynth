//! WCAG 2.1 relative luminance and contrast checks for theme tokens.

use egui::Color32;
use reelsynth_ui_theme::{Tokens, ACCENT_UI};

/// WCAG relative luminance for sRGB color (0..1).
pub fn relative_luminance(c: Color32) -> f32 {
    fn channel(v: f32) -> f32 {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    let r = channel(c.r() as f32 / 255.0);
    let g = channel(c.g() as f32 / 255.0);
    let b = channel(c.b() as f32 / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Contrast ratio between foreground and background (1..21).
pub fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Fail when contrast ratio is below `min_ratio`.
pub fn assert_min_contrast(label: &str, fg: Color32, bg: Color32, min_ratio: f32) {
    let ratio = contrast_ratio(fg, bg);
    assert!(
        ratio >= min_ratio - 0.01,
        "{label}: contrast {:.2}:1 below minimum {:.1}:1 (fg={fg:?} bg={bg:?})",
        ratio,
        min_ratio
    );
}

/// Verify all theme token pairs from the UI audit registry.
pub fn audit_theme_tokens() {
    let t = Tokens::default();
    assert_min_contrast("theme.text_on_bg", t.text, t.bg, 4.5);
    assert_min_contrast("theme.text_on_surface2", t.text, t.surface2, 4.5);
    assert_min_contrast("theme.text_muted_on_surface2", t.text_muted, t.surface2, 4.5);
    assert_min_contrast(
        "theme.text_secondary_on_surface2",
        t.text_secondary,
        t.surface2,
        4.5,
    );
    assert_min_contrast("theme.accent_on_on_accent", t.accent_on, t.accent, 4.5);
    assert_min_contrast("theme.accent_ui_on_bg", ACCENT_UI, t.bg, 3.0);
}

/// Scope trace colors used in the signal chain strip.
pub const SCOPE_TRACE_COLORS: [Color32; 4] = [
    Color32::from_rgb(0x5b, 0xc0, 0xde),
    Color32::from_rgb(0x9b, 0x7e, 0xde),
    Color32::from_rgb(0xde, 0x9b, 0x7e),
    Color32::from_rgb(0x4a, 0xde, 0x80),
];

pub fn audit_scope_trace_contrast(bg: Color32) {
    for (i, &color) in SCOPE_TRACE_COLORS.iter().enumerate() {
        assert_min_contrast(&format!("theme.scope_trace_colors[{i}]"), color, bg, 3.0);
    }
    assert_min_contrast("theme.wt_peak_dot", ACCENT_UI, bg, 3.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_white_contrast_is_21() {
        let ratio = contrast_ratio(Color32::WHITE, Color32::BLACK);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn theme_contrast_all_pairs() {
        audit_theme_tokens();
        audit_scope_trace_contrast(Tokens::default().bg);
    }
}
