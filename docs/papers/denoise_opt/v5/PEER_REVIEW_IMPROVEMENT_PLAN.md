# DenoiseOpt v5 Peer-Review Improvement Plan

> **Structure:** This plan follows the **writing-plans** skill layout: phased workstreams, checkbox tasks, exact files, effort, dependencies, done-when acceptance criteria, and risks.  
> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.  
> **Plan only.** Do not implement until the user explicitly starts execution.

**Goal:** Answer a peer-review-style critique of DenoiseOpt v5 by deepening evaluation honesty, baselines, wavetable-native realism, and theory clarity, while keeping claims narrowly scoped to cycle-local wavetable seam repair.

**Architecture:** Keep the existing DenoiseOpt hybrid (GA+PPO+PBT+NAS) and prolonged residual $R$ as the primary story. Extend depth inside that scope: cliff-stratified reporting, true self-supervised seam restorers, real instrument/wavetable cycles under the wrap-discontinuity protocol, and honest theory demotion. Do not pivot to general speech enhancement SOTA.

**Tech Stack:** Python/PyTorch benches in `reelsynth/scripts/`; Rust `sound_bench` + `export_sound_bench_tiles`; paper TeX in `denoise-opt-meta/paper/v5/` mirrored to `reelsynth/docs/papers/denoise_opt/v5/`.

**Canonical plan path:** `denoise-opt-meta/paper/v5/PEER_REVIEW_IMPROVEMENT_PLAN.md`  
**Mirror:** `reelsynth/docs/papers/denoise_opt/v5/PEER_REVIEW_IMPROVEMENT_PLAN.md`  
**Critique date:** 19 July 2026  
**Source critique:** peer-review paste (Strengths / Weaknesses / Suggestions) on DenoiseOpt v5 manuscript

## Global Constraints

- **Narrow claims only:** cycle-local wavetable / wrap-seam artifact repair. Never claim general speech or music enhancement SOTA.
- **OA-only cites:** every `\cite` / `\bibitem` must resolve to an open-access source already audited or newly OA-verified.
- **No fake PESQ on sine:** PESQ/STOI remain out of scope for non-speech tiles. Do not invent speech-metric numbers on synthetic cycles.
- **MUSHRA ignored** unless the user later explicitly requests a listening study.
- **No em-dash slop** in future prose notes, abstracts, or discussion edits (use periods or short clauses).
- **Speech/music LibriSpeech/MUSDB probes default OFF.** Wavetable-native realism first.
- **Holdout seed** `20260719`; overnight search seed `1902771841` (never thousand-separated in prose).
- **Primary metric remains** prolonged residual $R\in[0,1]$; secondary SNR/SDR/wrap-jump; optional seam-local metrics must be explicitly defined.

---

## Executive summary

The critique is partly venue mismatch (NeurIPS-style “real speech/music + modern DL SOTA”) and partly valid scientific pressure (identity $R$ looks too easy, wrap-jump barely moves, theory is thin, no true Noise2Noise baseline, synthetic-only evaluation).

**Locked stance:** stay niche and deepen. Treat DenoiseOpt as a DSP / audio-engineering contribution with a hybrid search protocol, not a general audio ML SOTA paper. Answer valid points with:

1. **Cliff-stratified reporting** so hard wrap cases carry the claim.
2. **Stronger learned baselines** (true N2N-style + lightweight seq model) on the same generator.
3. **Wavetable-native realism** (ReelSynth exports / OA instrument one-shots under wrap protocol), not a LibriSpeech bait-and-switch.
4. **Theory honesty** (demote trivial props; prove or delete Lemma 1).
5. **Writing / venue repositioning** toward DAFx / AES / arXiv DSP rather than top ML impact theater.

**Recommended first phase:** Phase A (metric & baseline honesty). Fast, high ROI, unblocks abstract/discussion updates, and reframes the identity-$R$ critique without new training runs.

---

## Critique triage map

Every weakness and suggestion is classified as **REAL** (must fix), **PARTIAL** (address within wavetable scope), or **REJECT** (venue mismatch / out of locked stance).

### Weaknesses

