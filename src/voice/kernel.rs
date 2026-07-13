//! Shared per-sample voice DSP kernel (offline + realtime).

use crate::osc::{sample_va, VaWaveform, WtWarpMode};
use crate::patch::{Envelope, Filter, Lfo, ModSlot, Oscillator, Patch};
use crate::wavetable::WavetableBank;

/// Per-voice DSP state shared by offline `render_note` and realtime voices.
#[derive(Clone, Debug)]
pub struct VoiceState {
    pub phases: Vec<f32>,
    pub amp_env_level: f32,
    pub amp_env_stage: u8,
    pub amp_env_time: f32,
    pub filt_env_level: f32,
    pub filt_env_stage: u8,
    pub filt_env_time: f32,
    pub svf_low: f32,
    pub svf_band: f32,
    pub svf2_low: f32,
    pub svf2_band: f32,
    pub noise_seed: u32,
}

impl VoiceState {
    pub fn new(patch: &Patch) -> Self {
        let phase_count: usize = patch
            .oscillators
            .iter()
            .map(|o| o.unison.max(1) as usize)
            .sum();
        Self {
            phases: vec![0.0; phase_count.max(1)],
            amp_env_level: 0.0,
            amp_env_stage: 0,
            amp_env_time: 0.0,
            filt_env_level: 0.0,
            filt_env_stage: 0,
            filt_env_time: 0.0,
            svf_low: 0.0,
            svf_band: 0.0,
            svf2_low: 0.0,
            svf2_band: 0.0,
            noise_seed: 1,
        }
    }

    pub fn reset(&mut self, patch: &Patch) {
        let phase_count: usize = patch
            .oscillators
            .iter()
            .map(|o| o.unison.max(1) as usize)
            .sum();
        self.phases.resize(phase_count.max(1), 0.0);
        self.phases.fill(0.0);
        self.amp_env_level = 0.0;
        self.amp_env_stage = 0;
        self.amp_env_time = 0.0;
        self.filt_env_level = 0.0;
        self.filt_env_stage = 0;
        self.filt_env_time = 0.0;
        self.svf_low = 0.0;
        self.svf_band = 0.0;
        self.svf2_low = 0.0;
        self.svf2_band = 0.0;
        self.noise_seed = self.noise_seed.wrapping_add(1);
    }
}

pub struct VoiceSampleContext<'a> {
    pub banks: &'a [WavetableBank],
    pub bank_for_osc: &'a dyn Fn(usize) -> usize,
    pub patch: &'a Patch,
    pub freq: f32,
    pub gate: bool,
    pub velocity: f32,
    pub time: f32,
    pub sample_index: u32,
    pub dt: f32,
    pub sr: f32,
}

