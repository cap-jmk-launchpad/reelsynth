//! Factory preset constructors.

use super::schema::*;

impl Patch {
    pub fn factory_va_bass() -> Self {
        Self {
            schema: SCHEMA_V2.into(),
            name: "VA Bass".into(),
            wavetable_id: Some("saw_morph".into()),
            oscillators: vec![
                Oscillator {
                    osc_type: "saw".into(),
                    level: 0.85,
                    detune: -7.0,
                    unison: 2,
                    ..Oscillator::default_va()
                },
                Oscillator {
                    osc_type: "saw".into(),
                    level: 0.55,
                    detune: 7.0,
                    unison: 1,
                    ..Oscillator::default_va()
                },
                Oscillator {
                    osc_type: "square".into(),
                    level: 0.15,
                    pulse_width: 0.25,
                    unison: 1,
                    ..Oscillator::default_va()
                },
            ],
            filter: Filter {
                cutoff: 180.0,
                resonance: 0.45,
                key_tracking: 0.35,
                drive: 0.35,
                ..Filter::default()
            },
            filter2: Filter {
                cutoff: 420.0,
                resonance: 0.3,
                filter_type: "lowpass".into(),
                key_tracking: 0.2,
                drive: 0.2,
            },
            filter_envelope: Envelope {
                attack: 0.003,
                decay: 0.45,
                sustain: 0.15,
                release: 0.35,
            },
            envelope: Envelope {
                attack: 0.005,
                decay: 0.25,
                sustain: 0.75,
                release: 0.2,
            },
            sub_level: 0.35,
            unison_stereo_spread: 0.85,
            ..Self::default_mono()
        }
    }

    /// WT lead with morph + unison spread.
    pub fn factory_wt_lead() -> Self {
        Self {
            schema: SCHEMA_V2.into(),
            name: "WT Lead".into(),
            wavetable_id: Some("saw_morph".into()),
            oscillators: vec![Oscillator {
                osc_type: "wavetable".into(),
                level: 0.9,
                position: 64.0,
                morph_a: 0.0,
                morph_b: 180.0,
                morph_amount: 0.55,
                warp_mode: "sync".into(),
                warp_amount: 0.35,
                unison: 4,
                detune: 12.0,
                pan: 0.0,
                wavetable_id: Some("saw_morph".into()),
                pulse_width: default_pulse_width(),
                fm_source: default_fm_none(),
                fm_ratio: default_fm_ratio(),
                fm_index: 0.0,
            }],
            filter: Filter {
                cutoff: 2800.0,
                resonance: 0.55,
                key_tracking: 0.65,
                drive: 0.15,
                ..Filter::default()
            },
            filter2: Filter {
                cutoff: 5200.0,
                resonance: 0.35,
                filter_type: "highpass".into(),
                key_tracking: 0.4,
                drive: 0.0,
            },
            filter_envelope: Envelope {
                attack: 0.08,
                decay: 0.5,
                sustain: 0.35,
                release: 0.6,
            },
            envelope: Envelope {
                attack: 0.02,
                decay: 0.35,
                sustain: 0.65,
                release: 0.45,
            },
            unison_stereo_spread: 1.0,
            ..Self::default_mono()
        }
    }

    /// FM bell: WT carrier + VA sine modulator (2→1).
    pub fn factory_fm_bell() -> Self {
        Self {
            schema: SCHEMA_V2.into(),
            name: "FM Bell".into(),
            wavetable_id: Some("sine".into()),
            oscillators: vec![
                Oscillator {
                    osc_type: "wavetable".into(),
                    level: 0.9,
                    position: 32.0,
                    fm_source: "osc2".into(),
                    fm_ratio: 3.5,
                    fm_index: 4.5,
                    wavetable_id: Some("sine".into()),
                    ..Oscillator::default_va()
                },
                Oscillator {
                    osc_type: "sine".into(),
                    level: 0.0,
                    fm_ratio: 1.0,
                    ..Oscillator::default_va()
                },
                Oscillator {
                    level: 0.0,
                    ..Oscillator::default_va()
                },
            ],
            filter: Filter {
                cutoff: 4200.0,
                resonance: 0.35,
                key_tracking: 0.75,
                ..Filter::default()
            },
            filter2: Filter {
                cutoff: 6800.0,
                resonance: 0.2,
                filter_type: "highpass".into(),
                key_tracking: 0.5,
                ..Filter::default()
            },
            envelope: Envelope {
                attack: 0.002,
                decay: 1.2,
                sustain: 0.05,
                release: 1.8,
            },
            filter_envelope: Envelope {
                attack: 0.001,
                decay: 0.8,
                sustain: 0.1,
                release: 1.5,
            },
            lfo: Lfo {
                rate: 0.35,
                depth: 0.15,
                target: "osc1_fm_index".into(),
                shape: default_lfo_shape(),
            },
            mod_matrix: vec![ModSlot {
                source: "lfo1".into(),
                target: "osc1_fm_index".into(),
                amount: 0.35,
                enabled: true,
            }],
            ..Self::default_mono()
        }
    }

    /// FM pluck: WT carrier with dual-mod algorithm preset (2+3→1).
    pub fn factory_fm_pluck() -> Self {
        Self {
            schema: SCHEMA_V2.into(),
            name: "FM Pluck".into(),
            wavetable_id: Some("metallic".into()),
            oscillators: vec![
                Oscillator {
                    osc_type: "wavetable".into(),
                    level: 0.85,
                    position: 48.0,
                    fm_source: "osc2_osc3".into(),
                    fm_ratio: 2.0,
                    fm_index: 3.2,
                    wavetable_id: Some("metallic".into()),
                    ..Oscillator::default_va()
                },
                Oscillator {
                    osc_type: "sine".into(),
                    level: 0.0,
                    fm_ratio: 1.0,
                    ..Oscillator::default_va()
                },
                Oscillator {
                    osc_type: "triangle".into(),
                    level: 0.0,
                    detune: 7.0,
                    ..Oscillator::default_va()
                },
            ],
            filter: Filter {
                cutoff: 3200.0,
                resonance: 0.5,
                key_tracking: 0.85,
                ..Filter::default()
            },
            filter2: Filter {
                cutoff: 5200.0,
                resonance: 0.3,
                filter_type: "bandpass".into(),
                key_tracking: 0.6,
                ..Filter::default()
            },
            envelope: Envelope {
                attack: 0.001,
                decay: 0.35,
                sustain: 0.0,
                release: 0.45,
            },
            filter_envelope: Envelope {
                attack: 0.001,
                decay: 0.25,
                sustain: 0.0,
                release: 0.35,
            },
            ..Self::default_mono()
        }
    }
}
