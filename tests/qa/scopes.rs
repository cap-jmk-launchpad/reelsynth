//! Phase 5 Q&A — scope previews, spectrum, FX tap.

use reelsynth::engine::note_to_freq;
use reelsynth::fx::{EffectSlot, FxChain};
use reelsynth::patch::Patch;
use reelsynth::{
    render_scope_previews, spectrum_magnitudes, PREVIEW_FIFTH_NOTE, PREVIEW_ROOT_NOTE,
};
use reelsynth::wavetable::WavetableBank;

use super::helpers::*;

#[test]
fn preview_power_chord_two_frequencies() {
    let root = note_to_freq(PREVIEW_ROOT_NOTE);
    let fifth = note_to_freq(PREVIEW_FIFTH_NOTE);
    assert!((fifth / root - 1.5).abs() < 0.02);
    let bank = WavetableBank::factory_saw_morph();
    let patch = Patch::default_mono();
    let previews = render_scope_previews(std::slice::from_ref(&bank), |_| 0, &patch, 128);
    assert!(peak(&previews.osc.samples) > 0.01);
    assert!(peak(&previews.out.samples) > 0.01);
}

#[test]
fn preview_filter_tap_changes_with_cutoff() {
    let bank = WavetableBank::factory_saw_morph();
    let mut bright = Patch::default_mono();
    bright.filter.cutoff = 9000.0;
    bright.filter.key_tracking = 0.0;
    let mut dark = Patch::default_mono();
    dark.filter.cutoff = 180.0;
    dark.filter.key_tracking = 0.0;
    let b = render_scope_previews(std::slice::from_ref(&bank), |_| 0, &bright, 64);
    let d = render_scope_previews(std::slice::from_ref(&bank), |_| 0, &dark, 64);
    assert!(
        zero_crossings(&b.filter.samples) > zero_crossings(&d.filter.samples),
        "bright filter tap should have more ZC"
    );
}

#[test]
fn scope_monitor_live_feed_nonzero_when_playing() {
    let patch = Patch::default_mono();
    let mut engine = reelsynth::engine::SynthEngine::new(
        WavetableBank::factory_saw_morph(),
        patch,
        QA_SR,
    );
    engine.note_on(0, 60, 1.0);
    let mut block = vec![0.0f32; 2048];
    engine.process(&mut block);
    let taps = engine.scope_monitor().snapshot();
    assert!(engine.scope_monitor().is_playing());
    let snap_peak = taps
        .out
        .snapshot(32)
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    assert!(snap_peak > 0.001, "scope peak={snap_peak}");
}

#[test]
fn spectrum_out_normalized() {
    let bank = WavetableBank::factory_saw_morph();
    let patch = Patch::default_mono();
    let previews = render_scope_previews(std::slice::from_ref(&bank), |_| 0, &patch, 64);
    let bars = spectrum_magnitudes(&previews.out.samples, 24);
    assert_eq!(bars.len(), 24);
    let max = bars.iter().copied().fold(0.0f32, f32::max);
    assert!(max <= 1.0 && max > 0.05);
}

#[test]
fn fx_tap_smears_with_delay_enabled() {
    let bank = WavetableBank::factory_saw_morph();
    let mut dry = Patch::default_mono();
    dry.effects.clear();
    let mut wet = Patch::default_mono();
    wet.effects.clear();
    let mut slot = EffectSlot::delay();
    slot.mix = 0.55;
    slot.time_ms = 150.0;
    wet.effects.push(slot);
    let dry_prev = render_scope_previews(std::slice::from_ref(&bank), |_| 0, &dry, 128);
    let wet_prev = render_scope_previews(std::slice::from_ref(&bank), |_| 0, &wet, 128);
    let dry_var = variance(&dry_prev.fx.samples);
    let wet_var = variance(&wet_prev.fx.samples);
    assert!(wet_var < dry_var * 0.95 || wet_var > dry_var * 1.05);
    let _ = FxChain::new(QA_SR);
}

fn variance(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    samples.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / samples.len() as f32
}
