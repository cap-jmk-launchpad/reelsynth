//! Meta-learning + literature-informed hyperparameter search (1500 trials).
//!
//! Outer loop: random / PBT-style search over (λ, fade, polish, pin, θ noise)
//! informed by AutoML / Bayesian HPO / population-based training practice.
//!
//! **Primary meta objective:** prolonged residual score ∈ [0, 1] (1 = best):
//! ideal multi-period reference (same seed, no open-wrap) vs tiled engine cycle
//! after DenoiseOpt. D/S wrap-energy proxies remain as report auxiliaries.
//!
//! Emits top-4 champions vs naive baseline matrix for the paper.

use crate::artifact_reduce::{periodize_with_algo, PeriodizeAlgo};
use crate::denoise_opt::{
    apply_denoise_opt, apply_denoise_theta, residual_score_prolonged, FROZEN_THETA, N_THETA,
    RESIDUAL_PROLONG_PERIODS,
};
use crate::seam::SeamStyle;
use crate::sound_bench::{
    crackle_fast, generate_sound, generate_sound_ideal, BenchFamily, BENCH_N,
};
use serde_json::json;

const N_TRIALS_DEFAULT: usize = 1500;
const VAL_FAST_DEFAULT: usize = 400;
const VAL_FINAL_DEFAULT: usize = 2000;
const PROLONG: usize = RESIDUAL_PROLONG_PERIODS;

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

/// Auxiliary D/S wrap-energy proxy (kept for reports; not meta ranking).
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

fn residual_for_cycle(ideal: &[f32], out: &[f32]) -> f32 {
    residual_score_prolonged(ideal, out, PROLONG)
}

/// Soft shape gate: keep mid-cycle preservation as a secondary constraint.
fn meta_rank(residual: f32, shape: f32) -> f32 {
    if shape >= 0.97 {
        residual
    } else {
        residual * 0.45
    }
}

fn eval_theta_fast(
    theta: &[f32; N_THETA],
    start_seed: u64,
    count: usize,
    n: usize,
    lambda: f32,
) -> (f32, f32, f32, f32, f32) {
    let mut sum_l = 0.0f32;
    let mut sum_d = 0.0f32;
    let mut sum_s = 0.0f32;
    let mut sum_r = 0.0f32;
    for k in 0..count {
        let seed = start_seed + k as u64;
        let (_, ideal) = generate_sound_ideal(seed, n);
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        apply_denoise_theta(&mut out, 0.0, theta);
        let (l, d, s, _) = score_with_lambda(&raw, &out, lambda);
        sum_l += l;
        sum_d += d;
        sum_s += s;
        sum_r += residual_for_cycle(&ideal, &out);
    }
    let c = count.max(1) as f32;
    let d = sum_d / c;
    let s = sum_s / c;
    let r = sum_r / c;
    (sum_l / c, d, s, 0.5 * (d + s), r)
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
        let mut qr = 0.0f32;
        let mut cnt = 0u32;
        let mut seed = fam.index() as u64 + 80_000;
        while cnt < 120 {
            if BenchFamily::from_seed(seed) == fam {
                let (_, ideal) = generate_sound_ideal(seed, n);
                let (_, raw) = generate_sound(seed, n);
                let mut out = raw.clone();
                apply_denoise_theta(&mut out, 0.0, theta);
                let (_, d, s, _) = score_with_lambda(&raw, &out, lambda);
                qd += d;
                qs += s;
                qr += residual_for_cycle(&ideal, &out);
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
            "residual": qr / cc,
        }));
    }
    fam_q
}

