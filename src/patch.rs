//! Patch schema parsing (reelsynth-preset-v2 with v1 migration).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCHEMA_V1: &str = "reelsynth-preset-v1";
pub const SCHEMA_V2: &str = "reelsynth-preset-v2";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Oscillator {
    #[serde(default = "default_wt_type", rename = "type")]
    pub osc_type: String,
    #[serde(default = "one")]
    pub level: f32,
    #[serde(default)]
    pub position: f32,
    #[serde(default)]
    pub detune: f32,
    #[serde(default = "default_unison")]
    pub unison: u32,
    #[serde(default)]
    pub pan: f32,
    #[serde(default)]
    pub wavetable_id: Option<String>,
}

fn default_wt_type() -> String {
    "wavetable".into()
}
fn one() -> f32 {
    1.0
}
fn default_unison() -> u32 {
    1
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Filter {
    #[serde(default = "default_lp", rename = "type")]
    pub filter_type: String,
    #[serde(default = "default_cutoff")]
    pub cutoff: f32,
    #[serde(default)]
    pub resonance: f32,
    /// 0 = no tracking, 1 = cutoff follows pitch 1:1 in semitones.
    #[serde(default = "default_key_tracking")]
    pub key_tracking: f32,
}

fn default_lp() -> String {
    "lowpass".into()
}
fn default_cutoff() -> f32 {
    1200.0
}
fn default_key_tracking() -> f32 {
    0.5
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Envelope {
    #[serde(default = "default_attack")]
    pub attack: f32,
    #[serde(default = "default_decay")]
    pub decay: f32,
    #[serde(default = "default_sustain")]
    pub sustain: f32,
    #[serde(default = "default_release")]
    pub release: f32,
}

fn default_attack() -> f32 {
    0.01
}
fn default_decay() -> f32 {
    0.2
}
fn default_sustain() -> f32 {
    0.6
}
fn default_release() -> f32 {
    0.4
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: default_attack(),
            decay: default_decay(),
            sustain: default_sustain(),
            release: default_release(),
        }
    }
}

