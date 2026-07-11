//! `.reelwt` + `.reelpreset` → Serum `.fxp` wavetable subset (v1).

use crate::export::{DroppedParam, ExportReport, SERUM_MOD_SLOTS_V1};
use crate::patch::Patch;
use crate::wavetable::WavetableBank;
use std::path::Path;

const FXP_MAGIC: &[u8; 4] = b"CcnK";
const FXP_WT_TAG: &[u8; 4] = b"RSWT";

pub fn export_serum_wt(
    bank: &WavetableBank,
    preset: &Patch,
    out_path: &Path,
    name: &str,
) -> ExportReport {
    let mut dropped = collect_dropped_mod(preset);
    if preset.lfo.depth > 0.0 {
        dropped.push(DroppedParam {
            path: "lfo.depth".into(),
            reason: "Serum v1 export maps WT position only; LFO depth not embedded".into(),
        });
    }
    if preset.sub_level > 0.0 {
        dropped.push(DroppedParam {
            path: "sub_level".into(),
            reason: "not supported in Serum WT subset v1".into(),
        });
    }
    if preset.noise_level > 0.0 {
        dropped.push(DroppedParam {
            path: "noise_level".into(),
            reason: "not supported in Serum WT subset v1".into(),
        });
    }

    let blob = build_fxp_blob(bank, preset, name);
    if let Some(parent) = out_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ExportReport::fail("serum", e.to_string());
        }
    }
    match std::fs::write(out_path, blob) {
        Ok(()) => ExportReport::ok("serum", out_path.display().to_string()).with_dropped(dropped),
        Err(e) => ExportReport::fail("serum", e.to_string()),
    }
}

fn collect_dropped_mod(preset: &Patch) -> Vec<DroppedParam> {
    preset
        .mod_matrix
        .iter()
        .enumerate()
        .skip(SERUM_MOD_SLOTS_V1)
        .map(|(i, slot)| DroppedParam {
            path: format!("mod_matrix[{i}]"),
            reason: format!(
                "Serum v1 supports {} mod slots; dropped {}→{}",
                SERUM_MOD_SLOTS_V1, slot.source, slot.target
            ),
        })
        .collect()
}

fn build_fxp_blob(bank: &WavetableBank, preset: &Patch, name: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(FXP_WT_TAG);
    payload.extend_from_slice(&(name.len() as u32).to_le_bytes());
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(&(bank.num_frames as u32).to_le_bytes());
    payload.extend_from_slice(&(bank.frame_size as u32).to_le_bytes());

    let wt_pos = preset
        .oscillators
        .first()
        .map(|o| o.position)
        .unwrap_or(0.0);
    payload.extend_from_slice(&wt_pos.to_le_bytes());
    payload.extend_from_slice(&preset.filter.cutoff.to_le_bytes());
    payload.extend_from_slice(&preset.envelope.attack.to_le_bytes());
    payload.extend_from_slice(&preset.envelope.release.to_le_bytes());

    for fi in 0..bank.num_frames {
        for &sample in bank.frame(fi) {
            payload.extend_from_slice(&sample.to_le_bytes());
        }
    }

    let mut out = Vec::new();
    out.extend_from_slice(FXP_MAGIC);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes()); // fx type preset
    out.extend_from_slice(b"ReSy");
    out.extend_from_slice(&0u32.to_le_bytes()); // preset index
    out.extend_from_slice(&(name.len() as u32).to_le_bytes());
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(&payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::import_serum_fxp;

    #[test]
    fn roundtrip_serum_frames() {
        let bank = WavetableBank::factory_sine();
        let preset = Patch::default_mono();
        let dir = std::env::temp_dir().join("reelsynth_rt_serum");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("patch.fxp");
        let report = export_serum_wt(&bank, &preset, &out, "test");
        assert!(report.success);
        let reimport = import_serum_fxp(out.to_str().unwrap()).unwrap();
        assert_eq!(reimport.num_frames, bank.num_frames);
        assert_eq!(reimport.frame_size, bank.frame_size);
    }
}
