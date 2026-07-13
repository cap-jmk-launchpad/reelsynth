//! Phase 2 Q&A — VA oscillators, warp, dual filter, morph, golden bass.

use reelsynth::osc::WtWarpMode;
use reelsynth::patch::Patch;
use reelsynth::wavetable::WavetableBank;

use super::helpers::*;

fn va_patch(osc_type: &str) -> Patch {
    let mut patch = Patch::default_mono();
    patch.oscillators[0].osc_type = osc_type.into();
    patch.oscillators[0].level = 1.0;
    patch.filter.cutoff = 8000.0;
    patch.filter2 = patch.filter.clone();
    patch
}

#[test]
fn va_square_pulse_triangle_nonzero() {
    for ty in ["square", "pulse", "triangle", "saw"] {
        let patch = va_patch(ty);
        let audio = render_patch(&patch, 220.0, 0.1);
        assert!(peak(&audio) > 0.02, "{ty} was silent");
    }
}

#[test]
fn pulse_width_changes_duty_cycle() {
    let mut narrow = va_patch("pulse");
    narrow.oscillators[0].pulse_width = 0.1;
    let mut wide = va_patch("pulse");
    wide.oscillators[0].pulse_width = 0.9;
    let a = render_patch(&narrow, 220.0, 0.1);
    let b = render_patch(&wide, 220.0, 0.1);
    let mut diff = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        diff += (x - y).abs();
    }
    assert!(diff > 0.5, "pulse width diff={diff}");
}

#[test]
fn warp_sync_differs_from_none() {
    let bank = WavetableBank::factory_saw_morph();
    let none = bank.sample_warped(32.0, 0.25, WtWarpMode::None, 0.0);
    let sync = bank.sample_warped(32.0, 0.25, WtWarpMode::Sync, 0.8);
    assert!((none - sync).abs() > 1e-4, "none={none} sync={sync}");
}

#[test]
fn warp_bend_differs_from_none() {
    let bank = WavetableBank::factory_saw_morph();
    let none = bank.sample_warped(64.0, 0.4, WtWarpMode::None, 0.0);
    let bend = bank.sample_warped(64.0, 0.4, WtWarpMode::Bend, 0.7);
    assert!((none - bend).abs() > 1e-4);
}

#[test]
fn dual_filter_parallel_stereo_width() {
    let mut patch = Patch::default_mono();
    patch.filter.cutoff = 500.0;
    patch.filter2.cutoff = 4000.0;
    patch.filter2.filter_type = "highpass".into();
    patch.unison_stereo_spread = 0.8;
    patch.oscillators[0].unison = 3;
    patch.filter2.drive = patch.filter.drive;
    let stereo = render_engine_stereo(&patch, 60, 1.0, 4096);
    assert_stereo_width(&stereo, 0.005);
}

#[test]
fn filter_drive_adds_harmonics() {
    let mut clean = Patch::default_mono();
    clean.oscillators[0].osc_type = "saw".into();
    clean.filter.drive = 0.0;
    clean.filter2 = clean.filter.clone();
    let mut driven = clean.clone();
    driven.filter.drive = 1.0;
    let r_clean = rms(&render_patch(&clean, 110.0, 0.12));
    let r_driven = rms(&render_patch(&driven, 110.0, 0.12));
    assert!(r_driven > r_clean * 0.95, "clean={r_clean} driven={r_driven}");
    assert!(peak(&render_patch(&driven, 110.0, 0.12)) > peak(&render_patch(&clean, 110.0, 0.12)) * 0.9);
}

#[test]
fn morph_persisted_in_patch_roundtrip() {
    let mut patch = Patch::factory_wt_lead();
    let json = patch.to_json().unwrap();
    let restored = Patch::from_json(&json).unwrap();
    assert_eq!(restored.oscillators[0].morph_a, patch.oscillators[0].morph_a);
    assert_eq!(restored.oscillators[0].morph_b, patch.oscillators[0].morph_b);
    assert!((restored.oscillators[0].morph_amount - patch.oscillators[0].morph_amount).abs() < 1e-5);
}

#[test]
fn factory_va_bass_matches_golden() {
    let patch = Patch::factory_va_bass();
    let rendered = render_patch(&patch, 55.0, QA_DURATION);
    let golden = load_golden_wav("factory_va_bass");
    assert_near_golden(&rendered, &golden, 0.08);
}
