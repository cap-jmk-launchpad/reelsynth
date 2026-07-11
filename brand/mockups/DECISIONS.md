# ReelSynth S6 Mockup — Design Decisions

Gate 1 static HTML/CSS mockups for user review before egui implementation.

## Layout rationale

### Full layout (`index.html` — 1280×820)

Three-column main area maximizes wavetable editing while keeping osc and filter controls within thumb reach:

| Zone | Width | Role |
|------|-------|------|
| Osc left column | 280px | Oscillator tabs, level/pan/coarse knobs, WT position slider, unison |
| Center hero | flex | WT frame strip (always visible), 2D waveform + 3D mesh (always visible) |
| Right rail | 240px | Filter, ADSR graph + knobs, LFO, stereo meter |

Below the main grid:

- **Mod matrix** — collapsible, default expanded, ~8 rows visible (~28px row height)
- **FX rack** — collapsible, default expanded, horizontal slot cards
- **Piano** — toggle in header; full-width keyboard in footer band
- **Status bar** — 36px footer with audio/MIDI telemetry

### S1 performance (`s1-performance.html`)

Stripped layout for the current playable app milestone:

- Header: text wordmark only (no preset bar, no transport)
- Center: preset hero (name, category, static spectrum SVG) + WT position strip
- Right rail: WT position + filter knobs with wired/live affordance; ADSR and LFO greyed out
- Footer: piano toggle + status (no osc/mod/FX/2D/3D)

### Narrow (`narrow.html` — 900×600)

Shows responsive collapse behaviour:

- Osc column shrinks to 220px, rail to 200px
- Mod matrix and FX sections collapsed (chevron + title bar only)
- Piano hidden (toggle off in header)
- WT strip + 2D/3D remain visible (compressed heights)

## Grid and spacing

All measurements align to an **8px base grid** (`--grid-unit: 8px`):

- Panel padding: 8px
- Column gutters: 8px internal padding
- Section header: 6px vertical padding
- Knob row gap: 12px (`--space-sm`)
- Header/footer heights: 48px / 36px (6×8 / 4.5×8 — snapped to nearest grid)

Spacing tokens from `brand/design/tokens.css` are used for larger rhythm (md 20, lg 32).

## Breakpoints

| Viewport | Behaviour |
|----------|-----------|
| ≥ 1280px | Full S6 layout (canonical) |
| 900–1279px | Narrow variant: tighter columns, collapsed mod/FX |
| < 900px | Out of scope for Gate 1; plugin window will enforce minimum size in egui |

## Colour and typography

- Dark theme only in mockups (`data-theme="reelsynth-dark"`)
- Tokens imported from `../design/tokens.css` — Majico palette:0
- Accent `#183d50` for fills and playhead; `--accent-ui: #2a6b8a` added in `mockups.css` for interactive highlights (knob arcs, wired badges, hover borders) where `#183d50` alone lacks contrast on dark surfaces
- Fonts via Google Fonts CDN: Inter (body), IBM Plex Sans (headings), JetBrains Mono (values, MIDI, mod amounts)

## Component philosophy

- **Hardware-in-software**: inset panel shadows, radial knob gradients, LED-style status dot, wired badges on live params
- **Not SaaS**: no cards with heavy drop shadows, no rounded pill nav, no hero marketing blocks
- **Realistic data**: Factory Lead preset, Saw Morph bank, frame 108/255, cutoff 1.2 kHz — matches current `reelsynth-app` defaults

## Explicit non-goals (Gate 1)

- No JavaScript interactivity (static review only)
- No egui / Rust implementation
- No competitor screenshots or reference pastes in mockup assets
- No `/flow/*` Majico routes (retired per workspace rules)

## Files

| File | Purpose |
|------|---------|
| `mockups.css` | Shared synth component styles |
| `index.html` | Full S6 layout |
| `s1-performance.html` | S1 playable-app target |
| `components.html` | Widget gallery |
| `narrow.html` | Collapsed mod/FX at 900×600 |
| `COMPONENT_SPEC.md` | HTML class → egui mapping |

## Opening locally

```bash
open brand/mockups/index.html
open brand/mockups/s1-performance.html
open brand/mockups/components.html
open brand/mockups/narrow.html
```

Requires network for Google Fonts on first load; tokens.css is local.
