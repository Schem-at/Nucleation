# Creating animated README illustrations

Handoff guide. This is everything learned building the animation engine and the
media pipeline, and the recipe for making a new illustration. The public API is
documented in [`docs/features/animation.md`](../../docs/features/animation.md);
this doc is the *authoring* side — how to turn the API into a GIF for the README.

---

## The big picture

```
Rust example (examples/readme_*.rs)
  build schematic ──► save .schem/.litematic     (the download beside the image)
                 └──► BuildAnimator + Timeline    (the animation, pure data)
                 └──► render transparent frames    (f0000.png … via the renderer)
                 └──► timing.json                  (optional: which code line is live per frame)
                                │
tools/readme-media/compose_code.py (optional)
  code panel + frame ──► SVG per frame ──► rsvg-convert ──► composite PNGs
                                │
ffmpeg  ──► GIF (transparent, or code-synced side-by-side)
```

Two hard rules that everything else follows from:

1. **nucleation renders the blocks; nothing else does.** The claim "every image
   was built and rendered by nucleation" stays true because Three.js / SVG only
   draw *chrome* (code panels, labels, grids-are-in-the-renderer). If an
   external tool ever rasterizes a block, that claim dies.
2. **Determinism is sacred.** `Timeline::seek(t)` is pure; frame times are
   `i*1000/fps` in f64; `Order::Random` is seeded. Regenerating a GIF must be
   byte-identical, the same property that let the wgpu 24→30 upgrade be verified
   by comparing hashes. Never introduce wall-clock or unseeded randomness.

---

## Prerequisites

- A **vanilla resource pack zip** (not committed). Local copy: `render_work/pack.zip`.
- **ffmpeg** on PATH (GIF assembly). Note: this machine's ffmpeg has **no SVG
  decoder and no drawtext** — do not rely on either.
- **rsvg-convert** on PATH (`brew install librsvg`) — the SVG rasterizer for the
  code compositor. This is the one that works here.
- Build with `--features rendering` (implies meshing).
- `render_work/` is gitignored — all output lands there, nothing to accidentally commit.

---

## Recipe: a new animation

### 1. The three-call core

```rust
let mut anim = BuildAnimator::from_schematic(&schem, Grouping::PerBlock);
anim.timeline_mut().add_staggered(
    presets::drop_and_pop(420.0, 6.0),               // the per-block move
    &Stagger::each(Order::Axis(Axis::Y, true), 60.0), // order + spacing (bottom-up)
    0.0,                                              // offset
);
let frames = anim.frames(20.0);                       // deterministic sampling
```

`Order` is the whole design — same move, different reveal order:
`Axis(Y,true)` bottom-up, `Index` build order, `Key(vec)` along a curve or a
build sequence, `DistanceFrom(p)` ripple, `Random(seed)`.

### 2. Mesh one MeshOutput per group

```rust
let meshes = schem.mesh_groups(&pack, &MeshConfig::default(), anim.groups())?;
```

`mesh_groups` keeps mesh *i* aligned with group *i* — the contract
`render_animation` relies on. Empty (all-air) groups still emit an entry so the
indices never slip.

### 3. Render transparent frames

```rust
let mut rc = RenderConfig::isometric();
rc.width = 480; rc.height = 400;
rc.sphere_fit = true;                        // steady framing while blocks arrive
rc.background = Some([0.0, 0.0, 0.0, 0.0]);  // transparent → drops into any README
render_animation_to_files(&meshes, &frames, &rc, None, "render_work/foo/f")?;
```

### 4. Assemble the GIF (transparent)

GIF has **1-bit** alpha, but the renderer has no MSAA, so edges are hard-cut and
the cutout is clean. The two flags that matter:

```bash
ffmpeg -y -framerate 20 -i 'render_work/foo/f%04d.png' \
  -vf "split[a][b];[a]palettegen=max_colors=192:reserve_transparent=1[p];\
[b][p]paletteuse=alpha_threshold=128" \
  -loop 0 render_work/foo/foo.gif
```

`reserve_transparent=1` + `alpha_threshold=128` — omit either and the background
comes back solid black.

Every `examples/readme_*.rs` follows this shape. Copy the closest one.

---

## Making it loop seamlessly

A closed cycle (build → hold → dissolve → empty, or a travelling wave on a
closed curve) loops if you **sample exactly one period and drop the last frame**
(frame N duplicates frame 0 and stutters). See `examples/render_trefoil.rs`:

- Start clips one full period early (`offset = -PERIOD`) so every group is
  mid-cycle at t=0.
- Verify: `frame_at(0)` and `frame_at(PERIOD)` must match (trefoil closes to
  3.6e-7 — pure float rounding). Empty-frame byte sizes are an easy check.

---

## Code-synced illustrations (the flagship)

"Highlight the code as the block it places drops in." Template:
`examples/readme_setblock.rs` + `tools/readme-media/compose_code.py`.

The generator emits `timing.json` beside the frames:

```json
{
  "title": "Placing blocks",
  "code": ["s = Schematic(\"pillar\")", "s.set_block(0,0,0,\"...\")", ...],
  "anim_w": 360, "anim_h": 420,
  "active": [0, 0, 1, 1, 2, ...]     // live code-line index, one per frame
}
```

`active[i]` is which line is lit at frame `i`. The generator computes it from the
stagger timing (block i starts at `intro + i*each_ms`). The compositor does the rest:

