//! CLI: `reelsynth-export`

use reelsynth::export::{
    export_preset, export_reelpack, export_wavetable, load_preset, parse_targets,
    resolve_bank_for_preset, ExportOptions, ExportReport, ExportTarget,
};
use reelsynth::wavetable::WavetableBank;
use std::env;
use std::path::PathBuf;

fn usage() -> &'static str {
    "Usage:\n  \
     reelsynth-export <target> <input> -o <output> [--targets vital,wav,...] [--name NAME]\n\n\
     Targets: vital, wav, serum, ableton, sfz, midi, audio, reelpack\n\n\
     Examples:\n  \
     reelsynth-export vital table.reelwt -o table.vitaltable\n  \
     reelsynth-export reelpack patch.reelpreset -o out/ --targets vital,wav,serum,ableton,sfz,midi,audio"
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("{}", usage());
        std::process::exit(1);
    }

    let target_str = &args[1];
    let input = PathBuf::from(&args[2]);
    let mut out = PathBuf::from(".");
    let mut targets_raw = String::new();
    let mut table_name = "reelsynth".to_string();

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("missing value for -o");
                    std::process::exit(1);
                }
                out = PathBuf::from(&args[i]);
            }
            "--targets" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("missing value for --targets");
                    std::process::exit(1);
                }
                targets_raw = args[i].clone();
            }
            "--name" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("missing value for --name");
                    std::process::exit(1);
                }
                table_name = args[i].clone();
            }
            "-h" | "--help" => {
                println!("{}", usage());
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let Some(target) = ExportTarget::parse(target_str) else {
        eprintln!("unknown target: {target_str}");
        std::process::exit(1);
    };

    let opts = ExportOptions {
        table_name,
        ..ExportOptions::default()
    };

    let report = if target == ExportTarget::Reelpack {
        let targets = if targets_raw.is_empty() {
            parse_targets("vital,wav,serum,ableton,sfz,midi,audio")
        } else {
            parse_targets(&targets_raw)
        };
        export_reelpack(&input, &out, &targets, &opts)
    } else if input.extension().and_then(|e| e.to_str()) == Some("reelwt") {
        let bank = match WavetableBank::read_file(input.to_str().unwrap()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };
        export_wavetable(&bank, target, &out, &opts)
    } else {
        let preset = match load_preset(&input) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };
        let bank = match resolve_bank_for_preset(&input, &preset) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };
        export_preset(&preset, &bank, target, &out, &opts)
    };

    print_report(&report);
    if !report.success {
        std::process::exit(1);
    }
}

fn print_report(report: &ExportReport) {
    if let Ok(text) = serde_json::to_string_pretty(report) {
        println!("{text}");
    }
}
