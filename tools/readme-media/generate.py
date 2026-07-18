#!/usr/bin/env python3
"""Generate the README's images and GIFs using nucleation itself.

Every picture in the README is produced by this script through the Python
binding — the same API the README documents. Regenerate after visual changes:

    pip install nucleation  # or a locally built wheel (bridge-full)
    python3 tools/readme-media/generate.py --pack /path/to/resource-pack.zip

The resource pack is any vanilla-format pack zip (assets/minecraft/...);
it is NOT committed to the repo. Output lands in docs/media/.
GIF assembly and image compositing need ffmpeg on PATH; there are no other
dependencies (no Pillow/numpy).

    --only hero,torus     regenerate a subset (see SCENES for names)
"""

import argparse
import colorsys
import json
import math
import os
import shutil
import subprocess
import tempfile

import nucleation as nu

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
OUT = os.path.join(ROOT, "docs", "media")

# GitHub-dark-ish navy, for scenes that read badly on transparency.
NAVY = (0.086, 0.098, 0.149, 1.0)


def render(schematic, pack, path, w=880, h=620, yaw=None, pitch=None, zoom=None,
           ortho=False, background=(0, 0, 0, 0)):
    cfg = nu.RenderConfig.create(w, h)
    cfg.set_isometric()
    if yaw is not None:
        cfg.set_yaw(yaw)
    if pitch is not None:
        cfg.set_pitch(pitch)
    if zoom is not None:
        cfg.set_zoom(zoom)
    if ortho:
        cfg.set_orthographic(True)
    r, g, b, a = background
    cfg.set_background(r, g, b, a)
    nu.Renderer.render_to_file(schematic, pack, cfg, path)
    print(f"  wrote {os.path.relpath(path, ROOT)}")


def assemble_gif(frame_dir, path, fps, max_colors=192, stats_mode="diff",
                 dither="bayer:bayer_scale=4"):
    """Palette-optimised GIF from f%03d.png frames in frame_dir.

    Flat-color scenes (e.g. the timelapse) want stats_mode="full" and
    dither="none"; textured turntables look better with the diff/bayer default.
    """
    palette = os.path.join(frame_dir, "palette.png")
    pattern = os.path.join(frame_dir, "f%03d.png")
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(fps),
                    "-i", pattern,
                    "-vf", f"palettegen=max_colors={max_colors}:stats_mode={stats_mode}",
                    palette], check=True)
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(fps),
                    "-i", pattern, "-i", palette,
                    "-lavfi", f"paletteuse=dither={dither}", "-loop", "0",
                    path], check=True)


