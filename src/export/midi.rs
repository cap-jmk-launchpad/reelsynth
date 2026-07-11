//! Simple SMF type-0 MIDI writer (single track, note events).

use crate::export::ExportReport;
use crate::patch::Patch;
use std::path::Path;

pub fn export_midi(preset: &Patch, out_path: &Path, opts: &crate::export::ExportOptions) -> ExportReport {
    let _ = preset;
    let ticks_per_quarter = 480u16;
    let duration_ticks = (opts.duration * ticks_per_quarter as f32) as u32;
    let track = build_track(opts.midi_note, duration_ticks, ticks_per_quarter);
    let file = build_smf(&track);
    if let Some(parent) = out_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ExportReport::fail("midi", e.to_string());
        }
    }
    match std::fs::write(out_path, file) {
        Ok(()) => ExportReport::ok("midi", out_path.display().to_string()),
        Err(e) => ExportReport::fail("midi", e.to_string()),
    }
}

fn build_track(note: u8, duration_ticks: u32, _ticks_per_quarter: u16) -> Vec<u8> {
    let mut events = Vec::new();
    events.extend(vlq(0));
    events.extend([0x90, note, 0x64]); // note on
    events.extend(vlq(duration_ticks));
    events.extend([0x80, note, 0x00]); // note off
    events.extend(vlq(0));
    events.extend([0xFF, 0x2F, 0x00]); // end of track

    let mut track = Vec::new();
    track.extend(b"MTrk");
    track.extend(&(events.len() as u32).to_be_bytes());
    track.extend(events);
    track
}

fn build_smf(track: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(b"MThd");
    out.extend(&6u32.to_be_bytes());
    out.extend(&0u16.to_be_bytes()); // type 0
    out.extend(&1u16.to_be_bytes()); // one track
    out.extend(&480u16.to_be_bytes());
    out.extend(track);
    out
}

fn vlq(value: u32) -> Vec<u8> {
    let mut buffer = value;
    let mut bytes = Vec::new();
    bytes.push((buffer & 0x7F) as u8);
    buffer >>= 7;
    while buffer > 0 {
        bytes.insert(0, ((buffer & 0x7F) as u8) | 0x80);
        buffer >>= 7;
    }
    bytes
}
