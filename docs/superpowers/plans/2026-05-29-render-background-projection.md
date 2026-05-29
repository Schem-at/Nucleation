# Render Background + Orthographic/Isometric Projection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a configurable RGBA background (with alpha→transparency) and orthographic/isometric projection to the renderer, exposed across the Rust core, Python, WASM, and FFI bindings, documented and tested.

**Architecture:** Two new fields on `RenderConfig` (`background: Option<[f32;4]>`, `projection: Projection`) flow through `CameraConfig` into the camera math (`compute_view_proj` gains an orthographic branch using a new `ortho()` matrix) and into the GPU clear color (a new pure `clear_color_for()` helper). Bindings expose getters/setters/constructors with names that keep the API-parity tool green.

**Tech Stack:** Rust, wgpu, PyO3 (Python), wasm-bindgen (WASM), C ABI (FFI), `image` crate for PNG.

---

## File Structure

- `src/rendering/camera.rs` — `Projection` enum, `CameraConfig` fields, `ortho()`, orthographic branch in `compute_view_proj`; unit tests.
- `src/rendering/mod.rs` — `RenderConfig` fields, `Default`, `to_camera()`, `RenderConfig::isometric()`, re-export `Projection`; unit tests.
- `src/rendering/gpu.rs` — `clear_color_for()` helper + wire into `render_to_view`; unit tests.
- `src/python/rendering.rs` — `PyProjection` enum + new methods on `PyRenderConfig`.
- `src/python/mod.rs` — register `PyProjection`.
- `src/wasm/rendering.rs` — new setters/getters + thread fields through `render_wasm`/`render_png_wasm`.
- `src/ffi.rs` — new `renderconfig_*` functions.
- `tools/api_parity_exclusions.txt` — projection naming exclusions.
- `docs/api-reference-python.md`, `docs/api-reference-wasm.md`, `docs/api-reference-ffi.md`, `docs/python/README.md`, `README-python.md`, `render_example.py` — docs.
- `tests/python_new_api_test.py`, `tests/node_new_api_test.mjs` — binding tests.

---

## Task 1: Core — `Projection` enum, `RenderConfig` fields, `isometric()`, `to_camera()`

**Files:**
- Modify: `src/rendering/camera.rs` (add `Projection`, extend `CameraConfig`)
- Modify: `src/rendering/mod.rs` (extend `RenderConfig`, `Default`, `to_camera`, add `isometric()`, re-export)
- Test: `src/rendering/mod.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test** — append to the bottom of `src/rendering/mod.rs`:

```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::rendering::camera::Projection;

    #[test]
    fn default_config_is_perspective_no_background() {
        let c = RenderConfig::default();
        assert_eq!(c.projection, Projection::Perspective);
        assert!(c.background.is_none());
    }

    #[test]
    fn isometric_sets_ortho_and_angles() {
        let c = RenderConfig::isometric();
        assert_eq!(c.projection, Projection::Orthographic);
        assert!((c.yaw - 45.0).abs() < 1e-4);
        assert!((c.pitch - 35.264).abs() < 1e-3);
    }

    #[test]
    fn to_camera_propagates_projection_and_background() {
        let mut c = RenderConfig::default();
        c.projection = Projection::Orthographic;
        c.background = Some([1.0, 0.0, 0.0, 0.5]);
        let cam = c.to_camera();
        assert_eq!(cam.projection, Projection::Orthographic);
        assert_eq!(cam.background, Some([1.0, 0.0, 0.0, 0.5]));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --features rendering --lib config_tests`
Expected: FAIL — compile errors (`Projection` undefined, `RenderConfig` has no `projection`/`background`, no `isometric`).

- [ ] **Step 3: Add `Projection` enum and extend `CameraConfig`** in `src/rendering/camera.rs`.

Add near the top of the file, after the module doc / `use` line:

```rust
/// Camera projection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    /// Standard perspective projection (default).
    Perspective,
    /// Parallel (orthographic) projection — no perspective foreshortening.
    Orthographic,
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective
    }
}
```

Then extend `CameraConfig` (add the two fields) and its `Default`:

```rust
pub struct CameraConfig {
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub zoom: f32,
    pub fov_deg: f32,
    /// Optional explicit orbit target. When set, the camera orbits and
    /// aims at this point instead of the model's bounding-box centroid.
    pub target: Option<[f32; 3]>,
    /// Projection mode.
    pub projection: Projection,
    /// Optional solid RGBA clear color (linear 0.0–1.0). `None` uses the
    /// default sky / HDRI behavior.
    pub background: Option<[f32; 4]>,
}
```

In `impl Default for CameraConfig`, add the two new fields:

```rust
            target: None,
            projection: Projection::Perspective,
            background: None,
```

- [ ] **Step 4: Extend `RenderConfig` in `src/rendering/mod.rs`.**

Add a re-export near the other `use`/`pub use` lines at the top of `mod.rs`:

```rust
pub use camera::Projection;
```

Add the two fields to the `RenderConfig` struct (after `target`):

```rust
    /// Optional solid RGBA clear color (linear 0.0–1.0). `None` keeps the
    /// default sky-blue clear (or the HDRI sky when HDRI is enabled). An
    /// alpha below 1.0 produces a transparent PNG. Ignored when HDRI is on.
    pub background: Option<[f32; 4]>,
    /// Camera projection mode (default `Perspective`).
    pub projection: Projection,