| ID | Critique item | Verdict | Action |
|----|---------------|---------|--------|
| W1 | Eval confined to synthetic single-cycle waveforms; limits top-ML impact | **PARTIAL** | Phase C: add real instrument / ReelSynth wavetable cycles under wrap protocol. Keep synthetic as primary controlled corpus. Reject “must do LibriSpeech to matter.” |
| W2 | No eval on real instrument / speech / polyphonic audio | **PARTIAL** / **REJECT split** | **PARTIAL:** real instruments + exported WT cycles (Phase C). **REJECT:** speech/polyphony as primary claim domain. OOD speech probe stays optional and OFF by default. |
| W3 | Honest “no general speech/music SOTA” limits contribution | **REJECT** (as a defect) | Keep honesty. Soften overclaim language only; reposition venue (Phase E). Narrow claims are a feature for DAFx/AES. |
| W4 | Props 1–2 trivial for ML audience | **REAL** | Phase D: demote to appendix “immediate consequences” or one-line remarks. |
| W5 | Prop 3 self-evident monotone map | **REAL** | Phase D: demote or fold into $R$ definition paragraph; no standalone “theorem theater.” |
| W6 | Lemma 1 is only a sketch; not a real lemma | **REAL** | Phase D: either rigorous proof under stated assumptions **or** delete and state “no convergence / wrap-closure guarantees for hybrid search.” |
| W7 | Identity $R$ deceptively high (~0.97 / ~0.965) | **REAL** | Phase A: explain residual-mass vs ideal RMS; stratify by cliff; make hard-cliff the operative regime in Results/Discussion. |
| W8 | Wrap-jump barely changes (0.91 → 0.90) while $R$ jumps | **REAL** | Phase A: report wrap-jump + optional edge RMSE / click energy on hard subset; clarify that favorite may repair tiled residual without fully zeroing endpoint jump; avoid claiming wrap-jump SOTA if numbers stay flat. |
| W9 | Cite Noise2Noise philosophy but no N2N-style baseline | **REAL** | Phase B: train true corrupt→corrupt or corrupt→ideal-sibling restorer without search leakage; distinguish from CNN-on-$R$ meta-fit. |

### Suggestions

| ID | Suggestion | Verdict | Action |
|----|------------|---------|--------|
| S1 | Test on real-world WT recordings or standard music/speech sets | **PARTIAL** | Phase C: real WT / instrument one-shots. **REJECT** LibriSpeech/MUSDB as required primary. Optional labeled OOD probe only if user insists later. |
| S2 | LibriSpeech / MUSDB for “relevance” | **REJECT** | Venue mismatch with locked narrow claims. Document rejection in Phase E “What we will NOT do.” |
| S3 | Modern DL baseline: small CNN/RNN self-supervised on seam problem | **REAL** | Phase B: N2N-style + lightweight LSTM/1D-CNN seq model predicting ideal from cracked tile. |
| S4 | Replace props with search convergence / error-landscape analysis | **PARTIAL** | Phase D: optional honest complexity/budget or landscape note only if non-fake; prefer removing weak theory over inventing guarantees. |
| S5 | Prove Lemma 1 conditions or remove | **REAL** | Phase D (same as W6). |
| S6 | Report top 10% high-cliff cases separately | **REAL** | Phase A: top 10% and top 25% wrap-jump / cliff-magnitude strata. |
| S7 | Add polynomial fitter / LSTM / SSM baselines | **REAL** / **PARTIAL** | Phase B: LSTM or small 1D CNN required; polynomial fitter optional small add if cheap; full SSM optional if timeboxed. |

### Strengths to preserve (do not regress)

| Strength | Keep / reinforce |
|----------|------------------|
| Clear cycle-local seam problem framing | Keep title/abstract claim freeze |
| Hybrid GA+PPO+PBT+NAS under one $R$ objective | Keep as method contribution |
| Modular Algorithms 1–8 | Keep; only clarify search vs fit vs score |
| Frozen eval protocol, seeds, compute, open source | Extend protocol with strata docs |
| Stats (Wilcoxon, multi-seed ±std, ablations) | Reuse on hard-cliff subsets |
| Classical DualCosine / FIR retained | Keep; document why Demucs/DiffWave screened |

