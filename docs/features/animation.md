# Animating a build

Nucleation can describe **which block is where, at what pose, at time *t***.
That description is a plain data model — no GPU, no rendering — so it drives
nucleation's own renderer, an exported JSON timeline, or anything else that can
draw a transform.

It answers the questions a schematic library actually gets asked: *show me this
build assembling itself*, *print it layer by layer*, *reveal it along the curve
it was built from*, *replay this diff*.

> **Status:** the data model and rendering both work end to end — see
> `examples/render_animation.rs`. Language bindings are not wired up yet
> (Rust only for now); dynamic lights and shadows are deliberately out of scope
> ([why](../plans/renderer-lighting-deferred.md)).

## The shape of it

```
positions ──> Grouping ──> [Group]  ──> Timeline ──seek(t)──> Frame
                                ▲                               │
                             Stagger                         [(GroupId, Pose)]
                          (order + delays)
```

- A **`Group`** is one animatable unit — a block, a layer, a chunk, a region.
- A **`Clip`** is what happens to a group: property tracks with keyframes,
  easing, delay, repeat, ping-pong.
- A **`Stagger`** decides the *order* groups animate in and *when* each starts.
- A **`Timeline`** binds clips to targets; `seek(t)` returns a **`Frame`** of
  poses.

## Quick start

```rust
use nucleation::animation::*;

let mut anim = BuildAnimator::from_schematic(&schem, Grouping::PerBlock);
anim.timeline_mut().add_staggered(
    presets::pop_in(200.0),                              // each block scales in
    &Stagger::each(Order::Axis(Axis::Y, true), 40.0),    // bottom to top, 40ms apart
    0.0,
);

for frame in anim.frames(30.0) {          // deterministic 30fps sampling
    for (id, pose) in &frame.poses {
        // pose.to_matrix() -> model matrix; pose.normal_matrix() -> normals
    }
}
```

Or skip the assembly and take a preset whole:

```rust
let anim = presets::assemble(&schem, 200.0, 40.0);
let anim = presets::print_layers(&schem, Axis::Y, 80.0);
```

## Ordering is the interesting part

Everything above is the same call with a different `Order`. That is the whole
design: *what moves* and *how it moves* stay fixed, and only the ranking changes.

| `Order` | Effect |
| --- | --- |
| `Index` | groups in the order they were built |
| `Axis(axis, ascending)` | bottom-up, top-down, left-to-right |
| `DistanceFrom(point)` | ripples outward from a point |
| `Key(Vec<f64>)` | any caller-supplied sort key |
| `Custom(Vec<usize>)` | an explicit permutation |
| `Random(seed)` | seeded shuffle — never unseeded |

`Key` is the general case, and two helpers produce the interesting keys.

### Along a shape's own curve

`ShapeEnum::parameter_at` gives the parametric `t` of a position along a line,
cylinder, cone, torus, pyramid or bezier — the same `t` a `curve_gradient` brush
uses to pick a colour. Feed it to the animator and blocks arrive **in the order
the curve sweeps**:

```rust
let anim = presets::along_shape(&schem, &shape, presets::drop_and_pop(300.0, 6.0), 2000.0);
```

A trefoil knot assembles itself head-to-tail instead of appearing all at once.

### In the order a brush painted them

Pass the sequence of placements and the animation replays the build:

```rust
let keys = presets::build_order_keys(&placement_sequence, anim.groups());
anim.timeline_mut().add_staggered(
    presets::pop_in(150.0),
    &Stagger::total(Order::Key(keys), 3000.0),
    0.0,
);
```

## Two easings, doing different jobs

This trips people up, so it is worth stating plainly:

- The easing inside a **`Clip`** shapes **how a group moves** once it starts.
- `Stagger::ease` shapes **when each group starts** — an accelerating or
  decelerating wave across the build.

```rust
Stagger::total(Order::Axis(Axis::Y, true), 2000.0)
    .from(StaggerFrom::Center)          // wave starts in the middle
    .eased(Easing::In(Power::Quad))     // and accelerates outward
```

`StaggerFrom` picks the origin: `First`, `Last`, `Center`, or `Index(n)`.

## Clips

A `Clip` bundles property tracks with timing.

```rust
let clip = Clip::new(400.0)
    .delay(100.0)
    .alternate(true)                    // ping-pong
    .repeat(Repeat::Times(3))
    .track(Track::tween(Property::Y, 8.0, 0.0, Easing::Out(Power::Cubic)))
    .track(Track::from_values(Property::RotZ, &[360.0, 0.0, -360.0], Easing::Linear));
```

