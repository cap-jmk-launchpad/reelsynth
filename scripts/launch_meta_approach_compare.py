#!/usr/bin/env python3
"""Launch meta-approach 5k bench with a single-instance lock (crash-safe resume).

Writes brand/artifacts/meta_approach_compare/bench.lock with PID.
Refuses to start a second copy while the lock holder is alive.
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "brand" / "artifacts" / "meta_approach_compare"
LOCK = OUT / "bench.lock"
PIDFILE = OUT / "bench.pid"
PY = ROOT / ".venv_gpu" / "Scripts" / "python.exe"
SCRIPT = ROOT / "scripts" / "bench_meta_approaches_5k.py"


def pid_alive(pid: int) -> bool:
    try:
        if os.name == "nt":
            import ctypes

            k = ctypes.windll.kernel32
            h = k.OpenProcess(0x1000, False, int(pid))
            if not h:
                return False
            k.CloseHandle(h)
            return True
        os.kill(pid, 0)
        return True
    except Exception:
        return False


def read_lock_pid() -> int | None:
    for path in (LOCK, PIDFILE):
        if not path.is_file():
            continue
        try:
            return int(path.read_text(encoding="utf-8").strip().splitlines()[0])
        except Exception:
            continue
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--iters", type=int, default=5000)
    ap.add_argument("--ckpt-every", type=int, default=25)
    ap.add_argument("--force", action="store_true", help="Ignore existing live lock")
    ap.add_argument(
        "--fresh",
        action="store_true",
        help="Wipe approach dirs + pass --no-resume (hard restart)",
    )
    args = ap.parse_args()

    OUT.mkdir(parents=True, exist_ok=True)
    existing = read_lock_pid()
    if existing and pid_alive(existing) and not args.force:
        print(f"ALREADY_RUNNING pid={existing}")
        print(f"Poll: {PY if PY.exists() else 'python'} scripts/meta_approach_status.py")
        return 0

    if args.fresh:
        import shutil

        for name in (
            "random",
            "cmaes",
            "reinforce",
            "aging_evo",
            "tpe",
            "hybrid_lstm",
        ):
            d = OUT / name
            if d.is_dir():
                shutil.rmtree(d, ignore_errors=True)
        for p in (
            OUT / "STATUS.json",
            OUT / "meta_approach_compare.json",
            OUT / "fig_meta_approach_compare.png",
        ):
            if p.is_file():
                p.unlink(missing_ok=True)
        print(f"FRESH wipe under {OUT}", flush=True)

    py = str(PY if PY.exists() else sys.executable)
    log = OUT / "bench_stdout.log"
    err = OUT / "bench_stderr.log"
    cmd = [
        py,
        "-u",
        str(SCRIPT),
        "--iters",
        str(args.iters),
        "--ckpt-every",
        str(args.ckpt_every),
        "--out-dir",
        str(OUT),
    ]
    if args.fresh:
        cmd.append("--no-resume")
    # Detach on Windows
    creationflags = 0
    if os.name == "nt":
        creationflags = subprocess.CREATE_NEW_PROCESS_GROUP | subprocess.DETACHED_PROCESS  # type: ignore[attr-defined]

    with log.open("a", encoding="utf-8") as lo, err.open("a", encoding="utf-8") as er:
        lo.write(f"\n--- launch {time.strftime('%Y-%m-%dT%H:%M:%S')} ---\n")
        lo.flush()
        proc = subprocess.Popen(
            cmd,
            cwd=str(ROOT),
            stdout=lo,
            stderr=er,
            creationflags=creationflags,
            close_fds=True,
        )
    LOCK.write_text(str(proc.pid), encoding="utf-8")
    PIDFILE.write_text(str(proc.pid), encoding="utf-8")
    print(f"LAUNCHED pid={proc.pid}")
    print(f"log={log}")
    print(f"Poll: {py} scripts/meta_approach_status.py")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
