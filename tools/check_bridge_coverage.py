#!/usr/bin/env python3
"""Check that src/bridge covers the old hand-written C FFI surface.

The old surface (every `pub extern "C" fn` in src/ffi/*.rs + src/store/ffi.rs, frozen
before deletion) lives in tools/bridge_coverage/old_ffi_surface.txt. This script parses
the bridge modules for `impl <Type> { pub fn <method> }` pairs, normalizes both sides
(lowercase, underscores stripped, type name prefixed), and reports any old function with
no bridge counterpart that isn't accounted for in exclusions.txt.

This replaces tools/check_api_parity.rs / check_jvm_parity.rs: with one generated
surface there is nothing to cross-diff — only completeness vs the pre-migration
baseline is worth checking, and only until the migration is accepted.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BRIDGE = ROOT / "src" / "bridge"
BASELINE = ROOT / "tools" / "bridge_coverage" / "old_ffi_surface.txt"
EXCLUSIONS = ROOT / "tools" / "bridge_coverage" / "exclusions.txt"

# Obsolete by construction: destructors are generated, buffers/strings are owned by
# DiplomatWrite, and error transport is Result-based instead of thread-local state.
# Bounded, allowlisted rename suffixes: a bridge method may add exactly one of these
# to an old name (encoding/representation changes mandated by PORTING.md), nothing else.
ALLOWED_RENAME_SUFFIXES = {"b64", "json", "with", "str", "snbt"}

OBSOLETE_PATTERNS = [
    re.compile(r"^free_"),
    re.compile(r"_free$"),
    re.compile(r"_destroy$"),
    re.compile(r"^schematic_last_error$"),
]


def norm(s: str) -> str:
    return s.replace("_", "").lower()


def bridge_methods() -> set[str]:
    """Set of normalized '<type><method>' strings across all bridge modules.

    Attribution is positional: each `pub fn` belongs to the nearest preceding
    `impl <Type>`. (No brace counting — string literals in `write!` format args
    contain unbalanced braces and derail any naive counter.)
    """
    out: set[str] = set()
    impl_re = re.compile(r"^\s*impl\s+([A-Za-z0-9_]+)\s*\{", re.M)
    fn_re = re.compile(r"^\s*pub\s+fn\s+([a-z0-9_]+)", re.M)
    for path in sorted(BRIDGE.glob("*.rs")):
        text = path.read_text()
        impls = [(m.start(), m.group(1)) for m in impl_re.finditer(text)]
        for fm in fn_re.finditer(text):
            preceding = [ty for start, ty in impls if start < fm.start()]
            if preceding:
                out.add(norm(preceding[-1]) + norm(fm.group(1)))
    return out


def load_exclusions() -> dict[str, str]:
    exclusions: dict[str, str] = {}
    if EXCLUSIONS.exists():
        for line in EXCLUSIONS.read_text().splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            name, _, reason = line.partition(" ")
            exclusions[name] = reason.strip("- ").strip()
    return exclusions


def main() -> int:
    old = [l.strip() for l in BASELINE.read_text().splitlines() if l.strip()]
    methods = bridge_methods()
    exclusions = load_exclusions()

    missing = []
    for fn in old:
        if any(p.search(fn) for p in OBSOLETE_PATTERNS):
            continue
        if fn in exclusions:
            continue
        # Candidate normalizations of the old name: as-is, accessor prefix dropped
        # (schematic_get_author -> Schematic::author), constructor rename
        # (blockstate_new -> BlockState::create).
        candidates = {norm(fn)}
        candidates.add(norm(re.sub(r"_(get|set)_", "_", fn)))
        if fn.endswith("_new"):
            candidates.add(norm(fn[: -len("_new")] + "_create"))
        # Old ABI prefixes that map to a differently-named bridge type.
        for old_prefix, new_ty in [("typed_executor_", "typedcircuitexecutor")]:
            if fn.startswith(old_prefix):
                rest = fn[len(old_prefix):]
                candidates.add(new_ty + norm(rest))
                candidates.add(new_ty + norm(re.sub(r"^(get|set)_", "", rest)))
        matched = False
        for n in candidates:
            if n in methods or any(n == m or n.endswith(m) or m.endswith(n) for m in methods):
                matched = True
                break
            if any(m.startswith(n) and m[len(n):] in ALLOWED_RENAME_SUFFIXES for m in methods):
                matched = True
                break
        if matched:
            continue
        n = norm(fn)
        # A bridge method covers an old fn if the normalized old name equals
        # '<type><method>' exactly, ends with the method (handles
        # schematic_builder_ vs SchematicBuilder), or differs only by an
        # allowlisted rename suffix (to_litematic -> to_litematic_b64,
        # get_blocks -> get_blocks_json, shape_union -> Shape::union_with).
        if n in methods or any(n == m or n.endswith(m) or m.endswith(n) for m in methods):
            continue
        renamed = any(
            m.startswith(n) and m[len(n):] in ALLOWED_RENAME_SUFFIXES
            for m in methods
        )
        if renamed:
            continue
        missing.append(fn)

    stale = [name for name in exclusions if name not in old]

    if stale:
        print("Stale exclusions (not in baseline):")
        for s in stale:
            print(f"  {s}")
    if missing:
        print(f"MISSING from bridge ({len(missing)} of {len(old)}):")
        for fn in missing:
            print(f"  {fn}")
        return 1
    print(f"bridge coverage OK: all {len(old)} baseline fns covered "
          f"({len(exclusions)} explicit exclusions)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
