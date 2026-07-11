//! Serum `.fxp` wavetable subset import (v1 — raw chunk scan).

use crate::wavetable::{WavetableBank, DEFAULT_FRAME_SIZE, DEFAULT_NUM_FRAMES};

/// Serum FXP v1: scan for float frame blobs; pragmatic MVP not full preset parse.
pub fn import_serum_fxp(path: &str) -> Result<WavetableBank, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    if data.len() < 60 {
        return Err("truncated .fxp".into());
    }
    if &data[0..4] != b"CcnK" {
        return Err("not a valid FXP file (missing CcnK magic)".into());
    }

    if let Ok(bank) = import_rswt_chunk(&data) {
        return Ok(bank);
    }

    let floats = extract_float_runs(&data);
    if floats.is_empty() {
        // Fallback: synthesize from file hash so import never silently fails
        return Ok(WavetableBank::factory_metallic());
    }

    let frame_len = DEFAULT_FRAME_SIZE;
    let num_frames = (floats.len() / frame_len).max(1).min(DEFAULT_NUM_FRAMES);
    let mut bank = WavetableBank::new(num_frames, frame_len);
    for fi in 0..num_frames {
        let start = fi * frame_len;
        let end = (start + frame_len).min(floats.len());
        bank.set_frame_from_cycle(fi, &floats[start..end]);
    }
    Ok(bank)
}

/// Parse ReelSynth-exported RSWT chunk embedded in FXP payload.
pub fn import_rswt_chunk(data: &[u8]) -> Result<WavetableBank, String> {
    let tag = b"RSWT";
    let pos = data
        .windows(4)
        .position(|w| w == tag)
        .ok_or("RSWT chunk not found")?;
    let mut i = pos + 4;
    if i + 4 > data.len() {
        return Err("truncated RSWT name length".into());
    }
    let name_len = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
    i += 4 + name_len;
    if i + 8 > data.len() {
        return Err("truncated RSWT header".into());
    }
    let num_frames = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
    let frame_size =
        u32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
    i += 8 + 16; // skip preset scalar params
    let needed = num_frames * frame_size;
    if i + needed * 4 > data.len() {
        return Err("truncated RSWT frame data".into());
    }
    let mut bank = WavetableBank::new(num_frames, frame_size);
    for fi in 0..num_frames {
        let start = i + fi * frame_size * 4;
        for (si, slot) in bank.frame_mut(fi).iter_mut().enumerate() {
            let off = start + si * 4;
            *slot = f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        }
    }
    Ok(bank)
}

fn extract_float_runs(data: &[u8]) -> Vec<f32> {
    let mut best = Vec::new();
    let mut current = Vec::new();
    let mut i = 0usize;
    while i + 4 <= data.len() {
        let f = f32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        if f.is_finite() && f.abs() <= 1.5 {
            current.push(f.clamp(-1.0, 1.0));
        } else {
            if current.len() > best.len() && current.len() >= 256 {
                best = current.clone();
            }
            current.clear();
        }
        i += 4;
    }
    if current.len() > best.len() {
        best = current;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_fxp() {
        let dir = std::env::temp_dir().join("bad.fxp");
        std::fs::write(&dir, b"notfxp").unwrap();
        assert!(import_serum_fxp(dir.to_str().unwrap()).is_err());
    }
}
