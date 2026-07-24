"""Render a portable FieldProgram gyroid through Nucleation's animation engine.

The geometry, grouping, renderer, GIF encoder, and saved schematic all use the
public Python binding. No external compositor invents the demonstrated subject.
"""
from __future__ import annotations

import os
from math import cos, sin
from pathlib import Path

from nucleation import (
    AnimationEffect,
    BuildAnimation,
    FieldProgramBinaryOp as B,
    FieldProgramBuilder,
    FieldProgramDistanceKind as DistanceKind,
    FieldProgramUnaryOp as U,
    FieldProgramValueType as ValueType,
    RenderConfig,
    Sdf,
)

RADIUS = 15
FREQUENCY = 0.42
THICKNESS = 0.30
PERIOD_MS = 6_000.0
FPS = 20.0


def gyroid() -> Sdf:
    """Build ``abs(sin x cos y + sin y cos z + sin z cos x) - t``."""
    p = FieldProgramBuilder.create()
    q = p.add_slot(ValueType.Vec3)
    distance = p.add_slot(ValueType.Scalar)
    p.set_output(distance)
    p.set_bounds(-RADIUS, -RADIUS, -RADIUS, RADIUS, RADIUS, RADIUS)
    p.set_distance_kind(DistanceKind.Implicit)

    p.push_pos()
    p.push_const_scalar(FREQUENCY)
    p.binary_op(B.Scale)
    p.store_local(q)

    for sine_axis, cosine_axis in (
        (U.VecX, U.VecY),
        (U.VecY, U.VecZ),
        (U.VecZ, U.VecX),
    ):
        p.load_local(q)
        p.unary_op(sine_axis)
        p.unary_op(U.Sin)
        p.load_local(q)
        p.unary_op(cosine_axis)
        p.unary_op(U.Cos)
        p.binary_op(B.Mul)
    p.binary_op(B.Add)
    p.binary_op(B.Add)
    p.unary_op(U.Abs)
    p.push_const_scalar(THICKNESS)
    p.binary_op(B.Sub)
    p.store_local(distance)

    return Sdf.from_program(p.build()).intersection_with(Sdf.sphere(RADIUS - 0.5))


def pulse() -> AnimationEffect:
    effect = AnimationEffect.create(PERIOD_MS)
    for at, scale in ((0.0, 0.97), (0.18, 1.035), (0.38, 0.97), (1.0, 0.97)):
        effect.add_keyframe("scale", at, scale, "inOutSine")
    for at, strength in ((0.0, 0.0), (0.14, 0.7), (0.31, 0.0), (1.0, 0.0)):
        effect.add_keyframe("emissiveR", at, strength * 0.18, "inOutSine")
        effect.add_keyframe("emissiveG", at, strength * 0.80, "inOutSine")
        effect.add_keyframe("emissiveB", at, strength, "inOutSine")
    effect.set_repeat_forever()
    return effect


def material(x: int, y: int, z: int) -> str:
    phase = sin(x * 0.31) + cos(z * 0.27) + y / (RADIUS * 1.8)
    if (x * 19 + y * 29 + z * 37) % 61 == 0:
        return "minecraft:sea_lantern"
    if phase > 1.0:
        return "minecraft:light_blue_concrete"
    if phase > 0.25:
        return "minecraft:prismarine_bricks"
    if phase > -0.55:
        return "minecraft:warped_wart_block"
    return "minecraft:dark_prismarine"


def build_animation() -> BuildAnimation:
    field = gyroid()
    animation = BuildAnimation.create("field-program-gyroid")
    animation.set_default_effect(pulse())

    groups = 0
    blocks = 0
    for y in range(-RADIUS, RADIUS + 1):
        positions = [
            (x, y, z)
            for x in range(-RADIUS, RADIUS + 1)
            for z in range(-RADIUS, RADIUS + 1)
            if field.eval_at(x, y, z) <= 0.0
        ]
        if not positions:
            continue
        animation.begin_keyed_group(float(y + RADIUS))
        for x, yy, z in positions:
            animation.set_block(x, yy, z, material(x, yy, z))
        animation.end_group()
        groups += 1
        blocks += len(positions)

    animation.set_stagger_total_ms(PERIOD_MS * 0.82)
    animation.set_stagger_offset_ms(-PERIOD_MS)
    animation.set_loop_period_ms(PERIOD_MS)
    camera = AnimationEffect.turntable(PERIOD_MS)
    camera.set_repeat_forever()
    animation.animate_camera(camera, 0.0)
    print(f"gyroid: {blocks} blocks in {groups} animated layers")
    return animation


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    media = root / "docs/media/features/sdf-and-fields"
    downloads = root / "docs/downloads/features/sdf-and-fields"
    media.mkdir(parents=True, exist_ok=True)
    downloads.mkdir(parents=True, exist_ok=True)

    animation = build_animation()
    config = RenderConfig.create(420, 420)
    config.set_isometric()
    config.set_sphere_fit(True)
    config.set_zoom(1.22)
    config.set_background(0.025, 0.035, 0.055, 1.0)

    pack = Path(
        os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")
    ).read_bytes()
    gif = media / "gyroid-bloom.gif"
    schematic = downloads / "gyroid-bloom.schem"
    frames = animation.render_gif(pack, config, str(gif), FPS, 0.0)
    animation.save_to_file(str(schematic))
    print(f"rendered {frames} frames to {gif}")
    print(f"saved {schematic}")


if __name__ == "__main__":
    main()