Animatable properties: `X`/`Y`/`Z`, `RotX`/`RotY`/`RotZ` (degrees),
`ScaleX`/`ScaleY`/`ScaleZ`/`ScaleUniform`, `Opacity`, `TintR/G/B/A`,
`EmissiveR/G/B`.

A clip **only overrides the channels it animates**, so clips layer: one for
position, another for rotation, added independently.

Before its delay elapses a clip holds its first frame; after it finishes it
holds its last. Nothing snaps back.

### Easing curves

`Linear`; `In`/`Out`/`InOut` over `Quad`, `Cubic`, `Quart`, `Quint`, `Sine`,
`Expo`, `Circ`; `Back` and `Elastic` (which deliberately overshoot);
`Bounce`; `Steps(n)`; and `CubicBezier(x1, y1, x2, y2)` with the same
parameterisation as CSS, so any curve you can write there works here.

### Colours in one call

Tint and emissive take colour strings and expand to per-channel tracks, so one
name writes a compound value instead of three tracks at the call site:

```rust
Clip::new(600.0)
    .tint(&["#ff0000", "#00ff00", "#0000ff"], Easing::Linear)
    .emissive(&["#000000", "#ffcc00"], Easing::Out(Power::Cubic))
```

`#rgb`, `#rrggbb` and `#rrggbbaa` all parse, with or without the leading `#`.
An unparseable colour becomes neutral white rather than failing, so a typo dulls
a colour instead of killing the render.

`Pose::opacity` folds into tint alpha at draw time, so there is a single alpha
source rather than two that can disagree.

### Modifiers

A `Modifier` post-processes a track's value — `SinCosBounce` drives the
`0.5 · (|sin v| + |cos v|)` arc when a track sweeps `0..4π`:

```rust
Track::tween(Property::Y, 0.0, 4.0 * std::f32::consts::PI, Easing::Linear)
    .with_modifier(Modifier::SinCosBounce)
```

Modifiers are a fixed set rather than callbacks, because closures cannot cross
the language bindings. Rust callers wanting arbitrary maths should pre-bake
keyframes instead.

## Grouping and cost

| `Grouping` | Unit | Notes |
| --- | --- | --- |
| `PerBlock` | one block | highest resolution, highest draw cost |
| `Layer(axis)` | one slice | the printer effect |
| `Chunk(n)` | an n³ cube | good default for large builds |
| `Custom(sets)` | whatever you pass | |

Greedy meshing batches geometry by `(texture, AO pattern)`, so splitting a build
per block **changes material batching and raises draw counts**. Use `PerBlock`
for hero shots of hundreds to low thousands of blocks; use `Layer` or `Chunk`
for anything large. Measure before relying on it.

Air is never grouped — air is absence, not a block.

## The camera is on the same clock

Target `Camera` and the clip drives the view instead of geometry, so an orbit
and an assembly share one timeline:

```rust
anim.timeline_mut().add(presets::turntable(4000.0), Target::Camera, 0.0);
```

The mapping is: `RotY → yaw`, `RotX → pitch`, `ScaleUniform → zoom`,
`X`/`Y`/`Z` → orbit target offset.

## Determinism

`Timeline::seek` is pure — no interior mutation, no wall-clock. The same time
always yields the same frame, sampling out of order changes nothing, and
`Order::Random` is seeded with no unseeded variant.

Frame times come from `i × 1000 ÷ fps` computed in `f64`, so they do not drift
the way accumulated sums would.

This is not incidental. Regenerated README media must be byte-identical, the
same property that let the wgpu 24→30 upgrade be verified by comparing render
hashes.

## Pivots

A group's pose pivots about its **centroid** by default, which is what makes
"scale in place" work without any arithmetic at the call site. Override
`Pose::pivot` to swing a group about a hinge instead.

`Pose::normal_matrix()` returns the inverse-transpose for transforming normals.
Renderers **must** apply it — skip it and rotated geometry shades wrong in a way
that reads as a lighting bug. Degenerate poses (a block at scale 0 mid-reveal)
return identity rather than emitting NaNs.

## Rendering it

Three calls: group, mesh per group, render.

```rust
use nucleation::animation::{presets, Axis, BuildAnimator, Grouping, Order, Stagger, Target};
use nucleation::rendering::{render_animation_to_files, RenderConfig};

let mut anim = BuildAnimator::from_schematic(&schem, Grouping::PerBlock);
anim.timeline_mut().add_staggered(
    presets::drop_and_pop(420.0, 6.0),
    &Stagger::each(Order::Axis(Axis::Y, true), 55.0),
    0.0,
);
let spin = presets::turntable(anim.duration_ms());
anim.timeline_mut().add(spin, Target::Camera, 0.0);

// One MeshOutput per group, index-aligned — this is the contract.
let meshes = schem.mesh_groups(&pack, &MeshConfig::default(), anim.groups())?;

let mut rc = RenderConfig::isometric();
rc.sphere_fit = true;          // steady framing while the camera orbits
render_animation_to_files(&meshes, &anim.frames(24.0), &rc, None, "out/f")?;
```

