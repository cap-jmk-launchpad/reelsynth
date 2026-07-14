//! Note scheduler — emit note on/off for a beat window.

use super::clock::BeatRange;
use super::schema::{Clip, ClipRef, SequenceProject, Track};
use super::transport::TransportState;

/// Dedicated MIDI channel for scheduled sequence notes (live uses 0).
pub const SEQ_CHANNEL: u8 = 15;

#[derive(Clone, Debug, PartialEq)]
pub enum SchedEvent {
    NoteOn {
        sample_offset: usize,
        channel: u8,
        note: u8,
        velocity: f32,
    },
    NoteOff {
        sample_offset: usize,
        channel: u8,
        note: u8,
    },
}

#[derive(Clone, Debug, Default)]
pub struct NoteScheduler {
    /// Notes currently held by the scheduler (for note-off on stop).
    active_notes: Vec<u8>,
    /// Session scene slots — when set, arrangement clips are ignored.
    session_slots: Option<Vec<Option<ClipRef>>>,
}

impl NoteScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_session(&mut self, slots: Option<Vec<Option<ClipRef>>>) {
        self.session_slots = slots;
    }

    pub fn session_slots(&self) -> Option<&[Option<ClipRef>]> {
        self.session_slots.as_deref()
    }

    /// Collect note on/off events for `range` from `project` tracks.
    pub fn process(
        &mut self,
        project: &SequenceProject,
        transport: &TransportState,
        range: BeatRange,
        buffer_frames: usize,
    ) -> Vec<SchedEvent> {
        if !transport.playing {
            return Vec::new();
        }

        let mut events = Vec::new();

        if let Some(slots) = self.session_slots.as_ref() {
            let solo_any = project.tracks.iter().any(|t| t.solo);
            for slot in slots.iter().flatten() {
                let Some(track) = project.tracks.get(slot.track) else {
                    continue;
                };
                if track.mute || (solo_any && !track.solo) {
                    continue;
                }
                let Some(clip) = track.clips.get(slot.clip) else {
                    continue;
                };
                collect_session_clip_events(clip, range, buffer_frames, &mut events);
            }
        } else {
            let solo_any = project.tracks.iter().any(|t| t.solo);
            for (ti, track) in project.tracks.iter().enumerate() {
                if track.mute {
                    continue;
                }
                if solo_any && !track.solo {
                    continue;
                }
                for clip in &track.clips {
                    collect_arrangement_clip_events(clip, range, buffer_frames, ti, &mut events);
                }
            }
        }

        events.sort_by_key(|e| event_sample_offset(e));
        self.track_active(&events);
        events
    }

    /// Force note-off for all scheduler-held notes (transport stop).
    pub fn all_note_offs(&self) -> Vec<SchedEvent> {
        self.active_notes
            .iter()
            .map(|&note| SchedEvent::NoteOff {
                sample_offset: 0,
                channel: SEQ_CHANNEL,
                note,
            })
            .collect()
    }

    pub fn clear_active(&mut self) {
        self.active_notes.clear();
    }

    fn track_active(&mut self, events: &[SchedEvent]) {
        for ev in events {
            match ev {
                SchedEvent::NoteOn { note, .. } => {
                    if !self.active_notes.contains(note) {
                        self.active_notes.push(*note);
                    }
                }
                SchedEvent::NoteOff { note, .. } => {
                    self.active_notes.retain(|n| n != note);
                }
            }
        }
    }
}

fn event_sample_offset(ev: &SchedEvent) -> usize {
    match ev {
        SchedEvent::NoteOn { sample_offset, .. } => *sample_offset,
        SchedEvent::NoteOff { sample_offset, .. } => *sample_offset,
    }
}

fn collect_arrangement_clip_events(
    clip: &Clip,
    range: BeatRange,
    buffer_frames: usize,
    _track_idx: usize,
    out: &mut Vec<SchedEvent>,
) {
    let clip_end = clip.start_beats + clip.length_beats;
    if range.end_beats <= clip.start_beats || range.start_beats >= clip_end {
        return;
    }

    for note in &clip.notes {
        let note_start = clip.start_beats + note.start_beats;
        let note_end = note_start + note.duration_beats;
        emit_note_window(note_start, note_end, note, range, buffer_frames, out);
    }
}

