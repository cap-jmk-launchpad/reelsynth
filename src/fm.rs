//! Inter-oscillator FM routing (phase modulation + type-aware carrier modes).

use crate::osc::{sample_stack, sample_va, uses_wave_stack, VaWaveform, WtWarpMode};
use crate::patch::Oscillator;
use crate::wavetable::WavetableBank;

/// FM modulator source for a carrier oscillator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FmSource {
    #[default]
    None,
    Osc2,
    Osc3,
    /// Osc2 + Osc3 summed modulators (algorithm preset 2+3→1).
    Osc2Osc3,
    Feedback,
}

impl FmSource {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "osc2" | "2" => Self::Osc2,
            "osc3" | "3" => Self::Osc3,
            "osc2_osc3" | "osc2+osc3" | "2+3" => Self::Osc2Osc3,
            "feedback" | "self" => Self::Feedback,
            _ => Self::None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Osc2 => "osc2",
            Self::Osc3 => "osc3",
            Self::Osc2Osc3 => "osc2_osc3",
            Self::Feedback => "feedback",
        }
    }
}

/// Apply algorithm preset labels from the UI (2→1, 3→1, 2+3→1).
pub fn fm_source_from_algorithm(idx: usize) -> FmSource {
    match idx {
        1 => FmSource::Osc2,
        2 => FmSource::Osc3,
        3 => FmSource::Osc2Osc3,
        _ => FmSource::None,
    }
}

pub fn fm_algorithm_index(source: FmSource) -> usize {
    match source {
        FmSource::Osc2 => 1,
        FmSource::Osc3 => 2,
        FmSource::Osc2Osc3 => 3,
        _ => 0,
    }
}

const PHASE_MOD_SCALE: f32 = 0.08;
const WT_POS_MOD_SCALE: f32 = 3.0;
const TRUE_FM_SCALE: f32 = 0.15;

/// Raw modulator waveform sample at `mod_phase` (0..1), using `source_osc` timbre.
pub fn modulator_sample(
    source_osc: &Oscillator,
    bank: &WavetableBank,
    mod_phase: f32,
    mod_phase_inc: f32,
    wt_pos: f32,
    warp: WtWarpMode,
    warp_amount: f32,
) -> f32 {
    if let Some(wave) = VaWaveform::from_osc_type(&source_osc.osc_type) {
        sample_va(wave, mod_phase, mod_phase_inc, source_osc.pulse_width)
    } else {
        bank.sample_warped(wt_pos, mod_phase, warp, warp_amount)
    }
}

/// Combined modulator signal for a carrier osc (includes feedback path).
pub fn fm_mod_signal(
    fm_source: FmSource,
    carrier_idx: usize,
    modulators: &[Oscillator],
    banks: &[WavetableBank],
    bank_for_osc: &dyn Fn(usize) -> usize,
    carrier_phase: f32,
    fm_ratio: f32,
    carrier_phase_inc: f32,
    feedback: f32,
) -> f32 {
    if fm_source == FmSource::None {
        return 0.0;
    }

    let mod_phase = (carrier_phase * fm_ratio.max(0.01)).fract();
    let mod_phase_inc = carrier_phase_inc * fm_ratio.max(0.01);

    let sample_from = |src_idx: usize| -> f32 {
        let osc = &modulators[src_idx];
        let bank_idx = bank_for_osc(src_idx);
        let bank = banks
            .get(bank_idx)
            .unwrap_or_else(|| banks.first().expect("at least one bank"));
        let wt_pos = osc.position;
        let warp = WtWarpMode::from_str(&osc.warp_mode);
        modulator_sample(
            osc,
            bank,
            mod_phase,
            mod_phase_inc,
            wt_pos,
            warp,
            osc.warp_amount,
        )
    };

    match fm_source {
        FmSource::None => 0.0,
        FmSource::Osc2 if carrier_idx != 1 && modulators.len() > 1 => sample_from(1),
        FmSource::Osc3 if carrier_idx != 2 && modulators.len() > 2 => sample_from(2),
        FmSource::Osc2Osc3 if carrier_idx == 0 && modulators.len() > 2 => {
            sample_from(1) + sample_from(2)
        }
        FmSource::Feedback => feedback,
        _ => 0.0,
    }
}

/// Effective carrier phase offset from phase-mod (radians scaled to cycles).
pub fn fm_phase_offset(mod_signal: f32, fm_index: f32) -> f32 {
    mod_signal * fm_index * PHASE_MOD_SCALE
}

