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

/// Default three-slot FX rack (chorus, delay, reverb bypassed).
pub fn default_effects() -> Vec<EffectSlot> {
    vec![
        EffectSlot::chorus(),
        EffectSlot::delay(),
        EffectSlot::reverb(),
    ]
}
