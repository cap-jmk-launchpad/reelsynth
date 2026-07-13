//! Preset schema migration (v1 → v2).

use serde_json::Value;

use super::schema::{default_filter2, default_filter_envelope, default_macros, Lfo, Patch, SCHEMA_V1, SCHEMA_V2};

pub(crate) fn migrate_fx_bypass(patch: &mut Patch) {
    if patch.effects.is_empty() {
        patch.effects = crate::fx::effects_from_bypass(&patch.fx_bypass);
    }
}

pub(crate) fn migrate_v1_to_v2(v: &mut Value) {
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
        f.entry("drive").or_insert(Value::Number(0.into()));
    }

    if !obj.contains_key("filter2") {
        obj.insert(
            "filter2".into(),
            serde_json::to_value(default_filter2()).unwrap(),
        );
    }

    obj.entry("unison_stereo_spread")
        .or_insert(Value::Number(serde_json::Number::from_f64(0.7).unwrap()));

    if !obj.contains_key("lfo2") {
        obj.insert("lfo2".into(), serde_json::to_value(Lfo::default()).unwrap());
    }
    if !obj.contains_key("macros") {
        obj.insert(
            "macros".into(),
            serde_json::to_value(default_macros()).unwrap(),
        );
    }

    if let Some(lfo) = obj.get_mut("lfo").and_then(|l| l.as_object_mut()) {
        lfo.entry("shape")
            .or_insert(Value::String("sine".into()));
    }
    if let Some(lfo) = obj.get_mut("lfo2").and_then(|l| l.as_object_mut()) {
        lfo.entry("shape")
            .or_insert(Value::String("sine".into()));
    }

    if let Some(arr) = obj.get_mut("oscillators").and_then(|a| a.as_array_mut()) {
        for osc in arr {
            if let Some(o) = osc.as_object_mut() {
                o.entry("pulse_width")
                    .or_insert(Value::Number(serde_json::Number::from_f64(0.5).unwrap()));
                o.entry("morph_a").or_insert(Value::Number(0.into()));
                o.entry("morph_b")
                    .or_insert(Value::Number(serde_json::Number::from_f64(255.0).unwrap()));
                o.entry("morph_amount").or_insert(Value::Number(0.into()));
                o.entry("warp_mode")
                    .or_insert(Value::String("none".into()));
                o.entry("warp_amount").or_insert(Value::Number(0.into()));
                o.entry("fm_source")
                    .or_insert(Value::String("none".into()));
                o.entry("fm_ratio")
                    .or_insert(Value::Number(serde_json::Number::from_f64(1.0).unwrap()));
                o.entry("fm_index").or_insert(Value::Number(0.into()));
            }
        }
    }
}
