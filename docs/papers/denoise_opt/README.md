# DenoiseOpt paper (arXiv-style)

**Author:** Julian M. Kleber  
**ORCID:** [0000-0001-5518-0932](https://orcid.org/0000-0001-5518-0932)  
**Email:** [julian.m.kleber@gmail.com](mailto:julian.m.kleber@gmail.com)

## Repositories

| Repo | Role |
|------|------|
| [reeldemo/reelsynth](https://github.com/reeldemo/reelsynth) | Synth DSP, frozen `FROZEN_THETA`, benches |
| [reeldemo/denoise-opt-meta](https://github.com/reeldemo/denoise-opt-meta) | Public paper PDF, 1500-trial JSON, figures, scripts |

Sources: `main.tex` (article class, arXiv-friendly). Figures under `figures/`. Compiled PDF: `main.pdf`.

## 1500-trial summary

Literature-informed meta-learning / HPO over six prior families (`bayes_local`, `pbt_exploit`, `mo_shape`, `evo_explore`, `racing_mid`, …). Champion: **`racing_mid_1043`**.

| Algorithm | \(Q\) (matrix, \(N{=}2000\)) |
|-----------|------------------------------|
| Naive DualCosine | ≈ 0.789 |
| Meta Top 1 (`racing_mid`) | ≈ 0.790 |
| Meta Top 2–4 | ≈ 0.790 |

Shape stays \(\mathcal{S}\approx 0.997\) on Top 1. Artifacts: `brand/artifacts/denoise_opt_meta_1500.json`. Frozen θ in `src/denoise_opt.rs` matches champion `theta`.

## Build

```bash
cd docs/papers/denoise_opt
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```

Or upload `main.tex` + `figures/` to Overleaf / arXiv. Mirror release: [denoise-opt-meta](https://github.com/reeldemo/denoise-opt-meta).

## Regenerating figures

```bash
cargo run -p reelsynth --release --bin bench_denoise_meta
python brand/artifacts/render_benchmark_matrix.py
python brand/artifacts/render_paper_figures.py
copy brand\artifacts\fig_*.png docs\papers\denoise_opt\figures\
```

## Literature

Harvested with **Klaut Research MCP** (local mode → OpenAlex/Crossref/arXiv):

- `brand/artifacts/literature_klaut_research.json`
- `brand/artifacts/literature_meta_audio.json`
- `brand/artifacts/literature_core_citations.json`

Paper distinguishes **used** vs **screened** sources in Related Work and the bibliography.
