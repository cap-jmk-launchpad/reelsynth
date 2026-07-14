//! Scale intervals, snapping, and degree → MIDI mapping.

use serde::{Deserialize, Serialize};

/// Supported scales and modes for performance input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scale {
    #[default]
    Major,
    /// Alias for natural minor in UI labels ("Minor").
    Minor,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    MajorPent,
    MinorPent,
    Blues,
    WholeTone,
    /// All chromatic notes — snapping disabled.
    Chromatic,
}

/// How incoming notes relate to the active scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleBehavior {
    /// Snap out-of-scale notes to the nearest scale tone.
    #[default]
    Snap,
    /// Only accept in-scale notes (drop others).
    Filter,
}

/// Performance keyboard layout mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceLayout {
    #[default]
    Piano,
    Scale,
    Chords,
}

impl Scale {
    /// Semitone intervals from the root (ascending within one octave).
    pub fn intervals(self) -> &'static [u8] {
        match self {
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor | Scale::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            Scale::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Scale::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            Scale::MajorPent => &[0, 2, 4, 7, 9],
            Scale::MinorPent => &[0, 3, 5, 7, 10],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
            Scale::WholeTone => &[0, 2, 4, 6, 8, 10],
            Scale::Chromatic => &[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
            ],
        }
    }

    pub fn is_chromatic(self) -> bool {
        matches!(self, Scale::Chromatic)
    }

    pub fn degree_count(self) -> usize {
        self.intervals().len()
    }
}

/// Whether a MIDI note belongs to the scale rooted at `root` (pitch class 0–11).
pub fn note_in_scale(note: u8, root: u8, scale: Scale) -> bool {
    if scale.is_chromatic() {
        return true;
    }
    let pc = note % 12;
    let rel = (pc + 12 - (root % 12)) % 12;
    scale.intervals().contains(&rel)
}

/// Snap `note` to the nearest scale tone; chromatic passes through unchanged.
pub fn snap_note(note: u8, root: u8, scale: Scale) -> u8 {
    if scale.is_chromatic() || note_in_scale(note, root, scale) {
        return note;
    }
    let root_pc = root % 12;
    let rel = (note % 12 + 12 - root_pc) % 12;
    let intervals = scale.intervals();
    let octave_base = (note as i32 / 12) * 12;

    let (below_idx, above_idx) = nearest_degree_indices(rel, intervals);
    let below = octave_base + root_pc as i32 + intervals[below_idx] as i32;
    let above = if intervals[above_idx] < intervals[below_idx] {
        octave_base + 12 + root_pc as i32 + intervals[above_idx] as i32
    } else {
        octave_base + root_pc as i32 + intervals[above_idx] as i32
    };

    let dist_below = (note as i32 - below).unsigned_abs();
    let dist_above = (above - note as i32).unsigned_abs();
    if dist_below <= dist_above {
        below.clamp(0, 127) as u8
    } else {
        above.clamp(0, 127) as u8
    }
}

/// Map a scale degree (0-based) and octave offset to a MIDI note.
pub fn scale_degree_to_midi(root: u8, scale: Scale, degree: usize, base_octave: i8) -> u8 {
    let intervals = scale.intervals();
    if intervals.is_empty() {
        return 60;
    }
    let deg = degree % intervals.len();
    let octave_shift = (degree / intervals.len()) as i32;
    let root_pc = root % 12;
    let base = (base_octave as i32 + 1) * 12 + root_pc as i32;
    let semitone = intervals[deg] as i32 + octave_shift * 12;
    (base + semitone).clamp(0, 127) as u8
}

fn nearest_degree_indices(rel: u8, intervals: &[u8]) -> (usize, usize) {
    let mut below = 0usize;
    let mut above = 0usize;
    for (i, &iv) in intervals.iter().enumerate() {
        if iv <= rel {
            below = i;
        }
        if iv >= rel {
            above = i;
            break;
        }
    }
    if intervals[above] < rel && above + 1 < intervals.len() {
        above += 1;
    }
    (below, above)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_includes_c_e_g() {
        let root = 0;
        assert!(note_in_scale(60, root, Scale::Major));
        assert!(note_in_scale(64, root, Scale::Major));
        assert!(note_in_scale(67, root, Scale::Major));
        assert!(!note_in_scale(61, root, Scale::Major));
    }

    #[test]
    fn snap_to_nearest_scale_tone() {
        assert_eq!(snap_note(61, 0, Scale::Major), 60);
        assert_eq!(snap_note(62, 0, Scale::Major), 62);
        // Eb is equidistant from D and E — tie prefers the lower scale tone.
        assert_eq!(snap_note(63, 0, Scale::Major), 62);
    }

    #[test]
    fn chromatic_snap_is_identity() {
        assert_eq!(snap_note(61, 0, Scale::Chromatic), 61);
    }

    #[test]
    fn scale_degree_c_major() {
        assert_eq!(scale_degree_to_midi(0, Scale::Major, 0, 4), 60);
        assert_eq!(scale_degree_to_midi(0, Scale::Major, 2, 4), 64);
        assert_eq!(scale_degree_to_midi(0, Scale::Major, 4, 4), 67);
    }

    #[test]
    fn pentatonic_has_five_degrees() {
        assert_eq!(Scale::MajorPent.degree_count(), 5);
    }
}
