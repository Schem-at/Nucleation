#!/usr/bin/env python3
"""Render a rotating Minecraft Earth directly to transparent ProRes 4444."""

from __future__ import annotations

import argparse
import json
import math
import subprocess
import urllib.parse
import urllib.request
from pathlib import Path

from nucleation import (
    AnimationEffect,
    BuildAnimation,
    PaletteBuilder,
    RenderConfig,
    ResourcePack,
    Schematic,
    VideoConfig,
)

BLUE_MARBLE = "Land ocean ice 2048.jpg"
SUN = (0.75, 0.40, 0.55)
NOISY_BLOCKS = """
prismarine carved jack_o command structure loom cartography crafting smithing
fletching barrel jukebox note_block tnt target piston observer dispenser dropper
furnace smoker sculk chiseled pumpkin melon mycelium podzol glazed shroomlight
froglight _ore raw_ bookshelf hay_block dried magma sponge cake spawner respawn
ancient reinforced suspicious infested mushroom coral sulfur cinnabar copper_grate
bulb daylight composter beehive bee_nest lodestone bedrock grass_block creaking
pale_moss resin_clump amethyst budding wart nylium oxidized weathered exposed
pillar lantern blue_ice packed_ice frosted crying gilded redstone_block slime honey
scaffolding kelp bamboo_block muddy root
""".split()


def run(*args: str) -> bytes:
    return subprocess.run(args, check=True, capture_output=True).stdout


def blue_marble(cache: Path) -> Path:
    """Download and cache NASA's Blue Marble image from Wikimedia Commons."""
    if cache.exists():
        return cache
    query = urllib.parse.urlencode({
        "action": "query", "titles": f"File:{BLUE_MARBLE}", "prop": "imageinfo",
        "iiprop": "url", "iiurlwidth": 1280, "format": "json",
    })
    headers = {"User-Agent": "nucleation-globe-example/1.0"}
    request = urllib.request.Request(
        f"https://commons.wikimedia.org/w/api.php?{query}", headers=headers
    )
    page = next(iter(json.load(urllib.request.urlopen(request))["query"]["pages"].values()))
    cache.parent.mkdir(parents=True, exist_ok=True)
    cache.write_bytes(urllib.request.urlopen(
        urllib.request.Request(page["imageinfo"][0]["thumburl"], headers=headers)
    ).read())
    return cache


def pixels(path: Path, width: int = 1024) -> tuple[int, int, bytes]:
    source_width, source_height = map(int, run(
        "ffprobe", "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=width,height", "-of", "csv=p=0", str(path),
    ).decode().strip().split(","))
    height = round(width * source_height / source_width)
    rgb = run(
        "ffmpeg", "-v", "error", "-i", str(path), "-vf", f"scale={width}:{height}",
        "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
    )
    return width, height, rgb


def minecraft_palette():
    builder = PaletteBuilder.create()
    builder.full_blocks_only()
    builder.exclude_tile_entities()
    builder.exclude_transparent()
    for keyword in NOISY_BLOCKS:
        builder.exclude_keyword(keyword)
    return builder.build().dithered()


def vivid_rgb(r: int, g: int, b: int) -> tuple[int, int, int]:
    source = [255.0 * (value / 255.0) ** 0.55 for value in (r, g, b)]
    luminance = 0.2126 * source[0] + 0.7152 * source[1] + 0.0722 * source[2]
    vivid = [max(0, min(255, round(luminance + (value - luminance) * 1.3))) for value in source]
    return vivid[0], vivid[1], vivid[2]


def build_globe(radius: float, texture: tuple[int, int, bytes]) -> Schematic:
    """Build one textured globe; Nucleation rotates and lights it at render time."""
    if radius <= 0.6:
        raise ValueError("radius must be greater than 0.6 blocks")
    width, height, rgb = texture
    palette = minecraft_palette()
    globe = Schematic.create("minecraft-earth")

    extent = int(radius) + 1
    for x in range(-extent, extent + 1):
        for y in range(-extent, extent + 1):
            for z in range(-extent, extent + 1):
                distance = math.sqrt(x * x + y * y + z * z)
                if not radius - 0.6 <= distance <= radius + 0.4:
                    continue
                nx, ny, nz = x / distance, y / distance, z / distance
                u = (-math.atan2(nz, nx) / (2.0 * math.pi)) % 1.0
                v = 0.5 - math.asin(max(-1.0, min(1.0, ny))) / math.pi
                offset = (min(height - 1, int(v * height)) * width + min(width - 1, int(u * width))) * 3
                color = vivid_rgb(*rgb[offset:offset + 3])
                globe.set_block(x, y, z, palette.closest_block_dithered(*color, x, y, z))
    return globe


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True, help="Minecraft resource-pack ZIP")
    parser.add_argument("--output", type=Path, default=Path("minecraft-earth.mov"))
    parser.add_argument("--frames", type=int, default=48)
    parser.add_argument("--fps", type=float, default=10.0)
    parser.add_argument("--size", type=int, default=840)
    parser.add_argument("--radius", type=float, default=120.0)
    args = parser.parse_args()
    if args.frames < 1 or args.fps < 1.0 or args.radius <= 0.6:
        parser.error("frames must be positive, fps must be at least 1, and radius must exceed 0.6")

    texture = pixels(blue_marble(Path.home() / ".cache/nucleation/blue-marble.jpg"))
    period_ms = args.frames / args.fps * 1000.0
    animation = BuildAnimation.from_schematic(build_globe(args.radius, texture))
    animation.animate_all(AnimationEffect.turntable(period_ms))
    animation.set_loop_period_ms(period_ms)

    view = RenderConfig.create(args.size, args.size)
    view.set_isometric()
    view.set_yaw(0.0)
    view.set_pitch(15.0)
    view.set_zoom(1.6)
    view.set_sphere_fit(True)
    view.set_background(0.0, 0.0, 0.0, 0.0)
    view.set_directional_light(*SUN, 1.0)
    view.set_ambient_light(0.18)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    pack = ResourcePack.from_bytes(args.pack.read_bytes())
    frame_count = animation.render_video_with_pack(
        pack, view, VideoConfig.prores_4444(args.fps), str(args.output), 0.0
    )
    print(f"rendered {frame_count} frames to {args.output}")


if __name__ == "__main__":
    main()