/// Session clip: playhead is clip-local; optional loop within `length_beats`.
fn collect_session_clip_events(
    clip: &Clip,
    range: BeatRange,
    buffer_frames: usize,
    out: &mut Vec<SchedEvent>,
) {
    let len = clip.length_beats.max(0.001);
    let max_loops = if clip.r#loop {
        ((range.end_beats / len).ceil() as usize).saturating_add(1)
    } else {
        1
    };

    for loop_idx in 0..max_loops {
        let loop_offset = loop_idx as f32 * len;
        if loop_offset >= range.end_beats {
            break;
        }
        for note in &clip.notes {
            let note_start = loop_offset + note.start_beats;
            let note_end = note_start + note.duration_beats;
            if note_end <= range.start_beats {
                continue;
            }
            if note_start >= range.end_beats {
                if !clip.r#loop {
                    break;
                }
                continue;
            }
            emit_note_window(note_start, note_end, note, range, buffer_frames, out);
        }
        if !clip.r#loop {
            break;
        }
    }
}

fn emit_note_window(
    note_start: f32,
    note_end: f32,
    note: &super::schema::MidiNote,
    range: BeatRange,
    buffer_frames: usize,
    out: &mut Vec<SchedEvent>,
) {
    if note_start >= range.start_beats && note_start < range.end_beats {
        let offset = beat_to_sample_offset(note_start, range, buffer_frames);
        out.push(SchedEvent::NoteOn {
            sample_offset: offset,
            channel: SEQ_CHANNEL,
            note: note.pitch,
            velocity: note.velocity,
        });
    }

    if note_end >= range.start_beats && note_end < range.end_beats {
        let offset = beat_to_sample_offset(note_end, range, buffer_frames);
        out.push(SchedEvent::NoteOff {
            sample_offset: offset,
            channel: SEQ_CHANNEL,
            note: note.pitch,
        });
    }
}

fn beat_to_sample_offset(beat: f32, range: BeatRange, buffer_frames: usize) -> usize {
    if range.beats_per_sample <= 0.0 {
        return 0;
    }
    let offset_beats = beat - range.start_beats;
    let sample = (offset_beats / range.beats_per_sample).round() as usize;
    sample.min(buffer_frames.saturating_sub(1))
}

/// Clips active at the current playhead (for session launch / UI).
pub fn clips_at_playhead<'a>(
    tracks: &'a [Track],
    playhead: f32,
) -> impl Iterator<Item = (&'a Track, &'a Clip)> + 'a {
    tracks.iter().flat_map(move |track| {
        track
            .clips
            .iter()
            .filter(move |c| {
                playhead >= c.start_beats && playhead < c.start_beats + c.length_beats
            })
            .map(move |c| (track, c))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence::schema::{MidiNote, SequenceProject};

    fn test_project_with_note() -> SequenceProject {
        let mut p = SequenceProject::default_template();
        p.tracks[0].clips.push(Clip {
            start_beats: 0.0,
            length_beats: 4.0,
            notes: vec![MidiNote {
                pitch: 60,
                start_beats: 1.0,
                duration_beats: 0.5,
                velocity: 0.8,
            }],
            r#loop: false,
            automation: Vec::new(),
        });
        p
    }

    #[test]
    fn emits_note_on_at_beat_boundary() {
        let project = test_project_with_note();
        let transport = TransportState {
            playing: true,
            bpm: 120.0,
            playhead_beats: 0.0,
            ..Default::default()
        };
        let range = BeatRange {
            start_beats: 0.0,
            end_beats: 2.0,
            beats_per_sample: (120.0 / 60.0) / 44100.0,
        };
        let mut sched = NoteScheduler::new();
        let events = sched.process(&project, &transport, range, 256);
        let ons: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SchedEvent::NoteOn { note: 60, .. }))
            .collect();
        assert_eq!(ons.len(), 1);
        let off: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SchedEvent::NoteOff { note: 60, .. }))
            .collect();
        assert_eq!(off.len(), 1);
    }

    #[test]
    fn session_scene_plays_slot_clip() {
        let mut p = test_project_with_note();
        p.tracks[0].clips[0].start_beats = 99.0; // arrangement position — ignored in session
        let slots = vec![Some(crate::sequence::schema::ClipRef {
            track: 0,
            clip: 0,
        })];
        let transport = TransportState {
            playing: true,
            bpm: 120.0,
            playhead_beats: 0.0,
            ..Default::default()
        };
        let range = BeatRange {
            start_beats: 0.0,
            end_beats: 2.0,
            beats_per_sample: (120.0 / 60.0) / 44100.0,
        };
        let mut sched = NoteScheduler::new();
        sched.set_session(Some(slots));
        let events = sched.process(&p, &transport, range, 256);
        let ons: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SchedEvent::NoteOn { note: 60, .. }))
            .collect();
        assert_eq!(ons.len(), 1);
    }
}