---

## What we will NOT do

- Full **LibriSpeech / MUSDB / general speech enhancement SOTA** chase.
- Invented **PESQ/STOI** on sine or non-speech wavetable tiles.
- **MUSHRA** listening panels unless user later requests.
- Fake **convergence theorems** for hybrid GA+PPO+PBT search.
- Pivoting abstract to “general audio denoising” or NeurIPS-impact cosplay.
- Training **full Demucs / DiffWave** as primary baselines (screened as compute/domain mismatch; document why).
- Em-dash-heavy prose rewrites.

---

## Suggested venue positioning

| Venue class | Fit | Notes |
|-------------|-----|-------|
| **DAFx** | Best primary target | DSP problem, bake operators, seam metrics, hybrid search as engineering method |
| **AES** (convention / journal short) | Strong secondary | Wavetable / synthesis artifact repair audience |
| **arXiv cs.SD / eess.AS** | Always | DSP-framed preprint; honest narrow abstract |
| **Top ML (NeurIPS/ICML/ICLR)** | Poor fit unless reframed | Critique’s impact complaint stands under current narrow scope; do not chase without speech/music SOTA (which we refuse) |

Phase E updates Related Work + Discussion to state this positioning explicitly.

---

## Priority order for execution

1. **Phase A** — Metric & baseline honesty (fast, high ROI)  
2. **Phase D** — Theory cleanup (can parallelize with A; mostly TeX)  
3. **Phase B** — Stronger learned baselines (needs training compute)  
4. **Phase C** — Wavetable-native realism (needs sample-pack decision)  
5. **Phase E** — Writing / positioning (after A numbers; refresh again after B/C)

Parallelization note: A and D can run in the same sprint. B and C can overlap once sample source and N2N scope are decided.

---

## Open decisions for user

1. **Real sample pack source?** Options: (a) ReelSynth factory / exported `.reelwt` cycles only; (b) license-clean OA instrument one-shot pack (name required); (c) both. Default recommendation: (a) first, then (b) if licenses clear.
2. **Noise2Noise baseline scope?** Options: (a) corrupt→corrupt same-seed sibling pair; (b) corrupt→ideal-sibling supervised proxy labeled as “oracle N2N sibling”; (c) both. Default recommendation: (c) with clear naming so search leakage is impossible to confuse with (b).
3. **Lemma 1 fate?** Prove under narrow assumptions vs delete + “no guarantees.” Default recommendation: **delete + honesty sentence** unless a short rigorous proof fits in <0.5 page.
4. **Optional seam-local metric?** Edge RMSE and/or click energy on tiled wrap region. Default recommendation: **edge RMSE** (simplest, reproducible).
5. **Venue commitment?** DAFx 2027-ish vs AES vs arXiv-only for now.

---

## File map (shared across phases)

| Path | Role |
|------|------|
| `reelsynth/scripts/overnight_gpu_rl_arch.py` | `make_batch`, residual $R$, overnight search |
| `reelsynth/scripts/bench_sota_matrix.py` | Multi-family SOTA matrix |
| `reelsynth/scripts/bench_canonical_eval_dataset.py` | Canonical holdout corpus stats |
| `reelsynth/scripts/metrics_snr_sdr.py` | SNR/SDR/wrap-jump helpers |
| `reelsynth/scripts/bench_rust_sound_bench_tiles.py` | Rust tile transfer bench |
| `reelsynth/src/sound_bench.rs` | Rust procedural families |
| `reelsynth/src/bin/export_sound_bench_tiles.rs` | Export 20 Rust tiles JSON |
| `denoise-opt-meta/paper/v5/subsections/methods.tex` | Props/Lemma/$R$ definition |
| `denoise-opt-meta/paper/v5/subsections/experiments.tex` | Protocol, dataset |
| `denoise-opt-meta/paper/v5/subsections/results.tex` | Tables, strata |
| `denoise-opt-meta/paper/v5/subsections/discussion.tex` | Identity-$R$ / hard-cliff narrative |
| `denoise-opt-meta/paper/v5/subsections/limitations.tex` | Scope honesty |
| `denoise-opt-meta/paper/v5/subsections/related_work.tex` | N2N contrast |
| `denoise-opt-meta/paper/v5/main.tex` | Abstract / keywords |
| `denoise-opt-meta/paper/v5/EVAL_PROTOCOL.md` | Frozen protocol extension |
| `denoise-opt-meta/paper/v5/figures/*.json` | Regenerated figure/table blobs |
| Mirror tree | `reelsynth/docs/papers/denoise_opt/v5/` (sync tex + this plan) |

