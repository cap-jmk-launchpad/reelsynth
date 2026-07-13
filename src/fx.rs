//! Post-voice FX chain — reorderable slots with parallel mix per effect.

use serde::{Deserialize, Serialize};

/// Effect type identifiers (persisted in presets).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    Chorus,
    Delay,
    Reverb,
    Distortion,
    Compressor,
}

impl EffectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chorus => "chorus",
            Self::Delay => "delay",
            Self::Reverb => "reverb",
            Self::Distortion => "distortion",
            Self::Compressor => "compressor",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Chorus => "Chorus",
            Self::Delay => "Delay",
            Self::Reverb => "Reverb",
            Self::Distortion => "Distortion",
            Self::Compressor => "Compressor",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "delay" => Self::Delay,
            "reverb" => Self::Reverb,
            "distortion" | "dist" => Self::Distortion,
            "compressor" | "comp" => Self::Compressor,
            _ => Self::Chorus,
        }
    }

    pub const ALL: [Self; 5] = [
        Self::Chorus,
        Self::Delay,
        Self::Reverb,
        Self::Distortion,
        Self::Compressor,
    ];
}

/// Serializable effect slot stored on `Patch`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EffectSlot {
    pub effect_type: EffectType,
    #[serde(default)]
    pub bypassed: bool,
    #[serde(default = "default_mix")]
    pub mix: f32,
    // Chorus
    #[serde(default = "default_chorus_rate")]
    pub rate: f32,
    #[serde(default = "default_chorus_depth")]
    pub depth: f32,
    // Delay
    #[serde(default = "default_delay_ms")]
    pub time_ms: f32,
    #[serde(default = "default_feedback")]
    pub feedback: f32,
    // Reverb
    #[serde(default = "default_reverb_size")]
    pub size: f32,
    #[serde(default = "default_reverb_damping")]
    pub damping: f32,
    // Distortion
    #[serde(default)]
    pub drive: f32,
    #[serde(default = "default_tone")]
    pub tone: f32,
    // Compressor
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    #[serde(default = "default_ratio")]
    pub ratio: f32,
    #[serde(default = "default_comp_attack")]
    pub attack: f32,
    #[serde(default = "default_comp_release")]
    pub release: f32,
}

fn default_mix() -> f32 {
    0.35
}
fn default_chorus_rate() -> f32 {
    0.8
}
fn default_chorus_depth() -> f32 {
    0.35
}
fn default_delay_ms() -> f32 {
    280.0
}
fn default_feedback() -> f32 {
    0.32
}
fn default_reverb_size() -> f32 {
    0.68
}
fn default_reverb_damping() -> f32 {
    0.42
}
fn default_tone() -> f32 {
    0.5
}
fn default_threshold() -> f32 {
    -18.0
}
fn default_ratio() -> f32 {
    4.0
}
fn default_comp_attack() -> f32 {
    0.01
}
fn default_comp_release() -> f32 {
    0.12
}

impl EffectSlot {
    pub fn chorus() -> Self {
        Self {
            effect_type: EffectType::Chorus,
            bypassed: false,
            mix: 0.24,
            rate: 0.8,
            depth: 0.35,
            ..Self::delay()
        }
    }

    pub fn delay() -> Self {
        Self {
            effect_type: EffectType::Delay,
            bypassed: false,
            mix: 0.28,
            time_ms: 280.0,
            feedback: 0.32,
            ..Self::reverb()
        }
    }

    pub fn reverb() -> Self {
        Self {
            effect_type: EffectType::Reverb,
            bypassed: true,
            mix: 0.35,
            size: 0.68,
            damping: 0.42,
            ..Self::distortion()
        }
    }

    pub fn distortion() -> Self {
        Self {
            effect_type: EffectType::Distortion,
            bypassed: true,
            mix: 0.4,
            drive: 0.35,
            tone: 0.5,
            ..Self::compressor()
        }
    }

    pub fn compressor() -> Self {
        Self {
            effect_type: EffectType::Compressor,
            bypassed: true,
            mix: 0.5,
            threshold: -18.0,
            ratio: 4.0,
            attack: 0.01,
            release: 0.12,
            rate: default_chorus_rate(),
            depth: default_chorus_depth(),
            time_ms: default_delay_ms(),
            feedback: default_feedback(),
            size: default_reverb_size(),
            damping: default_reverb_damping(),
            drive: 0.0,
            tone: default_tone(),
        }
    }

