//! Render invariant assertions for the QA matrix.

use super::helpers::{peak, rms};

/// Minimum RMS for an audible note at full velocity.
pub const RMS_EPSILON: f32 = 1e-5;

/// Hard peak ceiling (post soft-clip).
pub const PEAK_MAX: f32 = 1.0;

/// Every sample must be finite.
pub fn assert_render_finite(buf: &[f32]) {
    for (i, &s) in buf.iter().enumerate() {
        assert!(s.is_finite(), "sample[{i}] = {s} is not finite");
    }
}

/// Peak magnitude must not exceed `max`.
pub fn assert_peak_bounded(buf: &[f32], max: f32) {
    let p = peak(buf);
    assert!(p <= max, "peak {p} exceeds max {max}");
}

/// RMS must exceed epsilon (audible output).
pub fn assert_rms_above_epsilon(buf: &[f32], epsilon: f32) {
    let value = rms(buf);
    assert!(value > epsilon, "rms {value} should exceed {epsilon}");
}

/// Standard smoke check: finite, bounded peak, audible RMS.
pub fn assert_smoke_render(buf: &[f32]) {
    assert_render_finite(buf);
    assert_peak_bounded(buf, PEAK_MAX);
    assert_rms_above_epsilon(buf, RMS_EPSILON);
}

/// High-frequency stability: finite output with bounded peak (no RMS floor).
pub fn assert_stability_render(buf: &[f32]) {
    assert_render_finite(buf);
    assert_peak_bounded(buf, PEAK_MAX);
}

#[test]
fn invariants_reject_nan_and_inf() {
    assert_render_finite(&[0.0, 0.5, -0.5]);
}

#[test]
#[should_panic(expected = "not finite")]
fn invariants_catch_nan() {
    assert_render_finite(&[0.0, f32::NAN]);
}

#[test]
fn peak_bounded_accepts_unit_scale() {
    assert_peak_bounded(&[0.0, 0.9, -0.8], PEAK_MAX);
}

#[test]
fn rms_epsilon_detects_silence() {
    assert_rms_above_epsilon(&[0.0, 0.01, -0.01], RMS_EPSILON);
}