/// WT position offset for Serum-style FM on wavetable carriers.
pub fn fm_wt_position_offset(mod_signal: f32, fm_index: f32) -> f32 {
    mod_signal * fm_index * WT_POS_MOD_SCALE
}

/// True FM frequency multiplier for VA sine carriers.
pub fn fm_freq_multiplier(mod_signal: f32, fm_index: f32) -> f32 {
    1.0 + mod_signal * fm_index * TRUE_FM_SCALE
}

/// Sample a carrier with type-aware FM applied.
pub fn sample_carrier_with_fm(
    osc: &Oscillator,
    bank: &WavetableBank,
    banks: &[WavetableBank],
    wt_ids: &[String],
    carrier_phase: f32,
    carrier_phase_inc: f32,
    wt_pos: f32,
    warp: WtWarpMode,
    warp_amount: f32,
    mod_signal: f32,
    fm_index: f32,
) -> f32 {
    if fm_index <= 0.0 || mod_signal.abs() < 1e-8 {
        return sample_carrier_raw(
            osc,
            bank,
            banks,
            wt_ids,
            carrier_phase,
            carrier_phase_inc,
            wt_pos,
            warp,
            warp_amount,
        );
    }

    let phase_off = fm_phase_offset(mod_signal, fm_index);
    let wt_pos_off = fm_wt_position_offset(mod_signal, fm_index);
    let freq_mult = fm_freq_multiplier(mod_signal, fm_index);

    if uses_wave_stack(osc) {
        return sample_stack(
            osc,
            bank,
            banks,
            wt_ids,
            carrier_phase,
            carrier_phase_inc,
            wt_pos,
            warp,
            warp_amount,
            phase_off,
            wt_pos_off,
            freq_mult,
        );
    }

    if let Some(wave) = VaWaveform::from_osc_type(&osc.osc_type) {
        match wave {
            VaWaveform::Sine => {
                let modulated_inc = carrier_phase_inc * freq_mult;
                let effective_phase = (carrier_phase + phase_off).fract();
                sample_va(wave, effective_phase, modulated_inc, osc.pulse_width)
            }
            _ => {
                let effective_phase = (carrier_phase + phase_off).fract();
                sample_va(wave, effective_phase, carrier_phase_inc, osc.pulse_width)
            }
        }
    } else {
        let effective_phase = (carrier_phase + phase_off).fract();
        let max_pos = (bank.num_frames.saturating_sub(1)).max(1) as f32;
        let effective_pos = (wt_pos + wt_pos_off).clamp(0.0, max_pos);
        bank.sample_warped(effective_pos, effective_phase, warp, warp_amount)
    }
}

fn sample_carrier_raw(
    osc: &Oscillator,
    bank: &WavetableBank,
    banks: &[WavetableBank],
    wt_ids: &[String],
    phase: f32,
    phase_inc: f32,
    wt_pos: f32,
    warp: WtWarpMode,
    warp_amount: f32,
) -> f32 {
    if uses_wave_stack(osc) {
        return sample_stack(
            osc,
            bank,
            banks,
            wt_ids,
            phase,
            phase_inc,
            wt_pos,
            warp,
            warp_amount,
            0.0,
            0.0,
            1.0,
        );
    }

    if let Some(wave) = VaWaveform::from_osc_type(&osc.osc_type) {
        sample_va(wave, phase, phase_inc, osc.pulse_width)
    } else {
        bank.sample_warped(wt_pos, phase, warp, warp_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::Oscillator;
    use crate::wavetable::WavetableBank;

    #[test]
    fn fm_source_parses() {
        assert_eq!(FmSource::from_str("osc2"), FmSource::Osc2);
        assert_eq!(FmSource::from_str("feedback"), FmSource::Feedback);
        assert_eq!(FmSource::from_str("osc2_osc3"), FmSource::Osc2Osc3);
    }

    #[test]
    fn fm_changes_wt_sample() {
        let bank = WavetableBank::factory_sine();
        let osc = Oscillator {
            osc_type: "wavetable".into(),
            ..Oscillator::default_va()
        };
        let dry = sample_carrier_with_fm(
            &osc,
            &bank,
            std::slice::from_ref(&bank),
            &[],
            0.25,
            0.01,
            0.0,
            WtWarpMode::None,
            0.0,
            0.0,
            0.0,
        );
        let wet = sample_carrier_with_fm(
            &osc,
            &bank,
            std::slice::from_ref(&bank),
            &[],
            0.25,
            0.01,
            0.0,
            WtWarpMode::None,
            0.0,
            1.0,
            5.0,
        );
        assert!((dry - wet).abs() > 1e-4, "dry={dry} wet={wet}");
    }
}
