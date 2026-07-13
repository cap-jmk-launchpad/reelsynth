use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::ring_buffer::ScopeRingBuffer;

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

