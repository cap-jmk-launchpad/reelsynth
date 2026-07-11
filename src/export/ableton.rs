//! `.reelpreset` → Ableton Wavetable param JSON (offline map, no Live API).

use crate::export::{DroppedParam, ExportReport};
use crate::patch::Patch;
use serde_json::json;
use std::path::Path;

fn norm_cutoff_hz(cutoff: f32) -> f32 {
    let min = 20.0f32;
    let max = 20000.0f32;
    let c = cutoff.clamp(min, max);
    ((c.ln() - min.ln()) / (max.ln() - min.ln())).clamp(0.0, 1.0)
}

fn norm_time(seconds: f32, max: f32) -> f32 {
    (seconds / max).clamp(0.0, 1.0)
}

fn osc_position_norm(preset: &Patch) -> f32 {
    let pos = preset
        .oscillators
        .first()
        .map(|o| o.position)
        .unwrap_or(0.0);
    if pos <= 1.0 {
        pos.clamp(0.0, 1.0)
    } else {
        (pos / 255.0).clamp(0.0, 1.0)
    }
}

pub fn export_ableton_map(preset: &Patch, out_path: &Path) -> ExportReport {
    let mut dropped = Vec::new();
    if preset.mod_matrix.len() > 4 {
        for (i, _slot) in preset.mod_matrix.iter().enumerate().skip(4) {
            dropped.push(DroppedParam {
                path: format!("mod_matrix[{i}]"),
                reason: "Ableton v1 export provides macro hints only (4 slots)".into(),
            });
        }
    }
    if preset.sub_level > 0.0 {
        dropped.push(DroppedParam {
            path: "sub_level".into(),
            reason: "Wavetable device has no sub osc in v1 map".into(),
        });
    }
    if preset.noise_level > 0.0 {
        dropped.push(DroppedParam {
            path: "noise_level".into(),
            reason: "noise osc not mapped in v1".into(),
        });
    }

    let doc = json!({
        "schema": "reelsynth-ableton-wt-v1",
        "device": "ableton:wavetable",
        "contract_id": "ableton:wavetable",
        "patch_name": preset.name,
        "parameters": {
            "osc1_pos": osc_position_norm(preset),
            "filter_freq": norm_cutoff_hz(preset.filter.cutoff),
            "filter_res": preset.filter.resonance.clamp(0.0, 1.0),
            "amp_attack": norm_time(preset.envelope.attack, 5.0),
            "amp_release": norm_time(preset.envelope.release, 8.0),
        },
        "macro_hints": preset.mod_matrix.iter().take(4).enumerate().map(|(i, slot)| {
            json!({
                "macro": i + 1,
                "source": slot.source,
                "target": slot.target,
                "amount": slot.amount,
            })
        }).collect::<Vec<_>>(),
        "notes": "Load wav_frames/ manually or via OSC handoff; this JSON maps semantic params to Live Wavetable IDs.",
    });

    if let Some(parent) = out_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ExportReport::fail("ableton", e.to_string());
        }
    }
    match serde_json::to_string_pretty(&doc) {
        Ok(text) => {
            if let Err(e) = std::fs::write(out_path, text) {
                ExportReport::fail("ableton", e.to_string())
            } else {
                ExportReport::ok("ableton", out_path.display().to_string()).with_dropped(dropped)
            }
        }
        Err(e) => ExportReport::fail("ableton", e.to_string()),
    }
}
