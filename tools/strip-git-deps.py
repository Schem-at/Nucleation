#!/usr/bin/env python3
"""Prepare Cargo.toml for crates.io publishing.

crates.io rejects git dependencies without versions and cannot resolve local
workspace crates that have not themselves been published. The published crate
therefore ships WITHOUT `simulation` (MCHPRS) and the local-only `mc-tick`,
`routing`, and `hdl` features. `meshing`/`rendering` survive:
schematic-mesher is on crates.io and its dep is dual version+git, which
cargo publish handles itself (keeps the version, drops the git side).
Simulation stays available from the git repo:

    nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation"] }

Run in CI right before `cargo publish --allow-dirty`; mutates Cargo.toml
in place (the runner is ephemeral). For a local dry-run:

    python3 tools/strip-git-deps.py && cargo publish --dry-run --allow-dirty && git checkout Cargo.toml

Note: the local dry-run fails with "no matching package" for
schematic-mesher until that crate is actually published to crates.io.
"""

import re
import sys
from pathlib import Path

manifest = Path(__file__).resolve().parent.parent / "Cargo.toml"
text = manifest.read_text()
original = text

# Drop the git-only deps (mchprs_* and the simulation-only hematite-nbt).
# schematic-mesher is intentionally NOT dropped: it is a dual version+git
# dep, cargo publish keeps the version side.
text = re.sub(r"(?m)^mchprs_\w+ = \{[^}]*\}\n", "", text)
text = re.sub(r'(?m)^hematite-nbt = "\*"\n', "", text)

# Drop the [patch.crates-io] section (only patches hematite-nbt for MCHPRS).
text = re.sub(r"(?ms)^# Patch for MCHPRS compatibility\n\[patch\.crates-io\]\n.*?(?=^\[|\Z)", "", text)

# Drop the git-only feature definition.
text = re.sub(r"(?ms)^# Redstone simulation support\nsimulation = \[.*?\]\n", "", text)

# Drop its entry from bridge-full.
text = re.sub(r'(?m)^\s*"simulation",\n', "", text)

# These optional dependencies point at workspace crates which are not on
# crates.io. A version next to a path does not make them publishable: Cargo
# still requires that version to exist in the registry while preparing the
# root package. Remove their feature declarations, dependency/dev-dependency
# entries, and bridge-full members together.
local_only = ("mc-tick", "nucleation-routing", "pnr-core", "nucleation-hdl")
for dependency in local_only:
    text = re.sub(
        rf"(?m)^{re.escape(dependency)} = \{{[^}}]*\}}\n",
        "",
        text,
    )
for feature in ("mc-tick", "routing", "hdl"):
    text = re.sub(rf"(?m)^{re.escape(feature)} = \[[^\n]*\]\n", "", text)
    text = re.sub(rf'(?m)^\s*"{re.escape(feature)}",\n', "", text)
# Dev-only helpers are local workspace crates too.
for dependency in ("mc-tick-trace", "mc-test"):
    text = re.sub(
        rf"(?m)^{re.escape(dependency)} = \{{[^}}]*\}}\n",
        "",
        text,
    )

for marker in (
    "mchprs",
    'hematite-nbt = "*"',
    "simulation = [",
    '"simulation"',
    "[patch.crates-io]",
    'mc-tick = { path =',
    'nucleation-routing = { path =',
    'pnr-core = { path =',
    'nucleation-hdl = { path =',
    'mc-tick-trace = { path =',
    'mc-test = { path =',
):
    if marker in text:
        sys.exit(f"strip-git-deps: marker still present after rewrite: {marker!r}")
# Positive checks: meshing/rendering must survive, with a versioned mesher dep.
for marker in (
    'meshing = ["schematic-mesher"]',
    "rendering = [",
    'schematic-mesher = { version = "',
):
    if marker not in text:
        sys.exit(f"strip-git-deps: expected marker missing after rewrite: {marker!r}")
if text == original:
    sys.exit("strip-git-deps: no changes made — manifest layout drifted, update the patterns")

manifest.write_text(text)
print("Cargo.toml stripped for crates.io publish (git/local-only features removed)")
