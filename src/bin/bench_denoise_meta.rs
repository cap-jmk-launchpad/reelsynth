//! Meta-learning + hyperparameter search for DenoiseOpt.
//!
//! ```bash
//! cargo run -p reelsynth --release --bin bench_denoise_meta
//! ```

fn main() {
    eprintln!("Running 1500 literature-informed DenoiseOpt meta trials…");
    let report = reelsynth::denoise_meta::run_meta_learning_search_1500();
    eprintln!(
        "champion: {}",
        serde_json::to_string_pretty(&report["champion"]).unwrap()
    );
    eprintln!(
        "benchmark_matrix_5: {}",
        serde_json::to_string_pretty(&report["benchmark_matrix_5"]).unwrap()
    );
    eprintln!(
        "n_trials={} seconds={:.1}",
        report["n_trials"],
        report["seconds"].as_f64().unwrap_or(0.0)
    );
    eprintln!("wrote brand/artifacts/denoise_opt_meta_1500.json");
}