/// Process one output frame for a single voice (stereo).
pub fn process_sample(state: &mut VoiceState, ctx: &VoiceSampleContext<'_>) -> [f32; 2] {
    let amp_env = advance_envelope(
        &mut state.amp_env_level,
        &mut state.amp_env_stage,
        &mut state.amp_env_time,
        &ctx.patch.envelope,
        ctx.gate,
        ctx.dt,
    );
    let filt_env = advance_envelope(
        &mut state.filt_env_level,
        &mut state.filt_env_stage,
        &mut state.filt_env_time,
        &ctx.patch.filter_envelope,
        ctx.gate,
        ctx.dt,
    );

    let lfo = lfo_value(&ctx.patch.lfo, ctx.time);
    let mods = compute_mods(&ctx.patch.mod_matrix, lfo, amp_env, ctx.velocity);

    let amp_mod = mods.get("amp").copied().unwrap_or(0.0);
    let amplitude = (ctx.velocity + amp_mod).clamp(0.0, 1.0) * amp_env;

    let mut left = 0.0f32;
    let mut right = 0.0f32;
    let mut phase_idx = 0usize;
    let spread_cents = 15.0f32;
    let stereo_spread = ctx.patch.unison_stereo_spread.clamp(0.0, 1.0);

    for (oi, osc) in ctx.patch.oscillators.iter().enumerate() {
        if osc.level <= 0.0 {
            phase_idx += osc.unison.max(1) as usize;
            continue;
        }

        let bank_idx = (ctx.bank_for_osc)(oi);
        let bank = ctx
            .banks
            .get(bank_idx)
            .unwrap_or_else(|| ctx.banks.first().expect("at least one bank"));

        let pos_mod = mods
            .get(&format!("osc{}_position", oi + 1))
            .copied()
            .unwrap_or(0.0);
        let wt_pos = wt_position(osc, pos_mod, lfo, &ctx.patch.lfo, bank.num_frames);
        let det_mod = mods
            .get(&format!("osc{}_detune", oi + 1))
            .copied()
            .unwrap_or(0.0);
        let unison = osc.unison.max(1) as usize;
        let va_wave = VaWaveform::from_osc_type(&osc.osc_type);
        let warp = WtWarpMode::from_str(&osc.warp_mode);
        let warp_amount = osc.warp_amount;

        for u in 0..unison {
            let det_spread = if unison > 1 {
                spread_cents * (u as f32 / (unison - 1) as f32 - 0.5) * 2.0
            } else {
                0.0
            };
            let pan_spread = if unison > 1 {
                (u as f32 / (unison - 1) as f32 - 0.5) * 2.0 * stereo_spread
            } else {
                0.0
            };
            let det = osc.detune + det_mod + det_spread;
            let osc_freq = ctx.freq * 2.0f32.powf(det / 1200.0);
            let phase_inc = osc_freq / ctx.sr;
            let phase = &mut state.phases[phase_idx];
            *phase += phase_inc;
            if *phase >= 1.0 {
                *phase -= 1.0;
            }

            let osc_sample = if let Some(wave) = va_wave {
                sample_va(wave, *phase, phase_inc, osc.pulse_width)
            } else {
                bank.sample_warped(wt_pos, *phase, warp, warp_amount)
            } * osc.level
                * amplitude
                / unison as f32;

            let (pan_l, pan_r) = equal_power_pan(osc.pan + pan_spread);
            left += osc_sample * pan_l;
            right += osc_sample * pan_r;
            phase_idx += 1;
        }
    }

    if ctx.patch.sub_level > 0.0 {
        let sub_phase = state.phases.first().copied().unwrap_or(0.0) * 0.5;
        let sub = (sub_phase * std::f32::consts::TAU).sin() * ctx.patch.sub_level * amplitude * 0.5;
        left += sub;
        right += sub;
    }
    if ctx.patch.noise_level > 0.0 {
        let noise = pseudo_noise(state.noise_seed) * ctx.patch.noise_level * amplitude;
        state.noise_seed = state.noise_seed.wrapping_add(1);
        left += noise;
        right += noise;
    }

    let filt_env_level = filt_env;

    let cutoff_mod = mods.get("filter_cutoff").copied().unwrap_or(0.0)
        + lfo_for_target(&ctx.patch.lfo, lfo, "cutoff") * ctx.patch.filter.cutoff;
    let res_mod = mods.get("filter_resonance").copied().unwrap_or(0.0);

    let cutoff1 = compute_cutoff(
        &ctx.patch.filter,
        cutoff_mod,
        ctx.freq,
        filt_env_level,
        ctx.sr,
    );
    let resonance1 = (ctx.patch.filter.resonance + res_mod).clamp(0.0, 0.95);

    let cutoff2 = compute_cutoff(
        &ctx.patch.filter2,
        cutoff_mod * 0.5,
        ctx.freq,
        filt_env_level,
        ctx.sr,
    );
    let resonance2 = ctx.patch.filter2.resonance.clamp(0.0, 0.95);

    let driven_l = soft_drive(left, ctx.patch.filter.drive);
    let driven_r = soft_drive(right, ctx.patch.filter2.drive.max(ctx.patch.filter.drive));
    let filtered_l = svf_filter(
        &mut state.svf_low,
        &mut state.svf_band,
        driven_l,
        cutoff1,
        resonance1,
        &ctx.patch.filter.filter_type,
        ctx.sr,
        ctx.patch.filter.drive,
    );
    let filtered_r = svf_filter(
        &mut state.svf2_low,
        &mut state.svf2_band,
        driven_r,
        cutoff2,
        resonance2,
        &ctx.patch.filter2.filter_type,
        ctx.sr,
        ctx.patch.filter2.drive,
    );

    [
        filtered_l.clamp(-1.0, 1.0),
        filtered_r.clamp(-1.0, 1.0),
    ]
}

fn wt_position(
    osc: &Oscillator,
    pos_mod: f32,
    lfo: f32,
    lfo_cfg: &Lfo,
    num_frames: usize,
) -> f32 {
    let max_pos = (num_frames.saturating_sub(1)).max(1) as f32;
    let morph_pos = if osc.morph_amount > 0.0 {
        osc.morph_a + (osc.morph_b - osc.morph_a) * osc.morph_amount.clamp(0.0, 1.0)
    } else {
        osc.position
    };
    (morph_pos + pos_mod + lfo_for_target(lfo_cfg, lfo, "wt_position")).clamp(0.0, max_pos)
}

