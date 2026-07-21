# Build animator — design

**Status:** proposed, 2026-07-21
**Depends on:** wgpu 30 upgrade (done)
**Explicitly out of scope:** dynamic lights, shadow mapping — see
[renderer-lighting-deferred.md](renderer-lighting-deferred.md)

## Goal

Animate a build: which block is where, at what pose, at time *t*. A deterministic,
GPU-free data model in Rust, exposed to all seven languages, plus the minimal
renderer support needed to draw it.

Two motivating cases drive the design, and they pull in different directions:

1. **Brush passes** — blocks appear in the order the building tool placed them.
   Ordering comes from *build history*.
2. **Trefoil knot** — blocks fly into position along the curve. Ordering comes
   from *shape geometry* (`parameter_at`).

A third falls out for free: **layer printing** (the existing `printer.gif`), and
a fourth is nearly free: **diff playback** (animate `Diff::added`).

## Why this fits what already exists

- `ShapeEnum::parameter_at(x, y, z) -> Option<f64>` already exists for Line,
  Cylinder, Cone, Torus, Pyramid, BezierCurve, Hollow.
- `BuildingTool::fill_enum_masked` **already computes that `t` per point** and
  passes it to the brush. Recording it is a few lines in an existing loop.
- `ChunkLoadingStrategy` (`TopDown`, `BottomUp`, `CenterOutward`,
  `DistanceToCamera`, `Random`) is the project's existing ordering vocabulary.
  The animator reuses it rather than inventing a parallel one.
- `generate.py` already frame-steps the renderer for turntables, so
  deterministic frame capture is an established pattern.

**Naming constraint:** `Transform` is already taken by `src/diff/mod.rs`. The
animator's per-group state is called `Pose`.

## Architecture

Four layers, each usable without the one above it.

```
Layer 4  bindings          src/bridge/animation.rs      (Diplomat -> 7 languages)
Layer 3  renderer          src/rendering/                (per-draw pose + tint)
Layer 2  authoring         src/animation/presets.rs      (assemble, print, along_shape)
Layer 1  core (pure logic) src/animation/                (easing, tracks, timeline, stagger)
```

Layer 1 has **no GPU and no rendering dependency**. It can emit JSON for an
external renderer (Three.js) as easily as feed nucleation's own. That keeps the
option open without committing to it.

### File layout

```
src/animation/
  mod.rs        re-exports + BuildAnimator
  easing.rs     Easing::eval
  pose.rs       Pose, Pose::to_matrix
  track.rs      Keyframe, Track, Clip
  timeline.rs   Timeline, Frame, seek
  stagger.rs    Grouping, Order, Stagger
  presets.rs    ergonomic constructors
```

## Layer 1 — core

### `Pose`

Per-group animatable state. Identity by default.

```rust
pub struct Pose {
    pub translate: [f32; 3],
    pub rotate_deg: [f32; 3],   // Euler XYZ in DEGREES (anime.js convention)
    pub scale: [f32; 3],
    pub pivot: [f32; 3],        // rotate/scale origin; defaults to group centroid
    pub opacity: f32,           // 0..1
    pub tint: [f32; 4],         // multiplied into base colour
    pub emissive: [f32; 4],     // added after lighting
}
```

Degrees, not radians — matches the reference API and avoids `degToRad` noise at
every call site. `pivot` defaulting to the group centroid is what makes "scale
in place" work without the caller doing centroid math.

`Pose::to_matrix() -> [[f32; 4]; 4]` composes `translate * pivot * rotate *
scale * -pivot`.

### `Easing`

```rust
pub enum Easing {
    Linear,
    In(Power), Out(Power), InOut(Power),      // Quad/Cubic/Quart/Quint/Sine/Expo/Circ
    InBack(f32), OutBack(f32), InOutBack(f32),
    InElastic { amplitude: f32, period: f32 }, OutElastic { .. }, InOutElastic { .. },
    OutBounce, InBounce, InOutBounce,
    Steps(u32),
    CubicBezier(f32, f32, f32, f32),
}
```

`CubicBezier` is the escape hatch — any CSS/anime.js curve can be expressed, so
we are not chasing their catalogue forever.

### `Track`, `Keyframe`, `Clip`

