//! Live scope ring buffers written from the audio thread.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Samples retained per scope tap (recent history for UI waveform / FFT).
pub const SCOPE_RING_LEN: usize = 512;

/// UI-facing waveform / spectrum point count.
pub const SCOPE_DISPLAY_LEN: usize = 64;

/// Fixed-capacity ring buffer of recent audio samples.
#[derive(Clone, Debug)]
pub struct ScopeRingBuffer {
    data: Vec<f32>,
    write_pos: usize,
    len: usize,
}

impl Default for ScopeRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeRingBuffer {
    pub fn new() -> Self {
        Self {
            data: vec![0.0; SCOPE_RING_LEN],
            write_pos: 0,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0.0);
        self.write_pos = 0;
        self.len = 0;
    }

    pub fn push(&mut self, sample: f32) {
        self.data[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % SCOPE_RING_LEN;
        self.len = self.len.saturating_add(1).min(SCOPE_RING_LEN);
    }

    /// Recent samples in chronological order, resampled to `out_len` points.
    pub fn snapshot(&self, out_len: usize) -> Vec<f32> {
        let out_len = out_len.clamp(2, SCOPE_DISPLAY_LEN);
        if self.len == 0 {
            return vec![0.0; out_len];
        }

        let available = self.len.min(SCOPE_RING_LEN);
        let mut recent = Vec::with_capacity(available);
        let start = if self.len >= SCOPE_RING_LEN {
            self.write_pos
        } else {
            0
        };
        for i in 0..available {
            let idx = (start + i) % SCOPE_RING_LEN;
            recent.push(self.data[idx]);
        }

        if recent.len() == out_len {
            return recent;
        }

        let step = (recent.len() as f32 / out_len as f32).max(1.0);
        (0..out_len)
            .map(|i| {
                let idx = ((i as f32 * step).floor() as usize).min(recent.len().saturating_sub(1));
                recent[idx]
            })
            .collect()
    }
}

/// Four-tap live scope capture (Osc → Filter → FX → Out).
#[derive(Clone, Debug, Default)]
pub struct ScopeLiveTaps {
    pub osc: ScopeRingBuffer,
    pub filter: ScopeRingBuffer,
    pub fx: ScopeRingBuffer,
    pub out: ScopeRingBuffer,
    pub playing: bool,
}

impl ScopeLiveTaps {
    pub fn clear(&mut self) {
        self.osc.clear();
        self.filter.clear();
        self.fx.clear();
        self.out.clear();
        self.playing = false;
    }

    pub fn push_frame(&mut self, osc: f32, filter: f32, fx: f32, out: f32) {
        self.osc.push(osc);
        self.filter.push(filter);
        self.fx.push(fx);
        self.out.push(out);
        self.playing = true;
    }
}

/// Thread-safe scope monitor shared between audio and UI.
#[derive(Clone, Debug, Default)]
pub struct ScopeMonitor {
    taps: Arc<Mutex<ScopeLiveTaps>>,
    active: Arc<AtomicBool>,
}

impl ScopeMonitor {
    pub fn new() -> Self {
        Self {
            taps: Arc::new(Mutex::new(ScopeLiveTaps::default())),
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn write_frame(&self, osc: f32, filter: f32, fx: f32, out: f32, voices_active: bool) {
        if voices_active {
            if let Ok(mut taps) = self.taps.lock() {
                taps.push_frame(osc, filter, fx, out);
            }
            self.active.store(true, Ordering::Relaxed);
        } else {
            self.active.store(false, Ordering::Relaxed);
            if let Ok(mut taps) = self.taps.lock() {
                taps.playing = false;
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> ScopeLiveTaps {
        self.taps
            .lock()
            .map(|t| t.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_wraps_and_snapshots() {
        let mut ring = ScopeRingBuffer::new();
        for i in 0..SCOPE_RING_LEN + 8 {
            ring.push(i as f32 * 0.01);
        }
        let snap = ring.snapshot(16);
        assert_eq!(snap.len(), 16);
        assert!(snap.last().copied().unwrap_or(0.0) > snap.first().copied().unwrap_or(0.0));
    }
}