/// Configurable meta search (use `n_trials=40` for a fast sanity check).
pub fn run_meta_learning_search_n(
    n_trials: usize,
    val_fast: usize,
    val_final: usize,
) -> serde_json::Value {
    let t0 = std::time::Instant::now();
    let n = BENCH_N;
    let mut rng = Rng(0x15A0_1500);
    let val_start = 55_000u64;
    let n_trials = n_trials.max(1);
    let val_fast = val_fast.max(1);
    let val_final = val_final.max(1);

    // (meta_rank, hp, theta, loss, d, s, q, residual)
    let mut scored: Vec<(f32, TrialHp, [f32; N_THETA], f32, f32, f32, f32, f32)> =
        Vec::with_capacity(n_trials);

    for i in 0..n_trials {
        let (hp, theta) = sample_trial(&mut rng, i);
        let (loss, d, s, q, residual) =
            eval_theta_fast(&theta, val_start, val_fast, n, hp.lambda_shape);
        let meta = meta_rank(residual, s);
        scored.push((meta, hp, theta, loss, d, s, q, residual));
        if i % 250 == 0 || (n_trials <= 100 && i % 10 == 0) {
            eprintln!(
                "meta progress {i}/{n_trials} best_residual_rank={:.4}",
                scored
                    .iter()
                    .map(|t| t.0)
                    .fold(0.0f32, f32::max)
            );
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Refine top 12 with larger validation
    let top_refine = scored.len().min(12);
    let mut refined = Vec::new();
    for (meta, hp, theta, _, _, _, _, _) in scored.iter().take(top_refine) {
        let (loss, d, s, q, residual) =
            eval_theta_fast(theta, 70_000, val_final, n, hp.lambda_shape);
        let meta2 = meta_rank(residual, s);
        let fam = family_stress(theta, hp.lambda_shape, n);
        refined.push(json!({
            "name": hp.name,
            "prior": hp.prior,
            "meta_score_fast": meta,
            "meta_score": meta2,
            "residual": residual,
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
            "val": {
                "loss": loss,
                "denoise": d,
                "shape": s,
                "quality": q,
                "residual": residual,
            },
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

    let mut five = Vec::new();
    let mut nd = 0.0f32;
    let mut ns = 0.0f32;
    let mut nr = 0.0f32;
    for k in 0..val_final {
        let seed = 60_000 + k as u64;
        let (_, ideal) = generate_sound_ideal(seed, n);
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        periodize_with_algo(&mut out, 0.0, SeamStyle::Adaptive, PeriodizeAlgo::DualCosine);
        let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
        nd += d;
        ns += s;
        nr += residual_for_cycle(&ideal, &out);
    }
    let c = val_final as f32;
    five.push(json!({
        "algo": "naive_dual_cosine",
        "kind": "naive",
        "denoise": nd / c,
        "shape": ns / c,
        "quality": 0.5 * (nd + ns) / c,
        "residual": nr / c,
        "rank": 0,
    }));
    let mut nc = 0.0f32;
    let mut ncs = 0.0f32;
    let mut ncr = 0.0f32;
    for k in 0..val_final {
        let seed = 60_000 + k as u64;
        let (_, ideal) = generate_sound_ideal(seed, n);
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        periodize_with_algo(&mut out, 0.0, SeamStyle::Adaptive, PeriodizeAlgo::Classic);
        let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
        nc += d;
        ncs += s;
        ncr += residual_for_cycle(&ideal, &out);
    }
    let classic_row = json!({
        "algo": "naive_classic",
        "kind": "naive_ref",
        "denoise": nc / c,
        "shape": ncs / c,
        "quality": 0.5 * (nc + ncs) / c,
        "residual": ncr / c,
    });

    for (i, (name, th, lam)) in top4_algos.iter().enumerate() {
        let (_, d, s, q, residual) = eval_theta_fast(th, 60_000, val_final, n, *lam);
        five.push(json!({
            "algo": name,
            "kind": "meta",
            "lambda": lam,
            "denoise": d,
            "shape": s,
            "quality": q,
            "residual": residual,
            "rank": i + 1,
            "theta": th.as_slice(),
            "prior": top4[i]["prior"],
            "trial_name": top4[i]["name"],
        }));
    }

    let mut fd = 0.0f32;
    let mut fs = 0.0f32;
    let mut fr = 0.0f32;
    for k in 0..val_final {
        let seed = 60_000 + k as u64;
        let (_, ideal) = generate_sound_ideal(seed, n);
        let (_, raw) = generate_sound(seed, n);
        let mut out = raw.clone();
        apply_denoise_opt(&mut out, 0.0);
        let (_, d, s, _) = score_with_lambda(&raw, &out, 1.0);
        fd += d;
        fs += s;
        fr += residual_for_cycle(&ideal, &out);
    }

    let artifact = if n_trials >= N_TRIALS_DEFAULT {
        "brand/artifacts/denoise_opt_meta_1500.json"
    } else {
        "brand/artifacts/denoise_opt_meta_sanity.json"
    };

    let report = json!({
        "title": "DenoiseOpt literature-informed meta-learning (residual objective)",
        "n_trials": n_trials,
        "val_fast": val_fast,
        "val_final": val_final,
        "cycle_n": n,
        "prolong_periods": PROLONG,
        "seconds": t0.elapsed().as_secs_f64(),
        "literature_priors": [
            "bayes_local — densify around good λ (Bayesian HPO)",
            "pbt_exploit — mutate near champion (population-based training)",
            "mo_shape — higher λ multi-objective shape preference",
            "aggressive — low λ long fade",
            "evo_explore — wide evolutionary explore",
            "racing_mid — mid-band racing / algorithm configuration",
        ],
        "meta_objective": "maximize residual_score = clamp(1 - residual_rms/max(ideal_rms,eps), 0, 1) on prolonged (tiled) ideal vs engine; soft gate S>=0.97; D/S auxiliaries only",
        "residual_formula": "score = clamp(1 - rms(engine_tiled - ideal_tiled) / max(rms(ideal_tiled), 1e-6), 0, 1); ideal = generate_sound_ideal (no open-wrap); engine = tile(DenoiseOpt(generate_sound), N=16)",
        "champion": top4.first().cloned().unwrap_or(json!({})),
        "top4": top4,
        "benchmark_matrix_5": five,
        "naive_classic_ref": classic_row,
        "production_frozen": {
            "denoise": fd / c,
            "shape": fs / c,
            "quality": 0.5 * (fd + fs) / c,
            "residual": fr / c,
        },
        "pareto_top20_fast": scored.iter().take(20).map(|(meta, hp, theta, loss, d, s, q, residual)| json!({
            "meta_score": meta,
            "residual": residual,
            "name": hp.name,
            "prior": hp.prior,
            "lambda": hp.lambda_shape,
            "val_fast": { "loss": loss, "denoise": d, "shape": s, "quality": q, "residual": residual },
            "theta": theta.as_slice(),
        })).collect::<Vec<_>>(),
        "artifact_path": artifact,
        "sessionId": "0ab8f9",
        "runId": if n_trials >= N_TRIALS_DEFAULT { "meta-1500" } else { "meta-sanity" },
        "note_frozen_theta": "FROZEN_THETA left unchanged; re-lock only after a full 1500 residual-objective run if desired",
    });

    let _ = std::fs::create_dir_all("brand/artifacts");
    if let Ok(s) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(artifact, &s);
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
                "runId": report["runId"],
                "message": "meta trials complete (residual objective)",
                "data": {
                    "n_trials": n_trials,
                    "champion": report["champion"]["name"],
                    "champion_residual": report["champion"]["val"]["residual"],
                    "champion_meta": report["champion"]["meta_score"],
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

/// 1500 literature-informed meta trials + top-4 vs naive matrix.
pub fn run_meta_learning_search_1500() -> serde_json::Value {
    run_meta_learning_search_n(N_TRIALS_DEFAULT, VAL_FAST_DEFAULT, VAL_FINAL_DEFAULT)
}

/// Back-compat thin wrapper used by older bin.
pub fn run_meta_learning_search() -> serde_json::Value {
    run_meta_learning_search_1500()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::denoise_opt::{residual_score, tile_cycle};

    #[test]
    fn sample_trial_shapes_ok() {
        let mut rng = Rng(1);
        let (hp, theta) = sample_trial(&mut rng, 0);
        assert!(hp.lambda_shape > 0.0);
        assert_eq!(theta[7], 0.0);
        let (_, _, s, q, residual) = eval_theta_fast(&theta, 0, 50, 128, hp.lambda_shape);
        assert!(s > 0.8, "shape={s}");
        assert!(q > 0.5);
        assert!((0.0..=1.0).contains(&residual), "residual={residual}");
    }

    #[test]
    fn residual_perfect_match_is_one() {
        let ideal: Vec<f32> = (0..64)
            .map(|i| (i as f32 / 64.0 * std::f32::consts::TAU).sin())
            .collect();
        let score = residual_score_prolonged(&ideal, &ideal, 16);
        assert!(
            (score - 1.0).abs() < 1e-5,
            "perfect match should be ~1, got {score}"
        );
    }

    #[test]
    fn residual_huge_wrap_cliff_near_zero() {
        let ideal: Vec<f32> = (0..64)
            .map(|i| (i as f32 / 64.0 * std::f32::consts::TAU).sin())
            .collect();
        let mut cliff = ideal.clone();
        cliff[0] = -2.0;
        cliff[63] = 2.0;
        let score = residual_score_prolonged(&ideal, &cliff, 16);
        assert!(
            score < 0.55,
            "huge wrap cliff should score nearer 0, got {score}"
        );
        assert!(score < 0.99);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn residual_score_always_in_unit_interval() {
        for seed in 0..40u64 {
            let n = 128usize;
            let (_, ideal) = generate_sound_ideal(seed, n);
            let (_, raw) = generate_sound(seed, n);
            let mut out = raw;
            apply_denoise_theta(&mut out, 0.0, &FROZEN_THETA);
            let s = residual_for_cycle(&ideal, &out);
            assert!(
                (0.0..=1.0).contains(&s),
                "seed {seed} residual={s} out of [0,1]"
            );
        }
        // Empty / mismatched edge
        assert_eq!(residual_score(&[], &[]), 0.0);
        let a = [1.0f32, -1.0];
        let b = tile_cycle(&a, 3);
        assert_eq!(b.len(), 6);
        let s = residual_score(&b, &b);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ideal_matches_engine_when_no_wrap_applied() {
        // Seeds without open-wrap must share identical baked cycles.
        let mut matched = 0u32;
        for seed in 0..200u64 {
            let (_, a) = generate_sound_ideal(seed, 64);
            let (_, b) = generate_sound(seed, 64);
            if a == b {
                matched += 1;
                let s = residual_score_prolonged(&a, &b, 8);
                assert!((s - 1.0).abs() < 1e-5, "identical cycles residual={s}");
            }
        }
        assert!(matched > 20, "expected some wrap-free seeds, got {matched}");
    }
}
