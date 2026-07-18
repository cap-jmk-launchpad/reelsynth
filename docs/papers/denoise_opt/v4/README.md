# DenoiseOpt paper v4 — LaTeX layout

## Template choice

**Double-column arXiv-style preprint** using:

```latex
\documentclass[10pt,twocolumn,letterpaper]{article}
\usepackage{arxiv-twocolumn}
```

Local style file: [`arxiv-twocolumn.sty`](arxiv-twocolumn.sty).

### Why this approach (2024–2026 CS/ML practice)

| Option | Notes | Decision |
|--------|--------|----------|
| [`kourgeorge/arxiv-style`](https://github.com/kourgeorge/arxiv-style) (`arxiv.sty`, MIT) | Very common for ML preprints; NeurIPS-derived look | **Single-column** by default (`textwidth=6.5in`) — not used as-is |
| [`myst-templates/arxiv_two_column`](https://github.com/myst-templates/arxiv_two_column) / [Brenhin Keller preprint](https://github.com/brenhinkeller/preprint-template.tex) | True two-column arXiv look | Pulls `background` / TikZ crop marks / `scalerel` ORCID — heavier than needed for arXiv upload |
| Conference styles (NeurIPS/ICML) | Often one-column for review | User wants **double column** |
| **`article` + `twocolumn` + lean local sty** | Standard packages only; pdflatex / MiKTeX / arXiv-safe | **Chosen** |

`arxiv-twocolumn.sty` keeps the familiar preprint title rules, compact sectioning, Times text (`times`/`mathptmx`), and `fancyhdr` headers, while relying on the built-in `twocolumn` class option and `geometry`. No exotic fonts or non-TeX-Live packages.

Title/abstract span both columns via the standard `\twocolumn[{...}]` idiom; body text is two-column. Single-column figures use `width=\columnwidth`; wide panels use `figure*` + `width=\textwidth`.

## Build (Windows / MiKTeX)

```powershell
cd paper\v4
pdflatex -interaction=nonstopmode main.tex
pdflatex -interaction=nonstopmode main.tex
```

Output: `main.pdf`.

## License note

Style inspiration from MIT-licensed community templates (kourgeorge/arxiv-style; myst arxiv_two_column lineage). The local `.sty` is a clean-room lean adaptation for double-column + arXiv package constraints.
