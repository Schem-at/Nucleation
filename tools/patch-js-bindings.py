#!/usr/bin/env python3
"""Make Diplomat's TypeScript declarations valid under Node16/NodeNext ESM."""

from pathlib import Path
import re


ROOT = Path("bindings/js")
RELATIVE_IMPORT = re.compile(r'((?:from|import)\s*[\(]?["\']\./)([^"\']+)(["\'])')


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

    # Node16/NodeNext resolves an imported `./Thing.mjs` declaration to
    # `Thing.d.mts`. Keep Diplomat's `.d.ts` files for bundler compatibility
    # and emit ESM declaration companions from the same patched source.
    for path in declarations:
        companion = path.with_name(path.name.removesuffix(".d.ts") + ".d.mts")
        companion.write_text(path.read_text())


if __name__ == "__main__":
    main()
