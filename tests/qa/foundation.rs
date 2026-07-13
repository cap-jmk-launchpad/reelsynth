//! Phase 1 Q&A — stereo engine, envelopes, banks, velocity.

use reelsynth::engine::SynthEngine;
use reelsynth::patch::Patch;
use reelsynth::wavetable::WavetableBank;

use super::helpers::*;

#[test]
fn engine_stereo_pan_produces_width() {
    let mut patch = Patch::default_mono();
    patch.oscillators[0].pan = -0.9;
    patch.filter2 = patch.filter.clone();
    let stereo = render_engine_stereo(&patch, 60, 1.0, 4096);
    assert_stereo_width(&stereo, 0.01);
}

#[test]
fn filter_envelope_opens_cutoff_over_time() {
    let mut closed = Patch::default_mono();
    closed.filter.cutoff = 200.0;
    closed.filter.key_tracking = 0.0;
    closed.filter2 = closed.filter.clone();
    closed.filter_envelope.sustain = 0.0;
    closed.filter_envelope.decay = 0.001;
    let mut open = closed.clone();
    open.filter_envelope.sustain = 1.0;
    open.filter_envelope.attack = 0.08;
    let c_audio = render_patch(&closed, 220.0, 0.2);
    let o_audio = render_patch(&open, 220.0, 0.2);
    assert!(
        zero_crossings(&o_audio) > zero_crossings(&c_audio),
        "closed={} open={}",
        zero_crossings(&c_audio),
        zero_crossings(&o_audio)
    );
}

#[test]
fn key_tracking_shifts_cutoff_with_note() {
    let mut patch = Patch::default_mono();
    patch.filter.cutoff = 400.0;
    patch.filter.key_tracking = 1.0;
    patch.filter2 = patch.filter.clone();
    let low = render_patch(&patch, 110.0, 0.2);
    let high = render_patch(&patch, 880.0, 0.2);
    let c_low = spectral_centroid(&low, QA_SR);
    let c_high = spectral_centroid(&high, QA_SR);
    assert!(c_high > c_low * 1.2, "low={c_low} high={c_high}");
}

#[test]
fn three_distinct_banks_render() {
    let mut patch = Patch::default_mono();
    patch.ensure_oscillators(3);
    patch.oscillators[0].wavetable_id = Some("saw_morph".into());
    patch.oscillators[0].level = 0.4;
    patch.oscillators[1].wavetable_id = Some("sine".into());
    patch.oscillators[1].level = 0.4;
    patch.oscillators[2].wavetable_id = Some("metallic".into());
    patch.oscillators[2].level = 0.4;
    let set = bank_set_for_patch(&patch);
    assert_eq!(set.banks().len(), 3);
    let audio = render_patch(&patch, 220.0, 0.2);
    assert!(peak(&audio) > 0.02);
}

#[test]
fn velocity_zero_is_silent() {
    let patch = Patch::default_mono();
    let silent = render_engine_process(&patch, 60, 0.0, 4096);
    assert!(peak(&silent) < 1e-6, "peak={}", peak(&silent));
}

#[test]
fn engine_process_with_default_patch() {
    let patch = Patch::default_mono();
    let audio = render_engine_process(&patch, 57, 1.0, 8192);
    assert!(peak(&audio) > 0.01);
}

#[test]
fn sub_osc_adds_low_energy() {
    let mut dry = Patch::default_mono();
    dry.sub_level = 0.0;
    let mut wet = Patch::default_mono();
    wet.sub_level = 0.6;
    let dry_audio = render_patch(&dry, 110.0, 0.2);
    let wet_audio = render_patch(&wet, 110.0, 0.2);
    assert!(rms(&wet_audio) > rms(&dry_audio) * 1.1);
}

#[test]
fn noise_osc_adds_broadband() {
    let mut dry = Patch::default_mono();
    dry.noise_level = 0.0;
    let mut wet = Patch::default_mono();
    wet.noise_level = 0.5;
    let dry_audio = render_patch(&dry, 220.0, 0.15);
    let wet_audio = render_patch(&wet, 220.0, 0.15);
    let zc_dry = zero_crossings(&dry_audio);
    let zc_wet = zero_crossings(&wet_audio);
    assert!(zc_wet > zc_dry, "noise should add ZC: dry={zc_dry} wet={zc_wet}");
}
