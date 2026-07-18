//! Meta-learning + hyperparameter search over DenoiseOpt variants.
//!
//! Outer loop searches algorithm family + hyperparameters; inner loop fits θ
//! on a stratified bench slice. Selects the champion by validation quality.

use crate::artifact_reduce::{periodize_with_algo, PeriodizeAlgo};
use crate::denoise_opt::{
    apply_denoise_theta, apply_denoise_opt, FROZEN_THETA, N_THETA,
};
use crate::seam::SeamStyle;
use crate::sound_bench::{
    crackle_fast, eval_theta_bench, fit_denoise_on_bench, generate_sound, BenchFamily, BENCH_N,
};
use serde_json::json;

#[derive(Debug, Clone, Copy)]
pub struct HyperParams {
    pub name: &'static str,
    pub lambda_shape: f32,
    pub fade_scale_bias: f32,
    pub polish_bias: f32,
    pub pin_bias: f32,
    pub stride: usize,
    pub restarts: usize,
    pub sweeps: usize,
    pub train_count: usize,
    pub algo_seed: u64,
}

impl HyperParams {
    pub fn grid() -> Vec<HyperParams> {
        let mut out = Vec::new();
        let lambdas = [0.5f32, 1.0, 1.5, 2.0];
        let fades = [0.75f32, 1.0, 1.25];
        let polishes = [0.5f32, 0.85, 1.0];
        let pins = [0.85f32, 1.0];
        let mut i = 0usize;
        for &lam in &lambdas {
            for &fade in &fades {
                for &pol in &polishes {
                    for &pin in &pins {
                        // subsample grid for tractable meta search
                        if (i % 3) != 0 {
                            i += 1;
                            continue;
                        }
                        out.push(HyperParams {
                            name: "grid",
                            lambda_shape: lam,
                            fade_scale_bias: fade,
                            polish_bias: pol,
                            pin_bias: pin,
                            stride: 7,
                            restarts: 2,
                            sweeps: 1,
                            train_count: 4_000,
                            algo_seed: 1000 + i as u64,
                        });
                        i += 1;
                    }
                }
            }
        }
        // Named recipes (meta priors)
        out.push(HyperParams {
            name: "aggressive_denoise",
            lambda_shape: 0.6,
            fade_scale_bias: 1.35,
            polish_bias: 1.0,
            pin_bias: 1.0,
            stride: 5,
            restarts: 3,
            sweeps: 1,
            train_count: 6_000,
            algo_seed: 42,
        });
        out.push(HyperParams {
            name: "shape_first",
            lambda_shape: 2.0,
            fade_scale_bias: 0.8,
            polish_bias: 0.55,
            pin_bias: 0.9,
            stride: 5,
            restarts: 3,
            sweeps: 1,
            train_count: 6_000,
            algo_seed: 7,
        });
        out.push(HyperParams {
            name: "balanced_meta",
            lambda_shape: 1.0,
            fade_scale_bias: 1.05,
            polish_bias: 0.85,
            pin_bias: 1.0,
            stride: 5,
            restarts: 3,
            sweeps: 2,
            train_count: 8_000,
            algo_seed: 99,
        });
        out.push(HyperParams {
            name: "fft_overlay_specialist",
            lambda_shape: 0.85,
            fade_scale_bias: 1.2,
            polish_bias: 1.0,
            pin_bias: 1.0,
            stride: 5,
            restarts: 3,
            sweeps: 1,
            train_count: 6_000,
            algo_seed: 314,
        });
        out
    }
}

fn apply_hyper_biases(theta: &mut [f32; N_THETA], hp: &HyperParams) {
    theta[1] = (theta[1] * hp.fade_scale_bias).clamp(0.0, 1.0);
    theta[6] = (theta[6] * hp.polish_bias).clamp(0.0, 1.0);
    theta[11] = (theta[11] * hp.polish_bias).clamp(0.0, 1.0);
    theta[9] = (theta[9] * hp.pin_bias).clamp(0.0, 1.0);
    let _ = hp.lambda_shape; // used in scoring below
}

fn score_with_lambda(raw: &[f32], out: &[f32], lambda: f32) -> (f32, f32, f32, f32) {
    let c_raw = crackle_fast(raw);
    let c_out = crackle_fast(out);
    let denoise = if c_raw < 1e-6 {
        1.0
    } else {
        ((c_raw - c_out) / c_raw).clamp(0.0, 1.0)
    };
    let n = raw.len();
    let guard = (n / 8).max(4).min(n / 3);
    let mut mae = 0.0f32;
    let mut cnt = 0u32;
    for i in guard..n.saturating_sub(guard) {
        mae += (out[i] - raw[i]).abs();
        cnt += 1;
    }
    mae /= cnt.max(1) as f32;
    let rms = (raw.iter().map(|x| x * x).sum::<f32>() / n as f32).sqrt();
    let shape = 1.0 - (mae / (rms + 1e-6)).clamp(0.0, 1.0);
    let loss = (1.0 - denoise) + lambda * (1.0 - shape);
    (loss, denoise, shape, 0.5 * (denoise + shape))
}

