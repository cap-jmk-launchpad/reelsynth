//! Pitch coverage grids for the QA matrix (see `docs/qa/MATRIX.md`).

use reelsynth::engine::note_to_freq;

/// First key on an 88-key piano (A0).
pub const PIANO_FIRST: u8 = 21;
/// Last key on an 88-key piano (C8).
pub const PIANO_LAST: u8 = 108;

/// Full 88-key piano range: MIDI 21..=108.
pub const PIANO_FULL: [u8; 88] = {
    let mut notes = [0u8; 88];
    let mut i = 0usize;
    while i < 88 {
        notes[i] = (PIANO_FIRST as usize + i) as u8;
        i += 1;
    }
    notes
};

/// Every MIDI note number 0..=127.
pub const MIDI_ALL: [u8; 128] = {
    let mut notes = [0u8; 128];
    let mut i = 0usize;
    while i < 128 {
        notes[i] = i as u8;
        i += 1;
    }
    notes
};

/// PR smoke trio: low, middle, high (MIDI 33, 69, 105).
pub const SMOKE_PITCHES: [u8; 3] = [33, 69, 105];

/// Direct-frequency inputs (Hz) — includes Nyquist-edge stability probe.
pub const FREQ_CUSTOM: [f32; 9] = [
    27.5, 55.0, 110.0, 220.0, 440.0, 880.0, 1760.0, 8000.0, 18000.0,
];

/// Equal-temperament frequency for a MIDI note.
pub fn pitch_freq(note: u8) -> f32 {
    note_to_freq(note)
}

/// Run `f` for each smoke-tier MIDI note (33, 69, 105).
pub fn for_each_smoke_pitch<F: FnMut(f32)>(mut f: F) {
    for &note in &SMOKE_PITCHES {
        f(pitch_freq(note));
    }
}

#[test]
fn piano_full_covers_88_keys() {
    assert_eq!(PIANO_FULL.len(), 88);
    assert_eq!(PIANO_FULL[0], PIANO_FIRST);
    assert_eq!(PIANO_FULL[87], PIANO_LAST);
    for (i, &note) in PIANO_FULL.iter().enumerate() {
        assert_eq!(note, PIANO_FIRST + i as u8);
    }
}

#[test]
fn midi_all_covers_full_range() {
    assert_eq!(MIDI_ALL.len(), 128);
    assert_eq!(MIDI_ALL[0], 0);
    assert_eq!(MIDI_ALL[127], 127);
}

#[test]
fn smoke_pitches_are_spread() {
    assert_eq!(SMOKE_PITCHES, [33, 69, 105]);
    assert!(SMOKE_PITCHES[0] < SMOKE_PITCHES[1]);
    assert!(SMOKE_PITCHES[1] < SMOKE_PITCHES[2]);
}

#[test]
fn freq_custom_includes_reference_and_edge() {
    assert!((FREQ_CUSTOM[4] - 440.0).abs() < 1e-3);
    assert!(FREQ_CUSTOM[8] >= 18000.0);
}
