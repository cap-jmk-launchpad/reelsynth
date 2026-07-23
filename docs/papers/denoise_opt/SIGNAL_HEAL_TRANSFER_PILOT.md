# Appendix stub — sci/eng wrap-heal transfer pilot

**Date:** 20260723T201503Z

Pilot transfer of DenoiseOpt’s winning outer loop (**hybrid GA–PPO / `hybrid_lstm`**) to public cycle-local wrap tasks (CWRU bearings, MIT-BIH ECG; MFPT if available). Period length $L=256$; score = prolonged residual $R$ vs ideal sibling.

## Results (prolonged $R$, higher better)

### cwru_bearings

| Method | $R$ | Label |
|--------|-----|-------|
| `ours_hybrid_lstm` | 0.8705 | Ours (hybrid GA–PPO / hybrid_lstm outer loop) |
| `cot_linear_periodize` | 0.8675 | bearings classical bad-COT control (passthrough of linear resample) |
| `no_bake` | 0.8675 | classical / passthrough |
| `endpoint_pin_mean` | 0.7990 | classical endpoint pin |
| `linear_fade` | 0.7929 | classical linear fade |
| `seam_fir3` | 0.7865 | classical seam FIR3 |
| `cot_cubic_then_dualcosine` | 0.7846 | bearings classical: DualCosine on cracked (not published deep COT) |
| `dual_cosine` | 0.7846 | classical DualCosine fade |
| `soft_periodize_hann` | 0.7703 | classical Hann soft-periodize |

### mitbih_ecg

| Method | $R$ | Label |
|--------|-----|-------|
| `ours_hybrid_lstm` | 0.8792 | Ours (hybrid GA–PPO / hybrid_lstm outer loop) |
| `endpoint_pin_mean` | 0.7796 | classical endpoint pin |
| `spline_join` | 0.7134 | ECG classical spline/FIR join (not Cycle-GAN) |
| `seam_fir3` | 0.6615 | classical seam FIR3 |
| `no_bake` | 0.6403 | classical / passthrough |
| `linear_fade` | 0.2719 | classical linear fade |
| `beat_average_sbmm_lite` | 0.2135 | ECG classical SBMM-lite beat average (not BeatDiff/Cycle-GAN) |
| `dual_cosine` | 0.2009 | classical DualCosine fade |
| `soft_periodize_hann` | 0.1479 | classical Hann soft-periodize |

## Caveats

- Classical board + domain classical proxies; not a claim of beating published deep SOTA unless those models were executed.
- Modest outer-loop budget (250 outer iters / dataset, fit_steps=40) — pilot hours, not multi-day.
- Real content is z-scored per period; musical/clinical absolute scale not preserved.
- **Reporting:** table $R$ is holdout-refit. Search-time champion $R$ was higher on CWRU (≈0.887) and MIT-BIH (≈0.883); use holdout for comparisons.
- CWRU: classical DualCosine / fades **hurt** vs no-bake; Ours only narrowly beats no-bake on holdout — transfer is weak on this wrap construction.
- ECG: DualCosine collapses morphology; Ours clearly leads the classical board (endpoint pin is the strongest classical).

Artifacts live in reelsynth `brand/artifacts/signal_heal_transfer/`.

