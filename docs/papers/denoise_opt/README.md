# DenoiseOpt paper (local mirror)

Canonical versioned paper: **[reeldemo/denoise-opt-meta](https://github.com/reeldemo/denoise-opt-meta)** → `paper/v8/`.

**Application lit survey:** [SIGNAL_HEALING_APPLICATIONS_LIT.md](SIGNAL_HEALING_APPLICATIONS_LIT.md) — where wrap/seam/periodize repair could transfer (wavetable, granular, PSOLA, graphics seams, ECG, …).

## Meta objective (residual)

\[
R=\mathrm{clamp}\!\left(1-\frac{\mathrm{rms}(y_{\mathrm{engine}}-y_{\mathrm{ideal}})}{\max(\mathrm{rms}(y_{\mathrm{ideal}}),\varepsilon)},\,0,\,1\right)
\]

- Ideal: `generate_sound_ideal`, tiled $N{=}16$
- Engine: DenoiseOpt(`generate_sound`), tiled $N{=}16$
- Soft gate: $\mathcal{S}\ge 0.97$ else $\times 0.45$
- Nested inner loss opt on $L=(1-\mathcal{D})+\lambda(1-\mathcal{S})$

## Headline (1500 trials)

| Algorithm | Residual |
|-----------|----------|
| Naive DualCosine | 0.698 |
| Meta Top 1 `evo_explore_515` | **0.824** |

## Reproduce

```bash
cargo run -p reelsynth --release --bin bench_denoise_meta
python brand/artifacts/render_benchmark_matrix.py
# Full paper:
#   cd ../denoise-opt-meta/paper/v8 && pdflatex main.tex
```

## v8 (2026-07-24) — current

Review-response rewrite complete (W0–W5): IMRaD Methods + math formalization; vibrato/hear/WT gallery; HP ±50% OAT sensitivity (500 iters, seed `1902771841`); Discussion/Limitations honesty; `REVIEW_RESPONSE.md` filled.
See upstream: `denoise-opt-meta/paper/v8` (+ `REVIEW_RESPONSE.md`).

## v7 (2026-07-19) — archived

Weakness elimination F1–F5 snapshot. Upstream: `denoise-opt-meta/paper/v7`.

## v5 (2026-07-19) — archived

Draft snapshot under local `v5/` if present. Canonical upstream: `denoise-opt-meta/paper/v5`.

## v4 (2026-07-18) — superseded archive

See `v4/` (canonical upstream: `denoise-opt-meta/paper/v4`).