---

# Phase A — Metric & baseline honesty

**Effort:** S–M  
**Dependencies:** None (uses existing tensors / benches)  
**ROI:** Highest. Directly answers W7, W8, S6.

### Intent

- Stratify by wrap-jump / cliff magnitude (top 10% and top 25% hardest tiles).
- Report $R$, SNR/SDR, wrap-jump for identity vs favorite (and DualCosine) on hard subsets.
- Clarify in text why identity $R$ can be high (small residual mass relative to ideal RMS) and that hard-cliff is the operative regime.
- Optional: secondary seam-local metric (edge RMSE).

### Tasks

#### Task A1: Cliff-stratum scoring script

**Files:**
- Create: `reelsynth/scripts/bench_cliff_strata.py`
- Modify: `reelsynth/scripts/metrics_snr_sdr.py` (export helpers if needed)
- Create: `denoise-opt-meta/paper/v5/figures/cliff_strata.json`
- Mirror copy of JSON under `reelsynth/docs/papers/denoise_opt/v5/figures/`

**Interfaces:**
- Consumes: `overnight_gpu_rl_arch.make_batch`, favorite bake cfg from overnight freeze, DualCosine, identity
- Produces: JSON with keys `all`, `top25_wrap`, `top10_wrap` each containing per-method `{R_mean, R_std, snr_mean, sdr_mean, wrap_jump_mean, n}`

- [ ] **Step 1:** Draw a large holdout batch (e.g. 4096 cycles, seed `20260719`) with engine + ideal; compute per-tile wrap-jump and cliff amplitude.
- [ ] **Step 2:** Define strata thresholds as empirical percentiles of wrap-jump (document exact cutoffs in JSON `meta`).
- [ ] **Step 3:** Score identity / DualCosine / favorite / `seam_fir3` on each stratum; write `cliff_strata.json`.
- [ ] **Step 4:** Unit sanity: assert `top10` mean wrap-jump > `top25` > `all`; assert `n` counts match.
- [ ] **Step 5:** Commit script + JSON (no TeX yet).

**Done-when:** JSON exists; hard strata show identity $R$ drop and/or favorite $\Delta R$ vs DualCosine larger than on `all`. If not, document that honestly (still publish strata).

**Risks:** Hard subset may be small; use at least 256 tiles in top 10% or widen draw. Do not cherry-pick cutoffs after seeing favorite numbers.

#### Task A2: Optional edge RMSE / click energy

**Files:**
- Modify: `reelsynth/scripts/metrics_snr_sdr.py`
- Modify: `reelsynth/scripts/bench_cliff_strata.py`
- Modify: `denoise-opt-meta/paper/v5/EVAL_PROTOCOL.md`

- [ ] **Step 1:** Define `edge_rmse` = RMS of `(y - r*)` on indices `[0:SEAM_W] U [L-SEAM_W:L]` after bake (document formula in EVAL_PROTOCOL).
- [ ] **Step 2:** Optional `click_energy` = mean square of first difference across the tiled wrap boundary (samples at $kL-1$ and $kL$).
- [ ] **Step 3:** Add both to stratum JSON; mark optional in TeX until numbers are stable.

**Done-when:** Metric definitions are unambiguous and reproducible from JSON + protocol text.

**Risks:** Click energy can correlate with intentional bright saw content; restrict narrative to wrap-local indices.

#### Task A3: Manuscript text + table for strata

**Files:**
- Modify: `denoise-opt-meta/paper/v5/subsections/results.tex`
- Modify: `denoise-opt-meta/paper/v5/subsections/discussion.tex`
- Modify: `denoise-opt-meta/paper/v5/subsections/experiments.tex`
- Modify: `denoise-opt-meta/paper/v5/main.tex` (abstract hard-cliff clause once numbers land)
- Sync mirror tex

