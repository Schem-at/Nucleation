# Nucleation API Upgrade — Design Spec

**Status:** Approved 2026-05-03
**Source proposal:** `api_upgrade.md` (Python redesign)
**Scope expansion:** Apply analogous polish to JS/WASM and minimal-but-honest convenience to FFI.

## Goal

Apply the API improvements in `api_upgrade.md` while preserving full backwards compatibility, working `pip install`, and the existing GitHub Actions wheel-build pipeline. Where the Python ergonomics translate to JS, ship them in a thin JS wrapper over wasm-bindgen output. Where they don't translate (FFI/C), add only the small convenience entry points that are honestly useful.

## Architecture

### Python — switch to mixed-layout package

Today the wheel ships a single compiled module named `nucleation`. We split into:

- `nucleation._native` — the compiled PyO3 extension (renamed from `nucleation`).
- `nucleation` — pure-Python package at `python/nucleation/__init__.py` providing the polished API on top of `_native`.

Mechanics:

- `pyproject.toml`: `module-name = "nucleation._native"`, add `python-source = "python"`, include `py.typed` and `_native.pyi`.
- `src/python/mod.rs`: `#[pymodule] fn _native(...)` (rename, was `nucleation`). Internal-only — users don't see this name.
- `python/nucleation/__init__.py`: re-export everything that used to be at `nucleation.*` (every Py* class) for full backcompat, plus the new polished surface.
- `python/nucleation/__init__.pyi`: type-hinted public surface.
- `python/nucleation/_native.pyi`: types for the extension module.

This lets us write the polished API in real Python with dataclasses, kwargs, `with` blocks, and overloads — none of which are pleasant to write in PyO3.

### Python — new public surface

All implemented in `python/nucleation/`:

- `Schematic` — wraps a `_native.PySchematic` instance. New methods `Schematic.new()`, `.open()`, `.from_template()` classmethods. Mutable, chainable (`set_block`/`map`/`fill`/`with_pack`/`copy`/`simulate` return `self`). `__init__(name_or_path)` preserves the old constructor's polymorphism. Context-manager support. `save(path, format=None)` infers format from extension. `render(path, config=None, **kwargs)` and `export_mesh(path, *, pack=None)` use the bound pack. `set_block` accepts both `(x, y, z, block)` and `(pos, block, *, state, nbt)`.
- `Block` — frozen dataclass `(id, state, nbt)`. `Block.parse(s)` parses `id[k=v]{snbt}` form. `with_state`/`with_nbt` return new instances.
- `Cursor` — `pos`, `step`. `place(block, *, state, nbt, offset)`, `advance(n)`, `reset()`. Returns self.
- `UseBlock`, `ButtonPress` — frozen dataclasses with `pos: tuple[int,int,int]`. Used as `events=[UseBlock((0,1,3))]`.
- `SchematicBuilder` — kept as deprecated shim emitting `DeprecationWarning`. About 20 lines.
- `BlockLike = str | Block` type alias.

### Python — backcompat shims

- `nucleation.Schematic` is the new wrapper, but its `__init__` and method signatures accept all old call patterns:
  - `Schematic("foo.schem")` loads, `Schematic("untitled")` creates blank.
  - `set_block(x, y, z, "id")` still works (positional-arg overload).
  - `set_block_with_properties`, `set_block_with_nbt`, `set_block_from_string`, `fill_cuboid`, `fill_sphere`, `copy_region`, every existing format I/O method passes through to `_native`.
  - `render_to_file(pack, path, config)` kept, calls `render` under the hood, emits `DeprecationWarning`.
  - `to_mesh(pack)` kept similarly; new `export_mesh(path, *, pack=None)` is the polished replacement that writes directly.
  - `create_simulation_world()` returns the underlying `_native.PyMchprsWorld` unchanged. New `simulate(*, ticks=1, events=None)` is the convenience wrapper.
- All `Py*` classes re-exported under their existing names so `from nucleation import PySchematicBuilder` etc. keeps working.

### WASM — JS wrapper module

Add `js/nucleation.mjs` (and a CJS shim if needed) shipped alongside the wasm-bindgen `pkg/` output via the existing `build-wasm.sh`. Provides:

- `Schematic` class wrapping `SchematicWrapper`.
- Static `Schematic.new(name?, opts?)`, `Schematic.open(buffer, opts?)`, `Schematic.fromTemplate(s, opts?)`.
- `setBlock([x, y, z], "id", { state, nbt })` — accepts both `[x,y,z]` array and three positional args.
- `cursor({ origin, step })` returning a `Cursor`.
- `simulate({ ticks, events })` wrapping the existing simulation world.
- `render({ path?, width, height, ... })` — kwargs-style.
- All raw `SchematicWrapper` methods remain accessible via `schem.raw` for power users.

The `pkg/` JS output stays untouched — wrapper is additive. `package.json` `main` updated to point to the wrapper.

### FFI — small honest additions

Most of the proposal's wins (kwargs, dataclasses, overloads, with-blocks, chained returns) don't translate to C. Add:

- `schematic_simulate_use_block(handle, ticks, events_xyz, n_events) -> bool` — runs a simulation world, fires `n_events` use-block events (each 3 ints in `events_xyz`), ticks, syncs back. One-call simulate analog.

Skip everything else for FFI. C consumers chain manually; wrapper layers in Python/JS provide the ergonomics.

## Testing

- `tests/python_new_api_test.py` — pytest covering: `Schematic.new/open/from_template`, chainable returns, `Block.parse` round-trip, `Cursor` placement, `set_block` tuple form + state + nbt, `simulate(events=[UseBlock(...)])`, `with` block, deprecated `SchematicBuilder` still works, `set_block(x,y,z,...)` positional still works, `render_to_file` shim.
- `tests/node_new_api_test.mjs` — Node test for the JS wrapper covering set_block array form, cursor, simulate, fromTemplate.
- `tests/test_ffi_simulate.rs` — integration test for the new FFI entry point (gated `#[cfg(feature = "ffi")]` if needed).
- All existing tests must continue to pass.

## Build & release

- `pyproject.toml` updated: `module-name = "nucleation._native"`, `python-source = "python"`, include `py.typed`, `python/nucleation/__init__.pyi`, `python/nucleation/_native.pyi`. Drop legacy top-level `nucleation.pyi` once the new one is in place (or keep it as a one-line re-export).
- `Cargo.toml` lib name unchanged (`libnucleation`).
- GitHub Actions: zero changes required. `maturin build`/`maturin-action` picks up the new layout automatically, abi3 wheels still produced.
- Version bump: minor (e.g. `0.2.0`) given the surface expansion. Patch-level if we want to be conservative — backcompat is preserved.

## Open items resolved during brainstorming

- **Mixed layout vs PyO3-only:** mixed layout chosen — much cleaner for dataclasses/kwargs/with-blocks.
- **JS via Rust JsValue vs JS wrapper:** JS wrapper chosen — ~200 LOC vs 600+ LOC of Rust glue with worse errors.
- **FFI scope:** kept minimal-and-honest, one new entry point.
- **Schematic mutability:** stays mutable as in the proposal; `.copy()` available.

## Out of scope

- Restructuring core Rust types (`UniversalSchematic`, `BlockState`, etc).
- Rewriting the rendering / meshing class hierarchy.
- Adding `BlockLike` parsing in core Rust (lives in the language wrappers).
- Changing the on-disk schematic formats.