    pub fn for_type(effect_type: EffectType) -> Self {
        match effect_type {
            EffectType::Chorus => Self::chorus(),
            EffectType::Delay => Self::delay(),
            EffectType::Reverb => Self::reverb(),
            EffectType::Distortion => Self::distortion(),
            EffectType::Compressor => Self::compressor(),
        }
    }
}

pub fn default_effects() -> Vec<EffectSlot> {
    vec![
        EffectSlot::chorus(),
        EffectSlot::delay(),
        EffectSlot::reverb(),
    ]
}

/// Legacy bypass flags — migrated into `effects` on load.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FxBypass {
    #[serde(default)]
    pub chorus_bypassed: bool,
    #[serde(default)]
    pub delay_bypassed: bool,
    #[serde(default = "default_true")]
    pub reverb_bypassed: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FxBypass {
    fn default() -> Self {
        Self {
            chorus_bypassed: false,
            delay_bypassed: false,
            reverb_bypassed: true,
        }
    }
}

pub fn effects_from_bypass(bypass: &FxBypass) -> Vec<EffectSlot> {
    let mut effects = default_effects();
    if let Some(slot) = effects.iter_mut().find(|s| s.effect_type == EffectType::Chorus) {
        slot.bypassed = bypass.chorus_bypassed;
    }
    if let Some(slot) = effects.iter_mut().find(|s| s.effect_type == EffectType::Delay) {
        slot.bypassed = bypass.delay_bypassed;
    }
    if let Some(slot) = effects.iter_mut().find(|s| s.effect_type == EffectType::Reverb) {
        slot.bypassed = bypass.reverb_bypassed;
    }
    effects
}

/// Post-voice FX chain with reorderable slots.
#[derive(Clone, Debug)]
pub struct FxChain {
    slots: Vec<EffectSlot>,
    processors: Vec<EffectProcessor>,
    sample_rate: f32,
}

impl Default for FxChain {
    fn default() -> Self {
        Self::new(44100)
    }
}

impl FxChain {
    pub fn new(sample_rate: u32) -> Self {
        let slots = default_effects();
        let processors = slots
            .iter()
            .map(|s| EffectProcessor::new(s, sample_rate as f32))
            .collect();
        Self {
            slots,
            processors,
            sample_rate: sample_rate as f32,
        }
    }

    pub fn set_effects(&mut self, slots: Vec<EffectSlot>) {
        self.slots = slots;
        self.processors = self
            .slots
            .iter()
            .map(|s| EffectProcessor::new(s, self.sample_rate))
            .collect();
    }

    /// Legacy API — maps fixed chorus/delay/reverb bypass flags.
    pub fn set_bypass(&mut self, bypass: FxBypass) {
        self.set_effects(effects_from_bypass(&bypass));
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.process_stereo(input, input)[0]
    }

    pub fn process_stereo(&mut self, left: f32, right: f32) -> [f32; 2] {
        let mut l = left;
        let mut r = right;

        for (slot, proc) in self.slots.iter().zip(self.processors.iter_mut()) {
            if slot.bypassed {
                continue;
            }
            let mix = slot.mix.clamp(0.0, 1.0);
            if mix <= 0.0001 {
                continue;
            }
            let [wet_l, wet_r] = proc.process_stereo(l, r, slot);
            l = l * (1.0 - mix) + wet_l * mix;
            r = r * (1.0 - mix) + wet_r * mix;
        }

        [soft_clip(l), soft_clip(r)]
    }
}

fn soft_clip(sample: f32) -> f32 {
    // Gentle master limiter — replaces hard clamp.
    (sample * 1.15).tanh()
}

// ── Runtime processors ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum EffectProcessor {
    Chorus(ChorusProc),
    Delay(DelayProc),
    Reverb(ReverbProc),
    Distortion(DistortionProc),
    Compressor(CompressorProc),
}