```rust
pub enum Property {
    X, Y, Z, RotX, RotY, RotZ, ScaleX, ScaleY, ScaleZ, ScaleUniform,
    Opacity, TintR, TintG, TintB, TintA, EmissiveR, EmissiveG, EmissiveB, EmissiveA,
}

pub struct Keyframe { pub at: f32, pub value: f32, pub ease: Easing }  // at: 0..1

pub struct Track {
    pub property: Property,
    pub keys: Vec<Keyframe>,
    pub modifier: Option<Modifier>,
}

pub struct Clip {
    pub duration_ms: f32,
    pub delay_ms: f32,
    pub tracks: Vec<Track>,
    pub alternate: bool,
    pub repeat: Repeat,          // Once | Times(u32) | Forever
}
```

Per-property tracks (rather than keyframing a whole `Pose`) because the
reference animation uses **different easing per property** — that's the point of
the abstraction.

**`Modifier`** covers the reference's `y: { to: [0, 4π], modifier: v => 0.5 *
(|sin v| + |cos v|) }` bounce. Rust closures cannot cross Diplomat, so this is a
named enum, not a function pointer:

```rust
pub enum Modifier { Abs, AbsSin, AbsCos, SinCosBounce, Fract, Round, Clamp01, Negate }
```

Rust callers who want arbitrary math can pre-bake keyframes instead. Documented
limitation, not a hidden one.

### `Timeline` and `Frame`

```rust
pub struct Timeline { /* entries: (Clip, Target, offset_ms) */ }

pub enum Target { Group(GroupId), Groups(Vec<GroupId>), All, Camera }

pub struct Frame {
    pub time_ms: f32,
    pub poses: Vec<(GroupId, Pose)>,
    pub camera: Option<CameraPose>,
}

impl Timeline {
    pub fn add(&mut self, clip: Clip, target: Target, offset_ms: f32) -> &mut Self;
    pub fn duration_ms(&self) -> f32;
    pub fn seek(&self, t_ms: f32) -> Frame;     // pure; no interior mutation
}
```

`Target::Camera` folds camera motion into the same timeline, so a turntable and
a block assembly share one clock instead of two.

## Layer 2 — grouping and stagger

### `Grouping` — what becomes an animatable unit

```rust
pub enum Grouping {
    PerBlock,
    Layer(Axis),        // the printer effect
    Chunk(u32),
    Region,             // named regions
    Custom(Vec<Vec<BlockPos>>),
}
```

Granularity is a caller decision because it trades animation resolution against
draw calls (see Performance).

### `Order` — the heart of the design

```rust
pub enum Order {
    Chunk(ChunkLoadingStrategy),      // reuse the existing vocabulary
    Axis(Axis, bool),                 // ascending/descending
    DistanceFrom([f32; 3]),
    ShapeParameter(ShapeEnum),        // <- trefoil: order along the curve
    BuildOrder,                       // <- brush pass: the order blocks were placed
    Custom(Vec<usize>),
    Random(u64),                      // seeded: determinism is non-negotiable
}
```

`ShapeParameter` calls the existing `parameter_at`. `BuildOrder` consumes a
`BuildRecord` (below). Everything else is positional.

### `Stagger`

```rust
pub struct Stagger {
    pub order: Order,
    pub from: StaggerFrom,            // First | Last | Center | Index(usize)
    pub spread: Spread,               // EachMs(f32) | TotalMs(f32)
    pub ease: Easing,                 // distribution easing, not motion easing
}
```

`ease` here shapes *when* each group starts (accelerating/decelerating waves),
distinct from the easing inside a `Clip`. Both matter and conflating them is the
usual mistake.

### `BuildRecord` — capturing build order

A small addition to `BuildingTool`, reusing the `t` the fill loop already has:

```rust
pub struct Placement { pub pos: (i32, i32, i32), pub t: Option<f64>, pub seq: u32 }
pub struct BuildRecord { pub placements: Vec<Placement> }

impl BuildingTool<'_> {
    /// As `fill_enum_masked`, but records what was placed, in order.
    pub fn fill_enum_recorded(&mut self, shape: &ShapeEnum, brush: &BrushEnum,
                              mode: &FillMode) -> BuildRecord;
}
```

Existing `fill*` methods are untouched — no cost imposed on callers who do not
animate.

## Layer 3 — renderer

Minimal, additive, and **no lighting work**.

```rust
pub struct SceneItem<'a> { pub mesh: &'a MeshOutput, pub pose: Pose }

pub fn render_scene(items: &[SceneItem], config: &RenderConfig,
                    hdri: Option<&HdriData>) -> Result<Vec<u8>, RenderError>;
