#!/usr/bin/env python3
"""Prepare Cargo.toml for crates.io publishing.

crates.io rejects git dependencies without versions and wildcard version
requirements, so the published crate ships WITHOUT the features whose
deps are git-only: `simulation` (MCHPRS), and `meshing`/`rendering`
(schematic-mesher — publish that crate to crates.io to win these back).
They stay available from the git repo:

    nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation", "rendering"] }

Run in CI right before `cargo publish --allow-dirty`; mutates Cargo.toml
in place (the runner is ephemeral). For a local dry-run:

    python3 tools/strip-git-deps.py && cargo publish --dry-run --allow-dirty && git checkout Cargo.toml
"""

import re
import sys
from pathlib import Path

manifest = Path(__file__).resolve().parent.parent / "Cargo.toml"
text = manifest.read_text()
original = text

# Drop the git-only deps (mchprs_*, simulation-only hematite-nbt, and
# the schematic-mesher meshing backend).
text = re.sub(r"(?m)^mchprs_\w+ = \{[^}]*\}\n", "", text)
text = re.sub(r'(?m)^hematite-nbt = "\*"\n', "", text)
text = re.sub(r"(?m)^schematic-mesher = \{[^}]*\}\n", "", text)

# Drop the [patch.crates-io] section (only patches hematite-nbt for MCHPRS).
text = re.sub(r"(?ms)^# Patch for MCHPRS compatibility\n\[patch\.crates-io\]\n.*?(?=^\[|\Z)", "", text)

# Drop the git-only feature definitions.
text = re.sub(r"(?ms)^# Redstone simulation support\nsimulation = \[.*?\]\n", "", text)
text = re.sub(r"(?m)^# Meshing support - generate 3D meshes from schematics\nmeshing = \[\"schematic-mesher\"\]\n", "", text)
text = re.sub(r"(?m)^# GPU rendering support - render schematics to images\nrendering = \[[^\]]*\]\n", "", text)

# Drop their entries from bridge-full.
text = re.sub(r'(?m)^\s*"(simulation|meshing|rendering)",\n', "", text)

# Drop the [[example]] blocks needing stripped features (examples/ is
# excluded from the package anyway, so the targets are dead weight).
text = re.sub(r"(?m)^# Examples requiring the meshing feature\n", "", text)
text = re.sub(
    r'(?ms)^\[\[example\]\]\nname = "[^"]+"\nrequired-features = \["(?:rendering|meshing)"\]\n\n?',
    "",
    text,
)

for marker in (
    "mchprs",
    'hematite-nbt = "*"',
    "schematic-mesher",
    "simulation = [",
    '"simulation"',
    "meshing = [",
    '"meshing"',
    "rendering = [",
    '"rendering"',
    "[patch.crates-io]",
):
    if marker in text:
        sys.exit(f"strip-git-deps: marker still present after rewrite: {marker!r}")
if text == original:
    sys.exit("strip-git-deps: no changes made — manifest layout drifted, update the patterns")

manifest.write_text(text)
print("Cargo.toml stripped for crates.io publish (simulation is git-only)")
