//! Analytical single-cycle previews for scope strip (Phase 1 placeholder).

use crate::fx::FxChain;
use crate::patch::{Oscillator, Patch};
use crate::voice::{process_sample, VoiceSampleContext, VoiceState};
use crate::wavetable::WavetableBank;

const PREVIEW_SR: f32 = 48_000.0;
const PREVIEW_NOTE: f32 = 130.81; // C3

/// One scope tap buffer (normalized waveform samples).
#[derive(Clone, Debug, Default)]
pub struct ScopeTap {
    pub samples: Vec<f32>,
}

/// Four-tap signal chain preview (Osc → Filter → FX → Out).
#[derive(Clone, Debug, Default)]
pub struct ScopePreviews {
    pub osc: ScopeTap,
    pub filter: ScopeTap,
    pub fx: ScopeTap,
    pub out: ScopeTap,
}

/// Render analytical previews for the scope strip UI.
pub fn render_scope_previews(
    banks: &[WavetableBank],
    bank_for_osc: impl Fn(usize) -> usize + Copy,
    patch: &Patch,
    sample_count: usize,
) -> ScopePreviews {
    let count = sample_count.max(16);
    let gate_samples = (count as f32 * 0.75) as usize;

    let mut osc_buf = vec![0.0f32; count];
    let mut filt_buf = vec![0.0f32; count];
    let mut fx_buf = vec![0.0f32; count];
    let mut out_buf = vec![0.0f32; count];

    for i in 0..count {
        let phase = i as f32 / count as f32;
        osc_buf[i] = preview_osc_sample(banks, bank_for_osc, patch, phase);
    }

    let mut voice = VoiceState::new(patch);
    let mut fx = FxChain::new(PREVIEW_SR as u32);
    fx.set_effects(patch.effects.clone());

    for i in 0..count {
        let t = i as f32 / PREVIEW_SR;
        let gate = i < gate_samples;

        let ctx = VoiceSampleContext {
            banks,
            bank_for_osc: &bank_for_osc,
            patch,
            freq: PREVIEW_NOTE,
            gate,
            velocity: 0.85,
            time: t,
            sample_index: i as u32,
            dt: 1.0 / PREVIEW_SR,
            sr: PREVIEW_SR,
        };

        let [l, r] = process_sample(&mut voice, &ctx);
        let filtered = (l + r) * 0.5;
        filt_buf[i] = filtered;
        let [fx_l, fx_r] = fx.process_stereo(filtered, filtered);
        fx_buf[i] = (fx_l + fx_r) * 0.5;
        out_buf[i] = fx_buf[i];
    }

    ScopePreviews {
        osc: ScopeTap { samples: osc_buf },
        filter: ScopeTap { samples: filt_buf },
        fx: ScopeTap { samples: fx_buf },
        out: ScopeTap { samples: out_buf },
    }
}

fn preview_osc_sample(
    banks: &[WavetableBank],
    bank_for_osc: impl Fn(usize) -> usize,
    patch: &Patch,
    phase: f32,
) -> f32 {
    use crate::osc::{sample_va, VaWaveform, WtWarpMode};

    let mut sum = 0.0f32;
    for (oi, osc) in patch.oscillators.iter().enumerate() {
        if osc.level <= 0.0 {
            continue;
        }
        if let Some(wave) = VaWaveform::from_osc_type(&osc.osc_type) {
            sum += sample_va(wave, phase.fract(), 1.0 / 2048.0, osc.pulse_width) * osc.level;
            continue;
        }
        let bank_idx = bank_for_osc(oi);
        let bank = banks.get(bank_idx).or_else(|| banks.first());
        let Some(bank) = bank else {
            continue;
        };
        let wt_pos = preview_wt_position(osc, bank.num_frames);
        let warp = WtWarpMode::from_str(&osc.warp_mode);
        sum += bank.sample_warped(wt_pos, phase, warp, osc.warp_amount) * osc.level;
    }
    sum.clamp(-1.0, 1.0)
}

fn preview_wt_position(osc: &Oscillator, num_frames: usize) -> f32 {
    let max_pos = (num_frames.saturating_sub(1)).max(1) as f32;
    if osc.morph_amount > 0.0 {
        let pos = osc.morph_a + (osc.morph_b - osc.morph_a) * osc.morph_amount.clamp(0.0, 1.0);
        pos.clamp(0.0, max_pos)
    } else {
        osc.position.clamp(0.0, max_pos)
    }
}
