#!/usr/bin/env python3
"""End-to-end demo of the diff + fingerprint engines from Python.

Uses the published wheel's real surface (`pip install nucleation`):
static `Fingerprint.*` / `Diff.*` entry points taking schematics as
arguments — see docs/api-reference-python.md.

Run:
    python examples/diff_fingerprint.py [build_a] [build_b]

With one path, B is derived from A by adding a few blocks (so the diff is
non-trivial). With two paths, A and B are diffed directly.
"""
import json
import sys

from nucleation import Diff, Fingerprint, Schematic

# `shape` is orientation-blind but still an identity on content: one added
# block makes a build unique. `structural` is deliberately coarser — it sees
# only solid massing (glass and redstone are invisible to it), so it is NOT
# usable for deduplication. See docs/features/analysis.md for the preset table.
PRESET_FP = "shape"
PRESET_DIFF = "exact"  # material- + orientation-sensitive edit distance


def load_pair(argv):
    path_a = argv[1] if len(argv) > 1 else "tests/fixtures/4bit_adder.litematic"
    a = Schematic.load_from_file(path_a)
    if len(argv) > 2:
        return a, Schematic.load_from_file(argv[2]), argv[2]
    # Derive B from A: same build plus three glass blocks (real "added" cells).
    b = Schematic.load_from_file(path_a)
    for k in range(3):
        b.set_block(-2 - k, 0, 0, "minecraft:glass")
    return a, b, f"{path_a} + 3 glass"


def main():
    a, b, b_label = load_pair(sys.argv)
    print(f"A = {sys.argv[1] if len(sys.argv) > 1 else 'tests/fixtures/4bit_adder.litematic'}")
    print(f"B = {b_label}\n")

    # ── Fingerprint: canonical digest + fuzzy/dedup helpers ──
    print(f"fingerprint A ({PRESET_FP}): {Fingerprint.compute(a, PRESET_FP)}")
    print(f"fingerprint B ({PRESET_FP}): {Fingerprint.compute(b, PRESET_FP)}")
    print(f"is_duplicate              : {Fingerprint.is_duplicate(a, b, PRESET_FP)}")
    print(f"footprint_distance        : {Fingerprint.footprint_distance(b, a, PRESET_FP):.4f}")
    print(f"signature A               : {Fingerprint.signature_json(a, PRESET_FP)[:80]}…\n")

    # ── Diff: structural edit distance + projections ──
    d = Diff.compute(a, b, PRESET_DIFF)
    print(f"diff ({PRESET_DIFF}): distance={d.distance()}  support={d.support():.3f}")
    print(f"summary_json: {json.dumps(json.loads(d.summary_json()))[:200]}\n")

    # Lossless round-trip: to_json() reconstructs a full Diff.
    d2 = Diff.from_json(d.to_json())
    assert d2.distance() == d.distance(), "round-trip preserved distance"
    print(f"to_json round-trip OK (distance still {d2.distance()})")

    # Projections are schematics you can save / render.
    d.markers().save_to_file("diff_markers.schem")
    print("wrote diff_markers.schem (added=lime, removed=red, changed=yellow, swapped=blue)")


if __name__ == "__main__":
    main()
