# DenoiseOpt paper (arXiv-style)

Sources: `main.tex` (article class, arXiv-friendly). Figures under `figures/`.

## Build

```bash
cd docs/papers/denoise_opt
pdflatex main.tex
```

Or upload `main.tex` + `figures/` to Overleaf / arXiv.

## Regenerating figures

```bash
cargo run -p reelsynth --release --bin bench_denoise_meta
python brand/artifacts/render_paper_figures.py
copy brand\artifacts\fig_*.png docs\papers\denoise_opt\figures\
```

## Literature

Harvested with **Klaut Research MCP** (local mode → OpenAlex/Crossref/arXiv):

`brand/artifacts/literature_klaut_research.json`

Cursor MCP install: `.cursor/mcp.json` (`klaut-research`). Reload MCP after clone.
Hosted `https://research.klaut.pro` was misrouted during this run; local in-process mode works.
