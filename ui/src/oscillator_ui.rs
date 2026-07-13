//! Per-oscillator UI state (unlimited count).

use reelsynth::patch::Oscillator;

use crate::osc_column::{fm_algorithm_index, fm_source_index, osc_type_index, warp_mode_index};

pub const MIN_OSCILLATORS: usize = 1;

#[derive(Debug, Clone)]
pub struct OscillatorUi {
    pub osc_type: usize,
    pub level: f32,
    pub pan: f32,
    pub coarse: f32,
    pub unison: u32,
    pub position: f32,
    pub pulse_width: f32,
    pub warp_mode: usize,
    pub warp_amount: f32,
    pub fm_source: usize,
    pub fm_algorithm: usize,
    pub fm_ratio: f32,
    pub fm_index: f32,
    pub morph_a: f32,
    pub morph_b: f32,
    pub morph_amount: f32,
}

impl Default for OscillatorUi {
    fn default() -> Self {
        Self::new_silent()
    }
}

impl OscillatorUi {
    pub fn new_silent() -> Self {
        Self {
            osc_type: 0,
            level: 0.0,
            pan: 0.0,
            coarse: 0.0,
            unison: 1,
            position: 0.0,
            pulse_width: 0.5,
            warp_mode: 0,
            warp_amount: 0.0,
            fm_source: 0,
            fm_algorithm: 0,
            fm_ratio: 1.0,
            fm_index: 0.0,
            morph_a: 0.0,
            morph_b: 255.0,
            morph_amount: 0.0,
        }
    }

    pub fn new_active() -> Self {
        Self {
            level: 0.85,
            unison: 3,
            position: 108.0,
            ..Self::new_silent()
        }
    }

    pub fn from_patch(osc: &Oscillator) -> Self {
        Self {
            osc_type: osc_type_index(&osc.osc_type),
            level: osc.level,
            pan: osc.pan,
            coarse: osc.detune,
            unison: osc.unison,
            position: osc.position,
            pulse_width: osc.pulse_width,
            warp_mode: warp_mode_index(&osc.warp_mode),
            warp_amount: osc.warp_amount,
            fm_source: fm_source_index(&osc.fm_source),
            fm_algorithm: fm_algorithm_index(&osc.fm_source),
            fm_ratio: osc.fm_ratio,
            fm_index: osc.fm_index,
            morph_a: osc.morph_a,
            morph_b: osc.morph_b,
            morph_amount: osc.morph_amount,
        }
    }
}
