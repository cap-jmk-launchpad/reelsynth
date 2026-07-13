//! Patch ↔ `UiState` synchronization (extracted for Q&A roundtrip tests).

use reelsynth::patch::{Envelope, Patch};
use crate::{
    effect_slots_from_patch, effect_slots_to_patch, factory_label, fm_algorithm_index,
    fm_source_from_index, fm_source_index, mod_slots_from_patch, mod_slots_to_patch,
    osc_type_from_index, osc_type_index, warp_mode_from_index, warp_mode_index, UiState,
};

pub fn lfo_shape_from_index(idx: usize) -> &'static str {
    match idx {
        1 => "tri",
        2 => "saw",
        3 => "sh",
        _ => "sine",
    }
}

pub fn lfo_shape_index(shape: &str) -> usize {
    match shape.to_ascii_lowercase().as_str() {
        "tri" | "triangle" => 1,
        "saw" => 2,
        "sh" | "s&h" => 3,
        _ => 0,
    }
}

pub fn filter_mode_from_type(filter_type: &str) -> usize {
    match filter_type.to_ascii_lowercase().as_str() {
        "highpass" | "hp" => 1,
        "bandpass" | "bp" => 2,
        "notch" => 3,
        _ => 0,
    }
}

pub fn filter_type_from_mode(mode: usize) -> &'static str {
    match mode {
        1 => "highpass",
        2 => "bandpass",
        3 => "notch",
        _ => "lowpass",
    }
}

fn preset_category_label(patch: &Patch) -> String {
    let wt = patch
        .wavetable_id
        .as_deref()
        .unwrap_or("wavetable")
        .replace('_', " ");
    format!("Preset · Wavetable · {wt}")
}

pub fn sync_state_from_patch(state: &mut UiState, patch: &Patch) {
    state.preset_name = patch.name.clone();
    state.preset_category = preset_category_label(patch);
    state.wt_bank_name = patch
        .wavetable_id
        .as_deref()
        .and_then(factory_label)
        .map(str::to_string)
        .unwrap_or_else(|| {
            patch
                .wavetable_id
                .as_deref()
                .unwrap_or("wavetable")
                .replace('_', " ")
        });
    state.wt_position = patch
        .oscillators
        .first()
        .map(|o| o.position)
        .unwrap_or(0.0);
    for i in 0..3 {
        if let Some(osc) = patch.oscillators.get(i) {
            state.osc_level[i] = osc.level;
            state.osc_pan[i] = osc.pan;
            state.osc_coarse[i] = osc.detune;
            state.osc_unison[i] = osc.unison;
            state.osc_position[i] = osc.position;
            state.osc_type[i] = osc_type_index(&osc.osc_type);
            state.osc_pulse_width[i] = osc.pulse_width;
            state.osc_warp_mode[i] = warp_mode_index(&osc.warp_mode);
            state.osc_warp_amount[i] = osc.warp_amount;
            state.osc_fm_source[i] = fm_source_index(&osc.fm_source);
            state.osc_fm_algorithm[i] = fm_algorithm_index(&osc.fm_source);
            state.osc_fm_ratio[i] = osc.fm_ratio;
            state.osc_fm_index[i] = osc.fm_index;
        }
    }
    state.unison_stereo_spread = patch.unison_stereo_spread;
    state.filter_drive = patch.filter.drive;
    state.filter2_cutoff = patch.filter2.cutoff;
    state.filter2_resonance = patch.filter2.resonance;
    state.filter2_mode = filter_mode_from_type(&patch.filter2.filter_type);
    state.filter2_drive = patch.filter2.drive;
    for i in 0..3 {
        if let Some(osc) = patch.oscillators.get(i) {
            state.osc_morph_a[i] = osc.morph_a;
            state.osc_morph_b[i] = osc.morph_b;
            state.osc_morph_amount[i] = osc.morph_amount;
        }
    }
    let idx = state.osc_tab.min(2);
    state.wt_morph_a = state.osc_morph_a[idx];
    state.wt_morph_b = state.osc_morph_b[idx];
    state.wt_morph_amount = state.osc_morph_amount[idx];
    state.sub_level = patch.sub_level;
    state.noise_level = patch.noise_level;
    state.filter_cutoff = patch.filter.cutoff;
    state.filter_resonance = patch.filter.resonance;
    state.filter_key_tracking = patch.filter.key_tracking;
    state.filter_mode = filter_mode_from_type(&patch.filter.filter_type);
    state.env_attack = patch.envelope.attack;
    state.env_decay = patch.envelope.decay;
    state.env_sustain = patch.envelope.sustain;
    state.env_release = patch.envelope.release;
    state.filt_env_attack = patch.filter_envelope.attack;
    state.filt_env_decay = patch.filter_envelope.decay;
    state.filt_env_sustain = patch.filter_envelope.sustain;
    state.filt_env_release = patch.filter_envelope.release;
    state.lfo_rate = patch.lfo.rate;
    state.lfo_depth = patch.lfo.depth;
    state.lfo_shape = lfo_shape_index(&patch.lfo.shape);
    state.lfo2_rate = patch.lfo2.rate;
    state.lfo2_depth = patch.lfo2.depth;
    state.lfo2_shape = lfo_shape_index(&patch.lfo2.shape);
    for (i, mac) in patch.macros.iter().enumerate().take(4) {
        state.macro_values[i] = mac.value;
    }
    state.mod_routes = mod_slots_from_patch(&patch.mod_matrix);
    state.mod_route_total = state.mod_routes.len().max(24);
    state.fx_slots = effect_slots_from_patch(&patch.effects);
}

