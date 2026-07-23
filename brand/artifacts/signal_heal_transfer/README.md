# Signal-heal transfer pilot

Generated: `20260723T201503Z`

## Method under test

- **Ours:** hybrid GA–PPO outer loop (`hybrid_lstm` in `bench_meta_approaches_5k.py`).
- FitCell / SeamCell / arch search reused; period length fixed to `N=256`.
- Metric: DenoiseOpt prolonged residual $R$ (same formula as wavetable).
- Pilot budget: modest outer iters (see `config`); not full industrial overnight.

## Wrap construction

- **CWRU bearings:** DE @12 kHz; per-rev windows via RPM; **ideal** = cubic resample to $L$; **engine** = linear resample (bad-COT proxy) + DenoiseOpt-style wrap cliff + seam noise.
- **MFPT:** same protocol when zip available; fixed shaft-rate periods.
- **MIT-BIH ECG:** R–R normal beats → $L$; **ideal** = local mean template + mild endpoint equalize (SBMM-lite classical); **engine** = single beat + wrap cliff.

## Seeds

- Search / construction seed: `1902771841`
- Holdout sample seed: `20260719`

## Honesty / limits

- Baselines are a **classical board** (+ domain classical COT / SBMM-lite). We do **not** claim BeatDiff / Cycle-GAN / deep order-tracking SOTA unless those weights ran.
- Do not wipe `brand/artifacts/meta_approach_compare/`.
- Optional KIT CNC / IEEE PMU / BMRB NMR skipped if login/paywall.

### Skipped optional

- **mfpt_bearings:** MFPT zip URL returned HTML (login/wall); skipped
- **kit_cnc:** skipped — KIT CNC DOI browser/login flow
- **ieee_pmu:** skipped — IEEE DataPort free-account wall
- **bmrb_nmr:** skipped — BMRB FID deferred

## Results table

See `results_table.json` and `fig_signal_heal_transfer.{png,pdf}`.

## Reproduce

```bash
.venv_gpu/Scripts/python scripts/_tmp_dl_signal_heal.py   # or built-in ensure
.venv_gpu/Scripts/python scripts/bench_signal_heal_transfer.py --iters 250
```

