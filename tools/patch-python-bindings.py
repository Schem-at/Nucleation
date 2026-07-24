#!/usr/bin/env python3
"""Apply stable Python-only compatibility shims after Diplomat generation."""

from pathlib import Path


BINDING = Path("bindings/python/src/sub_modules/nucleation/Schematic_binding.cpp")


def replace_once(source: str, old: str, new: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one generated binding fragment, found {count}: {old}")
    return source.replace(old, new)


def main() -> None:
    source = BINDING.read_text()
    source = replace_once(
        source,
        '#include "Schematic.hpp"\n',
        '#include "Schematic.hpp"\n#include "schematic_compat.hpp"\n',
    )
    source = replace_once(
        source,
        '        .def_static("open", std::move(maybe_op_unwrap(&nucleation::Schematic::open)), "path"_a)\n',
        '        .def_static("open", &nucleation::python_compat::schematic_open, "path"_a)\n',
    )
    source = replace_once(
        source,
        '        .def("save", &nucleation::Schematic::save, "path"_a)\n',
        '        .def("save", &nucleation::python_compat::schematic_save, "path"_a, nb::kw_only(), "format"_a = nb::none())\n',
    )
    BINDING.write_text(source)


if __name__ == "__main__":
    main()
