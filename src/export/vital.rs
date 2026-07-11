//! `.reelwt` → Vital `.vitaltable` JSON.

use crate::export::ExportReport;
use crate::wavetable::WavetableBank;
use serde_json::json;
use std::path::Path;

pub fn export_vital(bank: &WavetableBank, out_path: &Path, name: &str) -> ExportReport {
    let mut samples = Vec::with_capacity(bank.num_frames);
    for fi in 0..bank.num_frames {
        samples.push(bank.frame(fi).to_vec());
    }
    let doc = json!({
        "name": name,
        "samples": samples,
    });
    if let Some(parent) = out_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ExportReport::fail("vital", e.to_string());
        }
    }
    match serde_json::to_string_pretty(&doc) {
        Ok(text) => {
            if let Err(e) = std::fs::write(out_path, text) {
                ExportReport::fail("vital", e.to_string())
            } else {
                ExportReport::ok("vital", out_path.display().to_string())
            }
        }
        Err(e) => ExportReport::fail("vital", e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{export_wavetable, ExportOptions, ExportTarget};
    use crate::import::import_vital;

    #[test]
    fn roundtrip_vital() {
        let bank = WavetableBank::factory_saw_morph();
        let dir = std::env::temp_dir().join("reelsynth_rt_vital");
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join("table.vitaltable");
        let report = export_wavetable(
            &bank,
            ExportTarget::Vital,
            &out,
            &ExportOptions {
                table_name: "saw".into(),
                ..Default::default()
            },
        );
        assert!(report.success);
        let reimport = import_vital(out.to_str().unwrap()).unwrap();
        assert_eq!(reimport.num_frames, bank.num_frames);
        assert_eq!(reimport.frame_size, bank.frame_size);
        let rmse = frame_rmse(&bank, &reimport);
        assert!(rmse < 1e-5, "RMSE {rmse}");
    }

    fn frame_rmse(a: &WavetableBank, b: &WavetableBank) -> f32 {
        let n = a.num_frames.min(b.num_frames);
        let mut sum = 0.0f64;
        let mut count = 0usize;
        for fi in 0..n {
            let fa = a.frame(fi);
            let fb = b.frame(fi);
            let len = fa.len().min(fb.len());
            for i in 0..len {
                let d = (fa[i] - fb[i]) as f64;
                sum += d * d;
                count += 1;
            }
        }
        (sum / count.max(1) as f64).sqrt() as f32
    }
}
