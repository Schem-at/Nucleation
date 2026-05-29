# Render background + orthographic/isometric projection — Design

Date: 2026-05-29

## Goal

Add two capabilities to Nucleation's rendering API, exposed consistently across
the Rust core and the Python, WASM, and FFI bindings:

1. **Configurable background** — a solid RGBA clear color, where an alpha below
   1.0 produces a transparent (or semi-transparent) PNG.
2. **Orthographic projection** plus an **isometric** convenience preset.

Both default to current behavior, so existing callers are unaffected.

## Scope

In scope: `src/rendering/`, `src/python/`, `src/wasm/`, `src/ffi.rs`,
`tools/api_parity_exclusions.txt`, docs, and tests.

**Out of scope: the JVM binding (`nucleation-jvm/`).** It exposes no rendering
API today (only `BlockState`, `MeshResult`, `ResourcePack`, `BuildingTool`,
`SchematicBuilder`, `Shape`, `Nucleation`), so there is nothing to extend. A
follow-up can add JVM rendering wholesale if desired.

## Background facts (current pipeline)

- `RenderConfig` (`src/rendering/mod.rs:25`) fields: `width, height, yaw, pitch,
  zoom, fov, target`. `to_camera()` maps it to `CameraConfig`.
- `compute_view_proj` (`src/rendering/camera.rs`) is perspective-only: it fits
  the camera distance from the bounding-box corners projected through the fov,
  then builds `perspective(fov, aspect, near, far)`.
- The headless render target is `Rgba8UnormSrgb` and readback returns RGBA
  bytes; `encode_png` uses `image::RgbaImage`, so **alpha is preserved
  end-to-end**. Transparency only needs a configurable clear-alpha.
- The clear color is hardcoded in `render_to_view` (`src/rendering/gpu.rs:730`):
  sky-blue `(0.529, 0.808, 0.922, 1.0)` normally, black when HDRI is enabled.
- When HDRI is enabled a skybox is drawn over the cleared frame, so a custom or
  transparent background is overdrawn and effectively ignored in that mode.

## Design decisions

- **Background representation:** `background: Option<[f32; 4]>`. `None` keeps
  current behavior; `Some([r,g,b,a])` is a solid clear. One field covers both
  solid color and transparency and is fully backward-compatible.
- **Color space:** background components are **linear** 0.0–1.0 values passed
  directly to the wgpu clear, consistent with the existing hardcoded sky
  constant (so output matches the current convention; no gamma conversion).
- **Projection:** a `Projection { Perspective, Orthographic }` enum in the Rust
  core and Python; a `bool` (`orthographic`) in WASM and FFI (idiomatic for
  those surfaces). Plus an `isometric` convenience that sets orthographic +
  yaw 45° + pitch 35.264° in one call.
- **HDRI interaction:** documented — when HDRI is enabled the skybox overdraws
  the background, so `background` (including transparency) has no effect there.

## Rust core changes (`src/rendering/`)

### `mod.rs`

- Add `pub enum Projection { Perspective, Orthographic }`
  (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, default `Perspective`).
- Add to `RenderConfig`:
  - `pub background: Option<[f32; 4]>`
  - `pub projection: Projection`
- `Default for RenderConfig`: `background: None`, `projection: Perspective`.
- `RenderConfig::isometric()` → `Self { projection: Orthographic, yaw: 45.0,
  pitch: 35.264, ..Default::default() }`.
- `to_camera()` copies `projection` and `background` into `CameraConfig`.

### `camera.rs`

- Add to `CameraConfig`: `pub projection: Projection`, `pub background:
  Option<[f32; 4]>` (default `Perspective` / `None`).
- Add `pub fn ortho(left, right, bottom, top, near, far) -> [[f32; 4]; 4]`
  using the same NDC z range [0, 1] and handedness as the existing
  `perspective`.