def turntable_gif(schematic, pack, path, frames=40, w=560, h=400, pitch=None,
                  zoom=None, seconds=4.0):
    """Render a full-rotation turntable and assemble a looping GIF."""
    tmp = tempfile.mkdtemp(prefix="nuc-turntable-")
    try:
        for i in range(frames):
            cfg = nu.RenderConfig.create(w, h)
            cfg.set_isometric()
            cfg.set_yaw(360.0 * i / frames)
            if pitch is not None:
                cfg.set_pitch(pitch)
            if zoom is not None:
                cfg.set_zoom(zoom)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(schematic, pack, cfg,
                                       os.path.join(tmp, f"f{i:03}.png"))
        assemble_gif(tmp, path, fps=frames / seconds)
        print(f"  wrote {os.path.relpath(path, ROOT)} ({frames} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ── Scenes ───────────────────────────────────────────────────────────────────

def scene_hero(pack):
    """SDF floating island with material rules — the front-door image."""
    sdf = {
        "type": "smoothUnion", "k": 2.5,
        "a": {"type": "displace", "amplitude": 7.0, "frequency": 0.06, "seed": 7,
              "child": {"type": "ellipsoid", "radii": [30, 12, 30]}},
        "b": {"type": "translate", "offset": [26, 29, -14],
              "child": {"type": "sphere", "radius": 7.5}},
    }
    rules = {
        "fill": [
            {"when": {"depthBelowSurface": {"min": 0, "max": 0},
                      "yRange": {"min": 11, "max": 64}},
             "block": "minecraft:snow_block"},
            {"when": {"depthBelowSurface": {"min": 0, "max": 0},
                      "yRange": {"min": -4, "max": 10}},
             "block": "minecraft:grass_block"},
            {"when": {"depthBelowSurface": {"min": 0, "max": 0}},
             "block": "minecraft:stone"},
            {"when": {"depthBelowSurface": {"min": 1, "max": 3}},
             "block": "minecraft:dirt"},
            {"when": {"yRange": {"min": -64, "max": 12}},
             "gradient": {"palette": {"ids": [
                 "minecraft:deepslate", "minecraft:cobbled_deepslate",
                 "minecraft:tuff", "minecraft:stone", "minecraft:andesite"]},
                 "from": [70, 68, 72], "to": [150, 148, 152],
                 "axis": "y", "range": [-19, 10]}},
            {"block": "minecraft:stone"},  # catch-all (companion sphere core)
        ],
        "surface": [
            {"density": 0.10, "on": "minecraft:grass_block",
             "blocks": ["minecraft:poppy", "minecraft:dandelion",
                        "minecraft:short_grass", "minecraft:oxeye_daisy"]},
        ],
    }
    s = nu.Sdf.schematic_from_sdf(json.dumps(sdf), json.dumps(rules), True,
                                  -38, -21, -38, 38, 40, 38)
    render(s, pack, os.path.join(OUT, "hero.png"), w=1200, h=760, pitch=30, zoom=1.33)
    turntable_gif(s, pack, os.path.join(OUT, "hero-turntable.gif"),
                  pitch=28, zoom=1.18)
    return s


def scene_gradient_torus(pack):
    """Rainbow torus: Shape.torus + an IDW point-gradient brush over wool."""
    positions, colors = [], []
    n = 12
    for i in range(n):  # 12 rainbow-colored gradient points around the ring
        a = 2 * math.pi * i / n
        r, g, b = colorsys.hsv_to_rgb(i / n, 0.95, 0.95)
        positions += [round(16 * math.cos(a)), 0, round(16 * math.sin(a))]
        colors += [int(r * 255), int(g * 255), int(b * 255)]
    s = nu.Schematic.create("torus")
    shape = nu.Shape.torus(0, 0, 0, 16, 6, 0, 1, 0)
    brush = nu.Brush.point_gradient(positions, bytes(colors), 4.0,
                                    nu.InterpolationSpace.Oklab)
    brush.set_palette(nu.Palette.wool())
    nu.BuildingTool.fill(s, shape, brush)
    render(s, pack, os.path.join(OUT, "gradient-torus.png"), pitch=32, zoom=1.25)
    return s


# Escape-time iteration counts for the mandelbrot scenes (pure Python, no deps).
def _mandel_iters(size, max_iter):
    grid = []
    for pz in range(size):
        row = []
        for px in range(size):
            c = complex(-2.15 + 2.8 * px / size, -1.4 + 2.8 * pz / size)
            z, it = 0j, 0
            while abs(z) <= 2 and it < max_iter:
                z = z * z + c
                it += 1
            row.append(it)
        grid.append(row)
    return grid

# Hand-picked concrete ramp: inferno-ish, dark exterior -> hot boundary.
MANDEL_RAMP = ["minecraft:black_concrete", "minecraft:blue_concrete",
               "minecraft:purple_concrete", "minecraft:magenta_concrete",
               "minecraft:pink_concrete", "minecraft:orange_concrete",
               "minecraft:yellow_concrete", "minecraft:white_concrete"]
MANDEL_SIZE = 128
MANDEL_MAX = 40


def _mandel_block(it):
    if it >= MANDEL_MAX:
        return "minecraft:black_concrete"          # inside the set
    idx = min(len(MANDEL_RAMP) - 1, int(len(MANDEL_RAMP) * it / 18.0))
    return MANDEL_RAMP[idx]


def scene_mandelbrot(pack):
    """Pixel-art workflow: value -> curated block ramp, top-down ortho."""
    iters = _mandel_iters(MANDEL_SIZE, MANDEL_MAX)
    s = nu.Schematic.create("mandelbrot")
    for pz in range(MANDEL_SIZE):
        for px in range(MANDEL_SIZE):
            s.set_block(px, 0, pz, _mandel_block(iters[pz][px]))
    render(s, pack, os.path.join(OUT, "mandelbrot.png"), w=1024, h=1024,
           yaw=0.0, pitch=89.9, ortho=True)
    return s


def scene_timelapse(pack):
    """Build timelapse: the mandelbrot appears in escape-time order."""
    iters = _mandel_iters(MANDEL_SIZE, MANDEL_MAX)
    tmp = tempfile.mkdtemp(prefix="nuc-timelapse-")
    thresholds = list(range(1, 19)) + [22, 26, 31, 36, MANDEL_MAX]
    try:
        frame = 0
        for t in thresholds:
            s = nu.Schematic.create("stage")
            for pz in range(MANDEL_SIZE):
                for px in range(MANDEL_SIZE):
                    it = iters[pz][px]
                    if it <= t:
                        s.set_block(px, 0, pz, _mandel_block(it))
            cfg = nu.RenderConfig.create(512, 512)
            cfg.set_isometric()
            cfg.set_yaw(0.0)
            cfg.set_pitch(89.9)
            cfg.set_orthographic(True)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(s, pack, cfg,
                                       os.path.join(tmp, f"f{frame:03}.png"))
            frame += 1
        # hold the finished build for a beat
        for _ in range(6):
            shutil.copyfile(os.path.join(tmp, f"f{frame - 1:03}.png"),
                            os.path.join(tmp, f"f{frame:03}.png"))
            frame += 1
        assemble_gif(tmp, os.path.join(OUT, "build-timelapse.gif"), fps=8,
                     max_colors=64, stats_mode="full", dither="none")
        print(f"  wrote docs/media/build-timelapse.gif ({frame} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def scene_shapes(pack):
    """One schematic, five shapes, five materials: Shape + BuildingTool.fill."""
    s = nu.Schematic.create("shapes")
    fill = nu.BuildingTool.fill
    fill(s, nu.Shape.sphere(0, 8, 0, 8), nu.Brush.solid("minecraft:red_concrete"))
    fill(s, nu.Shape.torus(24, 3, 0, 8, 3, 0, 1, 0), nu.Brush.solid("minecraft:gold_block"))
    fill(s, nu.Shape.cone(46, 16, 0, 0, -1, 0, 8, 16), nu.Brush.solid("minecraft:emerald_block"))
    fill(s, nu.Shape.pyramid(68, 0, 0, 8, 8, 14, 0, 1, 0), nu.Brush.solid("minecraft:quartz_block"))
    ribbon = nu.Shape.bezier([84, 3, 0, 94, 20, -8, 110, 0, 6, 120, 16, -2], 3.0, 64)
    fill(s, ribbon, nu.Brush.solid("minecraft:lapis_block"))
    render(s, pack, os.path.join(OUT, "shapes-gallery.png"), w=1400, h=480,
           yaw=12, pitch=16, zoom=1.09)
    return s


def scene_masked_fill(pack):
    """Before/after: fill_replacing weathers stone_bricks inside a sphere."""
    def castle():
        s = nu.Schematic.create("castle")
        stone = nu.Brush.solid("minecraft:stone_bricks")
        fill = nu.BuildingTool.fill
        fill(s, nu.Shape.cuboid(-13, 0, -13, 13, 0, 13), stone)  # courtyard
        walls = nu.Shape.cuboid(-14, 0, -14, 14, 8, 14).difference_with(
            nu.Shape.cuboid(-13, 0, -13, 13, 9, 13))
        fill(s, walls, stone)
        for tx, tz in [(-14, -14), (-14, 14), (14, -14), (14, 14)]:
            fill(s, nu.Shape.cylinder(tx, 0, tz, 0, 1, 0, 4, 13), stone)
            fill(s, nu.Shape.cone(tx, 13, tz, 0, 1, 0, 5, 6),
                 nu.Brush.solid("minecraft:dark_oak_planks"))
        fill(s, nu.Shape.cuboid(-5, 1, -5, 5, 14, 5).hollow(1), stone)
        fill(s, nu.Shape.pyramid(0, 15, 0, 7, 7, 6, 0, 1, 0),
             nu.Brush.solid("minecraft:dark_oak_planks"))
        return s

    before = castle()
    after = castle()
    # weather one corner: swap stone_bricks -> mossy/cracked inside a sphere
    decay = nu.Shape.sphere(-14, 2, -14, 16)
    weathered = nu.Brush.linear_gradient(-14, 0, -14, 90, 130, 70,
                                         2, 10, 2, 115, 118, 108,
                                         nu.InterpolationSpace.Oklab)
    weathered.set_palette(nu.Palette.from_block_ids(json.dumps(
        ["minecraft:mossy_cobblestone", "minecraft:mossy_stone_bricks",
         "minecraft:cracked_stone_bricks"])))
    nu.BuildingTool.fill_replacing(after, decay, weathered,
                                   json.dumps(["minecraft:stone_bricks"]))

    tmp = tempfile.mkdtemp(prefix="nuc-masked-")
    try:
        a = os.path.join(tmp, "before.png")
        b = os.path.join(tmp, "after.png")
        for s, path in [(before, a), (after, b)]:
            render(s, pack, path, w=660, h=560, yaw=225, pitch=24, zoom=1.18)
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", a, "-i", b,
                        "-filter_complex", "[0][1]hstack", os.path.join(OUT, "masked-fill.png")],
                       check=True)
        print("  wrote docs/media/masked-fill.png")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return after


