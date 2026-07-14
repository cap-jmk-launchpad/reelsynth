//! UI-facing performance settings (indices for dropdowns + labels).

mod header;

pub use header::{draw_performance_header, PerformanceHeaderActions};

use reelsynth::{
    ChordSet, ChordVoicing, PerformanceLayout, PerformanceSettings, Scale, ScaleBehavior,
};

pub const ROOT_NAMES: &[&str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub const SCALE_NAMES: &[&str] = &[
    "Major",
    "Minor",
    "Natural Minor",
    "Harmonic Minor",
    "Melodic Minor",
    "Dorian",
    "Phrygian",
    "Lydian",
    "Mixolydian",
    "Locrian",
    "Major Pent",
    "Minor Pent",
    "Blues",
    "Whole Tone",
    "Chromatic",
];

pub const LAYOUT_NAMES: &[&str] = &["Piano", "Scale", "Chords"];

pub const CHORD_DEGREE_LABELS: &[&str] = &["I", "ii", "iii", "IV", "V", "vi", "vii°"];

/// Mirror of [`PerformanceSettings`] using dropdown indices in [`UiState`].
#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceUi {
    pub root: usize,
    pub scale: usize,
    pub layout: usize,
    pub chord_set: usize,
    pub voicing: usize,
    pub base_octave: i8,
    pub scale_behavior: ScaleBehavior,
}

impl Default for PerformanceUi {
    fn default() -> Self {
        Self {
            root: 0,
            scale: 0,
            layout: 0,
            chord_set: 0,
            voicing: 0,
            base_octave: 4,
            scale_behavior: ScaleBehavior::default(),
        }
    }
}

impl PerformanceUi {
    pub fn from_settings(s: &PerformanceSettings) -> Self {
        Self {
            root: s.root.min(11) as usize,
            scale: scale_index(s.scale),
            layout: layout_index(s.layout),
            chord_set: match s.chord_set {
                ChordSet::Triads => 0,
                ChordSet::Sevenths => 1,
            },
            voicing: match s.voicing {
                ChordVoicing::Close => 0,
                ChordVoicing::Spread => 1,
                ChordVoicing::Root => 2,
            },
            base_octave: s.base_octave,
            scale_behavior: s.scale_behavior,
        }
    }

    pub fn to_settings(&self) -> PerformanceSettings {
        PerformanceSettings {
            root: self.root.min(ROOT_NAMES.len().saturating_sub(1)) as u8,
            scale: scale_from_index(self.scale),
            scale_behavior: self.scale_behavior,
            layout: layout_from_index(self.layout),
            chord_set: if self.chord_set == 1 {
                ChordSet::Sevenths
            } else {
                ChordSet::Triads
            },
            voicing: match self.voicing {
                1 => ChordVoicing::Spread,
                2 => ChordVoicing::Root,
                _ => ChordVoicing::Close,
            },
            base_octave: self.base_octave,
        }
    }
}

pub fn scale_index(scale: Scale) -> usize {
    match scale {
        Scale::Major => 0,
        Scale::Minor => 1,
        Scale::NaturalMinor => 2,
        Scale::HarmonicMinor => 3,
        Scale::MelodicMinor => 4,
        Scale::Dorian => 5,
        Scale::Phrygian => 6,
        Scale::Lydian => 7,
        Scale::Mixolydian => 8,
        Scale::Locrian => 9,
        Scale::MajorPent => 10,
        Scale::MinorPent => 11,
        Scale::Blues => 12,
        Scale::WholeTone => 13,
        Scale::Chromatic => 14,
    }
}

pub fn scale_from_index(idx: usize) -> Scale {
    match idx {
        1 => Scale::Minor,
        2 => Scale::NaturalMinor,
        3 => Scale::HarmonicMinor,
        4 => Scale::MelodicMinor,
        5 => Scale::Dorian,
        6 => Scale::Phrygian,
        7 => Scale::Lydian,
        8 => Scale::Mixolydian,
        9 => Scale::Locrian,
        10 => Scale::MajorPent,
        11 => Scale::MinorPent,
        12 => Scale::Blues,
        13 => Scale::WholeTone,
        14 => Scale::Chromatic,
        _ => Scale::Major,
    }
}

pub fn layout_index(layout: PerformanceLayout) -> usize {
    match layout {
        PerformanceLayout::Piano => 0,
        PerformanceLayout::Scale => 1,
        PerformanceLayout::Chords => 2,
    }
}

pub fn layout_from_index(idx: usize) -> PerformanceLayout {
    match idx {
        1 => PerformanceLayout::Scale,
        2 => PerformanceLayout::Chords,
        _ => PerformanceLayout::Piano,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_ui_roundtrip() {
        let settings = PerformanceSettings {
            root: 7,
            scale: Scale::Dorian,
            layout: PerformanceLayout::Chords,
            ..PerformanceSettings::default()
        };
        let ui = PerformanceUi::from_settings(&settings);
        let back = ui.to_settings();
        assert_eq!(back.root, 7);
        assert_eq!(back.scale, Scale::Dorian);
        assert_eq!(back.layout, PerformanceLayout::Chords);
    }
}
