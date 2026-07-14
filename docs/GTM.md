# ReelSynth GTM plan (v0.1)

This is a lightweight go-to-market plan for **ReelSynth** as an OSS instrument, with a plugin waitlist as the primary conversion event.

## Executive summary

- **Product:** ReelSynth — MIT wavetable synth engine + standalone UI (plugin not shipped yet).
- **Primary CTA:** Join the **ReelSynth plugin waitlist** (CLAP/VST3/AU target, S7 roadmap).
- **Audience (hero):** Bedroom / hobby producers who want a **free** wavetable synth workflow.
- **Motion:** Balanced — stars + waitlist + dev integrators + Reeldemo Studio cross-promo.
- **Channels:** GitHub + HN + Reddit (initial wave).

## What we ship (truthful scope)

**Works today**

- Standalone app: MIDI input, sound design, preset + wavetable save
- Export pipeline: Vital, WAV frames, Serum WT subset, Ableton JSON map, SFZ, reelpack
- Python + CLI offline rendering

**Not shipped yet (must be explicit)**

- DAW plugin: CLAP/VST3/AU host bindings (S7)
- In-app MIDI recording (record MIDI in your DAW)

Use the capability matrix in [README.md](../README.md) and the honest limits in [docs/WORKFLOW.md](WORKFLOW.md).

## Goals (90 days)

Balanced goals, ranked for decision-making:

1. **Plugin waitlist growth** (validates S7 priority)
2. **OSS credibility** (GitHub stars, contributors, presets)
3. **Developer integrators** (PyO3/Rust usage in other tools)
4. **Reeldemo Studio** discovery (soft CTA; not required)

## ICP and messaging

### Primary ICP: bedroom producers

**Message:** “Free wavetable instrument you can run today. Export to Vital. Record MIDI in your DAW.”

Proof points:
- MIT licensed
- Imports Vital/Serum/WAV cycles
- Exports `reelpack/` and Vital `.vitaltable`
- Docs: [GETTING_STARTED.md](GETTING_STARTED.md), [FREE_STACK.md](FREE_STACK.md)

### Secondary ICP: synth nerds / sound designers

**Message:** “Inspectable DSP + full mod routing + interop matrix; exports don’t lie.”

Proof points:
- `export_report.json` with dropped parameters
- [INTEROP.md](INTEROP.md), [FORMAT.md](FORMAT.md)

### Developer ICP: Rust/audio devs

**Message:** “Embed a real wavetable synth as a Rust crate or PyO3 module.”

Proof points:
- [SDK.md](SDK.md) (full public Rust API)

## Funnel and conversion points

### Primary conversion: plugin waitlist signup

Waitlist lives on **reeldemo.github.io/reelsynth** and writes to **Supabase shared with Studio**, tagged as `product='reelsynth_plugin'`.

### Secondary conversions

- GitHub star
- Discord join (optional; only if you want to moderate)
- Reeldemo Studio “Used in Studio” click-through

## Launch checklist

### Minimum launch assets

- [ ] GitHub Release `v0.1.0` exists (source + notes)
- [ ] `docs-images.zip` uploaded to that release so screenshots resolve
- [ ] Landing page has waitlist CTA + form
- [ ] One short demo (GIF or 30–60s screen recording): “design sound → export → Vital”

### Nice-to-have

- Preset pack (`.reelpreset` + `.reelwt`) published as a `reelpack/` example
- “FAQ / honest limits” section on landing

## Channel plan (first wave)

### Wave 1 (Day 0–2): GitHub + HN

- GitHub release notes: “What it is”, “What works today”, “What’s next (S7 plugin)”
- HN: Show HN post focusing on “open wavetable instrument in Rust + offline export”

### Wave 2 (Day 3–7): Reddit

- r/edmproduction: “free wavetable synth workflow + Vital export”
- r/audioengineering: interop + offline rendering + export reports
- r/rust: PyO3 + DSP engine, request contributors

## Metrics dashboard (full)

Track weekly:

- GitHub: stars, clones, top referrers
- Waitlist: signups/day, DAW breakdown, source/utm
- Docs: pageviews (landing + docs index)
- Reeldemo Studio: click-throughs + trial signups (secondary)
- Community: Discord joins (if enabled)

## Risks and mitigations

1. **“Where is the plugin?”** → make waitlist explicit, include roadmap and S7 status.
2. **Producers won’t compile Rust** → publish binaries when CI is unblocked; until then, document “build from source” clearly.
3. **Interop disappointment** → point to [INTEROP.md](INTEROP.md) and `export_report.json`.

