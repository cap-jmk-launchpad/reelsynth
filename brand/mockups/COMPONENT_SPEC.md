# ReelSynth Component Spec — HTML → egui Mapping

Reference for Phase B egui implementation. Sizes are physical pixels at 1× scale (1280×820 canonical viewport).

## Layout shell

| HTML class | egui widget / pattern | Size / notes |
|------------|----------------------|--------------|
| `.rs-app` | `egui::CentralPanel` + fixed viewport | 1280×820 default; `ViewportBuilder::with_inner_size` |
| `.rs-header` | `TopBottomPanel::top` | height 48px |
| `.rs-footer` | `TopBottomPanel::bottom` | height 36px |
| `.rs-main` | `ui.horizontal` + nested panels | grid: 280px \| flex \| 240px |
| `.rs-col--osc` | `SidePanel::left` | `min_width(280.0)` |
| `.rs-col--right` | `SidePanel::right` | `min_width(240.0)` |

## Typography

| Context | Font | Size |
|---------|------|------|
| `.rs-wordmark` | IBM Plex Sans (Proportional) | 15px semibold |
| `.rs-panel__title` | IBM Plex Sans | 11px uppercase |
| Body labels | Inter (Proportional) | 13px |
| `.rs-knob__value`, `.mono` | JetBrains Mono | 11px |
| `.rs-preset-name` | IBM Plex Sans | 14px |

Map via `reelsynth_ui_theme::apply_fonts` + custom `FontId` overrides per widget.

## Knobs (`.rs-knob`)

| Variant | Dial size | egui approach |
|---------|-----------|---------------|
| `--sm` | 48×48px | Custom `Knob` widget or `egui::DragValue` + painted arc |
| `--md` | 56×56px | Default rack knob |
| `--lg` | 64×64px | S1 performance emphasis |
| `--wired` | + glow ring | `Painter::circle_stroke` accent-ui + "Live" badge label |
| `--disabled` | same dial | `ui.add_enabled(false, …)` + grey pointer |

Arc: 270° sweep, stroke 3px, track `--border`, fill `--accent` / `--accent-ui`.  
Pointer: 2×38% height rect, rotated by normalized value.  
Label below: 10px uppercase muted; value: mono 11px.

## Sliders (`.rs-slider`)

| Part | Size | egui |
|------|------|------|
| Track | 6px tall | `egui::Slider::new` with custom `SliderStyle` |
| Thumb | 12×12px circle | `rail_height: 6.0`, handle radius 6 |
| Fill | accent gradient | `WidgetVisuals` active bg or custom paint |

WT position: linear 0–255. Cutoff: logarithmic 40–12000 Hz.

## Tabs (`.rs-tabs`, `.rs-tab`)

| State | Visual |
|-------|--------|
| Default | `--surface2` container, 2px gap |
| Active | `--accent` fill, `--accent-on` text |
| Disabled | opacity 0.35 |

egui: `ui.selectable_value` in horizontal group, or `egui_extras::StripBuilder`.  
Tab padding: 6×8px, radius 8px.

## WT strip (`.rs-wt-strip`)

| Element | Size | egui |
|---------|------|------|
| Container | 72px tall | `ui.allocate_ui_with_layout` fixed height |
| Frame cell | flex equal width | `ui.image` or custom paint per frame thumbnail |
| Playhead | 2px wide | vertical line at `position / num_frames` |
| Active frame | border `--accent-ui` | stroke on hovered frame |

Data: `WavetableBank` frame count (256 default), position from `SynthEngine::set_wt_position`.

## WT views

| Class | Size | egui |
|-------|------|------|
| `.rs-wt-view` (2D) | min 140px tall | `egui_plot::Plot` or custom waveform `Shape::line` |
| `.rs-wt-3d-mesh` | same | Phase C: mesh from bank; mockup = gradient placeholder |

2D/3D split 50/50 in center column below strip.

## Piano (`.rs-piano`)

| Element | Size | egui constant |
|---------|------|---------------|
| Container | 72px tall (`--piano-h`) | `layout::PIANO_HEIGHT` |
| White key | 15px wide (`--piano-white-w`) | `layout::PIANO_WHITE_KEY_WIDTH` |
| Black key | 58% of white width, 56% of piano height | `PIANO_BLACK_*_RATIO` |
| Active | `--accent-ui` gradient | accent fill on key |

14 white keys (2 octaves) centered in piano wrap; fixed key width (not flex-stretch). Toggle via footer `.rs-toggle`.

## Mod matrix

| Class | Size | egui |
|-------|------|------|
| `.rs-mod-row` | 28px row | `ui.horizontal` per route |
| Grid columns | 100 + flex + 72 + 72 + 48 px | `egui::Grid` or `StripBuilder` |
| `.rs-mod-cell` | 72×28px | `DragValue` or clickable label |
| Variants | positive/negative/bipolar | colour from sign |

~8 rows visible in expanded section; scroll for remainder.  
Collapse: `CollapsingHeader::new("Modulation Matrix")`.

## FX rack (`.rs-fx-slot`)

| Element | Size | egui |
|---------|------|------|
| Slot card | 160px wide | `Frame::group` |
| Active | `--accent-ui` border | stroke colour |

Collapse: `CollapsingHeader::new("Effects")`.

## ADSR graph (`.rs-adsr-graph`)

| Property | Value |
|----------|-------|
| Height | 80px |
| Curve | `Shape::line` through A/D/S/R nodes |
| Nodes | 3px circles, draggable in egui |

Pair with 4× `--sm` knobs below graph.

## Meter (`.rs-meter`)

| Property | Value |
|----------|-------|
| Size | 48px tall, 2 bars |
| Fill | green→accent gradient by peak level |

egui: `ui.add(egui::ProgressBar)` vertical or custom paint from audio peaks.

## Buttons and toggles

| Class | egui | Notes |
|-------|------|-------|
| `.rs-btn` | `Button` primary | accent fill |
| `.rs-btn--ghost` | `Button` inactive | border only |
| `.rs-toggle` | `SelectableLabel` or checkbox styled | piano visibility |

## Disabled group (`.rs-group--disabled`)

Wrap children in `ui.add_enabled_ui(false, |ui| { … })`.  
Opacity 0.38 equivalent via `Visuals::widgets.noninteractive`.

## Collapsible sections (`.rs-section`)

| State | Behaviour |
|-------|-------------|
| Expanded | chevron down, body visible |
| `.rs-section--collapsed` | chevron −90°, `display: none` on body |

egui: `CollapsingHeader` with `default_open(true)` for mod/FX in full layout; `false` in narrow.

## S1-specific

| Element | Notes |
|---------|-------|
| `.rs-spectrum-hero` | static bars SVG; future: FFT scope |
| `.rs-wire-badge` | indicates param wired to audio engine |
| ADSR/LFO panels | `rs-group--disabled` until S6 |

## Colour tokens (implementation)

Import from `ui-theme/src/lib.rs` / `brand/design/tokens.css`:

| CSS var | Hex | Use in mockup |
|---------|-----|---------------|
| `--accent` | `#183d50` | fills, playhead, active tab |
| `--accent-ui` | `#2a6b8a` | mockups only — interactive highlights |
| `--accent-muted` | `#061e2a` | hover, bipolar mod cell bg |
| `--border` | `#27272a` | panel edges, knob track |

## Motion (egui Phase B)

- Collapse chevron: 120ms ease `cubic-bezier(0.4, 0, 0.2, 1)`
- Hover rows: `color-mix(accent 14%, bg-muted)` — matches DESIGN.md §4
