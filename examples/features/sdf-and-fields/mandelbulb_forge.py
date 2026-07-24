"""Render the portable Mandelbulb FieldProgram as a radial forge animation."""
from __future__ import annotations

import os
import sys
from collections import defaultdict
from math import atan2, pi, sqrt
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "examples"))
from field_program_mandelbulb import mandelbulb  # noqa: E402

SCALE = 13.5
LIMIT = 18
GROUPS = 24
PERIOD_MS = 6_000.0
FPS = 20.0


def pulse() -> AnimationEffect:
    effect = AnimationEffect.create(PERIOD_MS)
    for at, scale in ((0.0, 0.96), (0.17, 1.045), (0.36, 0.96), (1.0, 0.96)):
        effect.add_keyframe("scale", at, scale, "inOutSine")
    for at, strength in ((0.0, 0.0), (0.12, 0.78), (0.30, 0.0), (1.0, 0.0)):
        effect.add_keyframe("emissiveR", at, strength, "inOutSine")
        effect.add_keyframe("emissiveG", at, strength * 0.32, "inOutSine")
        effect.add_keyframe("emissiveB", at, strength * 0.58, "inOutSine")
    effect.set_repeat_forever()
    return effect


def material(x: int, y: int, z: int) -> str:
    angle = (atan2(z, x) / (2.0 * pi)) % 1.0
    height = (y + LIMIT) / (2 * LIMIT)
    if (x * 17 + y * 31 + z * 13) % 43 == 0:
        return "minecraft:sea_lantern"
    if height > 0.68:
        return "minecraft:amethyst_block"
    if angle < 0.28:
        return "minecraft:magenta_concrete"
    if angle < 0.62:
        return "minecraft:blue_concrete"
    return "minecraft:purple_concrete"


def build_animation() -> BuildAnimation:
    # The program's 12-iteration loop runs in native code for every query.
    field = mandelbulb(iterations=12).scale(SCALE)
    shells: dict[int, list[tuple[int, int, int]]] = defaultdict(list)
    max_radius = float(LIMIT)
    for x in range(-LIMIT, LIMIT + 1):
        for y in range(-LIMIT, LIMIT + 1):
            for z in range(-LIMIT, LIMIT + 1):
                if field.eval_at(x, y, z) <= 0.0:
                    radius = sqrt(x * x + y * y + z * z)
                    shell = min(GROUPS - 1, int(radius / max_radius * GROUPS))
                    shells[shell].append((x, y, z))

    animation = BuildAnimation.create("field-program-mandelbulb")
    animation.set_default_effect(pulse())
    block_count = 0
    for shell in sorted(shells):
        animation.begin_keyed_group(float(shell))
        for x, y, z in shells[shell]:
            animation.set_block(x, y, z, material(x, y, z))
        animation.end_group()
        block_count += len(shells[shell])

    animation.set_stagger_total_ms(PERIOD_MS * 0.76)
    animation.set_stagger_offset_ms(-PERIOD_MS)
    animation.set_loop_period_ms(PERIOD_MS)
    camera = AnimationEffect.turntable(PERIOD_MS)
    camera.set_repeat_forever()
    animation.animate_camera(camera, 0.0)
    print(f"mandelbulb: {block_count} blocks in {len(shells)} radial shells")
    return animation


def main() -> None:
    media = ROOT / "docs/media/features/sdf-and-fields"
    downloads = ROOT / "docs/downloads/features/sdf-and-fields"
    media.mkdir(parents=True, exist_ok=True)
    downloads.mkdir(parents=True, exist_ok=True)

    animation = build_animation()
    config = RenderConfig.create(420, 420)
    config.set_isometric()
    config.set_sphere_fit(True)
    config.set_zoom(1.08)
    config.set_background(0.045, 0.018, 0.060, 1.0)

    pack = Path(
        os.environ.get("NUCLEATION_PACK", ROOT / "render_work/pack.zip")
    ).read_bytes()
    gif = media / "mandelbulb-forge.gif"
    schematic = downloads / "mandelbulb-forge.schem"
    frames = animation.render_gif(pack, config, str(gif), FPS, 0.0)
    animation.save_to_file(str(schematic))
    print(f"rendered {frames} frames to {gif}")
    print(f"saved {schematic}")


if __name__ == "__main__":
    main()
