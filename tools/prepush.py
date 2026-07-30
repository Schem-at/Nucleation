#!/usr/bin/env python3
"""Pre-push verification for the generated-bindings world.

Runs the same gates CI does: core tests, bindings freshness/determinism,
bridge coverage, and the local smoke tests. Fast-fails on the first error.
"""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

CHECKS: list[tuple[str, list[str]]] = [
    ("cargo test", ["cargo", "test"]),
    ("bridge builds", ["cargo", "build", "--lib", "--features", "bridge"]),
    ("bindings fresh", ["bash", "-c", "./tools/gen-bindings.sh && git diff --exit-code -- bindings"]),
    ("bridge coverage", ["python3", "tools/check_bridge_coverage.py"]),
    # The sdist is staged from a *copy* of the tree, so it can be broken by
    # something the in-tree build never notices. v0.9.0 shipped exactly that:
    # `[workspace] members = ["crates/*"]` resolved fine here and died in CI with
    # "failed to read crates/*/Cargo.toml", because the staging script did not
    # copy the members. Resolving the staged manifest is a couple of seconds and
    # catches that whole class; actually building it is CI's job.
    (
        "sdist manifest resolves",
        [
            "bash",
            "-c",
            "python3 tools/stage-python-sdist.py >/dev/null "
            "&& cargo metadata --manifest-path bindings/python/rust/Cargo.toml "
            "--no-deps --format-version 1 >/dev/null",
        ],
    ),
    ("smoke: C", ["./examples/bridge_smoke/c/run.sh"]),
    ("smoke: C++", ["./examples/bridge_smoke/cpp/run.sh"]),
    ("smoke: PHP", ["./examples/bridge_smoke/php/run.sh"]),
    ("smoke: JS", ["./examples/bridge_smoke/js/run.sh"]),
    ("smoke: Python", ["./examples/bridge_smoke/python/run.sh"]),
]


def main() -> int:
    for name, cmd in CHECKS:
        print(f"==> {name}")
        proc = subprocess.run(cmd, cwd=ROOT)
        if proc.returncode != 0:
            print(f"FAILED: {name}", file=sys.stderr)
            return proc.returncode
    print("all pre-push checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