impl EffectProcessor {
    fn new(slot: &EffectSlot, sr: f32) -> Self {
        match slot.effect_type {
            EffectType::Chorus => Self::Chorus(ChorusProc::new(sr)),
            EffectType::Delay => Self::Delay(DelayProc::new(sr)),
            EffectType::Reverb => Self::Reverb(ReverbProc::new(sr)),
            EffectType::Distortion => Self::Distortion(DistortionProc::new(sr)),
            EffectType::Compressor => Self::Compressor(CompressorProc::new(sr)),
        }
    }

    fn process_stereo(&mut self, l: f32, r: f32, slot: &EffectSlot) -> [f32; 2] {
        match self {
            Self::Chorus(p) => p.process(l, r, slot),
            Self::Delay(p) => p.process(l, r, slot),
            Self::Reverb(p) => p.process(l, r, slot),
            Self::Distortion(p) => p.process(l, r, slot),
            Self::Compressor(p) => p.process(l, r, slot),
        }
    }
}

// ── Chorus ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct ChorusProc {
    sr: f32,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    pos: usize,
    phase: f32,
}

impl ChorusProc {
    fn new(sr: f32) -> Self {
        let len = (sr * 0.05).max(64.0) as usize;
        Self {
            sr,
            buf_l: vec![0.0; len],
            buf_r: vec![0.0; len],
            pos: 0,
            phase: 0.0,
        }
    }

    fn process(&mut self, l: f32, r: f32, slot: &EffectSlot) -> [f32; 2] {
        let rate = slot.rate.clamp(0.05, 8.0);
        let depth = slot.depth.clamp(0.0, 1.0);
        let base_delay = 0.012 * self.sr;
        let mod_depth = depth * 0.004 * self.sr;

        self.phase += rate / self.sr;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let mod_l = (self.phase * std::f32::consts::TAU).sin();
        let mod_r = ((self.phase + 0.25) % 1.0 * std::f32::consts::TAU).sin();

        self.buf_l[self.pos] = l;
        self.buf_r[self.pos] = r;

        let dl = (base_delay + mod_l * mod_depth).clamp(1.0, self.buf_l.len() as f32 - 2.0);
        let dr = (base_delay + mod_r * mod_depth).clamp(1.0, self.buf_r.len() as f32 - 2.0);

        let out_l = read_interp(&self.buf_l, self.pos, dl);
        let out_r = read_interp(&self.buf_r, self.pos, dr);

        self.pos = (self.pos + 1) % self.buf_l.len();
        [out_l, out_r]
    }
}

// ── Delay (stereo ping-pong) ─────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct DelayProc {
    sr: f32,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    pos: usize,
}

impl DelayProc {
    fn new(sr: f32) -> Self {
        let len = (sr * 2.0).max(128.0) as usize;
        Self {
            sr,
            buf_l: vec![0.0; len],
            buf_r: vec![0.0; len],
            pos: 0,
        }
    }

    fn process(&mut self, l: f32, r: f32, slot: &EffectSlot) -> [f32; 2] {
        let time_ms = slot.time_ms.clamp(1.0, 2000.0);
        let fb = slot.feedback.clamp(0.0, 0.92);
        let delay_samp = (time_ms * 0.001 * self.sr)
            .clamp(1.0, self.buf_l.len() as f32 - 2.0);

        let delayed_l = read_interp(&self.buf_l, self.pos, delay_samp);
        let delayed_r = read_interp(&self.buf_r, self.pos, delay_samp * 1.07);

        // Ping-pong feedback cross-feed.
        self.buf_l[self.pos] = l + delayed_r * fb;
        self.buf_r[self.pos] = r + delayed_l * fb;

        self.pos = (self.pos + 1) % self.buf_l.len();
        [delayed_l, delayed_r]
    }
}

// ── Reverb (Schroeder-style) ─────────────────────────────────────────────────

const NUM_COMBS: usize = 4;
const NUM_ALLPASS: usize = 2;

#[derive(Clone, Debug)]
struct CombFilter {
    buf: Vec<f32>,
    pos: usize,
    delay: usize,
    feedback: f32,
    damp: f32,
    store: f32,
}

