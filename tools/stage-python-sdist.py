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
