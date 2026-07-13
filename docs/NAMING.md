# ReelSynth Naming Conventions

Single source of truth for code naming across the ReelSynth workspace.

## Layer rules

| Layer | Files | Types | Functions |
|-------|-------|-------|-----------|
| **DSP engine** (`src/`) | snake_case, domain dirs (`fx/`, `osc/`, `voice/`) | `PascalCase` nouns (`EffectSlot`, `VoiceState`) | `snake_case` verbs (`process_sample`, `render_note`) |
| **UI** (`ui/src/`) | panel names (`osc_column.rs`, `fx_rack.rs`, `scope_strip.rs`) — **no sprint IDs** | `Ui` suffix for bridge types (`EffectSlotUi`, `ModSlotUi`) | `draw_*` for egui entrypoints |
| **App** (`app/`) | role-based (`audio_host.rs`, `state_sync.rs`) | — | — |
| **Bins** | `reelsynth-app` (playable), `reelsynth-ui-proto` (widget demo), `reelsynth-export` | — | — |
| **Tests** | domain names (`tests/qa/voice.rs`), not `phaseN.rs` | — | `snake_case` matching feature |

## Banned patterns

- Sprint prefixes in code: `S1`, `S3`, `S6` in module/type names (OK in `brand/mockups/` and git history only)
- `_stub` suffix on shipped symbols — use `_placeholder`, `#[cfg(feature)]`, or `pending_` prefix
- Duplicate crate/binary names (`reelsynth-app` bin vs `reelsynth-app` lib)
- `kernel` for voice DSP — use `process` or `sample`
- `preview` for scope generation — use `scope_preview` or merge into `scope/`

## Effect vocabulary

Use **`Effect*`** prefix for effect-related types (matches `EffectSlot`, `EffectType` in patch):

- `EffectSlotUi` — UI bridge for a single FX slot
- `EffectRackState` — FX rack panel state
- `draw_effect_rack` — FX rack draw entrypoint (file may remain `fx_rack.rs`)

## Modulation vocabulary

- `ModSlotUi` — UI bridge for a modulation route (was `ModSlotUi`)
- `ModMatrixState` — mod matrix panel state

## Legacy fields

- `fx_bypass` on `Patch` — deprecated; kept with `#[serde(alias)]` + migration into `effects`

## Shell / UI state

- `UiState` — top-level UI state (was `UiState`)
- `draw_shell` — main shell draw entrypoint (was `draw_shell`)
- `ShellConfig`, `ShellActions`, `ShellLayout` — shell configuration types

## Bin names

| Binary | Purpose |
|--------|---------|
| `reelsynth-app` | Standalone playable app |
| `reelsynth-ui-proto` | Widget demo |
| `reelsynth-export` | Export CLI |
