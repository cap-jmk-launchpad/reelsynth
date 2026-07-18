# Literature grounding notes (v4)

Generated from OA arXiv PDFs under `paper/klaut_artifacts/hybrid-reinforcement-learning-and-genetic-algori-1e29c690/pdfs/` plus classical VA/BLEP anchors from prior versions.
Use only for claims supported by these sources; do not invent citations.

## Used (method / design anchors)

| cite_key | Paper | Relevance |
|----------|-------|-----------|
| lehtinen2018 | Noise2Noise | Label-free restoration philosophy for unsupervised residual scoring |
| engel2020 | DDSP | Modular/differentiable DSP context for seam bake ops |
| jaderberg2017 | PBT | Exploit–mutate population schedules |
| real2019 | Regularized / Aging Evolution | GA/evolutionary NAS prior |
| khadka2018 | ERL | Interleave GA population with RL policy updates |
| schulman2017 | PPO | Clipped surrogate for discrete arch mutations |
| pham2018 | ENAS | RL controller + sharing ancestor |
| elsken2019 | NAS survey | Search space / strategy / evaluation taxonomy |
| shazeer2017 | MoE | Soft gates over heterogeneous blocks |
| stoller2018 | Wave-U-Net | Tiny U-Net cell prior |
| luo2019 | Conv-TasNet | Time-domain / TCN-inspired cells |
| jansson2017 | Singing U-Net | Audio U-Net lineage |
| purwins2019 | Deep Learning for Audio | Domain survey |
| snoek2012 | Practical BO | Local Bayesian prior family |
| zhang2007 | MOEA/D | Multi-objective cues |
| valimaki2006va, esqueda2016* | VA / aliasing / BLAMP | Classical discontinuity control |

## Screened (contrast only)

| cite_key | Why screened |
|----------|----------------|
| finn2017 | MAML — no task-gradient through a deep net |
| liu2019 | DARTS — continuous relaxation; we keep discrete ops |
| dfossez2019 / dfossez2021 | Demucs / Hybrid Demucs — too heavy per trial |
| kong2021 | DiffWave — generative sampling stack, not seam bake |
| pascual2017 | SEGAN — full GAN loops screened for cost |

## Klaut workflow
- Paper id: `hybrid-reinforcement-learning-and-genetic-algori-1e29c690`
- Attach → fetch PDFs → analyze → plan → LLM draft → **author revision** (claim hygiene)
- Ollama model: `qwen3.5:9b`
