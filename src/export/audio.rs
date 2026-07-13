//! Offline render → 24-bit WAV stem.

use crate::export::ExportReport;
use crate::patch::Patch;
use crate::voice::render_note_single_bank;
use crate::wavetable::WavetableBank;
use std::path::Path;

pub fn export_audio_wav(
    preset: &Patch,
    bank: &WavetableBank,
    out_path: &Path,
    opts: &crate::export::ExportOptions,
) -> ExportReport {
    let mono = render_note_single_bank(bank, opts.freq, opts.duration, opts.sample_rate, preset);
    if mono.is_empty() {
        return ExportReport::fail("audio", "render produced empty buffer");
    }
    if let Some(parent) = out_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ExportReport::fail("audio", e.to_string());
        }
    }
    match write_wav_24bit(out_path, &mono, opts.sample_rate) {
        Ok(()) => ExportReport::ok("audio", out_path.display().to_string()),
        Err(e) => ExportReport::fail("audio", e),
    }
}

fn write_wav_24bit(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    let mut pcm = Vec::with_capacity(samples.len() * 3);
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 8388607.0) as i32;
        pcm.push((v & 0xFF) as u8);
        pcm.push(((v >> 8) & 0xFF) as u8);
        pcm.push(((v >> 16) & 0xFF) as u8);
    }
    let data_len = pcm.len() as u32;
    let riff_len = 36 + data_len;
    let mut out = Vec::with_capacity(44 + pcm.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 3;
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&3u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(&pcm);
    std::fs::write(path, out).map_err(|e| e.to_string())
}
