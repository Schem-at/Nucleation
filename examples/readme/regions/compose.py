#!/usr/bin/env python3
"""Label README frames with Minecraft's bitmap font and encode GIFs."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

from PIL import Image, ImageDraw

FPS = 20
ATLAS_PATH = "assets/minecraft/textures/font/ascii.png"

SCENES = {
    "hero": {
        "title": "REGION VS SCHEMATIC",
        "intro": "BUILD: Main + west_wing + east_wing",
        "ops": ["REGION: west_wing", "SCHEMATIC: all regions", "STAMP: merged copy"],
        "final": "RESULT",
    },
    "stamping": {
        "title": "STAMP REGION",
        "intro": "SOURCE: stall",
        "ops": ["STAMP: diamond", "STAMP: emerald", "ROTATE + FLIP: lapis"],
        "final": "3 PLACEMENTS",
    },
    "axes": {
        "title": "ROTATE REGIONS",
        "intro": "X / Y / Z",
        "ops": [
            "X +90: south/bottom -> north/bottom",
            "Y +90: stair east -> south",
            "Z +90: east/bottom -> east/top",
        ],
        "final": None,
    },
    "overlap": {
        "title": "REGION PRIORITY",
        "fixed": "Main > alpha > zeta  |  AIR MASKS RED",
        "panels": [(95, "zeta"), (300, "alpha"), (475, "Main"), (650, "merged")],
    },
}


def frame_window(start_ms: float, end_ms: float) -> tuple[int, int]:
    start = max(0, round(start_ms * FPS / 1000.0))
    end = max(start, round(end_ms * FPS / 1000.0) - 1)
    return start, end


class MinecraftFont:
    def __init__(self, atlas: Image.Image):
        self.atlas = atlas.convert("RGBA")
        self.tiles: dict[str, tuple[Image.Image, int]] = {}

    def glyph(self, char: str) -> tuple[Image.Image, int]:
        if char in self.tiles:
            return self.tiles[char]
        code = ord(char)
        if code > 255:
            char, code = "?", ord("?")
        tile = self.atlas.crop(((code % 16) * 8, (code // 16) * 8, (code % 16 + 1) * 8, (code // 16 + 1) * 8))
        alpha = tile.getchannel("A")
        bbox = alpha.getbbox()
        width = 4 if char == " " else (min(7, bbox[2]) + 1 if bbox else 4)
        self.tiles[char] = (alpha, width)
        return alpha, width

    def width(self, text: str, scale: int) -> int:
        return sum(self.glyph(char)[1] * scale for char in text)

    def draw_centered(
        self,
        image: Image.Image,
        text: str,
        center_x: int,
        y: int,
        scale: int = 2,
        color: tuple[int, int, int, int] = (244, 247, 255, 255),
    ) -> None:
        x = center_x - self.width(text, scale) // 2
        for char in text:
            mask, advance = self.glyph(char)
            mask = mask.resize((8 * scale, 8 * scale), Image.Resampling.NEAREST)
            shadow = Image.new("RGBA", mask.size, (19, 24, 32, 255))
            ink = Image.new("RGBA", mask.size, color)
            image.paste(shadow, (x + scale, y + scale), mask)
            image.paste(ink, (x, y), mask)
            x += advance * scale


def phase_for(scene: str, frame: int, receipts: list[dict], count: int) -> str | None:
    meta = SCENES[scene]
    if scene == "overlap":
        return meta["fixed"]
    if not receipts:
        raise RuntimeError(f"{scene}: missing receipts")
    first, _ = frame_window(0, receipts[0]["start_ms"])
    if frame < round(receipts[0]["start_ms"] * FPS / 1000.0):
        return meta["intro"]
    for receipt, label in zip(receipts, meta["ops"]):
        start, end = frame_window(receipt["start_ms"], receipt["start_ms"] + receipt["duration_ms"])
        if start <= frame <= end:
            return label
    return meta["final"]


def label_frames(scene_dir: Path, scene: str, font: MinecraftFont) -> Path:
    frames = sorted(scene_dir.glob("f[0-9][0-9][0-9][0-9].png"))
    if not frames:
        raise RuntimeError(f"{scene}: no frames")
    receipts = json.loads((scene_dir / "receipts.json").read_text())
    if scene != "overlap" and len(receipts) != len(SCENES[scene]["ops"]):
        raise RuntimeError(f"{scene}: receipt/label count mismatch")
    out = scene_dir / "labelled"
    if out.exists():
        shutil.rmtree(out)
    out.mkdir()

    for index, path in enumerate(frames):
        image = Image.open(path).convert("RGBA")
        width, height = image.size
        draw = ImageDraw.Draw(image, "RGBA")
        draw.rectangle((0, 0, width, 38), fill=(9, 17, 29, 232))
        font.draw_centered(image, SCENES[scene]["title"], width // 2, 11)
        phase = phase_for(scene, index, receipts, len(frames))
        if phase:
            draw.rectangle((0, height - 38, width, height), fill=(9, 17, 29, 232))
            font.draw_centered(image, phase, width // 2, height - 27)
        if scene == "overlap":
            for x, label in SCENES[scene]["panels"]:
                draw.rectangle((x - 55, 46, x + 55, 72), fill=(9, 17, 29, 210))
                font.draw_centered(image, label, x, 52)
        image.convert("RGB").save(out / path.name, compress_level=2)
    return out


def encode(frames: Path, output: Path) -> None:
    graph = (
        "[0:v]split[s0][s1];"
        "[s0]palettegen=stats_mode=diff[p];"
        "[s1][p]paletteuse=dither=sierra2_4a[v]"
    )
    subprocess.run(
        [
            "ffmpeg", "-y", "-loglevel", "error", "-framerate", str(FPS),
            "-i", str(frames / "f%04d.png"), "-filter_complex", graph,
            "-map", "[v]", "-loop", "0", str(output),
        ],
        check=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("pack", type=Path)
    args = parser.parse_args()
    with zipfile.ZipFile(args.pack) as archive:
        with archive.open(ATLAS_PATH) as source:
            font = MinecraftFont(Image.open(source).copy())
    for scene in SCENES:
        frames = label_frames(args.root / scene, scene, font)
        output = args.root / f"{scene}.gif"
        encode(frames, output)
        print(output)


if __name__ == "__main__":
    main()
