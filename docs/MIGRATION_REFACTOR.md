# Migration: Big-Bang Refactor

Breaking changes from the `refactor/naming-and-structure` branch.

## Search-replace table

| Old | New | Notes |
|-----|-----|-------|
| `cargo run -p reelsynth-app --bin reelsynth-app` | `cargo run -p reelsynth-app --bin reelsynth-app` | Binary rename |
| `UiState` | `UiState` | Plugin/editor imports |
| `draw_shell` | `draw_shell` | Plugin editor |
| `ShellActions` | `ShellActions` | |
| `ShellConfig` | `ShellConfig` | |
| `ShellMidiDevices` | `ShellMidiDevices` | |
| `ShellLayout` | `ShellLayout` | |
| `ShellLayoutOptions` | `ShellLayoutOptions` | |
| `APP_HEIGHT_COMPACT` | `APP_HEIGHT_COMPACT` | |
| `ui/src/s1.rs` | `ui/src/shell.rs` | |
| `EffectSlotUi` | `EffectSlotUi` | |
| `EffectRackState` | `EffectRackState` | |
| `draw_effect_rack` | `draw_effect_rack` | |
| `ModSlotUi` | `ModSlotUi` | |
| `app/src/patch_sync.rs` | `ui/src/state_sync.rs` | |
| `clap_entry_pending` | `clap_entry_pending` | |
| `draw_level_meter` | `draw_level_meter` | |
| `tests/qa/phase1.rs` | `tests/qa/foundation.rs` | |
| `tests/qa/phase2.rs` | `tests/qa/oscillator.rs` | |
| `tests/qa/phase3.rs` | `tests/qa/fm.rs` | |
| `tests/qa/phase4.rs` | `tests/qa/effects.rs` | |
| `tests/qa/phase5.rs` | `tests/qa/scopes.rs` | |
| `tests/qa/phase6.rs` | `tests/qa/modulation.rs` | |
| `src/patch.rs` | `src/patch/` module | |
| `src/fx.rs` | `src/fx/` module | |
| `src/voice/kernel.rs` | `src/voice/process.rs` + helpers | |
| `src/scope.rs` + `src/preview.rs` | `src/scope/` module | |
| `ui/src/osc.rs` | `ui/src/osc_column.rs` | |
| `ui/src/scope.rs` | `ui/src/scope_strip.rs` | |

## Python module

`grok_dsp` Python alias removed; use `reelsynth` module directly.
