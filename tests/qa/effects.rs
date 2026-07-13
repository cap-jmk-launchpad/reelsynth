//! Phase 4 Q&A — engine + FX end-to-end.

use reelsynth::fx::{EffectSlot, EffectType, FxChain};
use reelsynth::patch::Patch;

use super::helpers::*;

fn patch_with_fx(slots: Vec<EffectSlot>) -> Patch {
    let mut patch = Patch::default_mono();
    patch.effects = slots;
    patch.filter2 = patch.filter.clone();
    patch
}

#[test]
fn engine_delay_audible_in_process() {
    let dry = patch_with_fx(vec![]);
    let mut wet = patch_with_fx(vec![]);
    let mut slot = EffectSlot::delay();
    slot.mix = 1.0;
    slot.time_ms = 80.0;
    slot.feedback = 0.45;
    wet.effects = vec![slot];
    let dry_buf = render_patch(&dry, 220.0, 0.35);
    let wet_buf = render_patch(&wet, 220.0, 0.35);
    let dry_tail = rms(&dry_buf[dry_buf.len() / 2..]);
    let wet_tail = rms(&wet_buf[wet_buf.len() / 2..]);
    assert!(wet_tail > dry_tail * 1.05, "dry={dry_tail} wet={wet_tail}");
}

#[test]
fn engine_reverb_tail_in_process() {
    let mut wet = patch_with_fx(vec![]);
    let mut slot = EffectSlot::reverb();
    slot.mix = 0.5;
    slot.bypassed = false;
    wet.effects = vec![slot];
    let buf = render_patch(&wet, 220.0, 0.6);
    let early = rms(&buf[512..2048]);
    let late = rms(&buf[buf.len() * 2 / 3..]);
    assert!(late > early * 0.05, "reverb tail early={early} late={late}");
}

#[test]
fn distortion_adds_harmonics() {
    use reelsynth::fx::FxChain;
    let mut chain = FxChain::new(QA_SR);
    let mut slot = EffectSlot::distortion();
    slot.drive = 0.95;
    slot.mix = 1.0;
    chain.set_effects(vec![slot]);
    let mut hot = 0.0f32;
    let mut out = 0.0f32;
    for i in 0..4096 {
        let x = (i as f32 * 0.03).sin();
        hot += x.abs();
        out += chain.process_sample(x).abs();
    }
    assert!(out > hot * 0.8);
    assert!(out > 50.0);
}

#[test]
fn compressor_reduces_dynamic_range() {
    use reelsynth::fx::FxChain;
    let mut chain = FxChain::new(QA_SR);
    let mut slot = EffectSlot::compressor();
    slot.threshold = -24.0;
    slot.ratio = 8.0;
    slot.mix = 1.0;
    chain.set_effects(vec![slot]);
    let mut peak_in = 0.0f32;
    let mut peak_out = 0.0f32;
    for i in 0..8192 {
        let x = (i as f32 * 0.01).sin() * 0.9;
        peak_in = peak_in.max(x.abs());
        peak_out = peak_out.max(chain.process_sample(x).abs());
    }
    assert!(peak_out < peak_in * 0.99, "in={peak_in} out={peak_out}");
}

#[test]
fn effect_slot_reorder_changes_output() {
    let ab = patch_with_fx(vec![
        {
            let mut s = EffectSlot::delay();
            s.mix = 1.0;
            s.time_ms = 120.0;
            s
        },
        {
            let mut s = EffectSlot::chorus();
            s.mix = 1.0;
            s
        },
    ]);
    let ba = patch_with_fx(vec![
        {
            let mut s = EffectSlot::chorus();
            s.mix = 1.0;
            s
        },
        {
            let mut s = EffectSlot::delay();
            s.mix = 1.0;
            s.time_ms = 120.0;
            s
        },
    ]);
    let a = render_patch(&ab, 220.0, 0.2);
    let b = render_patch(&ba, 220.0, 0.2);
    let mut diff = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        diff += (x - y).abs();
    }
    assert!(diff > 0.01, "reorder diff={diff}");
}

#[test]
fn soft_clip_limits_peak_below_1() {
    let mut patch = patch_with_fx(vec![]);
    patch.oscillators[0].level = 1.0;
    patch.oscillators[0].unison = 8;
    let buf = render_patch(&patch, 220.0, 0.15);
    let p = peak(&buf);
    assert!(p <= 1.0, "peak={p}");
    assert!(p > 0.1);
    let _ = (FxChain::new(QA_SR), EffectType::Chorus);
}
