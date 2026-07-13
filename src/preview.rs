//! Analytical single-cycle previews for scope strip (Phase 1 placeholder).

use crate::fx::FxChain;
use crate::patch::Patch;
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
    fx.set_bypass(patch.fx_bypass.clone());

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
    let mut sum = 0.0f32;
    for (oi, osc) in patch.oscillators.iter().enumerate() {
        if osc.level <= 0.0 {
            continue;
        }
        let bank_idx = bank_for_osc(oi);
        let bank = banks.get(bank_idx).or_else(|| banks.first());
        let Some(bank) = bank else {
            continue;
        };
        let frame_idx = osc.position.round() as usize;
        let frame = bank.frame(frame_idx.min(bank.num_frames.saturating_sub(1)));
        if frame.is_empty() {
            continue;
        }
        let idx = ((phase.fract() * frame.len() as f32) as usize).min(frame.len() - 1);
        sum += frame[idx] * osc.level;
    }
    sum.clamp(-1.0, 1.0)
}