fn eval_theta_lambda(
    theta: &[f32; N_THETA],
    count: usize,
    stride: usize,
    n: usize,
    lambda: f32,
) -> (f32, f32, f32, f32) {
    let stride = stride.max(1);
    let mut sum_l = 0.0f32;
    let mut sum_d = 0.0f32;
    let mut sum_s = 0.0f32;
    let mut m = 0u32;
    let mut seed = 0u64;
    while seed < count as u64 {
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        apply_denoise_theta(&mut out, 0.0, theta);
        let (l, d, s, _) = score_with_lambda(&raw, &out, lambda);
        sum_l += l;
        sum_d += d;
        sum_s += s;
        m += 1;
        seed += stride as u64;
    }
    let c = m.max(1) as f32;
    let d = sum_d / c;
    let s = sum_s / c;
    (sum_l / c, d, s, 0.5 * (d + s))
}

/// One meta trial: fit θ then bias by hyperparameters; score on held-out slice.
fn run_trial(hp: &HyperParams, val_count: usize, n: usize) -> serde_json::Value {
    let t0 = std::time::Instant::now();
    let (mut theta, fit) = fit_denoise_on_bench(
        hp.train_count,
        hp.stride,
        n,
        hp.restarts,
        hp.sweeps,
    );
    apply_hyper_biases(&mut theta, hp);
    let (tr_l, tr_d, tr_s, tr_q) =
        eval_theta_lambda(&theta, hp.train_count, hp.stride.max(3), n, hp.lambda_shape);
    // Validation: offset seeds so not identical to train stream
    let mut sum_l = 0.0f32;
    let mut sum_d = 0.0f32;
    let mut sum_s = 0.0f32;
    let mut m = 0u32;
    let start = 50_000u64 + hp.algo_seed * 97;
    for k in 0..val_count {
        let seed = start + k as u64;
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        apply_denoise_theta(&mut out, 0.0, &theta);
        let (l, d, s, _) = score_with_lambda(&raw, &out, hp.lambda_shape);
        sum_l += l;
        sum_d += d;
        sum_s += s;
        m += 1;
    }
    let c = m.max(1) as f32;
    let val_d = sum_d / c;
    let val_s = sum_s / c;
    let val_q = 0.5 * (val_d + val_s);
    let val_l = sum_l / c;

    // Family stress: extreme_overlay + open_wrap + combo
    let mut fam_q = Vec::new();
    for fam in [
        BenchFamily::ExtremeOverlay,
        BenchFamily::OpenWrapBias,
        BenchFamily::Combo,
        BenchFamily::HarmonicFft,
        BenchFamily::Nonlinear,
    ] {
        let mut qd = 0.0f32;
        let mut qs = 0.0f32;
        let mut cnt = 0u32;
        let mut seed = fam.index() as u64 + 80_000;
        while cnt < 200 {
            if BenchFamily::from_seed(seed) == fam {
                let (_, raw) = generate_sound(seed, n);
                let mut out = raw.clone();
                apply_denoise_theta(&mut out, 0.0, &theta);
                let (_, d, s, _) = score_with_lambda(&raw, &out, hp.lambda_shape);
                qd += d;
                qs += s;
                cnt += 1;
            }
            seed += BenchFamily::ALL.len() as u64;
        }
        let cc = cnt.max(1) as f32;
        fam_q.push(json!({
            "family": fam.label(),
            "denoise": qd / cc,
            "shape": qs / cc,
            "quality": 0.5 * (qd + qs) / cc,
        }));
    }

    json!({
        "name": hp.name,
        "hyper": {
            "lambda_shape": hp.lambda_shape,
            "fade_scale_bias": hp.fade_scale_bias,
            "polish_bias": hp.polish_bias,
            "pin_bias": hp.pin_bias,
            "train_count": hp.train_count,
            "stride": hp.stride,
            "restarts": hp.restarts,
            "sweeps": hp.sweeps,
            "algo_seed": hp.algo_seed,
        },
        "theta": theta.as_slice(),
        "train": { "loss": tr_l, "denoise": tr_d, "shape": tr_s, "quality": tr_q },
        "val": { "loss": val_l, "denoise": val_d, "shape": val_s, "quality": val_q },
        "family_stress": fam_q,
        "fit_seconds": fit["fit_seconds"].clone(),
        "trial_seconds": t0.elapsed().as_secs_f64(),
    })
}

