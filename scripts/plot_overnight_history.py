#!/usr/bin/env python3
"""Publication plots from overnight GPU RL history.jsonl (twocolumn-safe).

Sized for arXiv twocolumn: ~3.3in column width at 220 dpi, large labels,
minimal in-plot annotation. DualCosine baseline when available.
Copies to denoise-opt-meta/paper/v4/figures/ and docs/papers/denoise_opt/v4/figures/.
"""
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
META_FIG = Path(
    r"C:\Users\Julian\Documents\Programming\github\reeldemo\denoise-opt-meta\paper\v4\figures"
)
DOCS_FIG = ROOT / "docs" / "papers" / "denoise_opt" / "v4" / "figures"

# Physical inches: readable when shrunk to \\columnwidth (~3.3in)
COL_W, COL_H = 5.6, 3.5
PANEL_H = 6.2
FONT = {
    "axes.titlesize": 11,
    "axes.labelsize": 11,
    "xtick.labelsize": 10,
    "ytick.labelsize": 10,
    "legend.fontsize": 9,
    "figure.titlesize": 11,
}

BRANCH_COLORS = {
    "rl": "#2a9d8f",
    "ppo": "#2a9d8f",
    "nas": "#c9a227",
    "combo": "#e76f51",
    "ga": "#457b9d",
    "pbt": "#6a4c93",
}


def load_rows(path: Path) -> list[dict]:
    rows: list[dict] = []
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def load_baseline(run_dir: Path, rows: list[dict]) -> float | None:
    meta = run_dir / "run_meta.json"
    if meta.is_file():
        try:
            d = json.loads(meta.read_text(encoding="utf-8"))
            for k in ("dual_cosine_baseline", "baseline_dual_cosine", "baseline"):
                if k in d and d[k] is not None:
                    return float(d[k])
        except Exception:
            pass
    latest = ROOT / "brand" / "artifacts" / "overnight_gpu_rl_arch_latest.json"
    if latest.is_file():
        try:
            d = json.loads(latest.read_text(encoding="utf-8"))
            if "baseline_dual_cosine" in d:
                return float(d["baseline_dual_cosine"])
        except Exception:
            pass
    _ = rows
    return None


def champ_events(rows: list[dict]) -> list[dict]:
    events: list[dict] = []
    best = -1.0
    for r in rows:
        c = r.get("champ")
        if c is None:
            continue
        c = float(c)
        if c > best + 1e-12:
            best = c
            events.append(
                {
                    "iter": int(r["iter"]),
                    "champ": c,
                    "branch": r.get("branch"),
                    "arch_id": r.get("arch_id") or r.get("tag"),
                }
            )
    return events


def series(rows: list[dict], *keys: str) -> list[float | None]:
    out: list[float | None] = []
    for r in rows:
        v = None
        for k in keys:
            if r.get(k) is not None:
                v = float(r[k])
                break
        out.append(v)
    return out


def style_axes(ax) -> None:
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.grid(True, alpha=0.25, linewidth=0.55)
    ax.tick_params(labelsize=10)


