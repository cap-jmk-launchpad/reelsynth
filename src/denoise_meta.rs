//! Meta-learning + literature-informed hyperparameter search (1500 trials).
//!
//! Outer loop: random / PBT-style search over (λ, fade, polish, pin, θ noise)
//! informed by AutoML / Bayesian HPO / population-based training practice.
//! Inner scoring: denoise+shape loss on held-out procedural cycles.
//! Emits top-4 champions vs naive baseline matrix for the paper.

use crate::artifact_reduce::{periodize_with_algo, PeriodizeAlgo};
use crate::denoise_opt::{apply_denoise_opt, apply_denoise_theta, FROZEN_THETA, N_THETA};
use crate::seam::SeamStyle;
use crate::sound_bench::{crackle_fast, generate_sound, BenchFamily, BENCH_N};
use serde_json::json;

const N_TRIALS: usize = 1500;
const VAL_FAST: usize = 400;
const VAL_FINAL: usize = 2000;

#[derive(Debug, Clone)]
struct TrialHp {
    name: String,
    lambda_shape: f32,
    fade_scale_bias: f32,
    polish_bias: f32,
    pin_bias: f32,
    detrend_bias: f32,
    ease_bias: f32,
    algo_seed: u64,
    prior: &'static str,
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

fn eval_theta_fast(
    theta: &[f32; N_THETA],
    start_seed: u64,
    count: usize,
    n: usize,
    lambda: f32,
) -> (f32, f32, f32, f32) {
    let mut sum_l = 0.0f32;
    let mut sum_d = 0.0f32;
    let mut sum_s = 0.0f32;
    for k in 0..count {
        let seed = start_seed + k as u64;
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        apply_denoise_theta(&mut out, 0.0, theta);
        let (l, d, s, _) = score_with_lambda(&raw, &out, lambda);
        sum_l += l;
        sum_d += d;
        sum_s += s;
    }
    let c = count.max(1) as f32;
    let d = sum_d / c;
    let s = sum_s / c;
    (sum_l / c, d, s, 0.5 * (d + s))
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }
    fn f01(&mut self) -> f32 {
        (self.next() >> 33) as f32 / (u32::MAX as f32)
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f01()
    }
}

/// Literature-informed prior families (HPO / PBT / multi-objective cues).
fn sample_trial(rng: &mut Rng, idx: usize) -> (TrialHp, [f32; N_THETA]) {
    let bucket = idx % 6;
    let (prior, lam_lo, lam_hi, fade_lo, fade_hi, pol_lo, pol_hi) = match bucket {
        // Bayesian-HPO style: denser around previously good λ≈0.85
        0 => ("bayes_local", 0.55, 1.25, 0.9, 1.35, 0.7, 1.05),
        // PBT exploit: mutate near champion overlay specialist
        1 => ("pbt_exploit", 0.7, 1.0, 1.05, 1.35, 0.85, 1.05),
        // Multi-objective / shape-first (higher λ)
        2 => ("mo_shape", 1.2, 2.2, 0.65, 1.0, 0.4, 0.75),
        // Aggressive denoise (low λ, long fade)
        3 => ("aggressive", 0.35, 0.75, 1.15, 1.5, 0.9, 1.1),
        // Evolutionary wide explore
        4 => ("evo_explore", 0.3, 2.5, 0.5, 1.6, 0.3, 1.1),
        // Algorithm-config racing: mid band
        _ => ("racing_mid", 0.8, 1.4, 0.85, 1.2, 0.6, 0.95),
    };
    let hp = TrialHp {
        name: format!("{prior}_{idx}"),
        lambda_shape: rng.range(lam_lo, lam_hi),
        fade_scale_bias: rng.range(fade_lo, fade_hi),
        polish_bias: rng.range(pol_lo, pol_hi),
        pin_bias: rng.range(0.8, 1.05),
        detrend_bias: rng.range(0.85, 1.05),
        ease_bias: rng.range(0.0, 1.0),
        algo_seed: 10_000 + idx as u64,
        prior,
    };
    // Base θ: frozen + small Gaussian-ish noise (PBT mutation)
    let mut theta = FROZEN_THETA;
    for t in theta.iter_mut() {
        let noise = (rng.f01() - 0.5) * 0.22;
        *t = (*t + noise).clamp(0.0, 1.0);
    }
    // Apply hyper biases
    theta[0] = (theta[0] * hp.detrend_bias).clamp(0.0, 1.0);
    theta[1] = (theta[1] * hp.fade_scale_bias).clamp(0.0, 1.0);
    theta[3] = hp.ease_bias.clamp(0.0, 1.0);
    theta[6] = (theta[6] * hp.polish_bias).clamp(0.0, 1.0);
    theta[9] = (theta[9] * hp.pin_bias).clamp(0.0, 1.0);
    theta[11] = (theta[11] * hp.polish_bias).clamp(0.0, 1.0);
    theta[7] = 0.0;
    (hp, theta)
}

