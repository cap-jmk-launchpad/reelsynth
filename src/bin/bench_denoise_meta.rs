//! Meta-learning + hyperparameter search for DenoiseOpt.
//!
//! Primary objective: prolonged residual score ∈ [0,1] (1 = best).
//!
//! ```bash
//! # Full 1500-trial paper run (slow)
//! cargo run -p reelsynth --release --bin bench_denoise_meta
//!
//! # Short sanity (e.g. 40 trials, smaller val)
//! cargo run -p reelsynth --release --bin bench_denoise_meta -- 40
//! ```

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_trials = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1500);
    let (val_fast, val_final) = if n_trials < 1500 {
        // Fast sanity: small validation budgets
        (40usize, 80usize)
    } else {
        (400usize, 2000usize)
    };
    eprintln!(
        "Running {n_trials} DenoiseOpt meta trials (residual objective, val_fast={val_fast}, val_final={val_final})…"
    );
    let report = reelsynth::denoise_meta::run_meta_learning_search_n(n_trials, val_fast, val_final);
    eprintln!(
        "champion: {}",
        serde_json::to_string_pretty(&report["champion"]).unwrap()
    );
    eprintln!(
        "benchmark_matrix_5: {}",
        serde_json::to_string_pretty(&report["benchmark_matrix_5"]).unwrap()
    );
    eprintln!(
        "production_frozen residual={:.4} quality={:.4}",
        report["production_frozen"]["residual"]
            .as_f64()
            .unwrap_or(0.0),
        report["production_frozen"]["quality"]
            .as_f64()
            .unwrap_or(0.0)
    );
    eprintln!(
        "n_trials={} seconds={:.1} artifact={}",
        report["n_trials"],
        report["seconds"].as_f64().unwrap_or(0.0),
        report["artifact_path"].as_str().unwrap_or("?")
    );
}
