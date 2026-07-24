# Rotating Minecraft Earth

This example projects NASA Blue Marble imagery onto a Minecraft-block sphere and
uses Nucleation's animation/video API to render one seamless, transparent
turntable directly to ProRes 4444.

## Requirements

- Nucleation built with native rendering support
- FFmpeg and FFprobe on `PATH`
- a Minecraft resource-pack ZIP
- network access on the first run; the Blue Marble image is cached at
  `~/.cache/nucleation/blue-marble.jpg`

## Run

```bash
python examples/gallery/globe/generate.py \
  --pack path/to/resource-pack.zip \
  --output minecraft-earth.mov
```

Default output:

```text
840 × 840
48 frames
10 FPS
4.8 seconds
transparent ProRes 4444 MOV
```

Use smaller settings for a fast preview:

```bash
python examples/gallery/globe/generate.py \
  --pack path/to/resource-pack.zip \
  --output preview.mov \
  --frames 8 --fps 8 --size 420 --radius 36
```

The script builds one schematic. Nucleation then meshes it once, rotates it as a
single animation group, applies a fixed world-space directional light, reuses one
GPU renderer, and streams RGBA frames to FFmpeg without intermediate PNG files.

See [`docs/animation-video-api.md`](../../../docs/animation-video-api.md) for the
complete API contract, transparency requirements, and troubleshooting.