- [ ] **Step 1:** Add Table `tab:cliff-strata` (all / top25 / top10) with identity, DualCosine, favorite columns for $R$ and wrap-jump (SNR/SDR if space).
- [ ] **Step 2:** Add one Results paragraph: identity $R$ is high because residual mass ≪ ideal RMS on mild cliffs; operative claim is hard-cliff.
- [ ] **Step 3:** Discussion: address wrap-jump flatness honestly; if favorite does not reduce wrap-jump much, claim tiled residual repair, not endpoint-zeroing.
- [ ] **Step 4:** Rebuild PDF; sync mirror.

**Acceptance criteria (Phase A):**
- [ ] Top 10% and top 25% strata reported in paper + JSON.
- [ ] Explicit prose explaining high identity $R$.
- [ ] No new PESQ/MUSHRA.
- [ ] Abstract/discussion mention hard-cliff regime once measured.

**Risks:** Abstract update too early before JSON freezes; freeze cutoffs in EVAL_PROTOCOL first.

---

# Phase B — Stronger learned baselines

**Effort:** M–L  
**Dependencies:** Phase A protocol freeze helpful but not strictly required; same generator as overnight.  
**Answers:** W9, S3, S7.

### Intent

- Add a **true Noise2Noise-style / self-supervised seam restorer** trained on the same generator, distinct from current “CNN-on-$R$” meta-fit.
- Add a **lightweight LSTM or small 1D CNN seq model** predicting ideal from cracked tile.
- Keep classical DualCosine/FIR; document why full Demucs/DiffWave remain screened.

### Tasks

#### Task B1: N2N-style baseline trainer

**Files:**
- Create: `reelsynth/scripts/train_n2n_seam_baseline.py`
- Create: `reelsynth/scripts/baselines/n2n_seam.py` (model + train/eval API)
- Create: `denoise-opt-meta/paper/v5/figures/n2n_baseline.json`
- Modify: `reelsynth/scripts/bench_sota_matrix.py` (register method)

**Interfaces:**
- Consumes: `make_batch` / family batch API producing `(ideal, engine)` or two independent corruptions
- Produces: checkpoint + eval row `{method, R, snr, sdr, wrap_jump, ms, params}`

Training modes (implement both; name clearly):
1. **`n2n_corrupt_corrupt`:** two independent cliff draws from same mid-cycle seed → predict one from the other (true N2N spirit).
2. **`n2n_sibling_supervised`:** engine → ideal sibling (oracle proxy; label as supervised sibling, not “unsupervised search”).

Constraints:
- No outer NAS / PPO / access to overnight champion search state.
- Fixed architecture (small 1D CNN or U-Net-lite), fixed Adam, fixed step budget documented in JSON.
- Same $L{=}256$, seed policy documented.

- [ ] **Step 1:** Implement model + train loop with seed `20260719` train / held-out eval seeds.
- [ ] **Step 2:** Smoke train 500 steps; assert loss decreases; assert eval $R$ > DualCosine on canonical holdout or document failure.
- [ ] **Step 3:** Full train (budget in JSON, e.g. 20k–50k steps); export metrics.
- [ ] **Step 4:** Add rows to SOTA matrix + cliff strata script.
- [ ] **Step 5:** Commit checkpoints metadata (paths) + JSON; large `.pt` via release/artifact policy already used by overnight models.

**Done-when:** At least one N2N-style method appears in Results tables, clearly distinguished from CNN-on-$R$ meta-fit and from DenoiseOpt favorite.

**Risks:** Sibling-supervised may beat favorite and weaken meta-search story; report honestly. Corrupt-corrupt may be weak; still valuable as baseline.

#### Task B2: Lightweight seq baseline (LSTM or 1D CNN)

**Files:**
- Create: `reelsynth/scripts/baselines/seq_seam_lstm.py` (or `seq_seam_cnn1d.py`)
- Create: `reelsynth/scripts/train_seq_seam_baseline.py`
- Modify: `bench_sota_matrix.py`, `bench_cliff_strata.py`

