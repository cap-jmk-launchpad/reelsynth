//! Shared render + assertion helpers for Q&A scenario tests.

use std::path::PathBuf;

use approx::relative_eq;
use reelsynth::engine::{BankSet, SynthEngine};
use reelsynth::patch::Patch;
use reelsynth::wavetable::WavetableBank;

pub const QA_SR: u32 = 44100;
pub const QA_DURATION: f32 = 0.5;

pub fn primary_bank_for_patch(patch: &Patch) -> WavetableBank {
    match patch.wavetable_id.as_deref() {
        Some("sine") => WavetableBank::factory_sine(),
        Some("metallic") => WavetableBank::factory_metallic(),
        Some("square_morph") => WavetableBank::factory_square_morph(),
        Some("formant") => WavetableBank::factory_formant(),
        _ => WavetableBank::factory_saw_morph(),
    }
}

/// Offline render through `SynthEngine` (voice + FX chain).
pub fn render_patch(patch: &Patch, freq: f32, duration: f32) -> Vec<f32> {
    let bank = primary_bank_for_patch(patch);
    let engine = SynthEngine::new(bank, patch.clone(), QA_SR);
    engine.render_offline(freq, duration)
}

/// Block-based mono `SynthEngine::process` after `note_on`.
pub fn render_engine_process(
    patch: &Patch,
    note: u8,
    velocity: f32,
    frames: usize,
) -> Vec<f32> {
    let bank = primary_bank_for_patch(patch);
    let mut engine = SynthEngine::new(bank, patch.clone(), QA_SR);
    engine.note_on(0, note, velocity);
    let mut out = vec![0.0f32; frames];
    engine.process(&mut out);
    out
}

/// Interleaved stereo `[L,R,…]` via `SynthEngine::process_stereo`.
pub fn render_engine_stereo(
    patch: &Patch,
    note: u8,
    velocity: f32,
    frames: usize,
) -> Vec<f32> {
    let bank = primary_bank_for_patch(patch);
    let mut engine = SynthEngine::new(bank, patch.clone(), QA_SR);
    engine.note_on(0, note, velocity);
    let mut out = vec![0.0f32; frames * 2];
    engine.process_stereo(&mut out);
    out
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    mean.sqrt()
}

pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
}

pub fn assert_rms_in_range(buf: &[f32], min: f32, max: f32) {
    let value = rms(buf);
    assert!(
        value >= min && value <= max,
        "rms {value} not in [{min}, {max}]"
    );
}

/// Zero-crossing rate proxy for brightness.
pub fn zero_crossings(samples: &[f32]) -> usize {
    samples
        .windows(2)
        .filter(|w| w[0].signum() != w[1].signum() && w[0].abs() > 1e-6 && w[1].abs() > 1e-6)
        .count()
}

/// Simple magnitude-weighted spectral centroid (Hz).
pub fn spectral_centroid(samples: &[f32], sr: u32) -> f32 {
    let n = samples.len();
    if n < 4 {
        return 0.0;
    }
    let mut num = 0.0f32;
    let mut den = 0.0f32;
    for k in 1..(n / 2).min(256) {
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let angle = std::f32::consts::TAU * k as f32 * i as f32 / n as f32;
            re += s * angle.cos();
            im += s * angle.sin();
        }
        let mag = (re * re + im * im).sqrt();
        let freq = k as f32 * sr as f32 / n as f32;
        num += freq * mag;
        den += mag;
    }
    if den < 1e-9 {
        0.0
    } else {
        num / den
    }
}

pub fn assert_spectral_centroid_above(buf: &[f32], sr: u32, hz: f32) {
    let c = spectral_centroid(buf, sr);
    assert!(c > hz, "centroid {c} Hz should exceed {hz} Hz");
}

/// Mean |L−R| for interleaved stereo buffer.
pub fn stereo_width(buf: &[f32]) -> f32 {
    let frames = buf.len() / 2;
    if frames == 0 {
        return 0.0;
    }
    let sum: f32 = (0..frames).map(|f| (buf[f * 2] - buf[f * 2 + 1]).abs()).sum();
    sum / frames as f32
}

pub fn assert_stereo_width(buf: &[f32], min: f32) {
    let w = stereo_width(buf);
    assert!(w > min, "stereo width {w} should exceed {min}");
}

fn fixtures_audio_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio")
}

/// Load mono 16-bit PCM golden WAV from `tests/fixtures/audio/{name}.wav`.
pub fn load_golden_wav(name: &str) -> Vec<f32> {
    let path = fixtures_audio_dir().join(format!("{name}.wav"));
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    parse_wav_mono_f32(&data).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn parse_wav_mono_f32(data: &[u8]) -> Result<Vec<f32>, String> {
    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err("not a WAV file".into());
    }
    let channels = u16::from_le_bytes([data[22], data[23]]);
    let bits = u16::from_le_bytes([data[34], data[35]]);
    if channels != 1 || bits != 16 {
        return Err(format!("expected mono 16-bit, got {channels}ch {bits}bit"));
    }
    if &data[36..40] != b"data" {
        return Err("expected data chunk at offset 36".into());
    }
    let pcm_len = u32::from_le_bytes([data[40], data[41], data[42], data[43]]) as usize;
    let samples_start = 44;
    let pcm = &data[samples_start..samples_start + pcm_len.min(data.len() - samples_start)];
    Ok(pcm
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect())
}

/// Compare buffers with RMS-relative tolerance (skip initial transient).
pub fn assert_near_golden(buf: &[f32], golden: &[f32], rtol: f32) {
    let skip = (golden.len() / 20).max(64).min(golden.len() / 2);
    let a = &buf[skip..buf.len().min(golden.len())];
    let b = &golden[skip..];
    let n = a.len().min(b.len());
    assert!(n > 100, "overlap too short for comparison");
    let a_rms = rms(&a[..n]);
    let b_rms = rms(&b[..n]);
    assert!(
        relative_eq!(a_rms, b_rms, max_relative = rtol),
        "rms mismatch: got {a_rms} golden {b_rms} (rtol {rtol})"
    );
}

pub fn bank_set_for_patch(patch: &Patch) -> BankSet {
    BankSet::from_primary(primary_bank_for_patch(patch), patch)
}
