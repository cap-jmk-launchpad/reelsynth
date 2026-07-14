//! Factory Lead × pitch QA matrix (see `docs/qa/MATRIX.md`).

use reelsynth::patch::Patch;

use super::helpers::{render_patch, QA_DURATION};
use super::invariants::assert_smoke_render;
use super::pitch_grid::{pitch_freq, PIANO_FULL, SMOKE_PITCHES};

#[test]
fn factory_lead_smoke_low_mid_high() {
    let patch = Patch::factory_lead();
    for &note in &SMOKE_PITCHES {
        let buf = render_patch(&patch, pitch_freq(note), QA_DURATION);
        assert_smoke_render(&buf);
    }
}

#[test]
#[ignore = "nightly: full 88-key piano sweep for Factory Lead"]
fn factory_lead_piano_full_sweep() {
    let patch = Patch::factory_lead();
    for &note in &PIANO_FULL {
        let buf = render_patch(&patch, pitch_freq(note), QA_DURATION);
        assert_smoke_render(&buf);
    }
}
