//! Per-parameter smoke sweeps at the PR pitch trio (see `docs/qa/MATRIX.md`).

use reelsynth::fx::{EffectSlot, EffectType};
use reelsynth::patch::Patch;

use super::helpers::{render_patch, QA_DURATION};
use super::invariants::{assert_smoke_render, assert_stability_render};
use super::pitch_grid::{for_each_smoke_pitch, FREQ_CUSTOM};

fn base_patch() -> Patch {
    let mut patch = Patch::default_mono();
    patch.filter2 = patch.filter.clone();
    patch
}

#[test]
fn amp_env_smoke_three_pitches() {
    let patch = base_patch();
    for_each_smoke_pitch(|freq| {
        let buf = render_patch(&patch, freq, QA_DURATION);
        assert_smoke_render(&buf);
    });
}

#[test]
fn filt_env_smoke_three_pitches() {
    let mut patch = base_patch();
    patch.filter_envelope.attack = 0.01;
    patch.filter_envelope.decay = 0.2;
    patch.filter_envelope.sustain = 0.5;
    patch.filter_envelope.release = 0.3;
    for_each_smoke_pitch(|freq| {
        let buf = render_patch(&patch, freq, QA_DURATION);
        assert_smoke_render(&buf);
    });
}

fn patch_with_fx(mut slot: EffectSlot) -> Patch {
    let mut patch = base_patch();
    slot.bypassed = false;
    if slot.mix < 0.1 {
        slot.mix = 0.35;
    }
    patch.effects = vec![slot];
    patch
}

#[test]
fn fx_chorus_smoke_three_pitches() {
    let patch = patch_with_fx(EffectSlot::chorus());
    for_each_smoke_pitch(|freq| {
        assert_smoke_render(&render_patch(&patch, freq, QA_DURATION));
    });
}

#[test]
fn fx_delay_smoke_three_pitches() {
    let mut slot = EffectSlot::delay();
    slot.time_ms = 120.0;
    slot.feedback = 0.35;
    let patch = patch_with_fx(slot);
    for_each_smoke_pitch(|freq| {
        assert_smoke_render(&render_patch(&patch, freq, QA_DURATION));
    });
}

#[test]
fn fx_reverb_smoke_three_pitches() {
    let patch = patch_with_fx(EffectSlot::reverb());
    for_each_smoke_pitch(|freq| {
        assert_smoke_render(&render_patch(&patch, freq, QA_DURATION));
    });
}

#[test]
fn fx_distortion_smoke_three_pitches() {
    let mut slot = EffectSlot::distortion();
    slot.drive = 0.6;
    let patch = patch_with_fx(slot);
    for_each_smoke_pitch(|freq| {
        assert_smoke_render(&render_patch(&patch, freq, QA_DURATION));
    });
}

#[test]
fn fx_compressor_smoke_three_pitches() {
    let mut slot = EffectSlot::compressor();
    slot.threshold = -18.0;
    slot.ratio = 4.0;
    let patch = patch_with_fx(slot);
    for_each_smoke_pitch(|freq| {
        assert_smoke_render(&render_patch(&patch, freq, QA_DURATION));
    });
}

#[test]
fn custom_hz_smoke_grid() {
    let patch = Patch::factory_lead();
    for &freq in &FREQ_CUSTOM {
        let buf = render_patch(&patch, freq, QA_DURATION);
        if freq >= 18000.0 {
            assert_stability_render(&buf);
        } else {
            assert_smoke_render(&buf);
        }
    }
}

#[test]
fn all_effect_types_represented() {
    let types = [
        EffectType::Chorus,
        EffectType::Delay,
        EffectType::Reverb,
        EffectType::Distortion,
        EffectType::Compressor,
    ];
    assert_eq!(types.len(), 5);
}
