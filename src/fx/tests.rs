use super::*;
use super::processors::soft_clip;

#[test]
fn bypass_skips_fx() {
    let mut chain = FxChain::new(44100);
    let mut slots = default_effects();
    for slot in &mut slots {
        slot.bypassed = true;
    }
    chain.set_effects(slots);
    assert_eq!(chain.process_sample(0.5), soft_clip(0.5));
}

#[test]
fn chorus_changes_sample_when_active() {
    let mut chain = FxChain::new(44100);
    let mut slots = vec![EffectSlot::chorus()];
    slots[0].mix = 1.0;
    chain.set_effects(slots);
    let mut max_delta = 0.0f32;
    for i in 0..4000 {
        let input = (i as f32 * 0.013).sin() * 0.6;
        let out = chain.process_sample(input);
        max_delta = max_delta.max((out - soft_clip(input)).abs());
    }
    assert!(max_delta > 0.001, "chorus delta {max_delta}");
}

#[test]
fn delay_is_audible_with_mix() {
    let mut chain = FxChain::new(44100);
    let mut slot = EffectSlot::delay();
    slot.mix = 1.0;
    slot.time_ms = 50.0;
    slot.feedback = 0.0;
    chain.set_effects(vec![slot]);
    let mut impulse_response = 0.0f32;
    for i in 0..8000 {
        let input = if i == 0 { 1.0 } else { 0.0 };
        impulse_response += chain.process_sample(input).abs();
    }
    assert!(impulse_response > 0.5, "delay tail energy {impulse_response}");
}

#[test]
fn reverb_sustains_tail() {
    let mut chain = FxChain::new(44100);
    let mut slot = EffectSlot::reverb();
    slot.bypassed = false;
    slot.mix = 1.0;
    chain.set_effects(vec![slot]);
    let mut tail = 0.0f32;
    for i in 0..12000 {
        let input = if i < 100 { 0.8 } else { 0.0 };
        tail = tail.max(chain.process_sample(input).abs());
    }
    assert!(tail > 0.01, "reverb tail peak {tail}");
}

#[test]
fn soft_clip_limits_hot_signal() {
    assert!(soft_clip(2.0).abs() < 1.0);
    assert!(soft_clip(-3.0).abs() < 1.0);
}

#[test]
fn migrate_bypass_to_effects() {
    let effects = effects_from_bypass(&FxBypass::default());
    assert_eq!(effects.len(), 3);
    assert!(!effects[0].bypassed);
    assert!(!effects[1].bypassed);
    assert!(effects[2].bypassed);
}