def pick_annotate(events: list[dict], max_n: int = 3) -> list[dict]:
    """Keep sparse labels: first, last, and one mid update spaced in iteration."""
    if not events:
        return []
    if len(events) <= max_n:
        return events
    first, last = events[0], events[-1]
    mid_candidates = [e for e in events[1:-1] if e["iter"] > first["iter"] + 30]
    mid = None
    if mid_candidates:
        # prefer a late mid update so labels do not pile on the left
        target = first["iter"] + 0.45 * (last["iter"] - first["iter"])
        mid = min(mid_candidates, key=lambda e: abs(e["iter"] - target))
    chosen = [first]
    if mid is not None:
        chosen.append(mid)
    chosen.append(last)
    return chosen[:max_n]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("history", type=Path, help="Path to history.jsonl")
    ap.add_argument("--out-dir", type=Path, default=None)
    ap.add_argument("--baseline", type=float, default=None)
    ap.add_argument("--dpi", type=int, default=220)
    ap.add_argument("--also-meta-v4", action="store_true", default=True)
    ap.add_argument("--also-docs-v4", action="store_true", default=True)
    args = ap.parse_args()

    try:
        import matplotlib

        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("ERROR: matplotlib required (pip install matplotlib)", flush=True)
        return 2

    plt.rcParams.update(FONT)

    rows = load_rows(args.history)
    if not rows:
        print("ERROR: no history rows", flush=True)
        return 1

    run_dir = args.history.parent
    baseline = args.baseline if args.baseline is not None else load_baseline(run_dir, rows)

    out_dir = args.out_dir
    if out_dir is None:
        out_dir = ROOT / "brand" / "artifacts" / "figures" / run_dir.name
    out_dir.mkdir(parents=True, exist_ok=True)

    iters = [int(r["iter"]) for r in rows]
    champ = [r.get("champ") for r in rows]
    resid = [r.get("residual") for r in rows]
    branches = [(r.get("branch") or "").lower() for r in rows]
    bb_rl = series(rows, "branch_best_rl", "branch_best_ppo")
    bb_nas = series(rows, "branch_best_nas")
    bb_combo = series(rows, "branch_best_combo")
    bb_ga = series(rows, "branch_best_ga")
    bb_pbt = series(rows, "branch_best_pbt")
    events = champ_events(rows)

    final_champ = float(champ[-1]) if champ[-1] is not None else float("nan")
    n = len(rows)

    # --- Fig 1: champion + baseline ---
    fig, ax = plt.subplots(figsize=(COL_W, COL_H))
    ax.plot(iters, champ, color="#1a5f7a", linewidth=1.8, label="Champion $R$")
    if baseline is not None:
        ax.axhline(
            baseline,
            color="#c45c26",
            linestyle="--",
            linewidth=1.4,
            label=f"DualCosine ({baseline:.3f})",
        )
    if events:
        ax.scatter(
            [e["iter"] for e in events],
            [e["champ"] for e in events],
            s=28,
            c="#0b3d4a",
            zorder=5,
            label="Updates",
        )
    ax.set_xlabel("Iteration")
    ax.set_ylabel(r"Residual $R$ (1 = best)")
    ax.set_title("Champion residual vs DualCosine")
    style_axes(ax)
    ax.legend(loc="lower right", frameon=False, fontsize=9)
    fig.tight_layout()
    p1 = out_dir / "champ_residual_vs_iter.png"
    fig.savefig(p1, dpi=args.dpi, bbox_inches="tight")
    plt.close(fig)

    # --- Fig 2: branch bests ---
    fig, ax = plt.subplots(figsize=(COL_W, COL_H))
    if any(v is not None for v in bb_rl):
        ax.plot(iters, bb_rl, color=BRANCH_COLORS["ppo"], linewidth=1.5, label="PPO/RL best")
    if any(v is not None for v in bb_nas):
        ax.plot(iters, bb_nas, color=BRANCH_COLORS["nas"], linewidth=1.5, label="NAS best")
    if any(v is not None for v in bb_ga):
        ax.plot(iters, bb_ga, color=BRANCH_COLORS["ga"], linewidth=1.5, label="GA best")
    if any(v is not None for v in bb_pbt):
        ax.plot(iters, bb_pbt, color=BRANCH_COLORS["pbt"], linewidth=1.5, label="PBT best")
    if any(v is not None for v in bb_combo):
        ax.plot(iters, bb_combo, color=BRANCH_COLORS["combo"], linewidth=1.5, label="Combo best")
    if baseline is not None:
        ax.axhline(baseline, color="#6c757d", linestyle="--", linewidth=1.2, label="DualCosine")
    ax.set_xlabel("Iteration")
    ax.set_ylabel(r"Branch-best $R$")
    ax.set_title("Branch competition")
    style_axes(ax)
    ax.legend(loc="lower right", frameon=False, fontsize=8, ncol=2)
    fig.tight_layout()
    p2 = out_dir / "branch_bests_vs_iter.png"
    fig.savefig(p2, dpi=args.dpi, bbox_inches="tight")
    plt.close(fig)

    # --- Fig 3: per-iter residual by branch ---
    fig, ax = plt.subplots(figsize=(COL_W, COL_H))
    step = max(1, n // 12000)
    present = sorted({b for b in branches if b})
    for bname in present:
        col = BRANCH_COLORS.get(bname, "#888888")
        xs = [iters[i] for i in range(0, n, step) if branches[i] == bname]
        ys = [resid[i] for i in range(0, n, step) if branches[i] == bname]
        if xs:
            ax.scatter(xs, ys, s=6, alpha=0.28, c=col, linewidths=0, label=bname.upper())
    ax.plot(iters, champ, color="#1a5f7a", linewidth=1.5, label="Champ")
    if baseline is not None:
        ax.axhline(baseline, color="#6c757d", linestyle="--", linewidth=1.1)
    ax.set_xlabel("Iteration")
    ax.set_ylabel(r"Per-trial $R$")
    ax.set_title("Trial residuals by branch")
    style_axes(ax)
    ax.legend(loc="lower right", frameon=False, fontsize=8, markerscale=2.2, ncol=2)
    fig.tight_layout()
    p3 = out_dir / "residual_by_branch.png"
    fig.savefig(p3, dpi=args.dpi, bbox_inches="tight")
    plt.close(fig)

    # --- Fig 4: champion timeline (sparse labels) ---
    fig, ax = plt.subplots(figsize=(COL_W, COL_H * 0.95))
    if events:
        xs = [e["iter"] for e in events] + [iters[-1]]
        ys = [e["champ"] for e in events] + [events[-1]["champ"]]
        ax.step(xs, ys, where="post", color="#1a5f7a", linewidth=1.8)
        labels = pick_annotate(events, max_n=3)
        for e in labels:
            ax.axvline(e["iter"], color="#adb5bd", linewidth=0.7, alpha=0.75)
        # Place last label below the plateau so it is not clipped at the top
        offsets = [(8, 8), (8, -16), (-36, -14)]
        for e, off in zip(labels, offsets):
            ax.annotate(
                f"{e['champ']:.3f}",
                (e["iter"], e["champ"]),
                textcoords="offset points",
                xytext=off,
                fontsize=10,
                color="#0b3d4a",
                clip_on=False,
            )
        y_vals = [e["champ"] for e in events]
        if baseline is not None:
            y_vals.append(baseline)
        ymin, ymax = min(y_vals), max(y_vals)
        pad = max(0.02, 0.08 * (ymax - ymin + 1e-6))
        ax.set_ylim(ymin - pad, min(1.002, ymax + pad))
    else:
        ax.plot(iters, champ, color="#1a5f7a", linewidth=1.6)
    if baseline is not None:
        ax.axhline(baseline, color="#c45c26", linestyle="--", linewidth=1.2)
    ax.set_xlabel("Iteration")
    ax.set_ylabel(r"Champion $R$")
    ax.set_title(f"Champion updates ($n$={len(events)})")
    style_axes(ax)
    fig.tight_layout()
    p4 = out_dir / "champion_timeline.png"
    fig.savefig(p4, dpi=args.dpi, bbox_inches="tight", pad_inches=0.2)
    plt.close(fig)

    # --- Fig 5: full-width panel ---
    fig, axes = plt.subplots(2, 1, figsize=(COL_W * 1.55, PANEL_H), sharex=True)
    axes[0].plot(iters, champ, color="#1a5f7a", linewidth=1.7, label="Champion $R$")
    if baseline is not None:
        axes[0].axhline(
            baseline,
            color="#c45c26",
            linestyle="--",
            linewidth=1.3,
            label=f"DualCosine ({baseline:.3f})",
        )
    axes[0].set_ylabel(r"Champion $R$")
    axes[0].set_title(f"Overnight monitoring ({n:,} steps, champ $R$={final_champ:.3f})")
    style_axes(axes[0])
    axes[0].legend(loc="lower right", frameon=False, fontsize=9)

    if any(v is not None for v in bb_rl):
        axes[1].plot(iters, bb_rl, color=BRANCH_COLORS["ppo"], linewidth=1.4, label="PPO/RL")
    if any(v is not None for v in bb_nas):
        axes[1].plot(iters, bb_nas, color=BRANCH_COLORS["nas"], linewidth=1.4, label="NAS")
    if any(v is not None for v in bb_ga):
        axes[1].plot(iters, bb_ga, color=BRANCH_COLORS["ga"], linewidth=1.4, label="GA")
    if any(v is not None for v in bb_pbt):
        axes[1].plot(iters, bb_pbt, color=BRANCH_COLORS["pbt"], linewidth=1.4, label="PBT")
    if any(v is not None for v in bb_combo):
        axes[1].plot(iters, bb_combo, color=BRANCH_COLORS["combo"], linewidth=1.4, label="Combo")
    if baseline is not None:
        axes[1].axhline(baseline, color="#6c757d", linestyle="--", linewidth=1.1)
    axes[1].set_xlabel("Iteration")
    axes[1].set_ylabel(r"Branch-best $R$")
    style_axes(axes[1])
    axes[1].legend(loc="lower right", frameon=False, fontsize=8, ncol=3)
    fig.tight_layout()
    p5 = out_dir / "overnight_panel.png"
    fig.savefig(p5, dpi=args.dpi, bbox_inches="tight")
    plt.close(fig)

    summary = {
        "run_dir": str(run_dir),
        "n_points": n,
        "final_iter": iters[-1],
        "final_champ": final_champ,
        "baseline_dual_cosine": baseline,
        "delta_vs_baseline": (final_champ - baseline) if baseline is not None else None,
        "n_champ_updates": len(events),
        "champ_events_tail": events[-10:],
        "figures": [p.name for p in (p1, p2, p3, p4, p5)],
    }
    (out_dir / "plot_summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")

    written = [p1, p2, p3, p4, p5, out_dir / "plot_summary.json"]
    if args.also_meta_v4:
        META_FIG.mkdir(parents=True, exist_ok=True)
        for p in written:
            shutil.copy2(p, META_FIG / p.name)
        print(f"also copied to {META_FIG}", flush=True)
    if args.also_docs_v4:
        DOCS_FIG.mkdir(parents=True, exist_ok=True)
        for p in written:
            shutil.copy2(p, DOCS_FIG / p.name)
        print(f"also copied to {DOCS_FIG}", flush=True)

    print(f"wrote {out_dir} ({n} points, champ={final_champ:.6f}, baseline={baseline})", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