fn soft_drive(input: f32, drive: f32) -> f32 {
    if drive <= 0.0 {
        return input;
    }
    (input * (1.0 + drive * 4.0)).tanh()
}

fn compute_cutoff(filter: &Filter, mod_hz: f32, freq: f32, filt_env: f32, sr: f32) -> f32 {
    let base = (filter.cutoff + mod_hz).max(25.0);
    let key_cutoff = key_tracked_cutoff(base, freq, filter.key_tracking);
    filter_env_cutoff(key_cutoff, filt_env, sr).min(sr * 0.45)
}

fn equal_power_pan(pan: f32) -> (f32, f32) {
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

fn advance_envelope(
    level: &mut f32,
    stage: &mut u8,
    time: &mut f32,
    env: &Envelope,
    gate: bool,
    dt: f32,
) -> f32 {
    if gate {
        match *stage {
            0 => {
                *time += dt;
                let a = env.attack.max(1e-4);
                *level = (*time / a).min(1.0);
                if *level >= 1.0 {
                    *stage = 1;
                    *time = 0.0;
                }
            }
            1 => {
                *time += dt;
                let d = env.decay.max(1e-4);
                let t = (*time / d).min(1.0);
                *level = 1.0 + t * (env.sustain - 1.0);
                if t >= 1.0 {
                    *stage = 2;
                }
            }
            2 => *level = env.sustain,
            3 => {
                *stage = 0;
                *time = 0.0;
            }
            _ => {}
        }
    } else if *stage != 3 {
        *stage = 3;
        *time = 0.0;
    } else {
        *time += dt;
        let r = env.release.max(1e-4);
        let t = (*time / r).min(1.0);
        *level *= 1.0 - t;
    }
    *level
}

fn svf_filter(
    low: &mut f32,
    band: &mut f32,
    input: f32,
    cutoff: f32,
    resonance: f32,
    mode: &str,
    sr: f32,
    drive: f32,
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

fn lfo_value(lfo: &Lfo, t: f32) -> f32 {
    (t * lfo.rate * std::f32::consts::TAU * 2.0).sin() * lfo.depth
}

fn lfo_for_target(lfo: &Lfo, value: f32, target: &str) -> f32 {
    if lfo.target == target {
        value
    } else {
        0.0
    }
}

fn compute_mods(
    slots: &[ModSlot],
    lfo: f32,
    env: f32,
    velocity: f32,
) -> std::collections::HashMap<String, f32> {
    let mut out = std::collections::HashMap::new();
    for slot in slots {
        if !slot.enabled {
            continue;
        }
        let src = match slot.source.as_str() {
            "lfo1" | "lfo" => lfo,
            "env1" | "env" => env,
            "velocity" | "vel" => velocity,
            "modwheel" => 0.0,
            _ => 0.0,
        };
        *out.entry(slot.target.clone()).or_insert(0.0) += src * slot.amount;
    }
    out
}

fn pseudo_noise(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    ((x >> 16) as f32 / 32768.0) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::Patch;

    fn single_bank_ctx<'a>(
        bank: &'a WavetableBank,
        patch: &'a Patch,
        freq: f32,
        gate: bool,
        velocity: f32,
        time: f32,
        dt: f32,
    ) -> VoiceSampleContext<'a> {
        VoiceSampleContext {
            banks: std::slice::from_ref(bank),
            bank_for_osc: &|_| 0,
            patch,
            freq,
            gate,
            velocity,
            time,
            sample_index: 0,
            dt,
            sr: 44100.0,
        }
    }

    #[test]
    fn velocity_scales_amplitude() {
        let bank = WavetableBank::factory_sine();
        let patch = Patch::default_mono();
        let mut low = VoiceState::new(&patch);
        let mut high = VoiceState::new(&patch);
        let dt = 1.0 / 44100.0;
        for i in 0..4410 {
            let t = i as f32 * dt;
            let ctx_low = single_bank_ctx(&bank, &patch, 440.0, true, 0.25, t, dt);
            let ctx_high = single_bank_ctx(&bank, &patch, 440.0, true, 1.0, t, dt);
            let [l_l, _] = process_sample(&mut low, &ctx_low);
            let [l_h, _] = process_sample(&mut high, &ctx_high);
            if i > 2000 {
                assert!(l_h.abs() > l_l.abs());
            }
        }
    }

    #[test]
    fn pan_moves_energy() {
        let bank = WavetableBank::factory_sine();
        let mut patch_left = Patch::default_mono();
        patch_left.oscillators[0].pan = -1.0;
        let mut patch_right = Patch::default_mono();
        patch_right.oscillators[0].pan = 1.0;
        let mut left_voice = VoiceState::new(&patch_left);
        let mut right_voice = VoiceState::new(&patch_right);
        let dt = 1.0 / 44100.0;
        let mut hard_left = 0.0f32;
        let mut soft_left = 0.0f32;
        let mut hard_right = 0.0f32;
        let mut soft_right = 0.0f32;
        for i in 0..4410 {
            let t = i as f32 * dt;
            let ctx_l = single_bank_ctx(&bank, &patch_left, 440.0, true, 1.0, t, dt);
            let ctx_r = single_bank_ctx(&bank, &patch_right, 440.0, true, 1.0, t, dt);
            let [l, r] = process_sample(&mut left_voice, &ctx_l);
            hard_left += l.abs();
            soft_left += r.abs();
            let [l2, r2] = process_sample(&mut right_voice, &ctx_r);
            soft_right += l2.abs();
            hard_right += r2.abs();
        }
        assert!(hard_left > soft_left * 2.0, "hard_left={hard_left} soft_left={soft_left}");
        assert!(hard_right > soft_right * 2.0, "hard_right={hard_right} soft_right={soft_right}");
    }

    #[test]
    fn va_saw_produces_signal() {
        let bank = WavetableBank::factory_sine();
        let mut patch = Patch::factory_va_bass();
        patch.oscillators.truncate(1);
        patch.oscillators[0].level = 1.0;
        let mut voice = VoiceState::new(&patch);
        let dt = 1.0 / 44100.0;
        let mut peak = 0.0f32;
        for i in 0..4410 {
            let t = i as f32 * dt;
            let ctx = single_bank_ctx(&bank, &patch, 55.0, true, 1.0, t, dt);
            let [l, r] = process_sample(&mut voice, &ctx);
            peak = peak.max(l.abs().max(r.abs()));
        }
        assert!(peak > 0.05, "va saw peak={peak}");
    }

    #[test]
    fn dual_filter_stereo_width() {
        let bank = WavetableBank::factory_sine();
        let mut patch = Patch::default_mono();
        patch.filter.cutoff = 400.0;
        patch.filter2.cutoff = 4000.0;
        patch.filter2.filter_type = "highpass".into();
        patch.oscillators[0].pan = 0.0;
        let mut voice = VoiceState::new(&patch);
        let dt = 1.0 / 44100.0;
        let mut diff = 0.0f32;
        for i in 0..4410 {
            let t = i as f32 * dt;
            let ctx = single_bank_ctx(&bank, &patch, 440.0, true, 1.0, t, dt);
            let [l, r] = process_sample(&mut voice, &ctx);
            if l.is_finite() && r.is_finite() {
                diff += (l - r).abs();
            }
        }
        assert!(diff > 5.0, "stereo diff={diff}");
    }

    #[test]
    fn unison_spread_widens_stereo() {
        let bank = WavetableBank::factory_sine();
        let mut narrow = Patch::default_mono();
        narrow.oscillators[0].unison = 4;
        narrow.unison_stereo_spread = 0.0;
        narrow.filter2 = narrow.filter.clone();
        let mut wide = Patch::default_mono();
        wide.oscillators[0].unison = 4;
        wide.unison_stereo_spread = 1.0;
        wide.filter2 = wide.filter.clone();
        let dt = 1.0 / 44100.0;
        let mut narrow_diff = 0.0f32;
        let mut wide_diff = 0.0f32;
        let mut v1 = VoiceState::new(&narrow);
        let mut v2 = VoiceState::new(&wide);
        for i in 0..4410 {
            let t = i as f32 * dt;
            let [l1, r1] = process_sample(
                &mut v1,
                &single_bank_ctx(&bank, &narrow, 440.0, true, 1.0, t, dt),
            );
            let [l2, r2] = process_sample(
                &mut v2,
                &single_bank_ctx(&bank, &wide, 440.0, true, 1.0, t, dt),
            );
            narrow_diff += (l1 - r1).abs();
            wide_diff += (l2 - r2).abs();
        }
        assert!(wide_diff > narrow_diff * 1.2, "narrow={narrow_diff} wide={wide_diff}");
    }
}
