"""Compose native SDF primitives into a small rocket and animate its assembly."""
from __future__ import annotations

import os
from collections import defaultdict
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig, Sdf

ROOT = Path(__file__).resolve().parents[3]
FPS = 20.0
EFFECT_MS = 5_000.0
STAGGER_MS = 1_500.0


def rocket_parts() -> tuple[Sdf, Sdf, Sdf, Sdf, Sdf, Sdf]:
    """Return the named terms used by the readable composition example."""
    body = Sdf.capped_cylinder(4.5, 8.0)
    nose = Sdf.capped_cone(4.0, 4.5, 0.0).translate(0.0, 12.0, 0.0)

    fin_x = Sdf.box_shape(2.0, 3.5, 1.2, 0.65)
    fin_z = Sdf.box_shape(1.2, 3.5, 2.0, 0.65)
    fins = fin_x.translate(4.5, -5.0, 0.0).union_with(
        fin_x.translate(-4.5, -5.0, 0.0)
    ).union_with(
        fin_z.translate(0.0, -5.0, 4.5)
    ).union_with(
        fin_z.translate(0.0, -5.0, -4.5)
    )
    nozzle = Sdf.capped_cone(2.0, 0.8, 1.9).translate(0.0, -10.0, 0.0)
    window_cut = Sdf.capped_cylinder(2.1, 1.2).rotate(90.0, 0.0, 0.0).translate(0.0, 3.5, 4.1)
    window_glass = Sdf.capped_cylinder(1.55, 0.55).rotate(90.0, 0.0, 0.0).translate(0.0, 3.5, 4.2)
    return body, nose, fins, nozzle, window_cut, window_glass


def rocket() -> Sdf:
    body, nose, fins, nozzle, window_cut, window_glass = rocket_parts()
    hull = body.smooth_union(nose, 1.1).smooth_union(fins, 0.75).union_with(nozzle)
    return hull.subtract(window_cut).union_with(window_glass)


def materialize() -> AnimationEffect:
    """Empty -> assembled hold -> empty, leaving construction as the focus."""
    effect = AnimationEffect.create(EFFECT_MS)
    for at, scale in (
        (0.00, 0.0),
        (0.04, 0.0),
        (0.15, 1.0),
        (0.76, 1.0),
        (0.89, 0.0),
        (1.00, 0.0),
    ):
        effect.add_keyframe("scale", at, scale, "inOutCubic")
    for at, opacity in (
        (0.00, 0.0),
        (0.05, 0.0),
        (0.13, 1.0),
        (0.78, 1.0),
        (0.90, 0.0),
        (1.00, 0.0),
    ):
        effect.add_keyframe("opacity", at, opacity, "inOutSine")
    return effect


def block_for(
    x: int, y: int, z: int, nose: Sdf, fins: Sdf, nozzle: Sdf, window_glass: Sdf
) -> str:
    if window_glass.eval_at(x, y, z) <= 0.0:
        return "minecraft:light_blue_stained_glass"
    if nozzle.eval_at(x, y, z) <= 0.0:
        return "minecraft:deepslate_tiles"
    if fins.eval_at(x, y, z) <= 0.0:
        return "minecraft:red_concrete"
    if nose.eval_at(x, y, z) <= 0.0:
        return "minecraft:red_concrete"
    return "minecraft:smooth_quartz"


def build_animation() -> BuildAnimation:
    body, nose, fins, nozzle, window_cut, window_glass = rocket_parts()
    field = rocket()

    # Small x-bands within each y-layer keep the arrival visibly block-like
    # without creating one render group per voxel.
    clusters: dict[tuple[int, int], list[tuple[int, int, int]]] = defaultdict(list)
    for y in range(-13, 17):
        for x in range(-8, 9):
            for z in range(-8, 9):
                if field.eval_at(x, y, z) <= 0.0:
                    clusters[(y, (x + 8) // 3)].append((x, y, z))

    animation = BuildAnimation.create("primitive-rocket")
    animation.set_default_effect(materialize())
    block_count = 0
    for order, key in enumerate(sorted(clusters)):
        animation.begin_keyed_group(float(order))
        for x, y, z in clusters[key]:
            animation.set_block(
                x, y, z, block_for(x, y, z, nose, fins, nozzle, window_glass)
            )
        animation.end_group()
        block_count += len(clusters[key])

    animation.set_stagger_total_ms(STAGGER_MS)
    print(f"rocket: {block_count} blocks in {len(clusters)} assembly clusters")
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
    config.set_zoom(1.18)
    config.set_background(0.025, 0.035, 0.055, 1.0)
    config.set_fitted_grid(1, 2, -10.502, False, 0.36, 0.45, 0.58, 0.18)

    pack = Path(
        os.environ.get("NUCLEATION_PACK", ROOT / "render_work/pack.zip")
    ).read_bytes()
    gif = media / "primitive-rocket.gif"
    schematic = downloads / "primitive-rocket.schem"
    frames = animation.render_gif(pack, config, str(gif), FPS, 0.0)
    animation.save_to_file(str(schematic))
    print(f"rendered {frames} frames to {gif}")
    print(f"saved {schematic}")


if __name__ == "__main__":
    main()
