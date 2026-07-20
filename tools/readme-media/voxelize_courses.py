#!/usr/bin/env python3
"""Voxelize every prepared course GLB into a .schem.

    python3 tools/readme-media/prep_course.py          # fetch + prepare GLBs
    cargo build --release --example voxelize_course --features voxelize
    python3 tools/readme-media/voxelize_courses.py     # -> .../schematics/

Writes target/readme-models/schematics/<slug>.schem plus a summary TSV. Pass
slugs to do a subset. TARGET_SIZE fits each course's largest axis to that many
blocks, so every course comes out comparably sized regardless of the units its
ripper used.
"""

import json
import os
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
GLB_DIR = os.path.join(ROOT, "target", "readme-models", "courses")
OUT_DIR = os.path.join(ROOT, "target", "readme-models", "schematics")
BIN = os.path.join(ROOT, "target", "release", "examples", "voxelize_course")
MANIFEST = os.path.join(os.path.dirname(os.path.abspath(__file__)), "courses.json")

TARGET_SIZE = "400"


def main():
    courses = json.load(open(MANIFEST))
    wanted = sys.argv[1:]
    if wanted:
        courses = [c for c in courses if c["slug"] in wanted]
    os.makedirs(OUT_DIR, exist_ok=True)
    if not os.path.exists(BIN):
        sys.exit(f"missing {BIN}\n"
                 "cargo build --release --example voxelize_course --features voxelize")

    rows, failed = [], []
    for c in courses:
        glb = os.path.join(GLB_DIR, c["slug"] + ".glb")
        if not os.path.exists(glb):
            failed.append((c["slug"], "no GLB (prep failed?)"))
            continue
        out = os.path.join(OUT_DIR, c["slug"] + ".schem")
        name = c["slug"].replace("-", "_")
        p = subprocess.run([BIN, glb, out, name, TARGET_SIZE],
                           capture_output=True, text=True)
        line = next((l for l in p.stdout.splitlines() if l.startswith("RESULT")), None)
        if p.returncode != 0 or not line:
            failed.append((c["slug"], (p.stderr or p.stdout).strip().splitlines()[-1:] or "?"))
            continue
        _, nm, kept, dropped, extent, comps, size, rt = line.split("\t")
        rows.append(dict(slug=c["slug"], title=c["title"], game=c["game"],
                         blocks=int(kept), dropped=int(dropped), extent=extent,
                         components=comps, bytes=int(size), roundtrip=rt))
        print(f"OK   {c['slug']:<26} {int(kept):>9,} blocks  {extent:<14} "
              f"comps {comps:<7} {rt}", flush=True)

    for slug, why in failed:
        print(f"FAIL {slug:<26} {why}", flush=True)

    tsv = os.path.join(OUT_DIR, "summary.tsv")
    with open(tsv, "w") as f:
        cols = ["slug", "title", "game", "blocks", "dropped", "extent",
                "components", "bytes", "roundtrip"]
        f.write("\t".join(cols) + "\n")
        for r in sorted(rows, key=lambda r: -r["blocks"]):
            f.write("\t".join(str(r[c]) for c in cols) + "\n")
    print(f"\n{len(rows)} ok, {len(failed)} failed -> {tsv}")


if __name__ == "__main__":
    main()
