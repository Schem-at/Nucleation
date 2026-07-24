# Animation and Video Rendering API

Nucleation can animate an existing schematic as one rigid model, light it in world
space, and stream rendered RGBA frames directly to FFmpeg. The streaming path
reuses the parsed resource pack, generated mesh, GPU renderer, and GPU staging
resources; it never retains rendered pixel frames or requires intermediate PNG
files. The small sampled timeline descriptors are prepared in memory before
rendering.

## Requirements

- A native Nucleation build with the `rendering` bridge enabled. Published Python
  wheels use the repository's `bridge-full` feature set.
- A Minecraft resource-pack ZIP.
- FFmpeg on `PATH` for video output. You can also provide an explicit executable
  with `VideoConfig.set_ffmpeg_path(...)`.

Video encoding is native-only. WebAssembly callers can render individual frames
and use a browser-side encoder instead.

## Complete Python example

```python
from pathlib import Path
from nucleation import (
    AnimationEffect,
    BuildAnimation,
    RenderConfig,
    ResourcePack,
    Schematic,
    VideoConfig,
)

schematic = Schematic.open("castle.litematic")
pack = ResourcePack.from_bytes(Path("pack.zip").read_bytes())

# One cloned schematic becomes one animation group and one mesh.
animation = BuildAnimation.from_schematic(schematic)
period_ms = 4_000.0
animation.animate_all(AnimationEffect.turntable(period_ms))
animation.set_loop_period_ms(period_ms)

view = RenderConfig.create(840, 840)
view.set_isometric()
view.set_sphere_fit(True)
view.set_background(0.0, 0.0, 0.0, 0.0)
view.set_directional_light(0.75, 0.40, 0.55, 1.0)
view.set_ambient_light(0.18)

video = VideoConfig.prores_4444(20.0)
frames = animation.render_video_with_pack(
    pack, view, video, "castle-turntable.mov", 0.0
)
print(f"rendered {frames} frames")
```

The corresponding H.264 preset is:

```python
video = VideoConfig.h264(20.0)
animation.render_video_with_pack(
    pack, view, video, "castle-turntable.mp4", 0.0
)
```

H.264 output is opaque. Use ProRes 4444 when alpha must survive encoding.

## API overview

### `BuildAnimation.from_schematic(schematic)`

Clones the complete schematic and records it as one logical animation group. The
original `Schematic` is not mutated. Air is excluded from the group bounds while
all schematic metadata, regions, entities, and block data remain in the clone.
Entity-only schematics are supported. Multiple models at the same integer position
(for example, a block and entity) are rejected because one grouped mesh position
cannot represent both without dropping one.

This path is intended for rigid animation of an already completed model. Use the
normal `BuildAnimation.create(...)` construction methods when individual build
operations need independent timing.

### `BuildAnimation.animate_all(effect)`

Assigns a cloned effect to every recorded group. It does not alter geometry or
create additional groups.

For a seamless turntable, set an explicit loop period equal to the effect period:

```python
period_ms = 4_800.0
animation.animate_all(AnimationEffect.turntable(period_ms))
animation.set_loop_period_ms(period_ms)
```

Loop captures sample the half-open interval `[0, period)`. The final 360-degree
endpoint is intentionally omitted, preventing a duplicate first/last frame.
Without `set_loop_period_ms`, finite animations include their final state.

### One-shot effects and stamp operations

`with_effect(effect)` remains a one-shot decoration for the next recorded
construction group. Instant stamp operations now consume and apply it to their
final stamped group:

```python
animation.with_effect(effect)
animation.stamp_box(
    source,
    min_x, min_y, min_z,
    max_x, max_y, max_z,
    target_x, target_y, target_z,
    "[]",  # exclusions_json
    0.0,   # duration_ms
)
```

A custom pending effect combined with a timed stamp is rejected. Timed stamps
already own a movement clip, so silently composing a second transform on the
same target would be ambiguous. Use an instant stamp followed by an explicit
group effect, or split the movement and post-stamp animation into separate
operations.

The pending effect is consumed even when a stamp is invalid, matching the
one-shot behavior of other construction calls.

### `RenderConfig.set_directional_light(x, y, z, intensity)`

Configures a directional light in world coordinates.

- The vector must be finite and non-zero.
- Intensity must be finite and non-negative.
- The shader normalizes the direction.
- Geometry transformations rotate surface normals, not the light vector. A model
  rotating under a fixed directional light therefore gets a moving terminator.

The backward-compatible defaults are:

```text
direction = (0.3, 1.0, 0.5)
intensity = 1.0
```

### `RenderConfig.set_ambient_light(ambient)`

