# Porting `src/ffi/*.rs` to `src/bridge/` (Diplomat)

Rules for porting a hand-written FFI module to a `#[diplomat::bridge]` module. Read the
three existing files first — they are the pattern: `shared.rs` (shared error enum +
POD structs), `schematic.rs` (the core opaque), `autostack.rs` (a ported module that
cross-references `Schematic` and uses the static-methods-on-a-dummy-opaque pattern).

Diplomat reference examples (many patterns, known-good): the fork checkout at
`/Users/harrison/code/stencil/diplomat/feature_tests/src/*.rs`.

## Layout

- One file per domain: `src/bridge/<domain>.rs`, mirroring the old `src/ffi/<domain>.rs`.
- Each file: `#[diplomat::bridge] pub mod ffi { ... }` at top level.
- Register in `src/bridge/mod.rs` with `pub mod <domain>;` — feature-gate the whole
  module there if the old ffi file was feature-gated (e.g.
  `#[cfg(feature = "meshing")] pub mod meshing;`).
- Cross-module type refs: `use super::super::shared::ffi::NucleationError;`,
  `use super::super::schematic::ffi::Schematic;`. The `Schematic` inner field is
  `pub(crate)` — access it as `&schematic.0` / `&mut schematic.0`.

## Type rules (violating these breaks at least one backend)

1. Opaques: `#[diplomat::opaque_mut]` if ANY method takes `&mut self`, else
   `#[diplomat::opaque]`. Constructors return `Box<Self>` (or `Result<Box<Self>, _>`).
   Flatten the old double-box `Wrapper(*mut T)` to `Wrapper(T)`.
2. Enums: C-like only. Do NOT derive `Copy`/`Clone` (the macro derives them; you'll get
   E0119). `#[derive(PartialEq, Eq)]` is fine if needed. Error enums get
   `#[diplomat::attr(auto, error)]`.
3. Every fallible method returns `Result<T, NucleationError>` (import from `shared`).
   **Never return `Option<T>`** — map `None` to `Err(NucleationError::NotFound)`.
4. Strings in: `name: &DiplomatStr` (it's `&[u8]`) — validate with
   `std::str::from_utf8(..).map_err(|_| NucleationError::InvalidArgument)?`.
5. Strings out: last param `out: &mut DiplomatWrite`, `use std::fmt::Write;`,
   `let _ = write!(out, ...)`. Method returns `()` or `Result<(), _>`.
   (`use diplomat_runtime::DiplomatWrite;`)
6. Binary data out (bytes, PNG, GLB, serialized blobs): **base64 string** through
   `DiplomatWrite` (add `_b64` suffix to the method name). `DiplomatWrite` is UTF-8-only
   and the JS/Kotlin backends decode it as text — raw bytes corrupt. Use the `base64`
   crate (already a dev-dep; add `base64 = "0.22"` to `[dependencies]` gated behind
   `bridge` if not present — check Cargo.toml first, coordinate via the feature's dep
   list).
7. Slices in: primitives only — `&[i32]`, `&[f32]`, `&[u8]`. **No struct slices**
   (`&[BlockPos]` breaks the JS and Kotlin backends). Positions cross as flat
   `&[i32]` chunked in threes, exactly like the old `schematic_set_blocks`.
8. No `Vec<T>` returns, no callbacks, no trait objects, no returning `&T` borrows —
   return owned `Box<T>` (clone the inner value if needed; the old ffi cloned at
   combinator boundaries too, e.g. `shape_union`).
9. JSON-string APIs stay JSON-string APIs (write the JSON through `DiplomatWrite`).
   Don't invent typed replacements in this pass.
10. Rust enums with payloads (simulation `Value` etc.): keep the existing manual
    pattern — opaque + `from_x` constructors + fallible `as_x` accessors +
    `type_name` writing a discriminant string.
11. Consuming operations (old `schematicbuilder_build`, `worldsink_finish`): the opaque
    holds `Option<Inner>`; the method takes `&mut self`, `self.0.take()`, and returns
    `Err(NucleationError::AlreadyConsumed)` on second call.
12. Free-standing old functions with no natural receiver: static methods on a dummy
    `#[diplomat::opaque] pub struct <Domain>;` namespace type (see `autostack.rs`).
13. Feature-dependent behavior *within* a module: `#[cfg]` inside the method body
    (both arms present), never on the method signature.
14. Methods with >7 params: `#[allow(clippy::too_many_arguments)]`.

## Error mapping

Old convention → new: null-check args (gone — references can't be null),
`set_last_error` + null/-1/-2 returns → the matching `NucleationError` variant
(`Parse`, `Serialize`, `Io`, `Lock`, `Store`, `Mesh`, `Render`, `Simulation`,
`InvalidArgument`, `NotFound`, `AlreadyConsumed`). Look at the old error string to pick
the variant; see `stencil/docs/nucleation-error.md`.

## Coverage contract

Every `#[no_mangle] extern "C" fn` in the old file must map to exactly one bridge
method, except: `free_*`, `*_free`, `*_destroy`, `schematic_last_error` (obsolete by
construction — destructors and error transport are generated). At the top of your
module file, list anything else you could not port:
`// Omitted from port: <old fn> — <reason>`.
Keep old doc comments (adapted).

## Verification loop (must pass before you are done)

```sh
cd <your worktree root>
cargo build --lib --features bridge[,<extra features your module needs>]

DT=/Users/harrison/code/stencil/diplomat/target/release/diplomat-tool
E=src/bridge/mod.rs
rm -rf /tmp/gen_check_$$ && mkdir -p /tmp/gen_check_$$
$DT c       /tmp/gen_check_$$/c  -e $E -s
$DT cpp     /tmp/gen_check_$$/cp -e $E -s
$DT js      /tmp/gen_check_$$/js -e $E -s
$DT kotlin  /tmp/gen_check_$$/kt -e $E -s --config-file /Users/harrison/code/stencil/configs/kotlin.toml
$DT nanobind /tmp/gen_check_$$/py -e $E -s --config-file /Users/harrison/code/stencil/configs/nanobind.toml
$DT php     /tmp/gen_check_$$/ph -e $E -s --config-file /Users/harrison/code/stencil/configs/nucleation-php.toml
```

All seven commands must exit 0 with no `Lowering error` output. If a backend rejects a
construct, fix the API shape (rules above) rather than disabling the method; use
`#[diplomat::attr(<backend>, disable)]` only as a documented last resort.
