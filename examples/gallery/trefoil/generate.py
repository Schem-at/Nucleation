"""Generate the seamless animated trefoil through Nucleation's public Python API."""

from __future__ import annotations

import colorsys
import os
from math import cos, pi, sin
from pathlib import Path

from nucleation import (
    AnimationEffect,
    Brush,
    BuildAnimation,
    Curve3D,
    InterpolationSpace,
    PaletteBuilder,
    RenderConfig,
    Shape,
)

STEPS = 520
TUBE_RADIUS = 2.85
SCALE = 11.0
GROUPS = 90
PERIOD_MS = 6_000.0
SPREAD_MS = PERIOD_MS * 0.40
APPEAR_END = 0.10
VANISH_START = 0.58
VANISH_END = 0.68
FLY_HEIGHT = 7.0
FPS = 20.0


def trefoil(t: float) -> tuple[float, float, float]:
    """Sample the closed trefoil at normalized parameter ``t``."""
    angle = t * 2.0 * pi
    return (
        SCALE * (sin(angle) + 2.0 * sin(2.0 * angle)),
        SCALE * (cos(angle) - 2.0 * cos(2.0 * angle)),
        SCALE * -sin(3.0 * angle),
    )


def rgb(hue: float, saturation: float = 0.82, value: float = 1.0) -> tuple[int, int, int]:
    red, green, blue = colorsys.hsv_to_rgb(hue % 1.0, saturation, value)
    return round(red * 255), round(green * 255), round(blue * 255)


def make_wave() -> AnimationEffect:
    wave = AnimationEffect.create(PERIOD_MS)
    for at, value, easing in (
        (0.00, 0.0, "linear"),
        (APPEAR_END, 1.0, "outBack"),
        (VANISH_START, 1.0, "linear"),
        (VANISH_END, 0.0, "inCubic"),
        (1.00, 0.0, "linear"),
    ):
        wave.add_keyframe("scale", at, value, easing)
    for at, value, easing in (
        (0.00, FLY_HEIGHT, "linear"),
        (APPEAR_END, 0.0, "outCubic"),
        (VANISH_START, 0.0, "linear"),
        (VANISH_END, FLY_HEIGHT, "inCubic"),
        (1.00, FLY_HEIGHT, "linear"),
    ):
        wave.add_keyframe("y", at, value, easing)

    # Brief warm emissive pulse as each group lands.
    for index in range(40):
        at = index / 39
        distance = abs(at - APPEAR_END) / 0.06
        strength = 0.0 if distance >= 1.0 else (1.0 - distance) ** 2 * 0.5
        wave.add_keyframe("emissiveR", at, strength, "linear")
        wave.add_keyframe("emissiveG", at, strength * 170 / 255, "linear")
        wave.add_keyframe("emissiveB", at, strength * 80 / 255, "linear")

    wave.set_repeat_forever()
    return wave


def build_animation() -> BuildAnimation:
    coordinates = [component for index in range(STEPS) for component in trefoil(index / STEPS)]
    curve = Curve3D.from_points(coordinates, True)
    shape = Shape.tube_along(curve, TUBE_RADIUS)

    palette_builder = PaletteBuilder.create()
    palette_builder.full_blocks_only()
    palette_builder.exclude_transparent()
    palette_builder.exclude_tile_entities()
    palette = palette_builder.build()

    stops = [index / 12 for index in range(13)]
    colors = [component for t in stops for component in rgb(1.0 - t)]
    brush = Brush.curve_gradient(stops, colors, InterpolationSpace.Oklab)
    brush.set_palette(palette)

    animation = BuildAnimation.create("trefoil")
    created_groups = animation.fill_along_parameter(shape, brush, GROUPS)
    if created_groups != GROUPS:
        raise RuntimeError(f"expected {GROUPS} non-empty groups, got {created_groups}")

    animation.set_default_effect(make_wave())
    animation.set_stagger_total_ms(SPREAD_MS)
    animation.set_stagger_offset_ms(-PERIOD_MS)
    animation.set_loop_period_ms(PERIOD_MS)

    camera = AnimationEffect.turntable(PERIOD_MS)
    camera.set_repeat_forever()
    animation.animate_camera(camera, 0.0)
    return animation


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    pack_path = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip"))
    gif_path = Path(
        os.environ.get("NUCLEATION_OUT", root / "render_work/trefoil/python-trefoil.gif")
    )
    schematic_path = Path(
        os.environ.get(
            "NUCLEATION_SCHEM_OUT",
            root / "render_work/trefoil/python-trefoil.schem",
        )
    )
    gif_path.parent.mkdir(parents=True, exist_ok=True)
    schematic_path.parent.mkdir(parents=True, exist_ok=True)

    animation = build_animation()
    config = RenderConfig.create(480, 480)
    config.set_isometric()
    config.set_sphere_fit(True)
    config.set_background(0.0, 0.0, 0.0, 0.0)

    frames = animation.render_gif(pack_path.read_bytes(), config, str(gif_path), FPS, 0.0)
    animation.save_to_file(str(schematic_path))
    print(f"groups: {animation.group_count()}")
    print(f"frames: {frames} at {FPS:g} FPS ({PERIOD_MS / 1000:g}s loop)")
    print(f"saved: {schematic_path}")
    print(f"rendered: {gif_path}")


if __name__ == "__main__":
    main()