impl CombFilter {
    fn new(delay: usize) -> Self {
        Self {
            buf: vec![0.0; delay.max(1)],
            pos: 0,
            delay: delay.max(1),
            feedback: 0.84,
            damp: 0.2,
            store: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buf[self.pos];
        self.store = delayed * (1.0 - self.damp) + self.store * self.damp;
        self.buf[self.pos] = input + self.store * self.feedback;
        self.pos = (self.pos + 1) % self.delay;
        delayed
    }
}

#[derive(Clone, Debug)]
struct AllpassFilter {
    buf: Vec<f32>,
    pos: usize,
    delay: usize,
}

impl AllpassFilter {
    fn new(delay: usize) -> Self {
        Self {
            buf: vec![0.0; delay.max(1)],
            pos: 0,
            delay: delay.max(1),
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buf[self.pos];
        let out = -input + buf_out;
        self.buf[self.pos] = input + buf_out * 0.5;
        self.pos = (self.pos + 1) % self.delay;
        out
    }
}

#[derive(Clone, Debug)]
struct ReverbProc {
    combs_l: [CombFilter; NUM_COMBS],
    combs_r: [CombFilter; NUM_COMBS],
    allpass_l: [AllpassFilter; NUM_ALLPASS],
    allpass_r: [AllpassFilter; NUM_ALLPASS],
}

impl ReverbProc {
    fn new(sr: f32) -> Self {
        let scale = (sr / 44100.0).max(0.5);
        let comb_delays = [
            (1116.0 * scale) as usize,
            (1188.0 * scale) as usize,
            (1277.0 * scale) as usize,
            (1356.0 * scale) as usize,
        ];
        let ap_delays = [(556.0 * scale) as usize, (441.0 * scale) as usize];
        Self {
            combs_l: [
                CombFilter::new(comb_delays[0]),
                CombFilter::new(comb_delays[1]),
                CombFilter::new(comb_delays[2]),
                CombFilter::new(comb_delays[3]),
            ],
            combs_r: [
                CombFilter::new(comb_delays[0] + 23),
                CombFilter::new(comb_delays[1] + 23),
                CombFilter::new(comb_delays[2] + 23),
                CombFilter::new(comb_delays[3] + 23),
            ],
            allpass_l: [
                AllpassFilter::new(ap_delays[0]),
                AllpassFilter::new(ap_delays[1]),
            ],
            allpass_r: [
                AllpassFilter::new(ap_delays[0] + 13),
                AllpassFilter::new(ap_delays[1] + 13),
            ],
        }
    }

    fn process(&mut self, l: f32, r: f32, slot: &EffectSlot) -> [f32; 2] {
        let size = slot.size.clamp(0.0, 1.0);
        let damping = slot.damping.clamp(0.0, 1.0);
        let fb = 0.7 + size * 0.25;

        for comb in self.combs_l.iter_mut().chain(self.combs_r.iter_mut()) {
            comb.feedback = fb;
            comb.damp = damping * 0.9;
        }

        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;
        for c in &mut self.combs_l {
            out_l += c.process(l);
        }
        for c in &mut self.combs_r {
            out_r += c.process(r);
        }
        out_l *= 0.25;
        out_r *= 0.25;

        for ap in &mut self.allpass_l {
            out_l = ap.process(out_l);
        }
        for ap in &mut self.allpass_r {
            out_r = ap.process(out_r);
        }

        [out_l, out_r]
    }
}

// ── Distortion ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct DistortionProc {
    tone_lp: f32,
    tone_state_l: f32,
    tone_state_r: f32,
}

impl DistortionProc {
    fn new(_sr: f32) -> Self {
        Self {
            tone_lp: 0.3,
            tone_state_l: 0.0,
            tone_state_r: 0.0,
        }
    }

    fn process(&mut self, l: f32, r: f32, slot: &EffectSlot) -> [f32; 2] {
        let drive = 1.0 + slot.drive.clamp(0.0, 1.0) * 12.0;
        let tone = slot.tone.clamp(0.0, 1.0);
        let lp = 0.05 + tone * 0.85;
        self.tone_lp = lp;

        let shaped_l = (l * drive).tanh();
        let shaped_r = (r * drive).tanh();

        self.tone_state_l += lp * (shaped_l - self.tone_state_l);
        self.tone_state_r += lp * (shaped_r - self.tone_state_r);
        [self.tone_state_l, self.tone_state_r]
    }
}

// ── Compressor (feed-forward RMS) ────────────────────────────────────────────

#[derive(Clone, Debug)]
struct CompressorProc {
    sr: f32,
    env: f32,
}

impl CompressorProc {
    fn new(sr: f32) -> Self {
        Self { sr, env: 0.0 }
    }

