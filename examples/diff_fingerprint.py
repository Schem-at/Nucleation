#!/usr/bin/env python3
"""End-to-end demo of the diff + fingerprint engines from Python.

Build the extension first:
    maturin develop --features python,meshing

Run:
    python examples/diff_fingerprint.py [build_a] [build_b]

With one path, B is derived from A by adding a few blocks (so the diff is
non-trivial). With two paths, A and B are diffed directly.

The glowing-overlay GLB step only runs if PACK points at a resource pack zip:
    PACK=render_work/pack.zip python examples/diff_fingerprint.py
"""
import os
import sys

from nucleation import Schematic

PRESET_FP = "structural"  # rotation/material-agnostic structural equivalence
PRESET_DIFF = "exact"     # material- + orientation-sensitive edit distance


def load_pair(argv):
    path_a = argv[1] if len(argv) > 1 else "4bit_adder.litematic"
    a = Schematic.open(path_a)
    if len(argv) > 2:
        return a, Schematic.open(argv[2]), argv[2]
    # Derive B from A: same build plus three glass blocks (real "added" cells).
    b = Schematic.open(path_a)
    for k in range(3):
        b.set_block(-2 - k, 0, 0, "minecraft:glass")
    return a, b, f"{path_a} + 3 glass"


def main():
    a, b, b_label = load_pair(sys.argv)
    print(f"A = {sys.argv[1] if len(sys.argv) > 1 else '4bit_adder.litematic'}")
    print(f"B = {b_label}\n")

    # ── Fingerprint: canonical hash + fuzzy/dedup helpers ──
    fa = a.fingerprint(PRESET_FP)
    fb = b.fingerprint(PRESET_FP)
    print(f"fingerprint A ({PRESET_FP}): {fa}")
    print(f"fingerprint B ({PRESET_FP}): {fb}")
    print(f"is_duplicate_of           : {a.is_duplicate_of(b, PRESET_FP)}")
    print(f"footprint_distance        : {a.footprint_distance(b, PRESET_FP):.4f}")
    print(f"signature A               : {a.signature(PRESET_FP)[:80]}…\n")

    # ── Diff: structural edit distance + projections ──
    d = a.diff(b, PRESET_DIFF)
    print(f"diff ({PRESET_DIFF}): distance={d.distance}  support={d.support:.3f}")
    summary = d.summary_json()
    print(f"summary_json: {summary}\n")

    # Lossless round-trip: to_json() reconstructs a full Diff.
    from nucleation import Diff
    d2 = Diff.from_json(d.to_json())
    assert d2.distance == d.distance, "round-trip preserved distance"
    print(f"to_json round-trip OK (distance still {d2.distance})")

    # Projections are schematics you can save / render.
    d.markers().save("diff_markers.schem")
    print("wrote diff_markers.schem (added=lime, removed=red, changed=yellow, swapped=blue)")

    # ── Glowing overlay GLB (needs meshing + a resource pack) ──
    pack_path = os.environ.get("PACK")
    if pack_path:
        try:
            from nucleation import ResourcePack
            pack = ResourcePack.from_file(pack_path)
            after_glb = b.to_mesh(pack).glb_data  # property, returns bytes
            overlay = d.to_overlay_glb(bytes(after_glb))
            with open("diff_overlay.glb", "wb") as f:
                f.write(overlay)
            print(f"wrote diff_overlay.glb ({len(overlay)} bytes, glowing markers on the after-build)")
        except Exception as e:  # meshing feature off, or pack missing
            print(f"overlay skipped: {e}")
    else:
        print("overlay skipped: set PACK=<resource pack zip> to render diff_overlay.glb")


if __name__ == "__main__":
    main()
