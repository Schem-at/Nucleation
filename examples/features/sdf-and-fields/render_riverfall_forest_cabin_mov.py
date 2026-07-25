#!/usr/bin/env python3
"""Render the canonical Riverfall forest/cabin escarpment as transparent ProRes."""

from __future__ import annotations

import argparse
import hashlib
import math
import os
import subprocess
import tempfile
from pathlib import Path

from nucleation import RenderConfig, Renderer, ResourcePack, Schematic


def view(width: int, height: int, yaw: float) -> RenderConfig:
    config = RenderConfig.create(width, height)
    config.set_orthographic(True)
    config.set_yaw(yaw)
    config.set_pitch(25.0)
    config.set_sphere_fit(True)
    config.set_zoom(1.23)
    config.set_background(0.0, 0.0, 0.0, 0.0)
    config.set_ambient_light(0.69)
    config.set_directional_light(-0.65, 0.85, -0.35, 1.0)
    return config


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("/tmp/riverfall-forest-cabin-escarpment.litematic"),
    )
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
            "render_work/world-generation/riverfall-forest-cabin-transparent.mov"
        ),
    )
    parser.add_argument("--frames", type=int, default=40)
    parser.add_argument("--fps", type=int, default=20)
    parser.add_argument("--width", type=int, default=800)
    parser.add_argument("--height", type=int, default=480)
    args = parser.parse_args()

    if args.frames < 2 or args.fps <= 0 or args.width <= 0 or args.height <= 0:
        raise SystemExit("frames must be >=2; fps and dimensions must be positive")
    if not args.source.is_file() or not args.pack.is_file():
        raise SystemExit("source litematic or resource pack is missing")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    partial = args.output.with_name(f".{args.output.stem}.partial.mov")
    partial.unlink(missing_ok=True)

    schematic = Schematic.load_from_file(str(args.source))
    pack = ResourcePack.from_bytes(args.pack.read_bytes())
    with tempfile.TemporaryDirectory(prefix="riverfall-forest-cabin-") as temporary:
        frame_dir = Path(temporary)
        hashes = []
        for index in range(args.frames):
            # Closed restrained sweep around the accepted 225-degree hero view.
            yaw = 225.0 + 22.0 * math.sin(math.tau * index / args.frames)
            frame = frame_dir / f"frame-{index:04d}.png"
            Renderer.render_to_file_with_pack(
                schematic, pack, view(args.width, args.height, yaw), str(frame)
            )
            digest = hashlib.sha256(frame.read_bytes()).hexdigest()
            hashes.append(digest)
            print(f"frame {index + 1:02d}/{args.frames}: yaw={yaw:.1f} {digest[:12]}")

        if len(set(hashes)) <= 1:
            raise RuntimeError("all rendered frames are identical")
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
