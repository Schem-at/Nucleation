#!/usr/bin/env python3
"""Normalize Diplomat's enum wrappers in a nanobind-generated type stub.

Diplomat represents an FFI enum as a small wrapper class containing a nested
nanobind enum with the same name. Runtime constants exported onto the wrapper
are implicitly converted wherever the wrapper is accepted. Stubgen cannot
express that implicit C++ conversion and the repeated class name also shadows
the wrapper, so model the exported constants directly as wrapper instances.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


TOP_LEVEL_CLASS = re.compile(r"^class ([A-Za-z_][A-Za-z0-9_]*):$")
NESTED_ENUM = re.compile(r"^    class ([A-Za-z_][A-Za-z0-9_]*)\(enum\.Enum\):$")


def normalize(source: str) -> tuple[str, int, int]:
    lines = source.splitlines()
    output: list[str] = []
    outer: str | None = None
    nested_count = 0
    constant_count = 0
    index = 0

    while index < len(lines):
        line = lines[index]
        top_level = TOP_LEVEL_CLASS.match(line)
        if top_level:
            outer = top_level.group(1)
        elif line and not line[0].isspace():
            outer = None

        nested = NESTED_ENUM.match(line)
        if outer is not None and nested and nested.group(1) == outer:
            nested_count += 1
            index += 1
            while index < len(lines):
                candidate = lines[index]
                if not candidate or candidate.startswith("        "):
                    index += 1
                    continue
                break
            continue

        if outer is not None:
            constant = re.fullmatch(
                rf"    ([A-Za-z_][A-Za-z0-9_]*): {re.escape(outer)} = "
                rf"{re.escape(outer)}\.\1",
                line,
            )
            if constant:
                name = constant.group(1)
                output.append(f"    {name}: {outer}")
                constant_count += 1
                index += 1
                continue

            if line == f"    def __eq__(self, arg: {outer}, /) -> bool: ...":
                output.append("    def __eq__(self, arg: object, /) -> bool: ...")
                index += 1
                continue

        output.append(line)
        index += 1

    return "\n".join(output) + "\n", nested_count, constant_count


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: normalize_stub.py PATH")
    path = Path(sys.argv[1])
    normalized, nested_count, constant_count = normalize(path.read_text())
    if nested_count == 0 or constant_count == 0:
        raise SystemExit("generated stub contained no Diplomat enum wrappers")
    path.write_text(normalized)
    print(
        f"normalized {nested_count} enum wrappers and {constant_count} constants "
        f"in {path}"
    )


if __name__ == "__main__":
    main()
