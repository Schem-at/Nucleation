#!/usr/bin/env python3
"""Apply stable Python-only compatibility shims after Diplomat generation."""

from pathlib import Path


SCHEMATIC_BINDING = Path("bindings/python/src/sub_modules/nucleation/Schematic_binding.cpp")
BUILDING_TOOL_BINDING = Path(
    "bindings/python/src/sub_modules/nucleation/BuildingTool_binding.cpp"
)


def replace_once(source: str, old: str, new: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one generated binding fragment, found {count}: {old}")
    return source.replace(old, new)


def main() -> None:
    source = SCHEMATIC_BINDING.read_text()
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
    SCHEMATIC_BINDING.write_text(source)

    source = BUILDING_TOOL_BINDING.read_text()
    source = replace_once(
        source,
        '#include "BuildingTool.hpp"\n',
        '#include "BuildingTool.hpp"\n#include "sdf_callback.hpp"\n',
    )
    source = replace_once(
        source,
        '        .def_static("fill", &nucleation::BuildingTool::fill, "schematic"_a, "shape"_a, "brush"_a)\n',
        '        .def_static("fill", &nucleation::BuildingTool::fill, "schematic"_a, "shape"_a, "brush"_a)\n'
        '        .def_static("fill_sdf_function", &nucleation::fill_sdf_function, '
        '"schematic"_a, "brush"_a, "min_x"_a, "min_y"_a, "min_z"_a, '
        '"max_x"_a, "max_y"_a, "max_z"_a, "function"_a, '
        '"normal"_a = nb::none(), "epsilon"_a = 0.5)\n',
    )
    BUILDING_TOOL_BINDING.write_text(source)

    for path in sorted(Path("bindings/python/src").rglob("*")):
        if path.suffix not in {".cpp", ".hpp"}:
            continue
        original = path.read_text()
        normalized = "\n".join(line.rstrip() for line in original.splitlines()) + "\n"
        if normalized != original:
            path.write_text(normalized)


if __name__ == "__main__":
    main()