- [ ] **Step 1:** Choose LSTM (~1–2 layers, hidden ≤64) **or** depthwise 1D CNN; document param count target ≤100k.
- [ ] **Step 2:** Train engine→ideal on same generator; evaluate on canonical + 20-family + hard strata.
- [ ] **Step 3:** Optional cheap polynomial/endpoint fitter baseline if <1 day.

**Done-when:** Seq baseline in Table `tab:sota-main` (or new baselines table) with ms/batch and params.

#### Task B3: Screened heavy models documentation

**Files:**
- Modify: `denoise-opt-meta/paper/v5/subsections/related_work.tex`
- Modify: `denoise-opt-meta/paper/v5/subsections/experiments.tex`
- Modify: `denoise-opt-meta/paper/v5/subsections/limitations.tex`

- [ ] **Step 1:** Explicit paragraph: Demucs/DiffWave screened for domain mismatch (full-band speech/music denoisers vs $L{=}256$ periodic seam), compute, and OA/eval protocol mismatch.
- [ ] **Step 2:** Point to new N2N/seq baselines as the appropriate modern learned controls.

**Acceptance criteria (Phase B):**
- [ ] True N2N-style baseline trained without meta-search leakage.
- [ ] LSTM or small 1D CNN seq baseline reported.
- [ ] Classical DualCosine/FIR retained.
- [ ] Demucs/DiffWave screening stated in TeX.
- [ ] Hard-strata numbers include new baselines.

**Risks:** Training instability; keep architectures tiny. Do not silently use favorite init weights.

---

# Phase C — Evaluation realism (wavetable-native)

**Effort:** M–L  
**Dependencies:** User decision on sample pack source; Phase A metric helpers.  
**Answers:** W1, W2 (partial), S1 (partial). **Rejects S2 as primary.**

### Intent

Prefer real instrument one-shots / exported wavetable cycles from ReelSynth or OA sample packs (license clean), run through the wrap discontinuity protocol. Speech/music OOD probes remain default OFF.

### Tasks

#### Task C1: Wrap protocol for real cycles

**Files:**
- Create: `reelsynth/scripts/real_wt_wrap_protocol.py`
- Modify: `denoise-opt-meta/paper/v5/EVAL_PROTOCOL.md`

Protocol (lock in docs):
1. Load mono cycle or extract one period ($L{=}256$ resample/crop with documented rule).
2. Create ideal = periodized / endpoint-matched reference (or original closed seam).
3. Apply open-wrap cliff of amplitude $\pm\mathcal{U}(0.08,0.43)$ over `SEAM_W=8` (same as synthetic).
4. Score methods with prolonged $R$, SNR/SDR, wrap-jump, edge RMSE.

- [ ] **Step 1:** Implement loader + cliff apply + score.
- [ ] **Step 2:** Unit test on one synthetic cycle fed through real loader path (bit-comparable cliff).

#### Task C2: ReelSynth / factory export corpus

**Files:**
- Create: `reelsynth/src/bin/export_reelsynth_wt_cycles.rs` (or extend export bin)
- Create: `brand/artifacts/real_wt_cycles/` or `artifacts/real_wt_cycles/` (license README)
- Create: `denoise-opt-meta/paper/v5/figures/real_wt_matrix.json`

- [ ] **Step 1:** Export ≥20 cycles from factory banks / patches (saw morph, lead stacks, etc.).
- [ ] **Step 2:** Run wrap protocol + method matrix (identity, DualCosine, FIR, favorite, N2N, seq).
- [ ] **Step 3:** Add Results subsection “Wavetable-native transfer” (secondary to synthetic primary if needed).

#### Task C3: Optional OA instrument pack

**Files:**
- Create: `denoise-opt-meta/paper/v5/SAMPLE_LICENSES.md`
- Same scoring pipeline as C2

- [ ] **Step 1:** User picks pack; record license + URL in SAMPLE_LICENSES.md.
- [ ] **Step 2:** Import only license-clean one-shots; no scraped commercial libs.
- [ ] **Step 3:** Score and report as additional secondary table.

#### Task C4: Expand Rust sound_bench if needed

