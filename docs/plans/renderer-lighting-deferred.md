# Deferred: dynamic lighting and shadow mapping

**Decision (2026-07-20):** postpone dynamic lights and shadow mapping. Ship the
animation work that does not depend on them.

## Why deferred

- The mesher already bakes AO into vertex colors (`MeshOutput`: *"Vertex tint
  colors (biome coloring, AO, lighting baked in)"*), and the shader already
  applies it (`tex_color * in.color`). Contact shading exists.
- The animations we want (staggered assembly, format carousel, layer printing)
  need **per-mesh transforms**, not shadows.
- Shadow mapping is the highest-risk item in the renderer and contributes least
  to the README goal. Voxel geometry is coplanar-face-heavy — an acne minefield.

## What ships instead (no lighting changes required)

1. **Per-mesh model matrix** — per-draw uniform, multiply in the vertex shader.
   ⚠️ Transform normals by the model matrix too (inverse-transpose, or the
   rotation part for rigid transforms), or rotating meshes will shade wrong.
2. **Per-draw tint + emissive uniform** — `vec4` multiply + `vec4` add. Doubles
   as diff-overlay highlighting and "ghost" preview blocks.
3. **`BuildAnimator` timeline** — tracks, keyframes, easing, `seek(t)`, stagger
   by index / grid / distance-from-centre. Pure logic, deterministic, testable,
   exposed through Diplomat to all seven languages.
4. **Ground plane + grid primitives** (optional, cheap) — most of the "studio"
   framing look.

The hardcoded `light_dir` in `shader.wgsl` stays untouched.

## How to resume, in this order

**Stage 0 — upgrade wgpu first.** Nucleation is on **wgpu 24**; current is
**30**. Do the upgrade *before* writing shadow code, not after — porting a new
shadow pass across six major versions is avoidable pain. Depth-comparison
sampler APIs are stable across that range, so nothing here blocks on it.

**Stage 1 — lights without shadows.** Replace the hardcoded
`light_dir = normalize(vec3(0.3, 1.0, 0.5))` with a small light set in a uniform
buffer: one directional plus N punctual (spot/point), each with colour,
intensity, position/direction, and cone angle. Low risk, no new passes. This
alone delivers the roaming-spotlight drama. **Validate the look here before
going further** — it may be enough.

**Stage 2 — one shadow-casting light.** Start with a **spot** light: a single
perspective frustum is the simplest correct case. For a directional light,
reuse `camera.rs`'s existing `merged_bounds()` / `sphere_fit` to fit one tight
ortho frustum to the build's bounding box. **No cascades needed** — cascades
exist for infinite outdoor scenes, and a schematic is bounded.

Mechanically: depth-only pass from the light → depth texture + comparison
sampler → `textureSampleCompare` + PCF in the fragment shader.

Budget note: rendering is **offline and headless**, so a 4096² map and
expensive PCF cost nothing perceptible. Do not optimise for realtime.

**Stage 3 — bias tuning.** Expect shadow acne on large coplanar voxel faces.
Normal-offset bias (using the per-face normals already in the vertex data) is
the standard cure; plain depth bias alone will cause peter-panning.

**Stage 4 — multi-light shadows.** Only if genuinely needed.

## Traps to remember

- **Double-darkening.** Baked AO + real shadows at full strength looks muddy.
  `set_ao_intensity(f32)` is already exposed (0.0–1.0); blend it down to ~0.3–0.5
  when shadows are on. Keep it a render-time dial, not an architecture choice.
- **Transparent layers.** `MeshOutput` has separate opaque/transparent layers
  with premultiplied vertex colours — decide deliberately whether transparent
  geometry casts shadows (usually: no).
- **Greedy meshing batches by `(texture_path, AO pattern)`.** Splitting geometry
  per-block for animation changes material batching and raises draw counts.
  `set_greedy_meshing(false)` exists; measure before promising it in an API.

## Rejected alternatives (checked 2026-07-20)

| Crate | Status | Why not |
| --- | --- | --- |
| `rend3` | last release **2022-02-12** | dead |
| `renderling` | 2024-09, ~20k downloads | too immature |
| `three-d` | active (0.19.0, 2026-04) | **glow/OpenGL**, deprecated on macOS; full backend swap |
| `bevy` | very active | ECS app framework, wrong shape for a 7-language library |

Conclusion: implement on the existing wgpu renderer. wgpu ships an official
shadow example to work from.
