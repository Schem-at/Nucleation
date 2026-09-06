#!/usr/bin/env python3
"""Repair generated ESM declarations and wasm32 pointer handling."""

from pathlib import Path
import re


ROOT = Path("bindings/js")
RELATIVE_IMPORT = re.compile(r'((?:from|import)\s*[\(]?["\']\./)([^"\']+)(["\'])')


def patch_wasm_pointers(root: Path) -> None:
    """Wasm i32 exports are signed JS numbers; byte offsets must be unsigned.

    Normalize pointers at allocation/read boundaries, never scalar coordinates
    or enum values. This keeps the upper half of wasm32 memory addressable.
    """
    runtime = root / "diplomat-runtime.mjs"
    source = runtime.read_text()
    for signature in [
        "export function readString8(wasm, ptr, len) {",
        "export function readString16(wasm, ptr, len) {",
        "export function ptrRead(wasm, ptr) {",
        "export function resultFlag(wasm, ptr, offset) {",
        "export function enumDiscriminant(wasm, ptr) {",
    ]:
        patched = signature + "\n    ptr >>>= 0; // wasm32 addresses are unsigned, including above 2 GiB."
        if patched not in source:
            if signature not in source:
                raise SystemExit(f"generated runtime changed: {signature}")
            source = source.replace(signature, patched)
    source = re.sub(
        r"((?:const ptr|this\.#buffer|this\.#ptr) = (?:this\.#)?wasm\.diplomat_(?:alloc|buffer_write_create)\([^\n]+\))(?=;)",
        r"\1 >>> 0", source,
    )
    for original, patched in [
        ("this.ptr = ptr;", "this.ptr = ptr >>> 0;"),
        ("new typedArrayKind(arrayBuffer, offset)", "new typedArrayKind(arrayBuffer, offset < 0 ? offset >>> 0 : offset)"),
        ("wasm.memory.buffer, buffer, 2", "wasm.memory.buffer, buffer >>> 0, 2"),
    ]:
        if original not in source and patched not in source:
            raise SystemExit(f"generated runtime changed: {original}")
        source = source.replace(original, patched)

    # An inner string allocation can grow memory and detach the previous view.
    # Also, a Uint32Array length counts elements, not bytes.
    old = "        const destination = new Uint32Array(wasm.memory.buffer, ptr, byteLength);\n"
    if old in source:
        source = source.replace(old, "")
        source = source.replace(
            "            destination[2 * i] = stringsAlloc[i].ptr;",
            "            const destination = new Uint32Array(wasm.memory.buffer, ptr, strings.length * 2);\n"
            "            destination[2 * i] = stringsAlloc[i].ptr;",
        )
    runtime.write_text(source)

    for path in root.glob("*.mjs"):
        if path == runtime:
            continue
        source = path.read_text()
        source = re.sub(
            r"(static _fromFFI\([^\n]*\bptr\b[^\n]*\) \{\n)(?!        ptr >>>= 0;)",
            r"\1        ptr >>>= 0; // unsigned wasm32 address; field values retain their signedness.\n",
            source,
        )
        path.write_text(source)


def patch_declaration(path: Path) -> int:
    source = path.read_text()

    def add_extension(match: re.Match[str]) -> str:
        specifier = match.group(2)
        if Path(specifier).suffix:
            return match.group(0)
        return f"{match.group(1)}{specifier}.mjs{match.group(3)}"

    patched, count = RELATIVE_IMPORT.subn(add_extension, source)
    path.write_text(patched)
    return count


def main() -> None:
    patch_wasm_pointers(ROOT)
    declarations = sorted(ROOT.glob("*.d.ts"))
    if not declarations:
        raise SystemExit("no generated JS declarations found")

    replacements = sum(patch_declaration(path) for path in declarations)
    if replacements == 0:
        raise SystemExit("no extensionless JS declaration imports found; generator shape changed")

    remaining = []
    for path in declarations:
        for match in RELATIVE_IMPORT.finditer(path.read_text()):
            if not Path(match.group(2)).suffix:
                remaining.append(f"{path}:{match.group(0)}")
    if remaining:
        raise SystemExit("extensionless imports remain:\n" + "\n".join(remaining))

    # Return a packed owned copy for renderer transfer windows. A borrowed WASM
    # view cannot survive memory.grow or schematic mutation; Array.from boxes it.
    schematic = ROOT / "Schematic.mjs"
    source = schematic.read_text()
    start = source.index("    regionBlockIndices(")
    end = source.index("\n    }", start)
    method = source[start:end]
    old = 'Array.from(new diplomatRuntime.DiplomatSlicePrimitive(wasm, diplomatReceive.buffer, "u32", aEdges).getValue())'
    new = 'new diplomatRuntime.DiplomatSlicePrimitive(wasm, diplomatReceive.buffer, "u32", aEdges).getValue().slice()'
    if old not in method:
        raise SystemExit("regionBlockIndices slice conversion changed")
    schematic.write_text(source[:start] + method.replace(old, new) + source[end:])
    declaration = ROOT / "Schematic.d.ts"
    declaration.write_text(declaration.read_text().replace(
        "regionBlockIndices(regionName: string, start: number, count: number): Array<number>",
        "regionBlockIndices(regionName: string, start: number, count: number): Uint32Array",
    ))

    # Node16/NodeNext resolves an imported `./Thing.mjs` declaration to
    # `Thing.d.mts`. Keep Diplomat's `.d.ts` files for bundler compatibility
    # and emit ESM declaration companions from the same patched source.
    for path in declarations:
        companion = path.with_name(path.name.removesuffix(".d.ts") + ".d.mts")
        companion.write_text(path.read_text())


if __name__ == "__main__":
    main()
