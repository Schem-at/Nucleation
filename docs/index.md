---
title: Nucleation
---

<style>
/* Two call-to-action links on the landing page. pagina ships no button
   primitive, so this is defined here — over pagina's own --pg-* token
   contract, so it follows the reader's light/dark theme — and travels with
   the article rather than depending on a host stylesheet. The `nu-` prefix
   keeps it out of pagina's `pg-` namespace. */
.nu-cta {
  display: inline-block;
  margin: 0 0.5rem 0.5rem 0;
  padding: 0.5rem 1rem;
  border: 1px solid var(--pg-line-strong, var(--pg-line));
  border-radius: var(--pg-radius);
  color: var(--pg-fg);
  font-weight: 600;
  text-decoration: none;
}
.nu-cta:hover { border-color: var(--pg-accent); }
.nu-cta--primary {
  background: var(--pg-accent);
  border-color: var(--pg-accent);
  color: var(--pg-accent-fg);
}
/* `<model-viewer>` defaults to height:150px in its own stylesheet, which beats
   an `aspect-ratio`, so the height is set outright or the model renders as a
   sliver in a letterbox. */
.nu-model {
  display: block;
  width: 100%;
  height: min(26rem, 56vh);
  background: var(--pg-bg-sunken);
  border-radius: var(--pg-radius);
}
</style>

# Minecraft, treated as programmable matter.

Nucleation reads, writes, builds, simulates, meshes, and renders Minecraft\
schematics. Rust is the implementation. Six generated bindings expose the same\
model to Python, JavaScript, Kotlin, PHP, C, and C++.

<model-viewer class="nu-model" src="media/capture-1779012187376.glb" alt="An interactive 3D view of a schematic exported from Nucleation as glTF" camera-controls="" touch-action="pan-y"></model-viewer>

[Start with a schematic](features/basics.md){ .nu-cta .nu-cta--primary }
[Read the format guarantees](features/formats-and-io.md){ .nu-cta }

<figure class="kg" data-scene="scenes/formats-and-io.mjs" data-controls="false" data-readout="false"></figure>

<figure>
  <img src="media/hero.gif" alt="A scorched animated 3 by 7 torus knot generated and rendered by Nucleation">
  <figcaption>Each frame is a separate schematic. The braid advances while a periodic field cuts scorched plates over a molten core.</figcaption>
</figure>

## Install

Choose the package for the process that owns the schematic.

=== "Python"

    ```console
    pip install nucleation
    ```

=== "JavaScript"

    ```console
    npm install nucleation
    ```

=== "Rust"

    ```console
    cargo add nucleation
    ```

Kotlin/JVM, PHP, C, and C++ are published as release archives. Their generated
surface follows the same bridge definitions and the naming rules of each
language.

!!! quote ""
    One address space survives the trip from file parser to tick engine and renderer.

## First file

This Python example creates an empty schematic, places three blocks, and writes
Sponge `.schem`. Coordinates grow the default region when they fall outside its
current extent.

```python
from nucleation import Schematic

build = Schematic.create("signal-lamp")
build.set_block(0, 0, 0, "minecraft:lever[facing=east]")
build.set_block(1, 0, 0, "minecraft:redstone_wire")
build.set_block(2, 0, 0, "minecraft:redstone_lamp")
build.save_to_file("signal-lamp.schem")
```

[Continue with block states and inspection](features/basics.md)

## Index

| Area | What it covers |
|---|---|
| [`build/data`](features/data-driven-generation.md) | Turn images, arrays, and external datasets into positions, block states, and block entities. |
| [`io/formats`](features/formats-and-io.md) | Litematica, Sponge, MCEdit import, Bedrock, NUSN, Anvil regions, and world containers. |
| [`build/geometry`](features/shapes-and-brushes.md) | Shapes, brushes, masked fills, palettes, fields, terrain, geodata, and mesh voxelization. |
| [`build/regions`](features/regions-and-transforms.md) | Named regions, rigid transforms, deterministic stamping, and composition. |
| [`sim/tick`](features/tick-simulation.md) | Block ticks, update order, fluids, pistons, entities, checkpoints, and snapshots. |
| [`sim/redstone`](features/redstone-simulation.md) | Compiled redstone execution, typed inputs and outputs, and Insign annotations. |
| [`output/mesh`](features/meshing-and-rendering.md) | NUCM, GLB, glTF, USDZ, headless stills, and deterministic animation frames. |
| [`world/segment`](features/world-segmentation.md) | Bounded world streams, substrate subtraction, clustering, stitching, and provenance. |
| [`api/bindings`](features/bindings-and-languages.md) | One generated API across Rust, Python, JavaScript, Kotlin, PHP, C, and C++. |
| [`output/gallery`](gallery.md) | Rendered builds with source: knots, terrain, fractals, maps, text, and voxelized models. |

## Known boundaries

- Legacy MCEdit `.schematic` is import-only.
- JavaScript runs in WASM. It has no local filesystem, so Node callers pass
  bytes and browser callers use bytes or a store callback.
- Format conversion preserves supported cell data. Unrecognised extensions and
  version-specific metadata need an explicit transformation policy.
- The headless renderer is native. Browser display uses the exported mesh data.