fn default_filter_envelope() -> Envelope {
    Envelope {
        attack: 0.005,
        decay: 0.35,
        sustain: 0.2,
        release: 0.5,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Lfo {
    #[serde(default = "default_lfo_rate")]
    pub rate: f32,
    #[serde(default)]
    pub depth: f32,
    #[serde(default = "default_lfo_target")]
    pub target: String,
}

fn default_lfo_rate() -> f32 {
    0.5
}
fn default_lfo_target() -> String {
    "wt_position".into()
}

impl Default for Lfo {
    fn default() -> Self {
        Self {
            rate: default_lfo_rate(),
            depth: 0.0,
            target: default_lfo_target(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModSlot {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub amount: f32,
    /// When false the route is ignored by the engine (S6 UI On/Off).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Patch {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub wavetable_id: Option<String>,
    #[serde(default)]
    pub oscillators: Vec<Oscillator>,
    #[serde(default)]
    pub filter: Filter,
    #[serde(default)]
    pub envelope: Envelope,
    #[serde(default = "default_filter_envelope")]
    pub filter_envelope: Envelope,
    #[serde(default)]
    pub lfo: Lfo,
    #[serde(default)]
    pub mod_matrix: Vec<ModSlot>,
    #[serde(default)]
    pub fx_bypass: crate::fx::FxBypass,
    #[serde(default)]
    pub sub_level: f32,
    #[serde(default)]
    pub noise_level: f32,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            filter_type: default_lp(),
            cutoff: default_cutoff(),
            resonance: 0.3,
            key_tracking: default_key_tracking(),
        }
    }
}

impl Patch {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let mut v: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        migrate_v1_to_v2(&mut v);
        // Accept "type" alias for oscillator/filter
        if let Some(arr) = v.get_mut("oscillators").and_then(|a| a.as_array_mut()) {
            for osc in arr {
                let ty = osc.get("type").and_then(|t| t.as_str()).map(str::to_string);
                if let Some(t) = ty {
                    osc.as_object_mut()
                        .unwrap()
                        .entry("osc_type")
                        .or_insert(Value::String(t));
                }
            }
        }
        if let Some(f) = v.get_mut("filter") {
            let ty = f.get("type").and_then(|t| t.as_str()).map(str::to_string);
            if let Some(t) = ty {
                f.as_object_mut()
                    .unwrap()
                    .entry("filter_type")
                    .or_insert(Value::String(t));
            }
        }
        let mut patch: Patch = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if patch.schema.is_empty() || patch.schema == SCHEMA_V1 {
            patch.schema = SCHEMA_V2.into();
        }
        Ok(patch)
    }

    pub fn to_json(&self) -> Result<String, String> {
        let mut patch = self.clone();
        patch.schema = SCHEMA_V2.into();
        serde_json::to_string_pretty(&patch).map_err(|e| e.to_string())
    }

    pub fn default_mono() -> Self {
        Self {
            schema: SCHEMA_V2.into(),
            name: "default".into(),
            wavetable_id: Some("saw_morph".into()),
            oscillators: vec![Oscillator {
                osc_type: "wavetable".into(),
                level: 1.0,
                position: 0.0,
                detune: 0.0,
                unison: 1,
                pan: 0.0,
                wavetable_id: None,
            }],
            filter: Filter::default(),
            envelope: Envelope::default(),
            filter_envelope: default_filter_envelope(),
            lfo: Lfo::default(),
            mod_matrix: vec![],
            fx_bypass: crate::fx::FxBypass::default(),
            sub_level: 0.0,
            noise_level: 0.0,
        }
    }

    /// Ensure at least `count` wavetable oscillators (S3 tri-osc UI).
    pub fn ensure_oscillators(&mut self, count: usize) {
        while self.oscillators.len() < count {
            self.oscillators.push(Oscillator {
                osc_type: "wavetable".into(),
                level: 0.0,
                position: 0.0,
                detune: 0.0,
                unison: 1,
                pan: 0.0,
                wavetable_id: None,
            });
        }
        if let Some(first) = self.oscillators.first_mut() {
            if first.level <= 0.0 {
                first.level = 1.0;
            }
        }
    }

    /// Unique wavetable IDs referenced by oscillators (deduped, order preserved).
    pub fn wavetable_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for osc in &self.oscillators {
            if let Some(id) = osc.wavetable_id.as_deref() {
                if !ids.iter().any(|existing: &String| existing == id) {
                    ids.push(id.to_string());
                }
            }
        }
        if ids.is_empty() {
            if let Some(id) = &self.wavetable_id {
                ids.push(id.clone());
            }
        }
        ids
    }
}

fn migrate_v1_to_v2(v: &mut Value) {
    let obj = match v.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    let schema = obj
        .get("schema")
        .and_then(|s| s.as_str())
        .unwrap_or(SCHEMA_V1);
    let is_v1 = schema.is_empty() || schema == SCHEMA_V1;

    if is_v1 {
        obj.insert("schema".into(), Value::String(SCHEMA_V2.into()));
    }

    if !obj.contains_key("filter_envelope") {
        obj.insert(
            "filter_envelope".into(),
            serde_json::to_value(default_filter_envelope()).unwrap(),
        );
    }

    if let Some(arr) = obj.get_mut("oscillators").and_then(|a| a.as_array_mut()) {
        for osc in arr {
            if let Some(o) = osc.as_object_mut() {
                o.entry("pan").or_insert(Value::Number(0.into()));
            }
        }
    }

    if let Some(f) = obj.get_mut("filter").and_then(|f| f.as_object_mut()) {
        f.entry("key_tracking")
            .or_insert(Value::Number(serde_json::Number::from_f64(0.5).unwrap()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_patch() {
        let p = Patch::from_json(r#"{"filter":{"type":"lowpass","cutoff":800}}"#).unwrap();
        assert_eq!(p.filter.cutoff, 800.0);
        assert_eq!(p.schema, SCHEMA_V2);
    }

    #[test]
    fn v1_migration_adds_v2_fields() {
        let json = r#"{"schema":"reelsynth-preset-v1","oscillators":[{"type":"wavetable","level":1.0}]}"#;
        let p = Patch::from_json(json).unwrap();
        assert_eq!(p.schema, SCHEMA_V2);
        assert_eq!(p.filter_envelope.attack, 0.005);
        assert_eq!(p.oscillators[0].pan, 0.0);
        assert_eq!(p.filter.key_tracking, 0.5);
    }

    #[test]
    fn wavetable_ids_dedupes() {
        let mut p = Patch::default_mono();
        p.ensure_oscillators(3);
        p.oscillators[0].wavetable_id = Some("saw_morph".into());
        p.oscillators[1].wavetable_id = Some("sine".into());
        p.oscillators[2].wavetable_id = Some("saw_morph".into());
        let ids = p.wavetable_ids();
        assert_eq!(ids, vec!["saw_morph", "sine"]);
    }
}