```

In `impl Default for RenderConfig`, add after `target: None,`:

```rust
            background: None,
            projection: Projection::Perspective,
```

In `RenderConfig::to_camera`, add the two fields to the returned `CameraConfig`:

```rust
            target: self.target,
            projection: self.projection,
            background: self.background,
```

Add an `isometric` constructor inside `impl RenderConfig` (same block as `to_camera`):

```rust
    /// A config preset for a true isometric view: orthographic projection at
    /// yaw 45° and pitch ≈35.264° (`arctan(1/√2)`).
    pub fn isometric() -> Self {
        Self {
            yaw: 45.0,
            pitch: 35.264,
            projection: Projection::Orthographic,
            ..Self::default()
        }
    }
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --features rendering --lib config_tests`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add src/rendering/camera.rs src/rendering/mod.rs
git commit -m "feat(render): add Projection enum + background/projection fields to RenderConfig"
```

---

## Task 2: Core — `ortho()` projection matrix

**Files:**
- Modify: `src/rendering/camera.rs` (add `ortho`)
- Test: `src/rendering/camera.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test** — append to the bottom of `src/rendering/camera.rs`:

```rust
#[cfg(test)]
mod ortho_tests {
    use super::*;

    // Matrices are column-major: ndc[j] = Σ_i m[i][j] * v[i].
    fn transform(m: [[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        for j in 0..4 {
            out[j] = m[0][j] * v[0] + m[1][j] * v[1] + m[2][j] * v[2] + m[3][j] * v[3];
        }
        out
    }

    #[test]
    fn ortho_maps_box_to_ndc() {
        // left/right/bottom/top = ±1, near=1, far=3. View-space looks down -z,
        // so view z = -near maps to NDC z 0, view z = -far maps to NDC z 1.
        let m = ortho(-1.0, 1.0, -1.0, 1.0, 1.0, 3.0);
        let ndc = transform(m, [0.5, -0.5, -2.0, 1.0]);
        assert!((ndc[0] - 0.5).abs() < 1e-5, "x={}", ndc[0]);
        assert!((ndc[1] + 0.5).abs() < 1e-5, "y={}", ndc[1]);
        assert!((ndc[2] - 0.5).abs() < 1e-5, "z={}", ndc[2]); // mid-depth
        assert!((ndc[3] - 1.0).abs() < 1e-5, "w={}", ndc[3]); // no perspective divide
    }

    #[test]
    fn ortho_near_and_far_planes() {
        let m = ortho(-2.0, 2.0, -2.0, 2.0, 1.0, 5.0);
        let near = transform(m, [0.0, 0.0, -1.0, 1.0]);
        let far = transform(m, [0.0, 0.0, -5.0, 1.0]);
        assert!((near[2] - 0.0).abs() < 1e-5, "near z={}", near[2]);
        assert!((far[2] - 1.0).abs() < 1e-5, "far z={}", far[2]);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --features rendering --lib ortho_tests`
Expected: FAIL — `ortho` not found.

- [ ] **Step 3: Add `ortho` in `src/rendering/camera.rs`** (immediately after the `perspective` fn):

```rust
/// Orthographic projection matrix matching the wgpu NDC convention (z in
/// [0, 1]) and the same right-handed, looking-down-`-z` view space as
/// [`perspective`]. Column-major storage to match the rest of this module.
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let rl = 1.0 / (right - left);
    let tb = 1.0 / (top - bottom);
    let nf = 1.0 / (near - far);
    [
        [2.0 * rl, 0.0, 0.0, 0.0],
        [0.0, 2.0 * tb, 0.0, 0.0],
        [0.0, 0.0, nf, 0.0],
        [-(right + left) * rl, -(top + bottom) * tb, near * nf, 1.0],
    ]
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --features rendering --lib ortho_tests`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/rendering/camera.rs
git commit -m "feat(render): add orthographic projection matrix"
```

---

## Task 3: Core — orthographic branch in `compute_view_proj`

**Files:**
- Modify: `src/rendering/camera.rs` (`compute_view_proj`)
- Test: `src/rendering/camera.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test** — append to the bottom of `src/rendering/camera.rs`:

```rust
#[cfg(test)]
mod view_proj_tests {
    use super::*;

    fn transform(m: [[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        for j in 0..4 {
            out[j] = m[0][j] * v[0] + m[1][j] * v[1] + m[2][j] * v[2] + m[3][j] * v[3];
        }
        out
    }

    #[test]
    fn orthographic_fits_all_corners_in_ndc() {
        let cam = CameraConfig {
            yaw_deg: 30.0,
            pitch_deg: 25.0,
            zoom: 1.0,
            fov_deg: 45.0,
            target: None,
            projection: Projection::Orthographic,
            background: None,
        };
        let (vp, _) = compute_view_proj([0.0, 0.0, 0.0], [4.0, 2.0, 6.0], 16.0 / 9.0, &cam);

        let corners = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [4.0, 2.0, 0.0],
            [0.0, 0.0, 6.0],
            [4.0, 0.0, 6.0],
            [0.0, 2.0, 6.0],
            [4.0, 2.0, 6.0],
        ];
        let mut max_xy = 0.0f32;
        for c in &corners {
            let ndc = transform(vp, [c[0], c[1], c[2], 1.0]);
            // Orthographic: w stays 1 (no perspective divide).
            assert!((ndc[3] - 1.0).abs() < 1e-4, "w={}", ndc[3]);
            // All geometry inside the clip box.
            assert!(ndc[0].abs() <= 1.05, "x out of range: {}", ndc[0]);
            assert!(ndc[1].abs() <= 1.05, "y out of range: {}", ndc[1]);
            assert!(ndc[2] >= -0.001 && ndc[2] <= 1.001, "z out of range: {}", ndc[2]);
            max_xy = max_xy.max(ndc[0].abs()).max(ndc[1].abs());
        }
        // The framing should roughly fill the viewport (1/1.1 ≈ 0.9).
        assert!(max_xy > 0.8, "geometry too small in frame: {}", max_xy);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --features rendering --lib view_proj_tests`
Expected: FAIL — perspective path keeps `w = -z ≠ 1`, so the `w == 1` assertion fails (no orthographic branch yet).

- [ ] **Step 3: Replace `compute_view_proj`** in `src/rendering/camera.rs` with the branching version. Replace the entire existing `pub fn compute_view_proj(...) { ... }` body with:

```rust
pub fn compute_view_proj(
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    aspect: f32,
    camera: &CameraConfig,
) -> ([[f32; 4]; 4], [[f32; 4]; 4]) {
    let center = camera.target.unwrap_or([
        (bounds_min[0] + bounds_max[0]) * 0.5,
        (bounds_min[1] + bounds_max[1]) * 0.5,
        (bounds_min[2] + bounds_max[2]) * 0.5,
    ]);

    let yaw = camera.yaw_deg.to_radians();
    let pitch = camera.pitch_deg.to_radians();
    let fov = camera.fov_deg.to_radians();

    let dir = normalize3([
        -(pitch.cos() * yaw.sin()),
        -(pitch.sin()),
        -(pitch.cos() * yaw.cos()),
    ]);

    let forward = dir;
    let right = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
    let up = cross3(right, forward);

    let corners = [
        [bounds_min[0], bounds_min[1], bounds_min[2]],
        [bounds_max[0], bounds_min[1], bounds_min[2]],
        [bounds_min[0], bounds_max[1], bounds_min[2]],
        [bounds_max[0], bounds_max[1], bounds_min[2]],
        [bounds_min[0], bounds_min[1], bounds_max[2]],
        [bounds_max[0], bounds_min[1], bounds_max[2]],
        [bounds_min[0], bounds_max[1], bounds_max[2]],
        [bounds_max[0], bounds_max[1], bounds_max[2]],
    ];

    let (view_proj, inv_view_proj) = match camera.projection {
        Projection::Perspective => {
            let half_fov_y = fov * 0.5;
            let half_fov_x = (half_fov_y.tan() * aspect).atan();

            let mut max_dist = 1.0f32;
            for c in &corners {
                let rel = sub3(*c, center);
                let proj_right = dot3(rel, right).abs();
                let proj_up = dot3(rel, up).abs();
                let proj_depth = -dot3(rel, forward);
                let dist_h = proj_right / half_fov_x.tan() + proj_depth;
                let dist_v = proj_up / half_fov_y.tan() + proj_depth;
                max_dist = max_dist.max(dist_h).max(dist_v);
            }

            let distance = max_dist * 1.1 * camera.zoom;
            let eye = [
                center[0] - dir[0] * distance,
                center[1] - dir[1] * distance,
                center[2] - dir[2] * distance,
            ];

            let view = look_at(eye, center, [0.0, 1.0, 0.0]);
            let near = distance * 0.01;
            let far = distance * 10.0;
            let proj = perspective(fov, aspect, near, far);
            let view_proj = mat4_mul(proj, view);
            (view_proj, mat4_inverse(view_proj))
        }
        Projection::Orthographic => {
            let mut ext_h = 0.0f32;
            let mut ext_v = 0.0f32;
            let mut ext_depth = 0.0f32;
            for c in &corners {
                let rel = sub3(*c, center);
                ext_h = ext_h.max(dot3(rel, right).abs());
                ext_v = ext_v.max(dot3(rel, up).abs());
                ext_depth = ext_depth.max(dot3(rel, forward).abs());
            }

            // Half-extents of the ortho window, fitting both axes, scaled by zoom.
            let half_h = (ext_v.max(ext_h / aspect)).max(0.5) * 1.1 * camera.zoom;
            let half_w = half_h * aspect;

            // Stand far enough back that all geometry sits between near and far.
            let standoff = ext_depth + ext_h + ext_v + 1.0;
            let eye = [
                center[0] - dir[0] * standoff,
                center[1] - dir[1] * standoff,
                center[2] - dir[2] * standoff,
            ];

            let view = look_at(eye, center, [0.0, 1.0, 0.0]);
            let near = 0.01;
            let far = standoff * 2.0 + 1.0;
            let proj = ortho(-half_w, half_w, -half_h, half_h, near, far);
            let view_proj = mat4_mul(proj, view);
            (view_proj, mat4_inverse(view_proj))
        }
    };

    (view_proj, inv_view_proj)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --features rendering --lib view_proj_tests ortho_tests`
Expected: PASS (all). Also run the whole rendering test set to confirm no perspective regression: `cargo test --features rendering --lib rendering` then `cargo test --features rendering --lib camera`.

- [ ] **Step 5: Commit**

```bash
git add src/rendering/camera.rs
git commit -m "feat(render): orthographic branch in compute_view_proj"
```

---

## Task 4: Core — configurable clear color (`clear_color_for` helper)

**Files:**
- Modify: `src/rendering/gpu.rs` (add `clear_color_for`, use it in `render_to_view`)
- Test: `src/rendering/gpu.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test** — append to the bottom of `src/rendering/gpu.rs`:

```rust
#[cfg(test)]
mod clear_color_tests {
    use super::*;

    #[test]
    fn custom_background_used_verbatim() {
        let c = clear_color_for(Some([0.2, 0.4, 0.6, 0.0]), false);
        assert!((c.r - 0.2).abs() < 1e-6);
        assert!((c.g - 0.4).abs() < 1e-6);
        assert!((c.b - 0.6).abs() < 1e-6);
        assert!((c.a - 0.0).abs() < 1e-6); // transparent
    }

    #[test]
    fn custom_background_wins_over_hdri() {
        let c = clear_color_for(Some([1.0, 1.0, 1.0, 1.0]), true);
        assert!((c.r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_no_hdri_is_sky_blue() {
        let c = clear_color_for(None, false);
        assert!((c.r - 0.529).abs() < 1e-3);
        assert!((c.g - 0.808).abs() < 1e-3);
        assert!((c.b - 0.922).abs() < 1e-3);
        assert!((c.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_with_hdri_is_black() {
        let c = clear_color_for(None, true);
        assert!((c.r) < 1e-6 && (c.g) < 1e-6 && (c.b) < 1e-6);
        assert!((c.a - 1.0).abs() < 1e-6);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --features rendering --lib clear_color_tests`
Expected: FAIL — `clear_color_for` not found.

- [ ] **Step 3: Add `clear_color_for`** in `src/rendering/gpu.rs` (place it as a free fn near the top of the file, after the imports):

```rust
/// Choose the render-pass clear color. A custom `background` (linear RGBA)
/// always wins; otherwise black when an HDRI sky is drawn, else the default
/// sky-blue.
fn clear_color_for(background: Option<[f32; 4]>, hdri_enabled: bool) -> wgpu::Color {
    if let Some(bg) = background {
        wgpu::Color {
            r: bg[0] as f64,
            g: bg[1] as f64,
            b: bg[2] as f64,
            a: bg[3] as f64,
        }
    } else if hdri_enabled {
        wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    } else {
        wgpu::Color { r: 0.529, g: 0.808, b: 0.922, a: 1.0 }
    }
}
```

- [ ] **Step 4: Wire it into `render_to_view`.** In `src/rendering/gpu.rs`, replace the existing hardcoded `let clear_color = if self.hdri_enabled { ... } else { ... };` block with:

```rust
        let clear_color = clear_color_for(camera.background, self.hdri_enabled);
```

(`camera` is the `&CameraConfig` parameter already in scope in `render_to_view`.)

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --features rendering --lib clear_color_tests`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit**

```bash
git add src/rendering/gpu.rs
git commit -m "feat(render): configurable clear color via camera.background"
```

> **Note on the GPU pixel test:** an end-to-end render-and-read-back-pixels test would require GPU access (absent in CI) and a resource-pack fixture. The pure `clear_color_for` test above covers the new branching logic deterministically. End-to-end visual confirmation is covered by the manual verification step in Task 11.

---

## Task 5: Python binding — `Projection` enum + `RenderConfig` methods

**Files:**
- Modify: `src/python/rendering.rs`
- Modify: `src/python/mod.rs` (register enum)
- Test: `tests/python_new_api_test.py`

- [ ] **Step 1: Write the failing test** — append to `tests/python_new_api_test.py`:

```python
def test_render_config_background_and_projection():
    from nucleation import RenderConfig, Projection

    # Background round-trips; alpha < 1 allowed.
    cfg = RenderConfig(width=64, height=64, background=(1.0, 0.0, 0.0, 0.5))
    assert cfg.background == (1.0, 0.0, 0.0, 0.5)
    cfg.clear_background()
    assert cfg.background is None
    cfg.set_background(0.0, 1.0, 0.0, 1.0)
    assert cfg.background == (0.0, 1.0, 0.0, 1.0)

    # Projection enum round-trips.
    assert cfg.projection == Projection.Perspective
    cfg.set_projection(Projection.Orthographic)
    assert cfg.projection == Projection.Orthographic

    # Isometric preset.
    iso = RenderConfig.isometric(width=128, height=128)
    assert iso.width == 128
    assert iso.projection == Projection.Orthographic
    assert abs(iso.yaw - 45.0) < 1e-4
    assert abs(iso.pitch - 35.264) < 1e-3
```

- [ ] **Step 2: Run test to verify it fails**

Run: `maturin develop --features python,rendering && python -m pytest tests/python_new_api_test.py::test_render_config_background_and_projection -v`
Expected: FAIL — `ImportError: cannot import name 'Projection'` (or `RenderConfig` has no `background`).

- [ ] **Step 3: Add the `PyProjection` enum** in `src/python/rendering.rs`. After the `use` lines, add:

```rust
use crate::rendering::Projection;

/// Camera projection mode.
#[pyclass(name = "Projection", eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum PyProjection {
    Perspective,
    Orthographic,
}

impl From<PyProjection> for Projection {
    fn from(p: PyProjection) -> Self {
        match p {
            PyProjection::Perspective => Projection::Perspective,
            PyProjection::Orthographic => Projection::Orthographic,
        }
    }
}

impl From<Projection> for PyProjection {
    fn from(p: Projection) -> Self {
        match p {
            Projection::Perspective => PyProjection::Perspective,
            Projection::Orthographic => PyProjection::Orthographic,
        }
    }
}
```

- [ ] **Step 4: Extend the constructor and add methods** in `src/python/rendering.rs`.

Replace the existing `#[new] #[pyo3(signature = ...)] pub fn new(...) -> Self { ... }` with:

```rust
    #[new]
    #[pyo3(signature = (width=1024, height=1024, yaw=45.0, pitch=30.0, zoom=1.0, fov=45.0, target=None, background=None, projection=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        zoom: f32,
        fov: f32,
        target: Option<(f32, f32, f32)>,
        background: Option<(f32, f32, f32, f32)>,
        projection: Option<PyProjection>,
    ) -> Self {
        Self {
            inner: RenderConfig {
                width,
                height,
                yaw,
                pitch,
                zoom,
                fov,
                target: target.map(|(x, y, z)| [x, y, z]),
                background: background.map(|(r, g, b, a)| [r, g, b, a]),
                projection: projection.unwrap_or(PyProjection::Perspective).into(),
            },
        }
    }
```

Add the following methods inside the `#[pymethods] impl PyRenderConfig` block (e.g. just before `fn __repr__`):

```rust
    /// Current background as `(r, g, b, a)` in linear 0.0–1.0, or `None`.
    #[getter]
    pub fn background(&self) -> Option<(f32, f32, f32, f32)> {
        self.inner.background.map(|c| (c[0], c[1], c[2], c[3]))
    }

    /// Set a solid RGBA clear color (linear 0.0–1.0). An alpha below 1.0
    /// produces a transparent PNG. Ignored when HDRI is enabled.
    pub fn set_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.background = Some([r, g, b, a]);
    }

    /// Clear the custom background — revert to the default sky / HDRI.
    pub fn clear_background(&mut self) {
        self.inner.background = None;
    }

    /// Current projection mode.
    #[getter]
    pub fn projection(&self) -> PyProjection {
        self.inner.projection.into()
    }

    #[setter]
    pub fn set_projection(&mut self, value: PyProjection) {
        self.inner.projection = value.into();
    }

    /// Preset for a true isometric view: orthographic at yaw 45° / pitch ≈35.264°.
    #[staticmethod]
    #[pyo3(signature = (width=1024, height=1024))]
    pub fn isometric(width: u32, height: u32) -> Self {
        let mut inner = RenderConfig::isometric();
        inner.width = width;
        inner.height = height;
        Self { inner }
    }
```

Replace `fn __repr__` to include the new state:

```rust
    fn __repr__(&self) -> String {
        let proj = match self.inner.projection {
            crate::rendering::Projection::Perspective => "perspective",
            crate::rendering::Projection::Orthographic => "orthographic",
        };
        format!(
            "<RenderConfig {}x{} yaw={} pitch={} zoom={} fov={} projection={} background={:?}>",
            self.inner.width,
            self.inner.height,
            self.inner.yaw,
            self.inner.pitch,
            self.inner.zoom,
            self.inner.fov,
            proj,
            self.inner.background
        )
    }
```

- [ ] **Step 5: Register the enum** in `src/python/mod.rs`. In the `#[cfg(feature = "rendering")]` block, add after the `PyRenderConfig` line:

```rust
        m.add_class::<rendering::PyProjection>()?;
```

- [ ] **Step 6: Run test to verify it passes**

Run: `maturin develop --features python,rendering && python -m pytest tests/python_new_api_test.py::test_render_config_background_and_projection -v`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/python/rendering.rs src/python/mod.rs tests/python_new_api_test.py
git commit -m "feat(python): expose background + Projection on RenderConfig"
```

---

## Task 6: WASM binding — setters/getters + thread fields through render

**Files:**
- Modify: `src/wasm/rendering.rs`
- Test: `tests/node_new_api_test.mjs`

- [ ] **Step 1: Write the failing test** — append to `tests/node_new_api_test.mjs` (follow the file's existing import/test style; this uses the same `RenderConfig` wrapper the file already references):

```javascript
import assert from "node:assert";
import { RenderConfig } from "../pkg/nucleation.js";

{
    const cfg = new RenderConfig();
    // Background setter/getter round-trips, alpha < 1 allowed.
    cfg.setBackground(1.0, 0.0, 0.0, 0.5);
    const bg = cfg.background;
    assert.ok(bg, "background should be set");
    assert.strictEqual(bg.length, 4);
    assert.ok(Math.abs(bg[3] - 0.5) < 1e-6, "alpha preserved");
    cfg.clearBackground();
    assert.strictEqual(cfg.background, null, "background cleared");

    // Orthographic toggle.
    assert.strictEqual(cfg.orthographic, false);
    cfg.setOrthographic(true);
    assert.strictEqual(cfg.orthographic, true);

    // Isometric static constructor.
    const iso = RenderConfig.isometric();
    assert.strictEqual(iso.orthographic, true);
    assert.ok(Math.abs(iso.yaw - 45.0) < 1e-4);
    assert.ok(Math.abs(iso.pitch - 35.264) < 1e-3);
    console.log("RenderConfig background/projection test passed");
}
```

> If `tests/node_new_api_test.mjs` already has a single default export / runner, place the assertions inside that runner following the existing pattern instead of a bare block, and import `RenderConfig` from the same module path the rest of the file uses.

- [ ] **Step 2: Build WASM and run test to verify it fails**

Run: `wasm-pack build --target nodejs --out-dir pkg -- --features rendering && node tests/node_new_api_test.mjs`
Expected: FAIL — `cfg.setBackground is not a function`.

- [ ] **Step 3: Add the methods** inside `#[wasm_bindgen] impl RenderConfigWrapper` in `src/wasm/rendering.rs` (e.g. after the `target` getter, before the closing `}` of the impl):

```rust
    /// Set a solid RGBA clear color (linear 0.0–1.0). An alpha below 1.0
    /// produces a transparent PNG. Ignored when HDRI is enabled.
    #[wasm_bindgen(js_name = setBackground)]
    pub fn set_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.background = Some([r, g, b, a]);
    }

    /// Clear the custom background — revert to the default sky / HDRI.
    #[wasm_bindgen(js_name = clearBackground)]
    pub fn clear_background(&mut self) {
        self.inner.background = None;
    }

    /// Current background as `[r, g, b, a]`, or `null` if none.
    #[wasm_bindgen(getter, js_name = background)]
    pub fn background(&self) -> Option<Vec<f32>> {
        self.inner.background.map(|c| c.to_vec())
    }

    /// Enable or disable orthographic projection (default: perspective).
    #[wasm_bindgen(js_name = setOrthographic)]
    pub fn set_orthographic(&mut self, value: bool) {
        self.inner.projection = if value {
            crate::rendering::Projection::Orthographic
        } else {
            crate::rendering::Projection::Perspective
        };
    }

    /// Whether orthographic projection is enabled.
    #[wasm_bindgen(getter, js_name = orthographic)]
    pub fn orthographic(&self) -> bool {
        matches!(self.inner.projection, crate::rendering::Projection::Orthographic)
    }

    /// Preset for a true isometric view: orthographic at yaw 45° / pitch ≈35.264°.
    #[wasm_bindgen(js_name = isometric)]
    pub fn isometric() -> RenderConfigWrapper {
        RenderConfigWrapper {
            inner: crate::rendering::RenderConfig::isometric(),
        }
    }
```

- [ ] **Step 4: Thread the fields through both render entry points.** In `render_wasm` and `render_png_wasm` (both in `src/wasm/rendering.rs`), the `let render_config = RenderConfig { ... }` literal currently lists fields explicitly. In **each** of the two, add the two new fields after `target: config.inner.target,`:

```rust
            target: config.inner.target,
            background: config.inner.background,
            projection: config.inner.projection,
```

- [ ] **Step 5: Build WASM and run test to verify it passes**

Run: `wasm-pack build --target nodejs --out-dir pkg -- --features rendering && node tests/node_new_api_test.mjs`
Expected: PASS — prints "RenderConfig background/projection test passed".

- [ ] **Step 6: Commit**

```bash
git add src/wasm/rendering.rs tests/node_new_api_test.mjs
git commit -m "feat(wasm): expose background + orthographic on RenderConfig"
```

---

## Task 7: FFI binding — `renderconfig_*` functions

**Files:**
- Modify: `src/ffi.rs`
- Test: `tests/ffi_helpers_test.rs` (add a Rust-level test exercising the new FFI fns)

- [ ] **Step 1: Write the failing test** — append to `tests/ffi_helpers_test.rs`:

```rust
#[cfg(feature = "rendering")]
#[test]
fn ffi_renderconfig_background_and_projection() {
    use nucleation::ffi::rendering_ffi::*;

    unsafe {
        let cfg = renderconfig_new(64, 64);
        assert!(!cfg.is_null());

        // These must not panic and must be null-safe.
        renderconfig_set_background(cfg, 1.0, 0.0, 0.0, 0.5);
        renderconfig_clear_background(cfg);
        renderconfig_set_orthographic(cfg, true);
        renderconfig_set_isometric(cfg);

        // Null-pointer safety.
        renderconfig_set_background(std::ptr::null_mut(), 0.0, 0.0, 0.0, 1.0);
        renderconfig_set_orthographic(std::ptr::null_mut(), true);

        renderconfig_free(cfg);
    }
}
```

> If `nucleation::ffi::rendering_ffi` is not publicly reachable, instead place this test inside `src/ffi.rs` under a `#[cfg(test)] mod ffi_render_tests` block in the same module so the `renderconfig_*` fns are in scope; adjust the `use` accordingly.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --features rendering --test ffi_helpers_test ffi_renderconfig_background_and_projection`
Expected: FAIL — the four new functions are undefined.

- [ ] **Step 3: Add the functions** in `src/ffi.rs`, in the `rendering_ffi` module, after `renderconfig_set_fov`:

```rust
    /// Set a solid RGBA clear color (linear 0.0–1.0). Alpha < 1.0 yields a
    /// transparent PNG. Ignored when HDRI is enabled.
    #[no_mangle]
    pub extern "C" fn renderconfig_set_background(
        ptr: *mut FFIRenderConfig,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.background = Some([r, g, b, a]) };
        }
    }

    /// Clear the custom background — revert to default sky / HDRI.
    #[no_mangle]
    pub extern "C" fn renderconfig_clear_background(ptr: *mut FFIRenderConfig) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.background = None };
        }
    }

    /// Enable (`true`) or disable orthographic projection.
    #[no_mangle]
    pub extern "C" fn renderconfig_set_orthographic(ptr: *mut FFIRenderConfig, orthographic: bool) {
        if !ptr.is_null() {
            unsafe {
                (*ptr).0.projection = if orthographic {
                    rendering::Projection::Orthographic
                } else {
                    rendering::Projection::Perspective
                };
            }
        }
    }

    /// Configure a true isometric view: orthographic at yaw 45° / pitch ≈35.264°
    /// (preserves the current width/height).
    #[no_mangle]
    pub extern "C" fn renderconfig_set_isometric(ptr: *mut FFIRenderConfig) {
        if !ptr.is_null() {
            unsafe {
                let w = (*ptr).0.width;
                let h = (*ptr).0.height;
                let mut iso = rendering::RenderConfig::isometric();
                iso.width = w;
                iso.height = h;
                (*ptr).0 = iso;
            }
        }
    }
```

(`rendering` is already imported at the top of the `rendering_ffi` module via `use crate::rendering::{self, RenderConfig};`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --features rendering --test ffi_helpers_test ffi_renderconfig_background_and_projection`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/ffi.rs tests/ffi_helpers_test.rs
git commit -m "feat(ffi): renderconfig background + orthographic/isometric setters"
```

---

## Task 8: API parity exclusions

**Files:**
- Modify: `tools/api_parity_exclusions.txt`

- [ ] **Step 1: Add projection-naming exclusions.** Append under the existing `# --- Rendering exclusions ---` area of `tools/api_parity_exclusions.txt`:

```
# Projection: Python exposes a Projection enum (set_projection/projection);
# WASM/FFI use an orthographic bool (setOrthographic/orthographic).
[no_wasm]
RenderConfig.set_projection = Python enum API (WASM uses setOrthographic bool)
RenderConfig.projection = Python enum API (WASM uses orthographic bool)

[no_python]
RenderConfig.set_orthographic = WASM bool API (Python uses set_projection enum)
RenderConfig.orthographic = WASM bool API (Python uses projection enum)
```

> `RenderConfig.*` is already excluded for FFI (`[no_ffi]`), and `renderconfig_*` is already `[ffi_only]`, so the FFI side needs no new entries. `set_background`/`clear_background`/`background`/`isometric` match between Python and WASM and need no exclusion.

- [ ] **Step 2: Build the parity checker and run it**

Run:
```bash
rustc tools/check_api_parity.rs -o target/check_api_parity && ./target/check_api_parity
```
Expected: exit 0 / "parity OK" (no unexpected mismatches). If it reports a still-missing `RenderConfig` method, add the matching exclusion line in the same style and re-run until clean.

- [ ] **Step 3: Commit**

```bash
git add tools/api_parity_exclusions.txt
git commit -m "chore(parity): exclude projection naming differences for RenderConfig"
```

---

## Task 9: Documentation

**Files:**
- Modify: `docs/api-reference-python.md`, `docs/api-reference-wasm.md`, `docs/api-reference-ffi.md`
- Modify: `docs/python/README.md`, `README-python.md`
- Modify: `render_example.py`

- [ ] **Step 1: Update `docs/api-reference-python.md`.** Find the `RenderConfig` section and add the new parameters and methods. Insert:

```markdown
#### Backgrounds & projection

`RenderConfig` accepts an optional solid background and a projection mode:

- `background=(r, g, b, a)` — linear RGBA 0.0–1.0. An alpha below `1.0`
  produces a transparent PNG. Defaults to `None` (sky-blue, or the HDRI sky
  when an HDRI is set — in which case `background` is ignored).
- `projection=Projection.Perspective | Projection.Orthographic`.

Methods: `set_background(r, g, b, a)`, `clear_background()`, `background`
(getter), `set_projection(p)`, `projection` (getter), and the static
`RenderConfig.isometric(width=1024, height=1024)` preset (orthographic at
yaw 45° / pitch ≈35.264°).

```python
from nucleation import Schematic, ResourcePack, RenderConfig, Projection

pack = ResourcePack.from_file("pack.zip")
schem = Schematic.open("f-117-nighthawk.litematic")

# Transparent PNG, isometric framing.
cfg = RenderConfig.isometric(width=1024, height=768)
cfg.set_background(0.0, 0.0, 0.0, 0.0)  # fully transparent
schem.render_to_file(pack, "night.png", cfg)
```
```

- [ ] **Step 2: Update `docs/api-reference-wasm.md`.** In the `RenderConfig` section add:

```markdown
#### Backgrounds & projection

- `setBackground(r, g, b, a)` / `clearBackground()` / `background` getter
  (returns `[r,g,b,a]` or `null`) — linear RGBA 0.0–1.0; alpha < 1.0 → transparent PNG.
- `setOrthographic(bool)` / `orthographic` getter.
- `RenderConfig.isometric()` — static constructor (orthographic, yaw 45° / pitch ≈35.264°).

```js
const cfg = RenderConfig.isometric();
cfg.setBackground(0, 0, 0, 0); // transparent
const png = await schem.renderPng(pack, cfg);
```
```

- [ ] **Step 3: Update `docs/api-reference-ffi.md`.** In the rendering section add:

```markdown
#### Background & projection

- `void renderconfig_set_background(RenderConfig*, float r, float g, float b, float a)` — linear RGBA 0.0–1.0; alpha < 1.0 → transparent PNG.
- `void renderconfig_clear_background(RenderConfig*)`
- `void renderconfig_set_orthographic(RenderConfig*, bool orthographic)`
- `void renderconfig_set_isometric(RenderConfig*)` — orthographic, yaw 45° / pitch ≈35.264°, preserves width/height.
```

- [ ] **Step 4: Update `docs/python/README.md` and `README-python.md`.** Add a short "Backgrounds & projection" subsection to each rendering section, reusing the Python example from Step 1 (transparent + isometric).

- [ ] **Step 5: Refresh `render_example.py`** to its final form:

```python
from nucleation import Schematic, ResourcePack, RenderConfig, Projection

pack = ResourcePack.from_file("pack.zip")
schem = Schematic.open("f-117-nighthawk.litematic")

# Isometric framing with a fully transparent background.
cfg = RenderConfig.isometric(width=1024, height=768)
cfg.zoom = 0.85          # tighten the framing (smaller zoom => bigger object)
cfg.set_background(0.0, 0.0, 0.0, 0.0)  # transparent PNG

# render_to_file(pack, path, config). For a perspective view with a solid
# background instead:
#   cfg = RenderConfig(width=1024, height=768, fov=28.0)
#   cfg.set_background(0.05, 0.05, 0.08, 1.0)
schem.render_to_file(pack, "night.png", cfg)
```

- [ ] **Step 6: Commit**

```bash
git add docs/api-reference-python.md docs/api-reference-wasm.md docs/api-reference-ffi.md docs/python/README.md README-python.md render_example.py
git commit -m "docs(render): document background + orthographic/isometric across bindings"
```

---

## Task 10: Full build + binding test sweep

**Files:** none (verification)

- [ ] **Step 1: Rust core tests**

Run: `cargo test --features rendering --lib`
Expected: PASS, including `config_tests`, `ortho_tests`, `view_proj_tests`, `clear_color_tests`.

- [ ] **Step 2: Format + clippy**

Run: `cargo fmt --all && cargo clippy --features rendering --lib -- -D warnings`
Expected: clean.

- [ ] **Step 3: Python build + tests**

Run: `maturin develop --features python,rendering && python -m pytest tests/python_new_api_test.py -v`
Expected: PASS.

- [ ] **Step 4: WASM build + node test**

Run: `wasm-pack build --target nodejs --out-dir pkg -- --features rendering && node tests/node_new_api_test.mjs`
Expected: PASS.

- [ ] **Step 5: FFI test**

Run: `cargo test --features rendering --test ffi_helpers_test`
Expected: PASS.

- [ ] **Step 6: Commit** (only if fmt changed files)

```bash
git add -A && git commit -m "chore: fmt after render background/projection work" || echo "nothing to commit"
```

---

## Task 11: Pre-push verification + manual visual check

**Files:** none (verification)

- [ ] **Step 1: Run the project pre-push hook checks**

Run: `bash pre-push.sh`
Expected: all 8 checks pass (format, builds, tests, maturin, wasm, node tests, version consistency, API parity).

- [ ] **Step 2: Manual visual confirmation (optional, requires a GPU + resource pack)**

Run: `python render_example.py` against a real `pack.zip` + schematic.
Expected: `night.png` is an isometric view with a transparent background (open in an editor that shows alpha; corners are transparent). Then flip to the perspective/solid-background variant in the file and confirm the background color renders.

- [ ] **Step 3: Final commit if anything was adjusted**

```bash
git add -A && git commit -m "chore: finalize render background + projection feature" || echo "clean"
```

---

## Self-Review Notes

- **Spec coverage:** background `Option<[f32;4]>` (Tasks 1, 4–7); `Projection` enum + `isometric()` (Tasks 1–3, 5–7); orthographic camera math (Tasks 2–3); clear-color wiring incl. HDRI precedence (Task 4); Python/WASM/FFI bindings (Tasks 5–7); parity (Task 8); docs (Task 9); tests at every layer (Tasks 1–7, 10–11). JVM intentionally excluded per spec.
- **Refinement vs spec:** the spec's `#[ignore]`d GPU pixel test is replaced by the pure `clear_color_for` unit test (Task 4) — deterministic, no GPU/fixture, covers the same logic. End-to-end visuals are a manual step (Task 11).
- **Type consistency:** `Projection { Perspective, Orthographic }` in core; `PyProjection` mirrors it with `From` both ways; WASM/FFI use a `bool` mapped to the same enum; `background: Option<[f32;4]>` is consistent everywhere (Python tuple ↔ array, WASM `Vec<f32>`/`null`, FFI 4 floats).
- **Feature gating:** all rendering code is built/tested with `--features rendering` (and `python`/WASM targets as appropriate), matching the existing `#[cfg(feature = "rendering")]` gates.
