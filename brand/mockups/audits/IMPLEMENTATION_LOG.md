# ReelSynth UI Implementation Log

Progress tracker for the UI platform plan (`reelsynth_ui_platform` + `reelsynth_ui_redesign`).

## 2026-07-12 — Loop iteration 2 (S2 complete)

### S1 — Standalone shell ✅

| Item | Status |
|------|--------|
| Preset Open/Save (`.reelpreset` via `rfd`) | ✅ |
| MIDI input device select + note routing (`midir`) | ✅ |
| Functional header (wordmark, Open/Save, MIDI, piano toggle, status) | ✅ |
| Wired WT + filter params | ✅ (prior) |
| Piano keyboard + QWERTY | ✅ (prior) |
| `cargo test --no-default-features -j 1` | ✅ |

**Commit:** `c252ca6` — preset I/O, MIDI routing, functional header

### S2 — WT editor ✅

| Item | Status |
|------|--------|
| WT position strip (center + rail knob synced) | ✅ |
| 2D waveform view from current bank frame (`view_2d.rs`) | ✅ |
| 3D mesh surface from bank slices + rib grid (`view_3d.rs`) | ✅ |
| Reveal panels via `S1ShellConfig::show_wt_editor` | ✅ app enables |
| Bank hot-swap on preset load | ✅ |
| WT header menu: Open/Save `.reelwt` + factory banks | ✅ |
| Save `.reelwt` via `WavetableBank::write_file` | ✅ |
| Center layout: hero → strip → 2D/3D views (COMPONENT_SPEC) | ✅ |
| Frame draw/edit | ⬜ S3 |
| Morph A→B | ⬜ S3 |
| Import Vital/WAV/Serum | ⬜ S3 |
| egui-in-plugin-host spike | ⬜ end of S2 / S3 |

**WT center audit vs `index.html`:**

| Landmark | Mockup | egui | Delta |
|----------|--------|------|-------|
| WT strip height | 72px | 72px (`WT_STRIP_HEIGHT`) | 0 |
| 2D/3D view min height | 140px | 140px (`WT_VIEW_MIN_HEIGHT`) | 0 |
| Views split | 50/50 grid | `horizontal` half-width panels | 0 |
| Strip label | `Saw Morph · 256 frames` | `{bank} · {n} frames · pos {i}` | +pos (S1 sync) |
| 2D fill + stroke | accent fill 35%, accent-ui 2px | convex fill + 2px line | match |
| 3D mesh | CSS gradient placeholder | slice polylines + depth ribs from bank | data-driven |
| Center column order | strip → views | hero → strip → views (S1+S2) | hero is S1-only |

**Commit:** _(pending this loop)_

### Next loop

1. Frame draw/edit stub or minimal pencil tool
2. Morph controls (position-only stub ok if engine supports)
3. Import Vital/WAV/Serum from WT menu
4. S2 end spike: minimal CLAP + egui embed

### Sprint summary

| Sprint | Status |
|--------|--------|
| S-brand | ✅ |
| S0 | ✅ |
| S1 | ✅ |
| S2 | ✅ |
| S3 | ⬜ |
| S4 | ⬜ |
| S5 | ⬜ |
| S6 | ⬜ |
| S7 | roadmap |