    fn process(&mut self, l: f32, r: f32, slot: &EffectSlot) -> [f32; 2] {
        let threshold_db = slot.threshold.clamp(-60.0, 0.0);
        let ratio = slot.ratio.clamp(1.0, 20.0);
        let attack = slot.attack.clamp(0.001, 0.5);
        let release = slot.release.clamp(0.01, 2.0);

        let mono = (l.abs() + r.abs()) * 0.5;
        let target = mono;

        let coeff = if target > self.env {
            (-1.0 / (attack * self.sr)).exp()
        } else {
            (-1.0 / (release * self.sr)).exp()
        };
        self.env = target + coeff * (self.env - target);

        let level_db = 20.0 * (self.env.max(1e-8)).log10();
        let over_db = level_db - threshold_db;
        let gain_db = if over_db > 0.0 {
            -over_db * (1.0 - 1.0 / ratio)
        } else {
            0.0
        };
        let gain = 10.0_f32.powf(gain_db / 20.0);
        [l * gain, r * gain]
    }
}

fn read_interp(buf: &[f32], write_pos: usize, delay: f32) -> f32 {
    let len = buf.len();
    if len < 2 {
        return 0.0;
    }
    let read_pos = (write_pos as f32 - delay).rem_euclid(len as f32);
    let idx = read_pos.floor() as usize % len;
    let frac = read_pos - read_pos.floor();
    let next = (idx + 1) % len;
    buf[idx] * (1.0 - frac) + buf[next] * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_skips_fx() {
        let mut chain = FxChain::new(44100);
        let mut slots = default_effects();
        for slot in &mut slots {
            slot.bypassed = true;
        }
        chain.set_effects(slots);
        assert_eq!(chain.process_sample(0.5), soft_clip(0.5));
    }

    #[test]
    fn chorus_changes_sample_when_active() {
        let mut chain = FxChain::new(44100);
        let mut slots = vec![EffectSlot::chorus()];
        slots[0].mix = 1.0;
        chain.set_effects(slots);
        let mut max_delta = 0.0f32;
        for i in 0..4000 {
            let input = (i as f32 * 0.013).sin() * 0.6;
            let out = chain.process_sample(input);
            max_delta = max_delta.max((out - soft_clip(input)).abs());
        }
        assert!(max_delta > 0.001, "chorus delta {max_delta}");
    }

    #[test]
    fn delay_is_audible_with_mix() {
        let mut chain = FxChain::new(44100);
        let mut slot = EffectSlot::delay();
        slot.mix = 1.0;
        slot.time_ms = 50.0;
        slot.feedback = 0.0;
        chain.set_effects(vec![slot]);
        let mut impulse_response = 0.0f32;
        for i in 0..8000 {
            let input = if i == 0 { 1.0 } else { 0.0 };
            impulse_response += chain.process_sample(input).abs();
        }
        assert!(impulse_response > 0.5, "delay tail energy {impulse_response}");
    }

    #[test]
    fn reverb_sustains_tail() {
        let mut chain = FxChain::new(44100);
        let mut slot = EffectSlot::reverb();
        slot.bypassed = false;
        slot.mix = 1.0;
        chain.set_effects(vec![slot]);
        let mut tail = 0.0f32;
        for i in 0..12000 {
            let input = if i < 100 { 0.8 } else { 0.0 };
            tail = tail.max(chain.process_sample(input).abs());
        }
        assert!(tail > 0.01, "reverb tail peak {tail}");
    }

    #[test]
    fn soft_clip_limits_hot_signal() {
        assert!(soft_clip(2.0).abs() < 1.0);
        assert!(soft_clip(-3.0).abs() < 1.0);
    }

    #[test]
    fn migrate_bypass_to_effects() {
        let effects = effects_from_bypass(&FxBypass::default());
        assert_eq!(effects.len(), 3);
        assert!(!effects[0].bypassed);
        assert!(!effects[1].bypassed);
        assert!(effects[2].bypassed);
    }
}