- `compute_view_proj` branches on `camera.projection`:
  - **Perspective:** unchanged.
  - **Orthographic:** reuse the corner loop to find the max projected extent on
    the `right` axis (`ext_h`) and `up` axis (`ext_v`), and the depth range.
    Compute `half_h = max(ext_v, ext_h / aspect) * 1.1 * zoom` and
    `half_w = half_h * aspect` so both axes fit. Place the eye along `-dir` at a
    distance covering the depth extent; set `near`/`far` to bound the geometry;
    build `ortho(-half_w, half_w, -half_h, half_h, near, far)`.

### `gpu.rs`

- In `render_to_view`, replace the hardcoded `clear_color` with:
  1. `Some(bg)` on `camera.background` → use it (RGBA → `wgpu::Color`),
  2. else if `hdri_enabled` → black,
  3. else → sky-blue default.
  Read from `camera.background` (already a parameter); no new struct field.

## Binding changes

Method names are aligned across bindings where practical so the API-parity hook
stays green; the projection-shape difference (enum vs bool) is covered by an
exclusion entry.

### Python (`src/python/rendering.rs`, register in `src/python/mod.rs`)

- Expose a PyO3 unit enum `Projection { Perspective, Orthographic }`.
- Constructor signature gains `projection=Projection.Perspective`,
  `background=None` (accepts a 4-tuple/list or `None`).
- Add: `set_background(r, g, b, a)`, `clear_background()`, `background` getter
  (returns `Option<(f32,f32,f32,f32)>`), `set_projection(p)`, `projection`
  getter, and a staticmethod `isometric(width=1024, height=1024)`.
- Update `__repr__` to include projection + background.

### WASM (`src/wasm/rendering.rs`)

- Add: `setBackground(r,g,b,a)`, `clearBackground()`, `background` getter
  (returns a `Float32Array` of length 4 or `null`), `setOrthographic(bool)`,
  `orthographic` getter, and a static `isometric()` constructor.
- Thread `background` and `projection` through the `RenderConfig` built in
  `render_wasm` / `render_png_wasm`.

### FFI (`src/ffi.rs`, `rendering_ffi`)

- Add: `renderconfig_set_background(ptr, r, g, b, a)`,
  `renderconfig_clear_background(ptr)`,
  `renderconfig_set_orthographic(ptr, bool)`,
  `renderconfig_set_isometric(ptr)`.

### Parity tool

- Add entries to `tools/api_parity_exclusions.txt` for the projection naming
  difference (Python `set_projection`/`projection` enum vs WASM/FFI
  `setOrthographic`/`orthographic` bool) so `check_api_parity` passes.

## Documentation

- `docs/api-reference-python.md`, `docs/api-reference-wasm.md`,
  `docs/api-reference-ffi.md`: document the new fields/methods.
- `docs/python/README.md` and `README-python.md`: a "Backgrounds & projection"
  subsection with a transparent-PNG example and an isometric example.
- Refresh `render_example.py` to show `background` + `isometric()`.

## Testing

All config-level tests must pass without a GPU (the pre-push hook runs node and
python tests in environments that may lack one).

- **Rust (pure math, always run):** unit tests for `ortho()` (known points map
  to expected NDC), the orthographic `compute_view_proj` branch, and
  `RenderConfig::isometric()` / `to_camera()` field mapping.
- **Rust (GPU, `#[ignore]`d):** a render test that renders a single-block
  schematic and asserts (a) a transparent background yields corner-pixel
  alpha 0, and (b) a solid color background yields that color at the corners.
  Gated `#[ignore]` because CI has no GPU and there is no existing render test.
- **Python:** extend `tests/python_new_api_test.py` — build a `RenderConfig`
  with `background` + `projection`, assert getters round-trip, assert the
  `Projection` enum exists, assert `RenderConfig.isometric()` sets the expected
  fields.
- **Node/WASM:** extend `tests/node_new_api_test.mjs` — set background and
  orthographic, read the getters back.

## Verification

Run the pre-push checks: format, Rust/maturin/WASM builds, node + python tests,
API parity, version consistency.
