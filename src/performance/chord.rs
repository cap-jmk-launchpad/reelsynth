//! Chord spelling and diatonic degree resolution.

use serde::{Deserialize, Serialize};

use super::{scale::scale_degree_to_midi, Scale};

/// Triads vs seventh chords for the chord row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChordSet {
    #[default]
    Triads,
    Sevenths,
}

/// Simple voicing presets applied after chord spelling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChordVoicing {
    #[default]
    Close,
    Spread,
    Root,
}

/// Diatonic chord quality for a scale degree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Dominant7,
    Major7,
    Minor7,
    HalfDim7,
    Dim7,
}

impl ChordQuality {
    fn intervals(self) -> &'static [i8] {
        match self {
            ChordQuality::Major => &[0, 4, 7],
            ChordQuality::Minor => &[0, 3, 7],
            ChordQuality::Diminished => &[0, 3, 6],
            ChordQuality::Dominant7 => &[0, 4, 7, 10],
            ChordQuality::Major7 => &[0, 4, 7, 11],
            ChordQuality::Minor7 => &[0, 3, 7, 10],
            ChordQuality::HalfDim7 => &[0, 3, 6, 10],
            ChordQuality::Dim7 => &[0, 3, 6, 9],
        }
    }
}

/// Diatonic triad or seventh quality for `degree` (0 = I) in `scale`.
pub fn diatonic_quality(scale: Scale, degree: usize, chord_set: ChordSet) -> ChordQuality {
    let d = degree % 7;
    match chord_set {
        ChordSet::Triads => triad_quality(scale, d),
        ChordSet::Sevenths => seventh_quality(scale, d),
    }
}

fn triad_quality(scale: Scale, degree: usize) -> ChordQuality {
    use ChordQuality::*;
    match scale {
        Scale::Major | Scale::Lydian | Scale::Mixolydian => match degree {
            0 | 3 | 4 => Major,
            1 | 2 | 5 => Minor,
            _ => Diminished,
        },
        Scale::Minor
        | Scale::NaturalMinor
        | Scale::Dorian
        | Scale::Phrygian
        | Scale::MelodicMinor => match degree {
            0 | 3 => Minor,
            2 | 5 | 6 => Major,
            1 | 4 => Diminished,
            _ => Minor,
        },
        Scale::HarmonicMinor => match degree {
            0 | 3 => Minor,
            1 | 4 => Diminished,
            2 | 5 | 6 => Major,
            _ => Minor,
        },
        Scale::Locrian => match degree {
            0 => Diminished,
            1 | 4 => Major,
            2 | 5 => Minor,
            3 | 6 => Minor,
            _ => Diminished,
        },
        Scale::MajorPent | Scale::MinorPent | Scale::Blues | Scale::WholeTone | Scale::Chromatic => {
            if degree % 2 == 0 {
                Major
            } else {
                Minor
            }
        }
    }
}

fn seventh_quality(scale: Scale, degree: usize) -> ChordQuality {
    use ChordQuality::*;
    match scale {
        Scale::Major | Scale::Lydian => match degree {
            0 | 3 => Major7,
            1 | 2 | 5 => Minor7,
            4 => Dominant7,
            _ => HalfDim7,
        },
        Scale::Mixolydian => match degree {
            0 | 3 => Major7,
            1 | 2 | 5 => Minor7,
            4 => Dominant7,
            _ => HalfDim7,
        },
        Scale::NaturalMinor | Scale::Minor | Scale::Dorian | Scale::Phrygian => match degree {
            0 | 3 => Minor7,
            2 | 5 | 6 => Major7,
            1 | 4 => HalfDim7,
            _ => Minor7,
        },
        Scale::HarmonicMinor => match degree {
            0 | 3 => Minor7,
            1 => HalfDim7,
            2 | 5 | 6 => Major7,
            4 => Dominant7,
            _ => Minor7,
        },
        Scale::MelodicMinor => match degree {
            0 => Minor7,
            1 => Minor7,
            2 => Major7,
            3 => Major7,
            4 => Dominant7,
            5 => HalfDim7,
            _ => Dim7,
        },
        Scale::Locrian => match degree {
            0 => HalfDim7,
            1 | 4 => Major7,
            2 | 5 => Minor7,
            _ => Minor7,
        },
        _ => triad_quality(scale, degree),
    }
}

/// Spell a chord from root MIDI note, quality, and voicing preset.
pub fn resolve_chord(root: u8, quality: ChordQuality, voicing: ChordVoicing) -> Vec<u8> {
    let mut notes: Vec<u8> = quality
        .intervals()
        .iter()
        .map(|&st| (root as i16 + st as i16).clamp(0, 127) as u8)
        .collect();
    apply_voicing(&mut notes, voicing);
    notes.sort_unstable();
    notes.dedup();
    notes
}

/// Root + diatonic quality for a scale degree in the current key.
pub fn resolve_diatonic_chord(
    root_pc: u8,
    scale: Scale,
    degree: usize,
    base_octave: i8,
    chord_set: ChordSet,
    voicing: ChordVoicing,
) -> Vec<u8> {
    let root = scale_degree_to_midi(root_pc, scale, degree, base_octave);
    let quality = diatonic_quality(scale, degree, chord_set);
    resolve_chord(root, quality, voicing)
}

fn apply_voicing(notes: &mut Vec<u8>, voicing: ChordVoicing) {
    if notes.len() < 2 {
        return;
    }
    match voicing {
        ChordVoicing::Close => {}
        ChordVoicing::Spread => {
            for i in 1..notes.len() {
                if i % 2 == 1 {
                    notes[i] = (notes[i] as i16 + 12).min(127) as u8;
                }
            }
        }
        ChordVoicing::Root => {
            let root = notes[0];
            notes.truncate(1);
            notes.push((root as i16 + 12).min(127) as u8);
            if notes.len() == 1 {
                notes.push((root as i16 + 7).min(127) as u8);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_triad() {
        let notes = resolve_chord(60, ChordQuality::Major, ChordVoicing::Close);
        assert_eq!(notes, vec![60, 64, 67]);
    }

    #[test]
    fn g7_spelling() {
        let notes = resolve_chord(67, ChordQuality::Dominant7, ChordVoicing::Close);
        assert_eq!(notes, vec![67, 71, 74, 77]);
    }

    #[test]
    fn diatonic_i_in_c_major() {
        let notes = resolve_diatonic_chord(
            0,
            Scale::Major,
            0,
            4,
            ChordSet::Triads,
            ChordVoicing::Close,
        );
        assert_eq!(notes, vec![60, 64, 67]);
    }

    #[test]
    fn diatonic_v7_in_c_major() {
        let notes = resolve_diatonic_chord(
            0,
            Scale::Major,
            4,
            4,
            ChordSet::Sevenths,
            ChordVoicing::Close,
        );
        assert!(notes.contains(&71));
        assert!(notes.len() >= 4);
    }
}
