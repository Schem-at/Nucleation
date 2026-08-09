#!/usr/bin/env python
"""Render the redstone-eda gallery artifacts to PNG with the textured renderer.

Every picture in `redstone-eda/README.md` is produced by this script, so the
docs cannot drift from the artifacts: each PNG is a direct render of the
tracked `.schem` that the generators sim-verified and baked.

Usage:
    python render_gallery.py            # render every target
    python render_gallery.py NAME ...   # render only these targets
    python render_gallery.py --list     # list target names

Needs the wheel built with rendering support and `pack.zip` at the repo root:
    NUCLEATION_FEATURES=bridge-full,routing,hdl ~/eda-venv/bin/pip install ./bindings/python
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]          # repo root
EDA = ROOT / "redstone-eda"
OUT = EDA / "docs" / "img"
PACK = ROOT / "pack.zip"

# name -> (schematic path, render kwargs)
# `zoom` tightens the sphere-fitted camera (default 1.6); sparse pieces whose
# bounding box is mostly empty space need more.
TARGETS: dict[str, tuple[Path, dict]] = {
    # --- cell-library / dense arithmetic ---
    "rca4_cells":        (EDA / "rca4_cells.schem",                    dict(yaw=135, pitch=28)),
    "adder4_cells":      (EDA / "showcase/adder4_cells.schem",         dict(yaw=135, pitch=28)),
    "ripple_carry_adder_4bit": (EDA / "ripple_carry_adder_4bit.schem", dict(yaw=135, pitch=30)),
    "kogge_stone_32bit": (EDA / "showcase/kogge_stone_32bit.schem",    dict(yaw=135, pitch=34, w=1600, h=1000)),
    "mult_4x4":          (EDA / "showcase/mult4x4_stacked.schem",      dict(yaw=135, pitch=32, w=1400, h=1000)),
    "alu8":              (EDA / "showcase/alu8.schem",                 dict(yaw=135, pitch=32, w=1400, h=950)),
    # --- sequential ---
    "counter4":          (EDA / "showcase/counter4.schem",             dict(yaw=135, pitch=30)),
    "accumulator4":      (EDA / "showcase/accumulator4.schem",         dict(yaw=135, pitch=30)),
    "dff":               (EDA / "showcase/dff.schem",                  dict(yaw=140, pitch=26, zoom=2.0)),
    "register4":         (EDA / "showcase/register4.schem",            dict(yaw=135, pitch=28, zoom=1.9)),
    # --- routing / bus fabric ---
    "router_gallery":    (EDA / "showcase/router_gallery.schem",       dict(yaw=135, pitch=30, zoom=1.9)),
    "bus8_run":          (EDA / "showcase/bus8_run.schem",             dict(yaw=135, pitch=26, zoom=1.75)),
    "bus_cross8_design": (EDA / "showcase/bus_cross8_design.schem",    dict(yaw=145, pitch=30, zoom=1.8)),
    "crossing_tiles":    (EDA / "crossing_tiles.schem",                dict(yaw=145, pitch=26, zoom=2.1)),
    "pivot_v2h":         (EDA / "showcase/pivot_v2h.schem",            dict(yaw=145, pitch=24, zoom=1.75)),
    "hexanalog_trunk":   (EDA / "showcase/hexanalog_trunk.schem",      dict(yaw=135, pitch=26, zoom=1.9)),
    # --- HDL compiler output ---
    "genlib_seg7":       (EDA / "showcase/genlib_seg7.schem",          dict(yaw=135, pitch=30)),
    "genlib_cmp4":       (EDA / "showcase/genlib_cmp4.schem",          dict(yaw=135, pitch=30)),
    # --- community corpus, contract-enhanced ---
    "ADD007_enhanced":   (ROOT / "computational_schematics/enhanced/ADD007_8bit_cca_matt_enhanced.schem",
                          dict(yaw=135, pitch=30, w=1400, h=950, zoom=1.75)),
}

MAX_PX = 1600
BG = (0.055, 0.065, 0.11, 1.0)     # near-black navy


def render_one(name: str) -> None:
    """Render a single target in this process (called via `--one`)."""
    import nucleation as nu

    path, kw = TARGETS[name]
    w = min(kw.pop("w", 1200), MAX_PX)
    h = min(kw.pop("h", 850), MAX_PX)

    schem = nu.Schematic.open(str(path))
    pack = nu.ResourcePack.from_bytes(PACK.read_bytes())

    cfg = nu.RenderConfig.create(w, h)
    cfg.set_isometric()
    if "yaw" in kw:
        cfg.set_yaw(float(kw["yaw"]))
    if "pitch" in kw:
        cfg.set_pitch(float(kw["pitch"]))
    cfg.set_sphere_fit(True)
    cfg.set_zoom(float(kw.get("zoom", 1.6)))
    # Solid dark ground, not transparent: alpha is not honoured uniformly across
    # sizes (a big piece came back opaque black while small ones were clear), and
    # one consistent backdrop reads on both GitHub themes.
    cfg.set_background(*BG)

    OUT.mkdir(parents=True, exist_ok=True)
    dest = OUT / f"{name}.png"
    nu.Renderer.render_to_file_with_pack(schem, pack, cfg, str(dest))
    print(f"OK {name} {w}x{h} -> {dest.relative_to(ROOT)} "
          f"({dest.stat().st_size // 1024} KB, {schem.block_count()} blocks)")


def main() -> int:
    argv = sys.argv[1:]
    if argv and argv[0] == "--list":
        print("\n".join(TARGETS))
        return 0
    if argv and argv[0] == "--one":
        render_one(argv[1])
        return 0

    names = argv or list(TARGETS)
    unknown = [n for n in names if n not in TARGETS]
    if unknown:
        print(f"unknown targets: {unknown}", file=sys.stderr)
        return 2

    # Each target renders in its own process: a piece that OOMs or crashes the
    # renderer must not take the rest of the gallery down with it.
    ok, failed = [], []
    for name in names:
        r = subprocess.run([sys.executable, __file__, "--one", name],
                           capture_output=True, text=True, cwd=str(ROOT))
        if r.returncode == 0:
            print(r.stdout.strip())
            ok.append(name)
        else:
            tail = (r.stderr.strip().splitlines() or ["<no stderr>"])[-1]
            print(f"FAIL {name}: {tail}")
            failed.append(name)

    print(f"\n{len(ok)} rendered, {len(failed)} failed")
    if failed:
        print("failed: " + ", ".join(failed))
    return 0


if __name__ == "__main__":
    sys.exit(main())