```

`render_meshes` stays, reimplemented as `render_scene` with identity poses — so
existing callers and the `render_smoke` golden hashes are unaffected.

Shader changes in `shader.wgsl`:

- per-draw uniform: `model: mat4x4<f32>`, `normal_mat: mat3x3<f32>`,
  `tint: vec4<f32>`, `emissive: vec4<f32>`
- vertex: `position = view_proj * model * pos`
- **`normal = normalize(normal_mat * normal)`** — without this, rotated meshes
  shade wrong. The single easiest thing to get wrong here.
- fragment: `base_color * tint`, then `+ emissive` after lighting
- `opacity` folds into `tint.a`

The hardcoded `light_dir` stays exactly as it is.

## Determinism

Non-negotiable, because README media must regenerate byte-identically (proven
by the wgpu upgrade: all four `render_smoke` hashes unchanged).

- `seek(t)` is pure — no interior mutability, no wall-clock.
- `Order::Random(seed)` is seeded; there is no unseeded random.
- Frame times: `t_i = i * 1000.0 / fps` computed in `f64`, then cast.
- `Frame.poses` is sorted by `GroupId`, so serialization is stable.

## Performance

`Grouping::PerBlock` on a large build means many draw calls, and greedy meshing
batches by `(texture_path, AO pattern)` — so per-block splitting **changes
material batching** and raises draw counts substantially.

Mitigations, in order of preference:
1. Animate at `Layer`/`Chunk` granularity for big builds.
2. `PerBlock` only for the hero shots (hundreds to low thousands of blocks).
3. Instanced draw with a pose storage buffer — deferred; only if measurements
   demand it.

Benchmark before `PerBlock` is promised in a public API.

## Bindings (Layer 4)

`src/bridge/animation.rs`, following existing conventions:

- Opaque handles: `BuildAnimator`, `Timeline`, `Clip`.
- Enums crossing the boundary as **strings** (`"center_outward"`, `"out_cubic"`),
  matching the existing `from_preset` style.
- `Frame` as JSON via a `frame_at_json(t_ms)` method, matching the existing
  `*_json` convention.
- Presets get first-class methods so the common cases are one call in every
  language.

## Presets (the ergonomic front door)

```rust
BuildAnimator::assemble(&schem, Stagger { order: Order::Axis(Axis::Y, true), .. })
BuildAnimator::print_layers(&schem, Axis::Y, 60.0)
BuildAnimator::along_shape(&schem, &shape, Stagger { .. })   // trefoil
BuildAnimator::from_build_record(&schem, &record, Stagger { .. })  // brush pass
BuildAnimator::from_diff(&diff, ..)                          // diff playback
```

Each returns a `BuildAnimator` whose `Timeline` can still be edited — presets
are starting points, not walls.

## Testing

Layer 1 is pure logic, so most of this is cheap and fast:

- **Easing:** `eval(0) == 0`, `eval(1) == 1`, monotonic where the curve is;
  `CubicBezier` matches known CSS values.
- **Seek determinism:** same `t` → identical `Frame`, repeated and out of order.
- **Stagger:** ordering correctness per `Order`; `TotalMs` spread spans exactly
  the requested window; `from: Center` is symmetric.
- **`Pose::to_matrix`:** against hand-computed matrices; pivot round-trips.
- **`ShapeParameter` ordering** on a bezier: monotonic in `t`.
- **`BuildRecord`:** sequence matches the fill iteration order.
- **Golden frames:** extend `examples/render_smoke.rs` with an animated case at
  fixed `t` values, hashed — the same technique that proved the wgpu upgrade.

## Staging

Each stage lands something usable on its own.

| Stage | Content | Ships |
| --- | --- | --- |
| 1 | `src/animation/` core + tests | JSON timelines, any renderer |
| 2 | renderer pose + tint | animated renders from Rust |
| 3 | `BuildRecord` in `BuildingTool` | brush-pass animation |
| 4 | presets + Diplomat bindings | all 7 languages |
| 5 | `generate.py` integration | README media |

Stage 1 is independently valuable, which is the point — if the holiday ends at
stage 2, nothing is left half-built.

## Open questions

1. **Opacity without alpha blending.** Fading blocks in requires the transparent
   pipeline. Does an animated opaque block move to the transparent pass mid-flight
   (sorting cost, possible popping), or do we fade via `tint` toward the
   background instead? Proposal: start with scale-only reveals, add opacity in
   stage 2 once we can see it.
2. **Group pivot for non-contiguous groups** (e.g. `Region` spanning disjoint
   boxes) — centroid may sit outside the geometry. Probably fine; confirm visually.
3. **Camera easing vs. sphere-fit.** `sphere_fit` keeps framing constant under
   yaw; an animated camera that also dollies may fight it. Test early.
