#!/usr/bin/env python3
"""Normalize deterministic whitespace in Diplomat's generated Kotlin sources."""

from pathlib import Path


ROOT = Path("bindings/kotlin/src/main/kotlin")


def main() -> None:
    files = sorted(ROOT.rglob("*.kt"))
    if not files:
        raise SystemExit(f"no generated Kotlin files found under {ROOT}")

    changed = 0
    for path in files:
        original = path.read_text()
        normalized = "\n".join(line.rstrip() for line in original.splitlines()) + "\n"
        if normalized != original:
            path.write_text(normalized)
            changed += 1

    if changed == 0:
        raise SystemExit("expected generated Kotlin whitespace to normalize, but changed nothing")


if __name__ == "__main__":
    main()