fn baseline_row(n: usize, val_count: usize) -> serde_json::Value {
    let mut rows = Vec::new();
    for algo in [
        PeriodizeAlgo::Classic,
        PeriodizeAlgo::DualCosine,
        PeriodizeAlgo::DenoiseOpt,
        PeriodizeAlgo::EnsembleV3,
    ] {
        let mut sum_d = 0.0f32;
        let mut sum_s = 0.0f32;
        let mut m = 0u32;
        for k in 0..val_count {
            let seed = 50_000u64 + k as u64;
            let (_, raw) = generate_sound(seed, n);
            let mut out = raw.clone();
            match algo {
                PeriodizeAlgo::DenoiseOpt => apply_denoise_opt(&mut out, 0.0),
                _ => periodize_with_algo(&mut out, 0.0, SeamStyle::Adaptive, algo),
            }
            let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
            sum_d += d;
            sum_s += s;
            m += 1;
        }
        let c = m.max(1) as f32;
        rows.push(json!({
            "algo": algo.label(),
            "denoise": sum_d / c,
            "shape": sum_s / c,
            "quality": 0.5 * (sum_d + sum_s) / c,
        }));
    }
    // Frozen θ explicit
    let (l, d, s, q) = eval_theta_bench(&FROZEN_THETA, val_count, 1, n);
    rows.push(json!({
        "algo": "frozen_theta",
        "denoise": d,
        "shape": s,
        "quality": q,
        "loss": l,
    }));
    json!(rows)
}

/// Full meta-learning + hyperparameter sweep.
pub fn run_meta_learning_search() -> serde_json::Value {
    let t0 = std::time::Instant::now();
    let n = BENCH_N;
    let val_count = 1_500usize;
    let grid = HyperParams::grid();
    let mut trials = Vec::new();
    let mut best_q = -1.0f32;
    let mut best: Option<serde_json::Value> = None;

    for hp in &grid {
        let trial = run_trial(hp, val_count, n);
        let q = trial["val"]["quality"].as_f64().unwrap_or(0.0) as f32;
        let s = trial["val"]["shape"].as_f64().unwrap_or(0.0) as f32;
        // Meta objective: quality with hard shape floor
        let meta_score = if s >= 0.97 { q } else { q * 0.5 };
        let mut trial = trial;
        trial["meta_score"] = json!(meta_score);
        if meta_score > best_q {
            best_q = meta_score;
            best = Some(trial.clone());
        }
        trials.push(trial);
    }

    let baselines = baseline_row(n, val_count);
    let champion = best.clone().unwrap_or(json!({}));

    // Pareto front (denoise vs shape)
    let mut pareto = trials.clone();
    pareto.sort_by(|a, b| {
        let qa = a["val"]["quality"].as_f64().unwrap_or(0.0);
        let qb = b["val"]["quality"].as_f64().unwrap_or(0.0);
        qb.partial_cmp(&qa).unwrap()
    });

    let report = json!({
        "title": "DenoiseOpt meta-learning + hyperparameter search",
        "n_trials": trials.len(),
        "val_count": val_count,
        "cycle_n": n,
        "seconds": t0.elapsed().as_secs_f64(),
        "meta_objective": "maximize 0.5*(denoise+shape) subject to shape>=0.97",
        "champion": champion,
        "baselines": baselines,
        "trials": trials,
        "pareto_top10": pareto.into_iter().take(10).collect::<Vec<_>>(),
        "sessionId": "0ab8f9",
        "runId": "meta-hparam",
    });

    let _ = std::fs::create_dir_all("brand/artifacts");
    if let Ok(s) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write("brand/artifacts/denoise_opt_meta_search.json", s);
    }
    // #region agent log
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug-0ab8f9.log")
    {
        use std::io::Write;
        let _ = writeln!(
            f,
            "{}",
            json!({
                "sessionId": "0ab8f9",
                "runId": "meta-hparam",
                "message": "meta search complete",
                "data": {
                    "n_trials": report["n_trials"],
                    "champion_name": report["champion"]["name"],
                    "champion_quality": report["champion"]["val"]["quality"],
                    "seconds": report["seconds"],
                },
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0),
            })
        );
    }
    // #endregion
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_search_smoke() {
        // Tiny smoke: one recipe only via direct trial
        let hp = HyperParams {
            name: "smoke",
            lambda_shape: 1.0,
            fade_scale_bias: 1.0,
            polish_bias: 1.0,
            pin_bias: 1.0,
            stride: 11,
            restarts: 1,
            sweeps: 1,
            train_count: 300,
            algo_seed: 1,
        };
        let trial = run_trial(&hp, 200, 128);
        assert!(trial["val"]["shape"].as_f64().unwrap() > 0.9);
        eprintln!("{}", serde_json::to_string_pretty(&trial).unwrap());
    }
}
