---
name: audit-reelsynth-ui
description: >-
  Audits ReelSynth egui UI against HTML mockups using screenshots or live
  captures. Compares layout regions, tokens, widgets, and sprint visibility
  rules; outputs severity-ranked findings with egui fix pointers. Use when the
  user attaches a screenshot, asks to audit UI, compare app vs mockup, run Gate
  1/2 review, or check visual parity for reelsynth-ui or reelsynth-app.
disable-model-invocation: true
---

# ReelSynth UI Screenshot Audit

Design-first workflow: **mockup approval → proto → app**. Layout changes require mockup update first.

## Invoke

- `@audit-reelsynth-ui` or ask to "audit ReelSynth UI"
- Shorthand: `/audit-ui` (same skill)

## Inputs

| Input | Action |
|-------|--------|
| Screenshot attached | Primary audit target — read the image |
| No screenshot | Run app or open mockup in browser, capture screenshot |
| "Compare to mockup" | Side-by-side audit against HTML reference |

## Quick start

1. **Identify sprint context** from visible panels (S1 vs full S6 vs narrow).
2. **Load reference** — see [Reference files](#reference-files).
3. **Run region pass** — header → center hero → WT strip → right rail → footer/piano.
4. **Score findings** — Critical / Major / Minor / Polish (definitions in [reference.md](reference.md)).
5. **Emit report** using the template below.
6. **If fixing** — mockup first for layout; then egui paths in [Fix routing](#fix-routing).

## Sprint context detection

| Visible in screenshot | Reference mockup | Sprint |
|----------------------|------------------|--------|
| Preset hero + WT strip + right rail only; no osc/mod/FX/2D/3D | `s1-performance.html` | **S1** |
| Full three-column + mod matrix + FX + piano | `index.html` | **S6** |
| Collapsed mod/FX, narrower columns, piano off | `narrow.html` | Responsive |
| Widget gallery (knob/piano/tabs rows) | `components.html` | Gate 1 components |

**Sprint visibility rule:** unshipped panels must be **hidden**, not dimmed placeholders. Extra panels in S1 = Major; missing shipped panels = Critical.

## Reference files

Always read before auditing:

| File | Purpose |
|------|---------|
| `brand/mockups/s1-performance.html` | S1 layout (current app target) |
| `brand/mockups/index.html` | Full S6 layout |
| `brand/mockups/narrow.html` | Responsive collapse |
| `brand/mockups/components.html` | Widget gallery |
| `brand/mockups/COMPONENT_SPEC.md` | HTML → egui sizes |
| `brand/mockups/DECISIONS.md` | Locked layout decisions |
| `brand/design/tokens.css` | Colour + spacing tokens |
| `brand/mockups/mockups.css` | `--accent-ui` and mockup-only tokens |

Plan context: `.cursor/plans/reelsynth_ui_redesign_cc8a6033.plan.md` (Gate 1/2, ≤4px parity target).

## Audit workflow

### A. Screenshot-only (user attaches image)

1. Read screenshot; note viewport size if inferable.
2. Map visible regions to mockup regions (see [reference.md](reference.md) region map).
3. Walk checklist: spacing, colours, typography, knobs, piano, disabled states, sprint panels.
4. Produce structured report (template below).
5. Optionally describe side-by-side deltas vs mockup; use browser MCP to open mockup HTML if available.

### B. Live comparison (no screenshot)

```bash
# S1 app (audio + preset I/O)
cargo run -p reelsynth-app --bin reelsynth-ui

# Gate 2 proto (widget feel, no audio required)
cargo run -p reelsynth-ui --bin reelsynth-ui-proto
```

Capture screenshot (user or browser MCP), then follow workflow A. Open mockup at `file://…/brand/mockups/s1-performance.html` for side-by-side.

### C. Gate reviews

| Gate | What to audit | Pass criteria |
|------|---------------|---------------|
| **Gate 1** | Static HTML mockups | Matches DECISIONS + COMPONENT_SPEC; user sign-off |
| **Gate 2** | Proto binary | Knob drag, piano keys, disabled groups feel correct |
| **S1 parity** | App vs `s1-performance.html` | Layout ≤4px tolerance; only S1 panels visible |

## Checklist (summary)

Full checklist: [reference.md](reference.md).

- **Layout:** 8px grid; header 48px; footer 36px; osc 280px / rail 240px (S6); S1 center+rail only
- **Colours:** `#0a0a0a` canvas, `#18181b` panels, `#183d50` accent, `#2a6b8a` interactive highlights
- **Typography:** IBM Plex headings, Inter body, JetBrains Mono values
- **Knobs:** 48/56/64px; 270° arc; wired glow + "Live" badge on live params
- **Piano:** 72px tall; 15px white keys; 14 keys (2 octaves); toggle in footer
- **Disabled:** ADSR/LFO greyed in S1 (`rs-group--disabled` / `panel_disabled`)
- **WT strip:** 72px; playhead; frame 108/255 default data

## Report template

```markdown
# ReelSynth UI Audit — [S1 | S6 | Components | Proto]

**Target:** [screenshot description / cargo run …]
**Reference:** [mockup file]
**Viewport:** [WxH if known]

## Summary
[1–2 sentences: pass/fail vs gate; top issues]

## Findings

| # | Severity | Region | Issue | Expected | Fix hint |
|---|----------|--------|-------|----------|----------|
| 1 | Critical | … | … | … | … |

## Region scorecard

| Region | Status | Notes |
|--------|--------|-------|
| Header | ✅ / ⚠️ / ❌ | … |
| Center hero | … | … |
| WT strip | … | … |
| Right rail | … | … |
| Footer / piano | … | … |

## Gate verdict
- [ ] Gate 1 mockup parity
- [ ] Gate 2 proto feel
- [ ] S1 app parity (≤4px)

## Recommended next steps
1. …
```

## Fix routing

**Layout or spacing change** → update mockup HTML/CSS first, get approval, then egui.

| Area | Rust paths |
|------|------------|
| Grid constants | `ui/src/layout.rs` |
| S1 shell | `ui/src/s1.rs` |
| Knobs | `ui/src/widgets/knob.rs` |
| Piano | `ui/src/widgets/piano.rs` |
| Panels / disabled chrome | `ui/src/widgets/panel.rs` |
| WT strip | `ui/src/wt/strip.rs` |
| Tabs | `ui/src/widgets/tabs.rs` |
| Theme + fonts | `ui-theme/src/lib.rs` |
| App entry + theme apply | `app/src/main.rs`, `ui/src/bin/proto.rs` |

### egui pitfalls (always check on visual bugs)

See [reference.md](reference.md) § egui pitfalls. Common: `reelsynth_ui_theme::apply(ctx)` not called in `eframe` creation callback; heading font family not bound; `ui.add_enabled(false, …)` missing on disabled groups.

## Optional canvas

For multi-screenshot regression reviews, a Cursor Canvas side-by-side layout is acceptable. Do **not** use canvas for a single quick audit — the markdown report is the deliverable.

## Do not

- Ship layout fixes without mockup update first
- Treat dimmed unshipped panels as acceptable (must hide)
- Use `/flow/*` Majico routes (retired)
- Commit screenshot PNGs to the repo

## Additional resources

- Full checklists, severity rubric, region map, egui pitfalls: [reference.md](reference.md)
- Existing skills survey (what this skill fills): [reference.md](reference.md) § Related skills
