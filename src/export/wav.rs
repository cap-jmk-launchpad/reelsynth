//! `.reelwt` → folder of single-cycle WAV files.

use crate::export::ExportReport;
use crate::wavetable::WavetableBank;
use std::path::Path;

pub fn export_wav_folder(bank: &WavetableBank, out_dir: &Path) -> ExportReport {
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        return ExportReport::fail("wav", e.to_string());
    }
    for fi in 0..bank.num_frames {
        let path = out_dir.join(format!("frame_{:03}.wav", fi));
        if let Err(e) = write_wav_mono(&path, bank.frame(fi), 44100) {
            return ExportReport::fail("wav", e);
        }
    }
    ExportReport::ok("wav", out_dir.display().to_string())
}

pub fn write_wav_mono(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        pcm.extend_from_slice(&v.to_le_bytes());
    }
    let data_len = pcm.len() as u32;
    let riff_len = 36 + data_len;
    let mut out = Vec::with_capacity(44 + pcm.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2;
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(&pcm);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{export_wavetable, ExportOptions, ExportTarget};
    use crate::import::import_wav_folder;

    #[test]
    fn roundtrip_wav() {
        let bank = WavetableBank::factory_square_morph();
        let dir = std::env::temp_dir().join("reelsynth_rt_wav");
        let frames_dir = dir.join("frames");
        let _ = std::fs::remove_dir_all(&dir);
        let report = export_wavetable(
            &bank,
            ExportTarget::Wav,
            &frames_dir,
            &ExportOptions::default(),
        );
        assert!(report.success);
        let reimport = import_wav_folder(frames_dir.to_str().unwrap()).unwrap();
        assert_eq!(reimport.num_frames, bank.num_frames);
    }
}