def scene_palette_ramps(pack):
    """A literal picture of Palette ramps: one row per palette, light-sorted."""
    s = nu.Schematic.create("ramps")
    palettes = [nu.Palette.wool(), nu.Palette.concrete(),
                nu.Palette.terracotta(), nu.Palette.wood()]
    widest = 17  # terracotta
    for row, palette in enumerate(palettes):
        ids = json.loads(palette.sorted_by_lightness().block_ids_json())
        x0 = (widest - len(ids)) // 2
        y0 = 3 * (len(palettes) - 1 - row)
        for col, block_id in enumerate(ids):
            s.set_block(x0 + col, y0, 0, block_id)
            s.set_block(x0 + col, y0 + 1, 0, block_id)
    render(s, pack, os.path.join(OUT, "palette-ramps.png"), w=1360, h=870,
           yaw=0.0, pitch=0.0, ortho=True)
    return s


SCENES = {
    "hero": scene_hero,
    "torus": scene_gradient_torus,
    "mandelbrot": scene_mandelbrot,
    "shapes": scene_shapes,
    "masked-fill": scene_masked_fill,
    "palette-ramps": scene_palette_ramps,
    "timelapse": scene_timelapse,
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pack", required=True, help="resource pack zip")
    ap.add_argument("--only", help="comma-separated scene names")
    args = ap.parse_args()
    os.makedirs(OUT, exist_ok=True)
    pack = open(args.pack, "rb").read()
    names = args.only.split(",") if args.only else list(SCENES)
    for name in names:
        print(f"scene: {name}")
        SCENES[name](pack)


if __name__ == "__main__":
    main()
