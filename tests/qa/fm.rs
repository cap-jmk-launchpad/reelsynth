//! Phase 3 Q&A — FM routing, golden bell/pluck.

use reelsynth::fm::{sample_carrier_with_fm, FmSource};
use reelsynth::osc::{VaWaveform, WtWarpMode};
use reelsynth::patch::{Oscillator, Patch};
use reelsynth::wavetable::WavetableBank;

use super::helpers::*;

#[test]
fn fm_feedback_produces_signal() {
    let mut patch = Patch::default_mono();
    patch.oscillators[0].osc_type = "sine".into();
    patch.oscillators[0].fm_source = "feedback".into();
    patch.oscillators[0].fm_index = 3.0;
    patch.filter.cutoff = 6000.0;
    patch.filter2 = patch.filter.clone();
    let audio = render_patch(&patch, 440.0, 0.15);
    assert!(peak(&audio) > 0.02);
}

#[test]
fn fm_osc2_osc3_dual_routing() {
    let mut dual = Patch::factory_fm_pluck();
    let mut single = dual.clone();
    single.oscillators[0].fm_source = "osc2".into();
    let a = render_patch(&dual, 440.0, 0.2);
    let b = render_patch(&single, 440.0, 0.2);
    let mut diff = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        diff += (x - y).abs();
    }
    assert!(diff > 0.1, "dual vs osc2 diff={diff}");
}

#[test]
fn va_sine_true_fm_differs_from_phase_mod() {
    let bank = WavetableBank::factory_sine();
    let mut phase_mod = Patch::default_mono();
    phase_mod.oscillators[0].osc_type = "triangle".into();
    phase_mod.oscillators[0].fm_source = "osc2".into();
    phase_mod.oscillators[0].fm_index = 5.0;
    phase_mod.ensure_oscillators(2);
    phase_mod.oscillators[1].osc_type = "sine".into();
    let mut true_fm = phase_mod.clone();
    true_fm.oscillators[0].osc_type = "sine".into();
    let a = render_patch(&phase_mod, 440.0, 0.1);
    let b = render_patch(&true_fm, 440.0, 0.1);
    let mut diff = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        diff += (x - y).abs();
    }
    assert!(diff > 0.1, "fm render diff={diff}");
    let _ = bank;
}

#[test]
fn wt_position_mod_fm_mode() {
    let bank = WavetableBank::factory_sine();
    let osc = Oscillator {
        osc_type: "wavetable".into(),
        ..Oscillator::default_va()
    };
    let dry = sample_carrier_with_fm(
        &osc,
        &bank,
        std::slice::from_ref(&bank),
        &[],
        0.3,
        0.01,
        16.0,
        WtWarpMode::None,
        0.0,
        0.0,
        0.0,
    );
    let wet = sample_carrier_with_fm(
        &osc,
        &bank,
        std::slice::from_ref(&bank),
        &[],
        0.3,
        0.01,
        16.0,
        WtWarpMode::None,
        0.0,
        1.0,
        4.0,
    );
    assert!((dry - wet).abs() > 1e-4);
    let _ = FmSource::Osc2; // routing enum available
}

#[test]
fn factory_fm_bell_matches_golden() {
    let patch = Patch::factory_fm_bell();
    let rendered = render_patch(&patch, 880.0, QA_DURATION);
    let golden = load_golden_wav("factory_fm_bell");
    assert_near_golden(&rendered, &golden, 0.15);
}

#[test]
fn factory_fm_pluck_matches_golden() {
    let patch = Patch::factory_fm_pluck();
    let rendered = render_patch(&patch, 440.0, QA_DURATION);
    let golden = load_golden_wav("factory_fm_pluck");
    assert_near_golden(&rendered, &golden, 0.15);
}

#[test]
fn factory_wt_lead_matches_golden() {
    let patch = Patch::factory_wt_lead();
    let rendered = render_patch(&patch, 440.0, QA_DURATION);
    let golden = load_golden_wav("factory_wt_lead");
    assert_eq!(rendered.len(), golden.len());
    assert_near_golden(&rendered, &golden, 0.2);
}
