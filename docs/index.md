---
title: Nucleation
hide:
  - toc
---

<section class="bb-hero">
  <div class="bb-hero__copy">
    <p class="bb-address">docs/index.md <span>·</span> crate 0.10.13</p>
    <h1>
      <span>Minecraft, treated as</span>
      <span>programmable matter.</span>
    </h1>
    <p class="bb-lede">
      Nucleation reads, writes, builds, simulates, meshes, and renders Minecraft
      schematics. Rust is the implementation. Six generated bindings expose the
      same model to Python, JavaScript, Kotlin, PHP, C, and C++.
    </p>
    <div class="bb-actions">
      <a class="md-button md-button--primary" href="features/basics/">Start with a schematic</a>
      <a class="md-button" href="features/formats-and-io/">Read the format guarantees</a>
    </div>
  </div>

  <figure class="bb-specimen">
    <div class="bb-specimen__header">
      <span class="bb-source"><span>examples/readme/hero/</span>generate_intrinsic_animation.py</span>
      <span class="bb-extent">x 118 · y 121 · z 046</span>
    </div>
    <div class="bb-specimen__stage">
      <img src="media/hero.gif" alt="A scorched animated 3 by 7 torus knot generated and rendered by Nucleation">
    </div>
    <div class="bb-specimen__status">
      <span><i aria-hidden="true"></i>97,961 cells · .schem · timing not recorded</span>
      <a href="https://github.com/Schem-at/Nucleation/tree/master/examples/readme/hero">source</a>
    </div>
    <figcaption>
      Each frame is a separate schematic. The braid advances while a periodic
      field cuts scorched plates over a molten core. The published artifact does
      not contain a render-time measurement.
    </figcaption>
  </figure>
</section>

<!-- Specimen facts verified on 2026-08-15 with nucleation 0.10.13 against
docs/downloads/readme/hero/scorched-3x7-frame-000.schem. -->

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

<p class="bb-aside">One address space survives the trip from file parser to tick engine and renderer.</p>

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

<nav class="bb-index" aria-label="Feature index">
  <a href="features/formats-and-io/">
    <code>io/formats</code>
    <span>Litematica, Sponge, MCEdit import, Bedrock, NUSN, Anvil regions, and world containers.</span>
  </a>
  <a href="features/shapes-and-brushes/">
    <code>build/geometry</code>
    <span>Shapes, brushes, masked fills, palettes, fields, terrain, geodata, and mesh voxelization.</span>
  </a>
  <a href="features/regions-and-transforms/">
    <code>build/regions</code>
    <span>Named regions, rigid transforms, deterministic stamping, and composition.</span>
  </a>
  <a href="features/tick-simulation/">
    <code>sim/tick</code>
    <span>Block ticks, update order, fluids, pistons, entities, checkpoints, and snapshots.</span>
  </a>
  <a href="features/redstone-simulation/">
    <code>sim/redstone</code>
    <span>Compiled redstone execution, typed inputs and outputs, and Insign annotations.</span>
  </a>
  <a href="features/meshing-and-rendering/">
    <code>output/mesh</code>
    <span>NUCM, GLB, glTF, USDZ, headless stills, and deterministic animation frames.</span>
  </a>
  <a href="features/world-segmentation/">
    <code>world/segment</code>
    <span>Bounded world streams, substrate subtraction, clustering, stitching, and provenance.</span>
  </a>
  <a href="features/bindings-and-languages/">
    <code>api/bindings</code>
    <span>One generated API across Rust, Python, JavaScript, Kotlin, PHP, C, and C++.</span>
  </a>
  <a href="gallery/">
    <code>output/gallery</code>
    <span>Rendered builds with source: knots, terrain, fractals, maps, text, and voxelized models.</span>
  </a>
</nav>

## Known boundaries

- Legacy MCEdit `.schematic` is import-only.
- JavaScript runs in WASM. It has no local filesystem, so Node callers pass
  bytes and browser callers use bytes or a store callback.
- Format conversion preserves supported cell data. Unrecognised extensions and
  version-specific metadata need an explicit transformation policy.
- The headless renderer is native. Browser display uses the exported mesh data.