Sets the minimum unoccluded light level for directional-light rendering. The
value must be finite and between `0.0` and `1.0` inclusive. The default is `0.4`.
Lower values create a darker night side.

Configure the camera preset before custom lighting and background values:

```python
view.set_isometric()          # resets the view to the isometric preset
view.set_background(...)
view.set_directional_light(...)
view.set_ambient_light(...)
```

### `VideoConfig.prores_4444(fps)`

Creates the alpha-preserving MOV preset. The native FFmpeg pipeline is equivalent
to:

```text
raw RGBA frames
  -> prores_ks
  -> profile 4 (ProRes 4444)
  -> yuva444p10le
  -> 16-bit alpha
  -> MOV
```

The output filename must end in `.mov`.

### `VideoConfig.h264(fps)`

Creates an opaque H.264 preset using `libx264`, CRF 18, `yuv420p`, and fast-start
metadata. Output must end in `.mp4` or `.mov`.

Both constructors reject non-finite frame rates and rates below 1 FPS.

### `BuildAnimation.render_video_with_pack(...)`

```python
frame_count = animation.render_video_with_pack(
    resource_pack,
    render_config,
    video_config,
    output_path,
    hold_ms,
)
```

The function:

1. meshes all animation groups once;
2. creates one GPU renderer;
3. prepares deterministic timeline descriptors (poses, visibility, and camera);
4. renders one RGBA frame;
5. writes that frame directly to FFmpeg stdin;
6. releases each CPU readback after writing it (only one rendered pixel frame is
   retained at a time; the CPU `Vec` itself is freshly allocated per frame);
7. waits for FFmpeg and, only after successful encoding, publishes the
   same-directory temporary video to the requested path. Encoder/render failures
   clean up the temporary file without replacing an existing destination.

`hold_ms` extends non-looping captures. Loop captures use their explicit loop
period and ignore a final hold.

`render_video(pack_zip_bytes, ...)` is available when the caller does not need to
retain a loaded `ResourcePack`. The loaded-pack variant avoids ZIP parsing across
multiple renders.

### Loaded resource-pack still rendering

These renderer entry points accept a previously parsed pack:

```python
Renderer.render_to_file_with_pack(schematic, pack, view, "frame.png")
pixels_b64 = Renderer.render_pixels_b64_with_pack(schematic, pack, view)
png_b64 = Renderer.render_png_b64_with_pack(schematic, pack, view)
```

The existing byte-buffer methods remain available for one-off operations.

## Transparency

Both the render target and encoder must preserve alpha:

```python
view.set_background(0.0, 0.0, 0.0, 0.0)
video = VideoConfig.prores_4444(20.0)
```

An alpha-capable codec does not create transparency by itself. Conversely, an
RGBA render encoded with H.264 loses alpha.

A useful validation command is:

```bash
ffmpeg -v error -i output.mov -vf alphaextract -frames:v 1 \
  -f rawvideo -pix_fmt gray - | python3 -c \
  'import sys; d=sys.stdin.buffer.read(); print(min(d), max(d))'
```

A transparent render with opaque content should normally report `0 255`.

## FFmpeg discovery and errors

By default Nucleation starts `ffmpeg` through the process `PATH`:

```python
video = VideoConfig.prores_4444(20.0)
```

To pin an executable:

```python
video.set_ffmpeg_path("/usr/local/bin/ffmpeg")
```

Common failures:

- **FFmpeg cannot start:** install FFmpeg or set an explicit executable path.
- **Invalid argument before rendering:** check the FPS and output extension.
- **Encoder exits unsuccessfully:** run the selected FFmpeg binary with
  `-version`; ensure `prores_ks` or `libx264` is present and the output directory
  is writable.
- **Opaque ProRes output:** verify both a zero-alpha background and the ProRes
  4444 preset.
- **One extra frame:** finite timelines include the final state. Set
  `set_loop_period_ms(period_ms)` for a seamless loop over `[0, period)`.

## Performance model

The streaming path prevents pixel-frame storage growth proportional to duration. For
example, retaining 4K RGBA frames for ten seconds at 24 FPS would require about
7.6 GB before encoding. Streaming keeps only the current RGBA frame, while the
small timeline descriptors, schematic, resource pack, mesh, texture atlas, and
GPU resources remain resident.

Large filled models can still consume substantial meshing memory. Avoid invisible
interior blocks when only an opaque surface is required.

## Minecraft Earth example

The complete rotating Earth example is in:

```text
examples/gallery/globe/generate.py
```

It keeps only Earth-specific responsibilities in Python: downloading Blue
Marble, projecting image pixels onto a sphere, and choosing a Minecraft palette.
Animation grouping, world-space lighting, resource-pack reuse, GPU reuse, frame
sampling, transparency, and ProRes encoding are handled by Nucleation.