**Files:**
- Modify: `reelsynth/src/sound_bench.rs`
- Modify: `reelsynth/src/bin/export_sound_bench_tiles.rs`

- [ ] **Step 1:** Only if real-cycle coverage gaps remain: add families that mimic instrument-like spectra while staying procedural.
- [ ] **Step 2:** Re-export ≥20 tiles; refresh Rust transfer table.

#### Task C5: Explicitly keep speech OOD OFF

**Files:**
- Modify: `limitations.tex`, `EVAL_PROTOCOL.md`

- [ ] **Step 1:** State LibriSpeech/MUSDB not used; optional future OOD probe would be labeled and non-claim-bearing.

**Acceptance criteria (Phase C):**
- [ ] ≥20 real or exported wavetable cycles scored under wrap protocol.
- [ ] License file present for any external samples.
- [ ] No LibriSpeech/MUSDB primary table.
- [ ] Results distinguish synthetic primary vs wavetable-native secondary.

**Risks:** Period extraction from one-shots is ambiguous; document cropping/resample rules tightly. License mistakes are blocking.

---

# Phase D — Theory cleanup

**Effort:** S  
**Dependencies:** None (TeX-first). Can parallel Phase A.  
**Answers:** W4–W6, S4 (partial), S5.

### Intent

Demote/remove trivial Props 1–2 (and Prop 3). Prove Lemma 1 rigorously under stated assumptions **or** delete and state no guarantees. Optional honest complexity/budget note only if non-fake.

### Tasks

#### Task D1: Demote Props 1–3

**Files:**
- Modify: `denoise-opt-meta/paper/v5/subsections/methods.tex`
- Optional create: `denoise-opt-meta/paper/v5/subsections/appendix_immediate.tex` (if venue allows appendix)

- [ ] **Step 1:** Replace Prop 1–2 environments with a short “Immediate consequences of Eq. (R)” paragraph (or appendix).
- [ ] **Step 2:** Fold Prop 3 into the definition of $R$ as one sentence (“$R$ is strictly decreasing in residual RMS when pre-clamp scores lie in (0,1)”).
- [ ] **Step 3:** Grep for `\ref{prop:` and fix dangling refs.

#### Task D2: Lemma 1 decision

**Files:**
- Modify: `methods.tex`, `discussion.tex` / `limitations.tex`

**Default path (recommended):** delete Lemma 1; add:

> We do not claim convergence guarantees for the hybrid GA+PPO+PBT outer loop, nor a general wrap-closure theorem for arbitrary bake cells. Empirical gains are reported under the frozen protocol.

**Alternate path:** write a full proof under explicit assumptions (endpoint-only difference; bake zeros jump; mid-cycle RMS penalty ≤ seam gain) with inequalities spelled out; rename to Proposition if still mild.

- [ ] **Step 1:** User confirms delete vs prove (Open Decision 3).
- [ ] **Step 2:** Apply chosen path; remove “sufficient sketch” language entirely.

#### Task D3: Optional honest addition

**Files:**
- Modify: `methods.tex` or `experiments.tex`

Only if true:
- Outer-loop complexity: population size × proposals × fit steps × batch cost (big-O style accounting), **or**
- Empirical landscape note: reward variance / plateau observations from overnight log (no fake convexity claims).

- [ ] **Step 1:** Add ≤1 paragraph; cite overnight freeze numbers already in Results.
- [ ] **Step 2:** Reject any “converges to global optimum” wording.

**Acceptance criteria (Phase D):**
- [ ] No trivial proposition theater in main Methods.
- [ ] Lemma 1 either proved or removed with explicit no-guarantee sentence.
- [ ] No invented convergence theorem.

**Risks:** Over-deleting leaves Methods looking thin; keep $R$ definition + algorithms crisp.

---

# Phase E — Writing / positioning

**Effort:** S–M  
**Dependencies:** Phase A numbers minimum; refresh after B/C.  
**Answers:** W3 (reposition), S1 framing, Related Work N2N contrast.

### Intent

Sharper Related Work vs Noise2Noise audio restorers; soften residual overclaim; emphasize DSP/audio-engineering venue fit; update abstract/discussion with hard-cliff results.