```bash
python3 tools/readme-media/compose_code.py render_work/setblock render_work/setblock/setblock.gif
```

It builds one SVG per frame (code panel + highlight + the animation frame
embedded as base64), rasterizes each with `rsvg-convert`, and assembles the GIF.
Reusable for any `readme_*` dir that has a `timing.json`.

**The code panel shows Python** (the README's primary language) even though the
generator is Rust — the panel teaches what a *user* writes, decoupled from what
produces the pixels. Change this if we decide otherwise.

---

## Overlays without code-sync: labels & leader lines

For "block X is at (x,y,z)" callouts, the renderer projects world→screen and a
compositor draws the line. No text-rendering in the library.

```rust
let vps = animation_view_projs(&meshes, &frames, &rc);   // GPU-free, per frame
let (px, py) = camera::project_point(&vps[i], [2.5,2.5,2.5], rc.width, rc.height).unwrap();
```

`examples/readme_assemble.rs` writes these to `anchors.json`; an SVG/ffmpeg
overlay draws a leader line that tracks the block. Proven: the line lands exactly
on the block's pixels.

---

## The reference grid

`rc.grid = Some(GridConfig { half_extent, spacing, plane_y, show_axes, line_rgba })`
draws a world-space grid + coloured axes (+X red, +Y green, +Z blue). `None` by
default → renders byte-identical to no-grid.

**Two known rough edges (not yet fixed):**
- The mesher is **corner-based** (block `(0,0,0)` spans `[0,1]³`), so grid lines
  at integers sit on block *edges* and the axes emanate from a block *corner*,
  which can read as "half a block off." To centre blocks in cells, shift the
  grid by +0.5 (add an origin offset to `GridConfig`).
- **Don't turntable with a grid** — an orbiting camera spins the "ground," which
  is disorienting. Use a static camera (drop the `Target::Camera` turntable) for
  grid shots; save turntables for grid-less hero shots.

---

## Hard-won gotchas

- **Per-block meshing = one draw call per block**, and it changes greedy-mesh
  batching (batched by `(texture, AO pattern)`). Fine for hundreds–low thousands
  of blocks; use `Grouping::Layer`/`Chunk` for big builds.
- **Dithering bloats GIFs.** A dithered palette alternates neighbouring pixels —
  exactly what LZW can't compress; it can *double* the file. With a wide palette
  the plain snap is already smooth. `render_trefoil.rs` has a `--no-dither` flag.
- **Emissive adds, so it clips to white.** A bright/wide emissive pulse erases
  the underlying colour. Keep flashes narrow and modest (`peak ≈ 0.5`).
- **Sponge `.schem` origin-normalises** to a non-negative corner — a build with
  no floor shifts down on round-trip. Fingerprints are translation-invariant so
  this doesn't change them, but `get_block(x,y,z)` will surprise you.
- **The normal matrix is load-bearing.** The shader does `normal_mat * normal`;
  omit it and rotated blocks shade wrong in a way that looks like a lighting bug.
  (Already wired; don't remove it.)
- **GIF size is the real constraint.** A big rotating multicoloured object is
  near GIF's worst case (every frame differs everywhere). Levers: fewer fps
  (12–15), smaller canvas, fewer colours, shorter cycle, static camera.

---

## Tooling reference

| Piece | Path |
| --- | --- |
| Animation engine | `src/animation/` (easing, pose, track, timeline, stagger, presets) |
| Renderer additions | `src/rendering/` — `render_animation*`, `set_poses`, `mesh_groups`, `GridConfig`, `project_point`, `animation_view_projs` |
| Shader | `src/rendering/shader.wgsl` — per-draw pose/tint/emissive + line pipeline |
| Regression harness | `examples/render_smoke.rs` — pixel hashes; run before/after any render change |
| Examples | `examples/readme_{setblock,assemble,formats}.rs`, `render_{animation,trefoil}.rs` |
| Compositor | `tools/readme-media/compose_code.py` |

Run the smoke harness after any renderer change: identity poses / no grid must
stay byte-identical (all four `render_smoke` hashes unchanged proves it).

---

## Status & what's next

**Done & verified:** animation engine (89 tests), per-mesh pose/tint/emissive
(byte-identical when identity), `mesh_groups`, transparent frames, seamless
loops, the code-sync compositor, projection overlays, the 3D grid, wgpu 30.
662+ tests green.

**Requested but not built yet:**
- **Place-with-NBT** illustration — cheapest: same template + `set_block_with_nbt`.
- **Redstone helpers** illustration — different subsystem (simulation API); a
  *state* animation (lever→lamp tick), not block placement. Its own small design.
  Belongs in `docs/features/redstone-simulation.md`.
- **Bounds / tight-bounds** illustration — wants the grid + leader labels, not
  code-sync. Its own design.

**Known bugs to consider:**
- Sponge `.schem` reader injects an empty `components` NBT tag on read (harmless
  but breaks bit-exact round-trip). Either stop injecting it, or have
  `stable_nbt_token` ignore empty compounds.
- Grid half-block offset + no origin option (see grid section).

**Not yet bridged:** the whole `animation` module is Rust-only. Exposing it to
Python/JS/etc. is a `src/bridge/animation.rs` + `gen-bindings.sh` job (Diplomat
can't pass `Vec<Frame>`/closures, so the bridge collapses the pipeline into one
`render_to_files`-style call and clips become array-based builder methods).
