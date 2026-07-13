//! Generate golden WAV fixtures for Q&A tests (`tests/fixtures/audio/`).

use reelsynth::engine::SynthEngine;
use reelsynth::export::write_wav_mono;
use reelsynth::patch::Patch;
use reelsynth::wavetable::WavetableBank;
use std::path::PathBuf;

const SR: u32 = 44100;
const DURATION: f32 = 0.5;

fn bank_for_patch(patch: &Patch) -> WavetableBank {
    match patch.wavetable_id.as_deref() {
        Some("sine") => WavetableBank::factory_sine(),
        Some("metallic") => WavetableBank::factory_metallic(),
        _ => WavetableBank::factory_saw_morph(),
    }
}

fn render_preset(patch: Patch, freq: f32) -> Vec<f32> {
    let bank = bank_for_patch(&patch);
    let engine = SynthEngine::new(bank, patch, SR);
    engine.render_offline(freq, DURATION)
}

fn main() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio");
    std::fs::create_dir_all(&out_dir).expect("create audio fixtures dir");

    let presets: [(&str, Patch, f32); 4] = [
        ("factory_va_bass", Patch::factory_va_bass(), 55.0),
        ("factory_wt_lead", Patch::factory_wt_lead(), 440.0),
        ("factory_fm_bell", Patch::factory_fm_bell(), 880.0),
        ("factory_fm_pluck", Patch::factory_fm_pluck(), 440.0),
    ];

    for (name, patch, freq) in presets {
        let audio = render_preset(patch, freq);
        let path = out_dir.join(format!("{name}.wav"));
        write_wav_mono(&path, &audio, SR).expect("write wav");
        println!("wrote {} ({} samples)", path.display(), audio.len());
    }
}
