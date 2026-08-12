#!/usr/bin/env python3
"""Stage the Rust core inside bindings/python for a self-contained sdist."""

from __future__ import annotations

import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "bindings" / "python" / "rust"

if DEST.exists():
    shutil.rmtree(DEST)
DEST.mkdir(parents=True)

for name in ("Cargo.toml", "Cargo.lock", "build.rs", "LICENSE", "README.md"):
    shutil.copy2(ROOT / name, DEST / name)

for name in ("src", "data"):
    shutil.copytree(ROOT / name, DEST / name)

# Cargo auto-discovers Rust examples/tests/benches in addition to explicitly
# declared targets. Source files are sufficient for manifest validation and do
# not drag fixtures or rendered assets into the source distribution.
for target_root in ("examples", "tests", "benches"):
    root = ROOT / target_root
    if not root.exists():
        continue
    for source in root.rglob("*.rs"):
        destination = DEST / source.relative_to(ROOT)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)

# The root manifest declares `[workspace] members = ["crates/*"]`, and cargo
# resolves that glob before it builds anything — so every member's manifest must
# exist in the sdist or the build dies at manifest load with "failed to read
# crates/*/Cargo.toml", long before a single feature is considered. `bridge-full`
# also turns on `mc-tick`, which is a real path dependency, so its sources have
# to be here regardless.
#
# Manifest plus `src` only, for the same reason examples and tests are copied as
# bare `.rs` above: mc-tick's `tests/` is 10 MB of oracle captures, and none of
# it is needed to build the library.
for member in sorted((ROOT / "crates").glob("*")):
    if not (member / "Cargo.toml").is_file():
        continue
    member_dest = DEST / "crates" / member.name
    member_dest.mkdir(parents=True, exist_ok=True)
    shutil.copy2(member / "Cargo.toml", member_dest / "Cargo.toml")
    shutil.copytree(member / "src", member_dest / "src")

    # Cargo validates conventional targets named by [[bin]], [[example]],
    # [[test]], and [[bench]] while loading a workspace manifest, even when the
    # requested build is `--lib`. Copy only Rust sources, keeping large test
    # fixtures and benchmark captures out of the sdist.
    for target_root in ("examples", "tests", "benches"):
        root = member / target_root
        if not root.exists():
            continue
        for source in root.rglob("*.rs"):
            destination = member_dest / source.relative_to(member)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)

    # Non-conventional explicit target paths need the same treatment. Paths in
    # a member manifest are relative to that member, not the workspace root.
    member_manifest = (member / "Cargo.toml").read_text()
    for relative in sorted(
        set(re.findall(r'^path\s*=\s*"([^"]+)"', member_manifest, re.MULTILINE))
    ):
        source = member / relative
        if source.is_file() and not relative.startswith("src/"):
            destination = member_dest / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)

# Cargo validates every explicitly declared target path even when only --lib is
# built. Copy those target sources without dragging unrelated examples/assets
# into the source distribution.
cargo_toml = (ROOT / "Cargo.toml").read_text()
for relative in sorted(set(re.findall(r'^path\s*=\s*"([^"]+)"', cargo_toml, re.MULTILINE))):
    source = ROOT / relative
    if source.is_file() and not relative.startswith("src/"):
        destination = DEST / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)

print(DEST)
