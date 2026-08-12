#!/usr/bin/env python
"""Render every corpus artifact to PNG with the textured renderer.

Same pipeline as `redstone-eda/docs/render_gallery.py` -- nucleation's mesher
plus `pack.zip` at the repo root -- pointed at `artifacts/*.schem` and driven by
each scenario's own `render` block, so a picture is always a direct render of
the .schem the solver produced and the harness verified.

    python render.py            # every artifact with a result
    python render.py X02 ...     # prefix match
    python render.py --one ID    # in-process (used by the runner)

An entry with no artifact (the router refused, so there is nothing to show)
is skipped and the gallery says so instead.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
PACK = os.path.join(ROOT, "pack.zip")
RESULTS = os.path.join(HERE, "results")
OUT = os.path.join(HERE, "renders")

MAX_PX = 1500
BG = (0.055, 0.065, 0.11, 1.0)      # the gallery's near-black navy


def targets():
    out = {}
    for f in sorted(os.listdir(RESULTS)):
        if not f.endswith(".json"):
            continue
        with open(os.path.join(RESULTS, f)) as fh:
            r = json.load(fh)
        if r.get("artifact"):
            out[r["id"]] = r
    return out


def render_one(sid):
    import nucleation as nu

    with open(os.path.join(RESULTS, sid + ".json")) as fh:
        r = json.load(fh)
    kw = dict((r.get("scenario") or {}).get("render") or {})
    w = min(int(kw.pop("w", 1200)), MAX_PX)
    h = min(int(kw.pop("h", 820)), MAX_PX)

    schem = nu.Schematic.open(os.path.join(HERE, r["artifact"]))
    with open(PACK, "rb") as fh:
        pack = nu.ResourcePack.from_bytes(fh.read())

    cfg = nu.RenderConfig.create(w, h)
    cfg.set_isometric()
    cfg.set_yaw(float(kw.get("yaw", 145)))
    cfg.set_pitch(float(kw.get("pitch", 28)))
    cfg.set_sphere_fit(True)
    cfg.set_zoom(float(kw.get("zoom", 1.6)))
    cfg.set_background(*BG)

    os.makedirs(OUT, exist_ok=True)
    dest = os.path.join(OUT, sid + ".png")
    nu.Renderer.render_to_file_with_pack(schem, pack, cfg, dest)
    print("OK   %-26s %dx%d -> %d KB" % (sid, w, h,
                                         os.path.getsize(dest) // 1024))
    return 0


def main(argv):
    if argv[:1] == ["--one"]:
        return render_one(argv[1])
    if not os.path.exists(PACK):
        print("missing %s -- the textured renderer needs the resource pack"
              % PACK)
        return 1
    have = targets()
    picks = [a for a in argv if not a.startswith("-")]
    names = [s for s in have if not picks or any(s.startswith(p)
                                                 for p in picks)]
    ok, bad = [], []
    for sid in names:
        # one process each: a piece that OOMs the renderer must not take the
        # rest of the gallery down with it
        p = subprocess.run([sys.executable, __file__, "--one", sid],
                           capture_output=True, text=True, cwd=HERE)
        if p.returncode == 0:
            print(p.stdout.rstrip())
            ok.append(sid)
        else:
            tail = (p.stderr.strip().splitlines() or ["<no stderr>"])[-1]
            print("FAIL %-26s %s" % (sid, tail[:140]))
            bad.append(sid)
    print("\n%d rendered, %d failed, %d entries had no artifact to render"
          % (len(ok), len(bad),
             len([f for f in os.listdir(RESULTS) if f.endswith(".json")])
             - len(have)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
