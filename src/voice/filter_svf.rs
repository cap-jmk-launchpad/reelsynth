//! SVF filter and cutoff helpers.

use crate::patch::Filter;

pub(crate) fn compute_cutoff(filter: &Filter, mod_hz: f32, freq: f32, filt_env: f32, sr: f32) -> f32 {
    let base = (filter.cutoff + mod_hz).max(25.0);
    let key_cutoff = key_tracked_cutoff(base, freq, filter.key_tracking);
    filter_env_cutoff(key_cutoff, filt_env, sr).min(sr * 0.45)
}

pub(crate) fn equal_power_pan(pan: f32) -> (f32, f32) {
    let p = pan.clamp(-1.0, 1.0);
    let angle = (p + 1.0) * 0.25 * std::f32::consts::PI;
    (angle.cos(), angle.sin())
}

fn key_tracked_cutoff(base: f32, freq: f32, key_tracking: f32) -> f32 {
    if key_tracking <= 0.0 {
        return base;
    }
    let semitones = 12.0 * (freq / 440.0).log2();
    base * 2.0f32.powf(semitones * key_tracking / 12.0)
}

fn filter_env_cutoff(base: f32, env_level: f32, sr: f32) -> f32 {
    let range = base * 3.0;
    (base + env_level * range).clamp(25.0, sr * 0.45)
}

pub(crate) fn svf_filter(
    low: &mut f32,
    band: &mut f32,
    input: f32,
    cutoff: f32,
    resonance: f32,
    mode: &str,
    sr: f32,
    drive: f32,
    _dt: f32,
    _os_idx: usize,
) -> f32 {
    let driven = if drive > 0.0 {
        (input * (1.0 + drive * 2.0)).tanh()
    } else {
        input
    };
    let fc = cutoff.clamp(20.0, sr * 0.49);
    let f = 2.0 * (std::f32::consts::PI * fc / sr).sin();
    let q = 1.0 - resonance.clamp(0.0, 0.95);

    *low += f * *band;
    let high = driven - *low - q * *band;
    *band += f * high;
    *low = low.clamp(-8.0, 8.0);
    *band = band.clamp(-8.0, 8.0);

    let out = match mode.to_ascii_lowercase().as_str() {
        "highpass" | "hp" => high,
        "bandpass" | "bp" => *band,
        "notch" => *low + high,
        _ => *low,
    };

    if drive > 0.0 {
        (out * (1.0 + drive)).tanh()
    } else {
        out
    }
}

