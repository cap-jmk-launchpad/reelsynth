#!/usr/bin/env python3
"""Download raw data for the signal-heal transfer pilot.

Fetches a small CWRU subset (Zenodo mirror), MIT-BIH record subset (PhysioNet),
and attempts MFPT if a real zip is reachable.
"""
from __future__ import annotations

import ssl
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / "brand" / "artifacts" / "signal_heal_transfer" / "raw"
CTX = ssl.create_default_context()

CWRU = {
    "cwru/97.mat": ["https://zenodo.org/records/10986655/files/97.mat?download=1"],
    "cwru/98.mat": ["https://zenodo.org/records/10986655/files/98.mat?download=1"],
    "cwru/105.mat": ["https://zenodo.org/records/10986655/files/105.mat?download=1"],
    "cwru/169.mat": ["https://zenodo.org/records/10986655/files/169.mat?download=1"],
    "cwru/209.mat": ["https://zenodo.org/records/10986655/files/209.mat?download=1"],
}

MFPT = {
    "mfpt/MFPT-Fault-Data-Sets.zip": [
        # Official page often returns HTML; keep attempt for future mirrors.
        "https://www.mfpt.org/wp-content/uploads/2020/02/MFPT-Fault-Data-Sets.zip",
    ],
}

MITDB_RECS = ["100", "101", "103", "105", "112", "113", "115", "117", "121", "123"]


def try_get(urls: list[str], dest: Path, min_bytes: int = 1000) -> bool:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.is_file() and dest.stat().st_size >= min_bytes:
        # Reject HTML masquerading as zip
        head = dest.read_bytes()[:32]
        if dest.suffix.lower() == ".zip" and not head.startswith(b"PK"):
            dest.unlink(missing_ok=True)
        else:
            print(f"exists {dest} ({dest.stat().st_size})")
            return True
    for u in urls:
        for unverified in (False, True):
            try:
                print(f"GET {u} unverified={unverified}")
                ctx = ssl._create_unverified_context() if unverified else CTX
                req = urllib.request.Request(u, headers={"User-Agent": "reelsynth-signal-heal/0.1"})
                with urllib.request.urlopen(req, context=ctx, timeout=120) as r:
                    data = r.read()
                if len(data) < min_bytes:
                    print(f"  too small {len(data)} (need {min_bytes})")
                    continue
                if dest.suffix.lower() == ".zip" and not data.startswith(b"PK"):
                    print("  not a zip (HTML/login wall?) — skip")
                    continue
                dest.write_bytes(data)
                print(f"OK {dest} ({len(data)})")
                return True
            except Exception as e:
                print(f"  fail {type(e).__name__}: {e}")
    return False


def dl_mitdb() -> bool:
    dest = RAW / "mitdb"
    dest.mkdir(parents=True, exist_ok=True)
    try:
        import wfdb
    except ImportError:
        wfdb = None
    ok = True
    for rec in MITDB_RECS:
        base = f"https://physionet.org/files/mitdb/1.0.0/{rec}"
        for ext, amin in ((".dat", 1000), (".hea", 40), (".atr", 40)):
            path = dest / f"{rec}{ext}"
            if path.is_file() and path.stat().st_size >= amin:
                continue
            if not try_get([base + ext], path, min_bytes=amin):
                if wfdb is not None:
                    try:
                        wfdb.dl_files("mitdb", str(dest), [f"{rec}{ext}"])
                    except Exception as e:
                        print(f"wfdb fail {rec}{ext}: {e}")
                        ok = False
                else:
                    ok = False
    return ok


def main() -> int:
    RAW.mkdir(parents=True, exist_ok=True)
    cwru_ok = all(try_get(urls, RAW / rel) for rel, urls in CWRU.items())
    print("CWRU ok=", cwru_ok)
    mfpt_ok = all(try_get(urls, RAW / rel, min_bytes=10_000) for rel, urls in MFPT.items())
    print("MFPT ok=", mfpt_ok, "(optional; HTML walls are skipped)")
    mit_ok = dl_mitdb()
    print("MITDB ok=", mit_ok)
    return 0 if (cwru_ok and mit_ok) else 1


if __name__ == "__main__":
    raise SystemExit(main())
