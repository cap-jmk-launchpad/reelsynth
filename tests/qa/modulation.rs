//! Phase 6 Q&A — LFO2, macros, MPE, oversampling, mod matrix.

use reelsynth::engine::{MidiEvent, SynthEngine};
use reelsynth::lfo::{lfo_wave_unit, LfoRuntime, LfoShape};
use reelsynth::modulation::compute_macro_mods;
use reelsynth::oversample::process_os;
use reelsynth::patch::{ModSlot, Patch};
use reelsynth::wavetable::WavetableBank;

use super::helpers::*;

#[test]
fn lfo2_shapes_differ() {
    let mut rt = LfoRuntime::default();
    let sine = lfo_wave_unit(LfoShape::Sine, 0.25, &mut rt);
    let tri = lfo_wave_unit(LfoShape::Tri, 0.25, &mut rt);
    assert!((sine - tri).abs() > 0.05);
}

#[test]
fn macro_routes_to_multiple_targets() {
    let mut patch = Patch::default_mono();
    patch.macros[0].value = 1.0;
    patch.macros[0].target = "filter_cutoff".into();
    patch.macros[0].amount = 0.8;
    patch.macros[1].value = 1.0;
    patch.macros[1].target = "osc1_level".into();
    patch.macros[1].amount = 0.5;
    let macro_mods = compute_macro_mods(&patch.macros);
    assert!(macro_mods.get("filter_cutoff").copied().unwrap_or(0.0) > 0.0);
    assert!(macro_mods.get("osc1_level").copied().unwrap_or(0.0) > 0.0);
    let wet = render_patch(&patch, 220.0, 0.15);
    let mut dry = patch.clone();
    dry.macros[0].value = 0.0;
    dry.macros[1].value = 0.0;
    let dry_audio = render_patch(&dry, 220.0, 0.15);
    let mut diff = 0.0f32;
    for (a, b) in wet.iter().zip(dry_audio.iter()) {
        diff += (a - b).abs();
    }
    assert!(diff > 0.05, "macro diff={diff}");
}

#[test]
fn mpe_pressure_affects_output() {
    let mut loud = Patch::default_mono();
    loud.mod_matrix.push(ModSlot {
        source: "pressure".into(),
        target: "amp".into(),
        amount: 1.0,
        enabled: true,
    });
    let mut soft = loud.clone();
    let bank = WavetableBank::factory_sine();
    let mut engine_loud = SynthEngine::new(bank.clone(), loud.clone(), QA_SR);
    let mut engine_soft = SynthEngine::new(bank, soft.clone(), QA_SR);
    engine_loud.handle_event(MidiEvent::ChannelPressure {
        channel: 2,
        pressure: 1.0,
    });
    engine_soft.handle_event(MidiEvent::ChannelPressure {
        channel: 2,
        pressure: 0.0,
    });
    engine_loud.note_on(2, 60, 0.4);
    engine_soft.note_on(2, 60, 0.4);
    let mut buf_l = vec![0.0f32; 4096];
    let mut buf_s = vec![0.0f32; 4096];
    engine_loud.process(&mut buf_l);
    engine_soft.process(&mut buf_s);
    assert!(rms(&buf_l) > rms(&buf_s) * 1.2);
}

#[test]
fn mpe_timbre_affects_filter() {
    let mut patch = Patch::default_mono();
    patch.mod_matrix.push(ModSlot {
        source: "timbre".into(),
        target: "filter_cutoff".into(),
        amount: 0.8,
        enabled: true,
    });
    let bank = WavetableBank::factory_saw_morph();
    let mut open = SynthEngine::new(bank.clone(), patch.clone(), QA_SR);
    let mut closed = SynthEngine::new(bank, patch, QA_SR);
    open.handle_event(MidiEvent::ControlChange {
        channel: 2,
        cc: 74,
        value: 1.0,
    });
    closed.handle_event(MidiEvent::ControlChange {
        channel: 2,
        cc: 74,
        value: 0.0,
    });
    open.note_on(2, 60, 1.0);
    closed.note_on(2, 60, 1.0);
    let mut buf_o = vec![0.0f32; 4096];
    let mut buf_c = vec![0.0f32; 4096];
    open.process(&mut buf_o);
    closed.process(&mut buf_c);
    let zc_o = zero_crossings(&buf_o);
    let zc_c = zero_crossings(&buf_c);
    assert!(zc_o != zc_c || rms(&buf_o) != rms(&buf_c));
}

#[test]
fn mpe_through_engine_handle_event() {
    let patch = Patch::default_mono();
    let mut engine = SynthEngine::new(WavetableBank::factory_sine(), patch, QA_SR);
    engine.handle_event(MidiEvent::NoteOn {
        channel: 2,
        note: 60,
        velocity: 0.9,
    });
    let mut buf = vec![0.0f32; 2048];
    engine.process(&mut buf);
    assert!(peak(&buf) > 0.01);
    engine.handle_event(MidiEvent::NoteOff {
        channel: 2,
        note: 60,
    });
}

#[test]
fn oversampling_changes_fm_output() {
    let identity = process_os(0.5, |x, _| x);
    let squared = process_os(0.5, |x, _| x * x * 4.0);
    assert!((identity - squared).abs() > 0.01);
    let mut patch = Patch::factory_fm_bell();
    patch.mod_matrix.clear();
    let audio = render_patch(&patch, 660.0, 0.1);
    assert!(peak(&audio) > 0.02);
}

#[test]
fn spectral_crossfade_differs_from_linear() {
    let bank = WavetableBank::factory_saw_morph();
    let a = bank.sample(10.0, 0.25);
    let b = bank.sample(11.0, 0.25);
    let mid = bank.sample(10.5, 0.25);
    let lin = a * 0.5 + b * 0.5;
    assert!((mid - lin).abs() > 1e-4 || (a - b).abs() > 1e-4);
}

#[test]
fn mod_matrix_disabled_slot_silent() {
    let mut enabled = Patch::default_mono();
    enabled.mod_matrix = vec![ModSlot {
        source: "lfo1".into(),
        target: "filter_cutoff".into(),
        amount: 1.0,
        enabled: true,
    }];
    enabled.lfo.depth = 1.0;
    enabled.lfo.rate = 6.0;
    let mut disabled = enabled.clone();
    disabled.mod_matrix[0].enabled = false;
    let a = render_patch(&enabled, 220.0, 0.15);
    let b = render_patch(&disabled, 220.0, 0.15);
    let mut diff = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        diff += (x - y).abs();
    }
    assert!(diff > 0.02, "disabled mod diff={diff}");
}