pub fn patch_from_state(state: &UiState, base: &Patch) -> Patch {
    let mut patch = base.clone();
    patch.name = state.preset_name.clone();
    patch.ensure_oscillators(3);
    for i in 0..3 {
        if let Some(osc) = patch.oscillators.get_mut(i) {
            osc.level = state.osc_level[i];
            osc.pan = state.osc_pan[i];
            osc.detune = state.osc_coarse[i];
            osc.unison = state.osc_unison[i];
            osc.position = state.osc_position[i];
            osc.osc_type = osc_type_from_index(state.osc_type[i]).into();
            osc.pulse_width = state.osc_pulse_width[i];
            osc.warp_mode = warp_mode_from_index(state.osc_warp_mode[i]).into();
            osc.warp_amount = state.osc_warp_amount[i];
            osc.morph_a = state.osc_morph_a[i];
            osc.morph_b = state.osc_morph_b[i];
            osc.morph_amount = state.osc_morph_amount[i];
            osc.fm_source = fm_source_from_index(state.osc_fm_source[i]).into();
            osc.fm_ratio = state.osc_fm_ratio[i];
            osc.fm_index = state.osc_fm_index[i];
            if state.osc_morph_amount[i] > 0.0 {
                osc.position = state.osc_morph_a[i]
                    + (state.osc_morph_b[i] - state.osc_morph_a[i]) * state.osc_morph_amount[i];
            }
        }
    }
    patch.filter.cutoff = state.filter_cutoff;
    patch.filter.resonance = state.filter_resonance;
    patch.filter.key_tracking = state.filter_key_tracking;
    patch.filter.drive = state.filter_drive;
    patch.filter.filter_type = filter_type_from_mode(state.filter_mode).into();
    patch.filter2.cutoff = state.filter2_cutoff;
    patch.filter2.resonance = state.filter2_resonance;
    patch.filter2.drive = state.filter2_drive;
    patch.filter2.filter_type = filter_type_from_mode(state.filter2_mode).into();
    patch.unison_stereo_spread = state.unison_stereo_spread;
    patch.envelope = Envelope {
        attack: state.env_attack,
        decay: state.env_decay,
        sustain: state.env_sustain,
        release: state.env_release,
    };
    patch.filter_envelope = Envelope {
        attack: state.filt_env_attack,
        decay: state.filt_env_decay,
        sustain: state.filt_env_sustain,
        release: state.filt_env_release,
    };
    patch.lfo.rate = state.lfo_rate;
    patch.lfo.depth = state.lfo_depth;
    patch.lfo.shape = lfo_shape_from_index(state.lfo_shape).into();
    patch.lfo2.rate = state.lfo2_rate;
    patch.lfo2.depth = state.lfo2_depth;
    patch.lfo2.shape = lfo_shape_from_index(state.lfo2_shape).into();
    for (i, mac) in patch.macros.iter_mut().enumerate().take(4) {
        mac.value = state.macro_values[i];
    }
    patch.sub_level = state.sub_level;
    patch.noise_level = state.noise_level;
    patch.mod_matrix = mod_slots_to_patch(&state.mod_routes);
    patch.effects = effect_slots_to_patch(&state.fx_slots);
    patch
}

#[cfg(test)]
mod tests {
    use super::*;
    use reelsynth::patch::Patch;

    #[test]
    fn factory_va_bass_roundtrip() {
        let original = Patch::factory_va_bass();
        let mut state = UiState::default();
        sync_state_from_patch(&mut state, &original);
        let restored = patch_from_state(&state, &Patch::default_mono());
        assert_eq!(restored.name, original.name);
        assert!((restored.filter.cutoff - original.filter.cutoff).abs() < 1e-3);
        assert_eq!(restored.oscillators[0].osc_type, original.oscillators[0].osc_type);
        assert!((restored.sub_level - original.sub_level).abs() < 1e-4);
    }

    #[test]
    fn factory_fm_bell_roundtrip() {
        let original = Patch::factory_fm_bell();
        let mut state = UiState::default();
        sync_state_from_patch(&mut state, &original);
        let restored = patch_from_state(&state, &Patch::default_mono());
        assert_eq!(restored.oscillators[0].fm_source, original.oscillators[0].fm_source);
        assert!((restored.oscillators[0].fm_index - original.oscillators[0].fm_index).abs() < 1e-3);
    }

    #[test]
    fn lfo_shape_roundtrip() {
        assert_eq!(lfo_shape_from_index(lfo_shape_index("triangle")), "tri");
        assert_eq!(lfo_shape_index(lfo_shape_from_index(3)), 3);
    }
}
