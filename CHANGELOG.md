# Changelog

All notable changes to ReelSynth are documented here.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- **Design Quant knobs (Layers)** — multi-curve Layers pane no longer traps selection on the last WT/residual (often L3): hovering/clicking L1 or L2 prefers that curve over overlapping Quant knobs; knobs follow `selected_layer_idx` for any editable layer
- **Design Quant knobs (Selected)** — right Selected column paints Quant knobs on the plot painter after the wave (and keeps the toolbar above the fill) so editable WT/residual layers always show draggable knobs when Quant > 0
- **Held-note dropout** — brief note-off→note-on while releasing no longer hard-resets soft-start (`filter_fade`); audio device switches re-voice held keys so sustain stays continuous
- **Quant wrap crackle** — quant frame resample periodizes the cycle seam (same idea as factory tables) so Hold/Linear edits do not reintroduce near-vertical WT wraps
- **Design Quant knobs** — knobs only on editable WT/residual curves when Quant > 0; Selected column always shows knobs + **All·…** / per-segment interp for those layers (VA shows a hint); Result/Layers siblings stay stroke-only
- **Design WT click→select** — clicking a layer curve in Result / Layers commits `selected_layer_idx`; Quant knobs appear only on the selected WT/residual curve in multi-curve panes (siblings stay stroke-only); Selected pane stays fully interactive. Knob proximity still wins over curve click
- **Compose piano roll** — notes now commit on pointer release (egui clears interact pos on drag end); track select no longer clears the active clip; default clip auto-ensured so Pencil works immediately

- **Held-note crackle** — widen VA/WT BLEP so saw/stack wrap cliffs are no longer near-vertical (was ~0.98 sample jump at A4); regressions cover Factory Lead mid/late sustain with FX. Bright saw overtones remain intentional; unintended wrap clicks are suppressed.
- **Design Result curve** — stack Result drawn only on the left 2D pane (distinct fill); right pane is layers-only; individual layer curves drag on both panes (Y=level, X=phase/WT)
- **Quant knobs** — dots snap to the selected layer curve (proximity hit + level/sign scale) on **both** Design panes; quantized edit polyline drawn through knobs for intuitive shaping
- **Right Layers pane** — follows selection (`Edit · Layer N · type`); selected curve gets fill + emphasis; dim siblings; Quant reshape when the selected layer is wavetable
- **Factory wavetables (WT menu)** — loading a factory/import bank now promotes a wavetable layer (and ducks VA siblings) so audio matches the wave editor, not just the bank label

### Changed

- **Result overlay method** — `add` / `avg` / `avg_equal` combo in the Result pane caption (writes `stack_mode`, live audio)

- **Compose clip strip** — collapsed by default (**Clips ▸**); piano roll is the primary surface with in-toolbar tool hints

- **Layer curve hover** — Design **Layers** / **Result** panes preview the nearest selectable waveform (thicker/brighter stroke, hand cursor, status `Hover · Ln · type`) before click; Quant knob hover still wins when the pointer is on a knob
- **Quant knob hover** — clearer snap feedback across Result / Layers / Selected: enlarged brighter knobs with glow + slot guide, thickened active curve, grab/grabbing cursor, status `Slot N · amp ±x.xx`
- **Design WT layout** — two panes replaced by three equal columns (Result / Layers / Selected); toolbar and per-layer quant moved to Selected column
- Cleared workspace `cargo check` warnings (`-D warnings` clean for reelsynth / reelsynth-ui / reelsynth-app); Cursor **beforeShellExecution** hook blocks `git push` unless a fresh `.cursor/compile-clean.stamp` exists (refresh via `node .cursor/hooks/require-clean-compile.js`)
- WT menu section renamed **Factory wavetables** (was “Factory banks”) to match layer-first Design
- **Settings** moved from floating modal window to a **Settings** dropdown in the top header navbar
- Removed Design pane animations (ambient waves, phase playhead scrub, idle repaint loops)
- README expanded with doc links and capability matrix
- **Design ↔ Compose** mode switch in header; Compose hides WT editor and osc column
- On-screen piano upgraded to **88 keys (A0–C8)** with horizontal scroll and scale-fold dimming

### Added

- **Per-segment Quant interp** — each layer stores a curve default (`quant_interp`) plus `quant_segment_interps` (`len = knobs−1`); Selected toolbar **All·…** fills every segment; clicking a knob edits that knob→next segment (last knob has no outgoing segment; shows `end · no next`). Modes: Hold, Linear, Spline, Poly, Expo, MA
- **Selectable audio output** — header **Audio** combo lists CPAL output devices; selection persists in app settings
- **Auto-select new audio output** — when a device appears (DI / interface hot-plug), switch to it with a status message (Settings toggle; default on). Only newly appeared devices trigger a switch — no thrash on every default-device poll
- **Three-column Design WT** — Result · Layers · Selected panes with per-column Quant roles; **Residual** wavetable layer created on first Result quant edit
- `.cursor/hooks/` — `require-clean-compile.js` runs `cargo check` (`-D warnings`) and writes a stamp; `verify-compile-stamp.js` gates agent `git push` on that stamp (hook host cannot spawn cargo)
- **Compose mode** — header toggle switches from Design (sound engineering) to a mini-DAW layout: transport bar, multi-track arrangement, piano roll editor, scene grid, 88-key keyboard strip
- **Ableton-style clip editor** — thin clip strip + dominant piano roll; playable key column with QWERTY glyphs; unified live audition (keys / QWERTY / MIDI / pencil); transport ▶ voices scheduled notes; scenes collapsed by default
- User documentation pack: GETTING_STARTED, UI, WORKFLOW, FREE_STACK, SDK, REELDEMO_INTEGRATION
- [docs/README.md](docs/README.md) documentation index
- [AGENTS.md](AGENTS.md), [CONTRIBUTING.md](CONTRIBUTING.md) for agents and contributors
- `.cursor/skills/reelsynth-workflow/` — workflow skill for Cursor agents
- `scripts/bundle-docs-images.sh` — zip screenshots for GitHub Release upload
- Screenshot URLs via GitHub Release assets (not committed to repo)

## [0.1.0] - 2026-07-12

### Added

- Standalone egui app with realtime audio (cpal) and MIDI input (midir)
- Wavetable voice, filter, ADSR, LFO, mod matrix, FX chain
- `.reelwt` / `.reelpreset` native formats
- Import: Vital, WAV folder, Serum WT subset
- Export CLI: Vital, WAV, Serum, Ableton JSON, SFZ, MIDI, audio, reelpack
- Python PyO3 bindings for render and export
- Plugin UI shell (CLAP entry stub, no host I/O)

[Unreleased]: https://github.com/reeldemo/reelsynth/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/reeldemo/reelsynth/releases/tag/v0.1.0
