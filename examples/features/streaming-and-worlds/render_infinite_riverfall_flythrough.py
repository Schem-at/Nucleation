#!/usr/bin/env python3
"""Render a moving window over the procedural infinite Riverfall world."""

from __future__ import annotations

import argparse
import hashlib
import math
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from nucleation import RenderConfig, Renderer, ResourcePack, Schematic, WorldSink

sys.path.insert(0, str(Path(__file__).resolve().parent))
from infinite_riverfall_world import CELL_X, MAX_Y, MIN_Y, make_source


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be positive")
    return parsed


def render_config(width: int, height: int) -> RenderConfig:
    config = RenderConfig.create(width, height)
    config.set_orthographic(True)
    config.set_yaw(225.0)
    config.set_pitch(25.0)
    config.set_sphere_fit(True)
    config.set_zoom(1.18)
    config.set_background(0.0, 0.0, 0.0, 0.0)
    config.set_ambient_light(0.69)
    config.set_directional_light(-0.65, 0.85, -0.35, 1.0)
    return config


def persist_generation(world_dir: Path) -> None:
    source = make_source()
    sink = WorldSink.create(str(world_dir), "")
    # Covers every 160x128-block moving window from district 0 to district 1.
    stream = source.stream(-5, -4, 16, 3)
    while stream.remaining() > 0:
        sink.write_chunk(stream.next().take_view())
    sink.finish()


def load_window(world_dir: Path, center_x: int) -> Schematic:
    min_x, max_x = center_x - 80, center_x + 79
    min_z, max_z = -64, 63
    scene = Schematic.from_world_directory_bounded(
        str(world_dir), min_x, MIN_Y, min_z, max_x, MAX_Y, max_z
    )
    # Invisible light blocks lock the fit bounds despite changing edge geometry.
    scene.set_block(min_x, MIN_Y, min_z, "minecraft:light[level=0]")
    scene.set_block(max_x, MAX_Y, max_z, "minecraft:light[level=0]")
    return scene


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--pack",
        type=Path,
        default=Path(
            os.environ.get("NUCLEATION_PACK", "/Users/harrison/Downloads/pack.zip")
        ),
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(
            "render_work/world-generation/infinite-riverfall-moving-window.mov"
        ),
    )
    parser.add_argument("--frames", type=positive_int, default=60)
    parser.add_argument("--fps", type=positive_int, default=20)
    parser.add_argument("--width", type=positive_int, default=640)
    parser.add_argument("--height", type=positive_int, default=384)
    args = parser.parse_args()

    if args.frames < 4:
        raise SystemExit("frames must be at least 4")
    if not args.pack.is_file():
        raise SystemExit(f"resource pack not found: {args.pack}")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    partial = args.output.with_name(f".{args.output.stem}.partial.mov")
    partial.unlink(missing_ok=True)

    with tempfile.TemporaryDirectory(prefix="infinite-riverfall-") as temporary:
        temporary = Path(temporary)
        world_dir = temporary / "generated-world"
        frame_dir = temporary / "frames"
        frame_dir.mkdir()
        persist_generation(world_dir)

        pack = ResourcePack.from_bytes(args.pack.read_bytes())
        config = render_config(args.width, args.height)
        hashes: list[str] = []
        for index in range(args.frames):
            # Cosine ping-pong is position- and velocity-continuous at the loop.
            phase = math.tau * index / args.frames
            center_x = round(CELL_X * (0.5 - 0.5 * math.cos(phase)))
            scene = load_window(world_dir, center_x)
            frame = frame_dir / f"frame-{index:04d}.png"
            Renderer.render_to_file_with_pack(scene, pack, config, str(frame))
            digest = hashlib.sha256(frame.read_bytes()).hexdigest()
            hashes.append(digest)
            print(
                f"frame {index + 1:02d}/{args.frames}: "
                f"center_x={center_x:3d} {digest[:12]}"
            )

        if len(set(hashes)) < args.frames // 4:
            raise RuntimeError("moving window produced too few distinct frames")
        subprocess.run(
            [
                "ffmpeg",
                "-y",
                "-framerate",
                str(args.fps),
                "-i",
                str(frame_dir / "frame-%04d.png"),
                "-c:v",
                "prores_ks",
                "-profile:v",
                "4",
                "-pix_fmt",
                "yuva444p10le",
                "-alpha_bits",
                "16",
                str(partial),
            ],
            check=True,
            env={**os.environ, "LC_ALL": "C"},
        )

    os.replace(partial, args.output)
    print(args.output.resolve())


if __name__ == "__main__":
    main()
