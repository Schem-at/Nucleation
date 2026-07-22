"""Generate the five-line beacon example for docs/features/basics.md."""

import os
from pathlib import Path

from nucleation import RenderConfig, Renderer, Schematic


def make_base() -> Schematic:
    scene = Schematic.create("beacon_base")

    # A clipped dark pedestal gives the tiny mineral pyramid a clear silhouette.
    for x in range(9):
        for z in range(9):
            dx, dz = x - 4, z - 4
            if abs(dx) + abs(dz) <= 7:
                block = (
                    "minecraft:chiseled_polished_blackstone"
                    if (abs(dx), abs(dz)) in {(4, 3), (3, 4)}
                    else "minecraft:polished_blackstone_bricks"
                )
                scene.set_block(x, 0, z, block)

    # Two small, valid beacon tiers.
    for x in range(2, 7):
        for z in range(2, 7):
            scene.set_block(x, 1, z, "minecraft:iron_block")
    for x in range(3, 6):
        for z in range(3, 6):
            scene.set_block(x, 2, z, "minecraft:gold_block")

    # Four low accents frame the empty beacon position without obscuring it.
    for x, z in ((2, 2), (2, 6), (6, 2), (6, 6)):
        scene.set_block(x, 2, z, "minecraft:light_blue_stained_glass")

    return scene


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    base_out = root / "docs/downloads/readme/basics/beacon-base.schem"
    image_out = root / "docs/media/readme/basics/beacon.png"
    pack_path = Path(
        os.environ.get(
            "NUCLEATION_PACK",
            root / "render_work/pack.zip",
        )
    )

    base = make_base()
    base_out.parent.mkdir(parents=True, exist_ok=True)
    image_out.parent.mkdir(parents=True, exist_ok=True)
    base.save_to_file(str(base_out))

    completed = Schematic.load_from_file(str(base_out))
    completed.set_block(4, 3, 4, "minecraft:beacon")

    view = RenderConfig.create(560, 420)
    view.set_isometric()
    view.set_yaw(28.0)
    view.set_pitch(24.0)
    view.set_zoom(1.18)
    view.set_sphere_fit(True)
    view.set_background(0, 0, 0, 0)
    Renderer.render_to_file(completed, pack_path.read_bytes(), view, str(image_out))

    print(f"saved {base_out}")
    print(f"rendered {image_out}")


if __name__ == "__main__":
    main()
