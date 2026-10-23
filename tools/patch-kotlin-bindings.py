#!/usr/bin/env python3
"""Normalize deterministic whitespace in Diplomat's generated Kotlin sources,
and give `regionBlockIndices` an owned-array shape."""

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

    patch_region_block_indices(ROOT / "at/schem/nucleation/Schematic.kt")


def patch_region_block_indices(path: Path) -> None:
    """`region_block_indices` fills a caller buffer (`&mut [u32]`), but the
    generated Kotlin copies the array into native memory and never back. Keep
    `regionBlockIndices(name, start, count): Result<UIntArray>`: pass a native
    buffer of `count` cells and return the cells written."""
    source = path.read_text()
    replacements = [
        (
            "fun regionBlockIndices(regionName: String, start: UInt, out: UIntArray): Result<UInt> {",
            "fun regionBlockIndices(regionName: String, start: UInt, count: UInt): Result<UIntArray> {",
        ),
        (
            "val outSliceMemory = PrimitiveArrayTools.borrow(out)",
            "val outSliceMemory = PrimitiveArrayTools.borrow(UIntArray(count.toInt()))",
        ),
    ]
    start = source.index("    fun regionBlockIndices(")
    end = source.index("\n    }\n", start)
    method = source[start:end]
    for old, new in replacements:
        if old not in method:
            raise SystemExit(f"regionBlockIndices binding changed: {old!r} not found")
        method = method.replace(old, new)
    ok = "return (nativeOkVal.toUInt()).ok()"
    if ok not in method:
        raise SystemExit("regionBlockIndices binding changed: result conversion not found")
    method = method.replace(
        ok,
        "return PrimitiveArrayTools.getUIntArray(outSliceMemory.slice)"
        ".copyOf(nativeOkVal.toUInt().toInt()).ok()",
    )
    path.write_text(source[:start] + method + source[end:])


if __name__ == "__main__":
    main()