Then assemble the frames with ffmpeg (below).

### Transparent GIFs for docs

Set an alpha-0 clear and the frames drop into a README on light *or* dark
backgrounds:

```rust
rc.background = Some([0.0, 0.0, 0.0, 0.0]);
```

GIF only has **1-bit** transparency — a pixel is fully opaque or fully gone. That
would normally fringe antialiased edges, but the renderer does not multisample
(`count: 1`), so edges are hard-cut and the cutout is clean. Two ffmpeg flags do
the work:

```bash
ffmpeg -y -framerate 24 -i 'out/f%04d.png' \
  -vf "split[a][b];[a]palettegen=max_colors=200:reserve_transparent=1[p];\
[b][p]paletteuse=alpha_threshold=128" \
  -loop 0 out.gif
```

`reserve_transparent=1` keeps a palette slot for the transparent colour;
`alpha_threshold=128` picks the opaque/clear cutoff. Omit either and the
background comes back as solid black.

For full 8-bit alpha, APNG is the alternative and GitHub renders it — but
expect roughly double the file size:

```bash
ffmpeg -y -framerate 24 -i 'out/f%04d.png' -plays 0 -f apng out.png
```

Keep README animations small: drop to 12–15fps, shrink the canvas, and lower
`max_colors`. A 480×360 55-frame clip lands around 430 KB as a GIF.

`mesh_groups` is what keeps mesh *i* aligned with group *i* — groups that
contain only air still produce an entry so the indices never slip.

The GPU renderer, atlas and geometry buffers are built **once** and reused for
every frame; only a small uniform buffer is rewritten per frame. Rendering an
animation is therefore much cheaper than rendering N stills.

`examples/render_animation.rs` is the runnable version of the above.

### What rendering costs

Each group is meshed independently, so faces between groups are **not** culled
against each other. That is what you want when groups move apart, and wasteful
when they never do. Per-block grouping also means one draw call per block.
Prefer `Layer` or `Chunk` for large builds.

## Reference grid and axes

For coordinate-space clarity, the renderer can draw a world-space grid on the
ground plane with optional coloured axes (+X red, +Y green, +Z blue):

```rust
use nucleation::rendering::GridConfig;
rc.grid = Some(GridConfig {
    half_extent: 16,          // spans -16..=16 blocks
    spacing: 1,               // a line every block
    plane_y: 0.0,
    show_axes: true,
    line_rgba: [0.55, 0.57, 0.62, 0.6],
});
```

Blocks occlude grid lines behind them, and glass blends over the grid. It is
`None` by default, so a render without a grid is bit-for-bit unchanged.

## Screen-space overlays (labels, leader lines, code)

Text and 2D chrome live in a **compositor**, not the 3D renderer — that keeps a
heavy text-rendering dependency out of the library. The renderer's job is to
say *where on screen* a block is; the compositor draws the callout.

```rust
use nucleation::rendering::{animation_view_projs, camera::project_point};

let view_projs = animation_view_projs(&meshes, &frames, &rc);   // GPU-free
// For frame i, the pixel position of the block centre at (2, 2, 2):
if let Some((px, py)) = project_point(&view_projs[i], [2.5, 2.5, 2.5], rc.width, rc.height) {
    // hand (px, py) to an SVG/ffmpeg overlay step
}
```

`examples/readme_assemble.rs` writes these anchors to `anchors.json`; a
compositor then draws a leader line and caption that track the block frame by
frame. A 2D screen-space grid is the same idea — evenly spaced lines drawn by
the compositor.

## Presets

| Preset | What it does |
| --- | --- |
| `pop_in(ms)` | scale 0→1 with a slight overshoot |
| `drop_in(ms, height)` | fall into place, decelerating |
| `drop_and_pop(ms, height)` | both together |
| `spin_in(ms, turns)` | spin while scaling in |
| `turntable(ms)` | a full camera orbit |
| `assemble(schem, ms, each)` | bottom-to-top reveal |
| `print_layers(schem, axis, ms)` | layer-by-layer print |
| `along_shape(schem, shape, clip, ms)` | reveal along a curve |

Presets return ordinary `Clip`s and `BuildAnimator`s — keep editing the timeline
if one is nearly right.
