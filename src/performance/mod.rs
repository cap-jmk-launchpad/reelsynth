//! Performance input: scales, chords, and preset settings.

mod chord;
mod scale;
mod settings;

pub use chord::{
    diatonic_quality, resolve_chord, resolve_diatonic_chord, ChordQuality, ChordSet, ChordVoicing,
};
pub use scale::{
    note_in_scale, scale_degree_to_midi, snap_note, PerformanceLayout, Scale, ScaleBehavior,
};
pub use settings::PerformanceSettings;