fn family_stress(theta: &[f32; N_THETA], lambda: f32, n: usize) -> Vec<serde_json::Value> {
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
        while cnt < 120 {
            if BenchFamily::from_seed(seed) == fam {
                let (_, raw) = generate_sound(seed, n);
                let mut out = raw.clone();
                apply_denoise_theta(&mut out, 0.0, theta);
                let (_, d, s, _) = score_with_lambda(&raw, &out, lambda);
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
    fam_q
}

fn eval_algo_matrix(
    algos: &[(String, [f32; N_THETA], f32)],
    naive_label: &str,
    n: usize,
    val_count: usize,
) -> serde_json::Value {
    let start = 60_000u64;
    let mut rows = Vec::new();

    // Naive = DualCosine (strong hand baseline) + Classic listed separately in paper
    for (label, use_denoise_opt, algo) in [
        ("naive_classic", false, PeriodizeAlgo::Classic),
        ("naive_dual_cosine", false, PeriodizeAlgo::DualCosine),
    ] {
        let mut sum_d = 0.0f32;
        let mut sum_s = 0.0f32;
        for k in 0..val_count {
            let (_, raw) = generate_sound(start + k as u64, n);
            let mut out = raw.clone();
            periodize_with_algo(&mut out, 0.0, SeamStyle::Adaptive, algo);
            let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
            sum_d += d;
            sum_s += s;
        }
        let c = val_count as f32;
        if label == naive_label || label == "naive_classic" {
            rows.push(json!({
                "algo": label,
                "kind": "naive",
                "denoise": sum_d / c,
                "shape": sum_s / c,
                "quality": 0.5 * (sum_d + sum_s) / c,
            }));
        }
        let _ = use_denoise_opt;
    }

    // Primary naive for matrix is DualCosine (best classical)
    // Replace first naive entry focus: keep both classic + dual, then top4 meta
    for (name, theta, lam) in algos {
        let (_, d, s, q) = eval_theta_fast(theta, start, val_count, n, *lam);
        rows.push(json!({
            "algo": name,
            "kind": "meta",
            "lambda": lam,
            "denoise": d,
            "shape": s,
            "quality": q,
            "theta": theta.as_slice(),
        }));
    }
    json!(rows)
}

/// 1500 literature-informed meta trials + top-4 vs naive matrix.
pub fn run_meta_learning_search_1500() -> serde_json::Value {
    let t0 = std::time::Instant::now();
    let n = BENCH_N;
    let mut rng = Rng(0x15A0_1500);
    let val_start = 55_000u64;

    let mut scored: Vec<(f32, TrialHp, [f32; N_THETA], f32, f32, f32, f32)> = Vec::with_capacity(N_TRIALS);

    for i in 0..N_TRIALS {
        let (hp, theta) = sample_trial(&mut rng, i);
        let (loss, d, s, q) = eval_theta_fast(&theta, val_start, VAL_FAST, n, hp.lambda_shape);
        let meta = if s >= 0.97 { q } else { q * 0.45 };
        scored.push((meta, hp, theta, loss, d, s, q));
        if i % 250 == 0 {
            eprintln!(
                "meta 1500 progress {i}/{N_TRIALS} best_so_far={:.4}",
                scored
                    .iter()
                    .map(|t| t.0)
                    .fold(0.0f32, f32::max)
            );
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Refine top 12 with larger validation
    let mut refined = Vec::new();
    for (meta, hp, theta, _, _, _, _) in scored.iter().take(12) {
        let (loss, d, s, q) = eval_theta_fast(theta, 70_000, VAL_FINAL, n, hp.lambda_shape);
        let meta2 = if s >= 0.97 { q } else { q * 0.45 };
        let fam = family_stress(theta, hp.lambda_shape, n);
        refined.push(json!({
            "name": hp.name,
            "prior": hp.prior,
            "meta_score_fast": meta,
            "meta_score": meta2,
            "hyper": {
                "lambda_shape": hp.lambda_shape,
                "fade_scale_bias": hp.fade_scale_bias,
                "polish_bias": hp.polish_bias,
                "pin_bias": hp.pin_bias,
                "detrend_bias": hp.detrend_bias,
                "ease_bias": hp.ease_bias,
                "algo_seed": hp.algo_seed,
            },
            "theta": theta.as_slice(),
            "val": { "loss": loss, "denoise": d, "shape": s, "quality": q },
            "family_stress": fam,
        }));
    }
    refined.sort_by(|a, b| {
        b["meta_score"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["meta_score"].as_f64().unwrap_or(0.0))
            .unwrap()
    });

    let top4: Vec<_> = refined.iter().take(4).cloned().collect();
    let mut top4_algos = Vec::new();
    for (i, t) in top4.iter().enumerate() {
        let theta_arr: Vec<f32> = t["theta"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        let mut th = [0.0f32; N_THETA];
        for (j, v) in theta_arr.iter().enumerate().take(N_THETA) {
            th[j] = *v;
        }
        let lam = t["hyper"]["lambda_shape"].as_f64().unwrap_or(1.0) as f32;
        top4_algos.push((format!("meta_top{}", i + 1), th, lam));
    }

    // Matrix: naive classic + naive dual + top4 → paper uses 5: naive_dual + top4
    let mut matrix_algos = top4_algos.clone();
    // Build 5-way: DualCosine as naive + top4
    let mut five = Vec::new();
    // score naive dual
    let mut nd = 0.0f32;
    let mut ns = 0.0f32;
    for k in 0..VAL_FINAL {
        let (_, raw) = generate_sound(60_000 + k as u64, n);
        let mut out = raw.clone();
        periodize_with_algo(&mut out, 0.0, SeamStyle::Adaptive, PeriodizeAlgo::DualCosine);
        let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
        nd += d;
        ns += s;
    }
    let c = VAL_FINAL as f32;
    five.push(json!({
        "algo": "naive_dual_cosine",
        "kind": "naive",
        "denoise": nd / c,
        "shape": ns / c,
        "quality": 0.5 * (nd + ns) / c,
        "rank": 0,
    }));
    let mut nc = 0.0f32;
    let mut ncs = 0.0f32;
    for k in 0..VAL_FINAL {
        let (_, raw) = generate_sound(60_000 + k as u64, n);
        let mut out = raw.clone();
        periodize_with_algo(&mut out, 0.0, SeamStyle::Adaptive, PeriodizeAlgo::Classic);
        let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
        nc += d;
        ncs += s;
    }
    let classic_row = json!({
        "algo": "naive_classic",
        "kind": "naive_ref",
        "denoise": nc / c,
        "shape": ncs / c,
        "quality": 0.5 * (nc + ncs) / c,
    });

    for (i, (name, th, lam)) in matrix_algos.iter().enumerate() {
        let (_, d, s, q) = eval_theta_fast(th, 60_000, VAL_FINAL, n, *lam);
        five.push(json!({
            "algo": name,
            "kind": "meta",
            "lambda": lam,
            "denoise": d,
            "shape": s,
            "quality": q,
            "rank": i + 1,
            "theta": th.as_slice(),
            "prior": top4[i]["prior"],
            "trial_name": top4[i]["name"],
        }));
    }

    // Also frozen current production
    let mut fd = 0.0f32;
    let mut fs = 0.0f32;
    for k in 0..VAL_FINAL {
        let (_, raw) = generate_sound(60_000 + k as u64, n);
        let mut out = raw.clone();
        apply_denoise_opt(&mut out, 0.0);
        let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
        fd += d;
        fs += s;
    }

    let report = json!({
        "title": "DenoiseOpt 1500-trial literature-informed meta-learning",
        "n_trials": N_TRIALS,
        "val_fast": VAL_FAST,
        "val_final": VAL_FINAL,
        "cycle_n": n,
        "seconds": t0.elapsed().as_secs_f64(),
        "literature_priors": [
            "bayes_local — densify around good λ (Bayesian HPO)",
            "pbt_exploit — mutate near champion (population-based training)",
            "mo_shape — higher λ multi-objective shape preference",
            "aggressive — low λ long fade",
            "evo_explore — wide evolutionary explore",
            "racing_mid — mid-band racing / algorithm configuration",
        ],
        "meta_objective": "maximize Q=0.5*(D+S) s.t. S>=0.97",
        "champion": top4.first().cloned().unwrap_or(json!({})),
        "top4": top4,
        "benchmark_matrix_5": five,
        "naive_classic_ref": classic_row,
        "production_frozen": {
            "denoise": fd / c,
            "shape": fs / c,
            "quality": 0.5 * (fd + fs) / c,
        },
        "pareto_top20_fast": scored.iter().take(20).map(|(meta, hp, theta, loss, d, s, q)| json!({
            "meta_score": meta,
            "name": hp.name,
            "prior": hp.prior,
            "lambda": hp.lambda_shape,
            "val_fast": { "loss": loss, "denoise": d, "shape": s, "quality": q },
            "theta": theta.as_slice(),
        })).collect::<Vec<_>>(),
        "sessionId": "0ab8f9",
        "runId": "meta-1500",
    });

    let _ = std::fs::create_dir_all("brand/artifacts");
    if let Ok(s) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write("brand/artifacts/denoise_opt_meta_1500.json", &s);
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
                "runId": "meta-1500",
                "message": "1500 meta trials complete",
                "data": {
                    "n_trials": N_TRIALS,
                    "champion": report["champion"]["name"],
                    "champion_q": report["champion"]["val"]["quality"],
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
    let _ = matrix_algos;
    report
}

/// Back-compat thin wrapper used by older bin.
pub fn run_meta_learning_search() -> serde_json::Value {
    run_meta_learning_search_1500()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_trial_shapes_ok() {
        let mut rng = Rng(1);
        let (hp, theta) = sample_trial(&mut rng, 0);
        assert!(hp.lambda_shape > 0.0);
        assert_eq!(theta[7], 0.0);
        let (_, _, s, q) = eval_theta_fast(&theta, 0, 50, 128, hp.lambda_shape);
        assert!(s > 0.8, "shape={s}");
        assert!(q > 0.5);
    }
}
