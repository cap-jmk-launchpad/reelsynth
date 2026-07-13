//! Live scope ring buffers written from the audio thread.

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