### Tasks

#### Task E1: Related Work contrast

**Files:**
- Modify: `denoise-opt-meta/paper/v5/subsections/related_work.tex`

- [ ] **Step 1:** Paragraph contrasting: prior N2N speech restorers operate on broadband noisy speech; we operate on periodic $L$-sample seams with procedural siblings; previously we cited N2N as ranking philosophy only; Phase B now adds a same-domain N2N-style control.
- [ ] **Step 2:** OA-only cite check.

#### Task E2: Soften overclaim + venue sentence

**Files:**
- Modify: `introduction.tex`, `discussion.tex`, `conclusion.tex`, `main.tex`

- [ ] **Step 1:** Ensure every bold claim is scoped to cycle-local seam repair.
- [ ] **Step 2:** Add venue-fit sentence: contribution aimed at DSP / synthesis artifact repair (DAFx/AES), not general speech enhancement leaderboards.
- [ ] **Step 3:** Remove any residual “meta-learning SOTA” flavor without evidence.

#### Task E3: Abstract / discussion refresh after measurements

**Files:**
- Modify: `main.tex`, `discussion.tex`, `results.tex`

- [ ] **Step 1:** Insert hard-cliff top-10% headline numbers from Phase A.
- [ ] **Step 2:** Mention N2N/seq baselines once Phase B lands.
- [ ] **Step 3:** Mention wavetable-native secondary once Phase C lands.
- [ ] **Step 4:** Prose pass: no em dashes; no semicolon stacks; no fake PESQ.

#### Task E4: Protocol + changelog sync

**Files:**
- Modify: `EVAL_PROTOCOL.md`, `denoise-opt-meta/paper/CHANGELOG.md`, `PLAN_PROGRESS.md`
- Sync mirror tree

- [ ] **Step 1:** Document strata, new baselines, real-cycle protocol.
- [ ] **Step 2:** Changelog entry for v5.x peer-review response (user-visible honesty upgrades).

**Acceptance criteria (Phase E):**
- [ ] Related Work explains prior missing N2N compare and new baseline.
- [ ] Venue positioning explicit.
- [ ] Abstract reflects hard-cliff (and later B/C) without widening claims.
- [ ] Mirror synced; OA cites intact.

**Risks:** Writing ahead of numbers; gate abstract numeric updates on frozen JSON.

---

## Cross-phase acceptance checklist

- [ ] Narrow claim freeze intact (no speech SOTA pivot).
- [ ] Cliff strata (10%/25%) published.
- [ ] Identity-$R$ explained; hard-cliff is operative regime in prose.
- [ ] True N2N-style + seq baselines in tables.
- [ ] Wavetable-native secondary eval (not LibriSpeech primary).
- [ ] Theory cleaned (props demoted; Lemma 1 proved or deleted).
- [ ] OA-only cites; no PESQ-on-sine; no MUSHRA; no em-dash slop.
- [ ] Plan mirrored under `reelsynth/docs/papers/denoise_opt/v5/`.

---

## Suggested commit sequence (when executing)

1. `feat(eval): cliff-stratum bench + JSON` (A1–A2)  
2. `docs(paper): hard-cliff table + identity-R prose` (A3)  
3. `docs(paper): demote trivial props; lemma honesty` (D)  
4. `feat(baselines): N2N seam + seq LSTM/CNN` (B)  
5. `feat(eval): real WT wrap protocol + matrix` (C)  
6. `docs(paper): related work, abstract, venue positioning` (E)

---

## Execution handoff

Plan complete and saved to:

- `denoise-opt-meta/paper/v5/PEER_REVIEW_IMPROVEMENT_PLAN.md`
- `reelsynth/docs/papers/denoise_opt/v5/PEER_REVIEW_IMPROVEMENT_PLAN.md` (mirror)

**Recommended first phase:** Phase A (metric & baseline honesty).

**Two execution options when ready:**

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks  
2. **Inline Execution** — execute with `superpowers:executing-plans`, batch with checkpoints

Do not start implementation until the user confirms phase start and open decisions (especially sample pack + N2N scope + Lemma 1 fate).
