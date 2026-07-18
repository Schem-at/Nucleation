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
import base64
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


def hstack(paths, out):
    """Composite same-height panels side by side."""
    inputs = [a for p in paths for a in ("-i", p)]
    refs = "".join(f"[{i}]" for i in range(len(paths)))
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *inputs,
                    "-filter_complex", f"{refs}hstack=inputs={len(paths)}", out],
                   check=True)
    print(f"  wrote {os.path.relpath(out, ROOT)}")


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
    """Render a full-rotation turntable and assemble a looping GIF.

    sphere_fit keeps the orbit distance constant across yaws so the subject
    doesn't "breathe" as its screen-space bounding box changes.
    """
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
            cfg.set_sphere_fit(True)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(schematic, pack, cfg,
                                       os.path.join(tmp, f"f{i:03}.png"))
        assemble_gif(tmp, path, fps=frames / seconds)
        print(f"  wrote {os.path.relpath(path, ROOT)} ({frames} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ── Scenes ───────────────────────────────────────────────────────────────────

def scene_hero(pack):
    """SDF volcano island with material rules — the front-door image."""
    base = {"type": "smoothUnion", "k": 8.0,
            "a": {"type": "ellipsoid", "radii": [28, 11, 24]},
            "b": {"type": "translate", "offset": [12, 2, 12],
                  "child": {"type": "ellipsoid", "radii": [18, 9, 16]}}}
    peak = {"type": "translate", "offset": [-10, 10, -6],
            "child": {"type": "cappedCone", "halfHeight": 20, "r1": 18, "r2": 8}}
    mass = {"type": "displace", "amplitude": 9.0, "frequency": 0.055, "seed": 7,
            "octaves": 4,
            "child": {"type": "smoothUnion", "k": 5.0, "a": base, "b": peak}}
    root = {"type": "displace", "amplitude": 5.0, "frequency": 0.10, "seed": 12,
            "child": {"type": "translate", "offset": [2, -16, 2],
                      "child": {"type": "cappedCone", "halfHeight": 15,
                                "r1": 1.5, "r2": 20}}}
    arch = {"type": "translate", "offset": [26, 1, 12],
            "child": {"type": "rotate", "angles": [90, 0, 25],
                      "child": {"type": "torus", "majorRadius": 12,
                                "minorRadius": 3.0}}}
    shards = {"type": "union", "children": [
        {"type": "translate", "offset": [36, -4, -18],
         "child": {"type": "displace", "amplitude": 2.0, "frequency": 0.25,
                   "seed": 21, "child": {"type": "sphere", "radius": 4.5}}},
        {"type": "translate", "offset": [-38, -4, 24],
         "child": {"type": "displace", "amplitude": 2.5, "frequency": 0.22,
                   "seed": 22, "child": {"type": "sphere", "radius": 5.5}}},
        {"type": "translate", "offset": [18, -14, 34],
         "child": {"type": "displace", "amplitude": 1.5, "frequency": 0.3,
                   "seed": 23, "child": {"type": "sphere", "radius": 3.5}}},
    ]}
    island = {"type": "union", "children": [
        {"type": "smoothUnion", "k": 3.0,
         "a": {"type": "smoothUnion", "k": 5.0, "a": mass, "b": root},
         "b": arch},
        shards]}
    crater = {"type": "translate", "offset": [-10, 32, -6],
              "child": {"type": "cappedCylinder", "radius": 5.5, "halfHeight": 8}}
    sdf = {"type": "smoothSubtract", "k": 1.0, "a": island, "b": crater}

    rules = {
        "fill": [
            {"when": {"depthBelowSurface": {"min": 0, "max": 0},
                      "yRange": {"min": 20, "max": 64},
                      "noise": {"threshold": -0.25, "frequency": 0.15, "seed": 3}},
             "block": "minecraft:snow_block"},
            {"when": {"depthBelowSurface": {"min": 0, "max": 0},
                      "yRange": {"min": -3, "max": 16}},
             "block": "minecraft:grass_block"},
            {"when": {"depthBelowSurface": {"min": 0, "max": 0}},
             "block": "minecraft:stone"},
            {"when": {"depthBelowSurface": {"min": 1, "max": 2},
                      "yRange": {"min": -3, "max": 16}},
             "block": "minecraft:dirt"},
            {"when": {"yRange": {"min": -64, "max": 15}},
             "gradient": {"palette": {"ids": [
                 "minecraft:deepslate", "minecraft:cobbled_deepslate",
                 "minecraft:tuff", "minecraft:stone", "minecraft:andesite"]},
                 "from": [70, 68, 72], "to": [150, 148, 152],
                 "axis": "y", "range": [-28, 14]}},
            {"block": "minecraft:stone"},
        ],
        "surface": [
            {"density": 0.10, "on": "minecraft:grass_block",
             "blocks": ["minecraft:poppy", "minecraft:dandelion",
                        "minecraft:short_grass", "minecraft:oxeye_daisy"]},
        ],
    }
    s = nu.Sdf.schematic_from_sdf(json.dumps(sdf), json.dumps(rules), True,
                                  -44, -34, -40, 48, 42, 44)
    # lava pool in the crater (fills only air, so the rim stays intact)
    nu.BuildingTool.fill_replacing(s, nu.Shape.cylinder(-10, 24, -6, 0, 1, 0, 6, 3),
                                   nu.Brush.solid("minecraft:lava"),
                                   json.dumps(["minecraft:air"]))
    cfg = nu.RenderConfig.create(1200, 760)
    cfg.set_isometric(); cfg.set_yaw(135.0); cfg.set_pitch(26.0)
    cfg.set_zoom(1.45); cfg.set_sphere_fit(True)
    cfg.set_background(0, 0, 0, 0)
    nu.Renderer.render_to_file(s, pack, cfg, os.path.join(OUT, "hero.png"))
    print("  wrote docs/media/hero.png")
    turntable_gif(s, pack, os.path.join(OUT, "hero-turntable.gif"),
                  pitch=28, zoom=1.4)
    return s


def scene_gradient_torus(pack):
    """Rainbow torus: a curve gradient runs along the ring's parameter."""
    stops = [i / 6 for i in range(7)]
    colors = [255, 40, 40,   255, 180, 0,   60, 200, 60,
              40, 180, 220,  60, 70, 230,   200, 60, 220,
              255, 40, 40]  # first == last -> seamless wrap
    s = nu.Schematic.create("torus")
    brush = nu.Brush.curve_gradient(stops, bytes(colors), nu.InterpolationSpace.Oklab)
    brush.set_palette(nu.Palette.wool())
    nu.BuildingTool.fill(s, nu.Shape.torus(0, 0, 0, 16, 6, 0, 1, 0), brush)
    render(s, pack, os.path.join(OUT, "gradient-torus.png"), pitch=32, zoom=1.25)
    return s


def scene_shaded_sphere(pack):
    """Lambertian-shaded brush: base color lit from the upper left, snapped
    to the terracotta palette."""
    s = nu.Schematic.create("shaded")
    brush = nu.Brush.shaded(224, 130, 84, -1.0, 0.7, -0.3)
    brush.set_palette(nu.Palette.terracotta())
    nu.BuildingTool.fill(s, nu.Shape.sphere(0, 0, 0, 16), brush)
    render(s, pack, os.path.join(OUT, "shaded-sphere.png"), w=620, h=540,
           yaw=45, pitch=25, zoom=1.2)
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
    """Before/after: fill_replacing ages a Greek temple inside a sphere."""
    def temple():
        s = nu.Schematic.create("temple")
        fill = nu.BuildingTool.fill
        bricks = nu.Brush.solid("minecraft:stone_bricks")
        # three-stepped platform (stylobate)
        for i in range(3):
            fill(s, nu.Shape.cuboid(-13 + i, i, -9 + i, 13 - i, i, 9 - i), bricks)
        # peripteral colonnade
        cols = [(x, z) for x in (-10, -5, 0, 5, 10) for z in (-6, 6)]
        cols += [(x, z) for x in (-10, 10) for z in (-2, 2)]
        for x, z in cols:
            fill(s, nu.Shape.cylinder(x, 3, z, 0, 1, 0, 0.8, 7), bricks)
        # altar in the cella
        fill(s, nu.Shape.cuboid(-1, 3, -1, 1, 4, 1), bricks)
        # architrave + frieze
        fill(s, nu.Shape.cuboid(-11, 10, -7, 11, 10, 7), bricks)
        fill(s, nu.Shape.cuboid(-11, 11, -7, 11, 11, 7),
             nu.Brush.solid("minecraft:chiseled_stone_bricks"))
        # stepped gable roof along the long axis
        for i, z in enumerate((6, 4, 2, 0)):
            fill(s, nu.Shape.cuboid(-12 + i, 12 + i, -z, 12 - i, 12 + i, z), bricks)
        return s

    before = temple()
    after = temple()
    # age one corner: swap stone_bricks -> mossy/cracked inside a sphere
    decay = nu.Shape.sphere(-13, 1, -9, 17)
    aged = nu.Brush.linear_gradient(-13, 0, -9, 80, 110, 60,
                                    1, 12, 1, 125, 122, 116,
                                    nu.InterpolationSpace.Oklab)
    aged.set_palette(nu.Palette.from_block_ids(json.dumps(
        ["minecraft:mossy_stone_bricks", "minecraft:mossy_cobblestone",
         "minecraft:cracked_stone_bricks", "minecraft:cobblestone"])))
    nu.BuildingTool.fill_replacing(after, decay, aged,
                                   json.dumps(["minecraft:stone_bricks"]))

    tmp = tempfile.mkdtemp(prefix="nuc-masked-")
    try:
        panels = []
        for s, name in [(before, "before"), (after, "after")]:
            p = os.path.join(tmp, f"{name}.png")
            cfg = nu.RenderConfig.create(700, 560)
            cfg.set_isometric(); cfg.set_yaw(215.0); cfg.set_pitch(24.0)
            cfg.set_zoom(1.12)
            cfg.set_background(0, 0, 0, 0)
            nu.Renderer.render_to_file(s, pack, cfg, p)
            panels.append(p)
        hstack(panels, os.path.join(OUT, "masked-fill.png"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return after


def scene_redstone(pack):
    """Before/after: the README's lever->wire->lamp circuit, simulated."""
    def circuit():
        s = nu.Schematic.create("lamp_circuit")
        for x in range(3):
            s.set_block(x, 0, 0, "minecraft:gray_concrete")
        s.set_block_from_string(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]")
        s.set_block_from_string(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]")
        s.set_block_from_string(2, 1, 0, "minecraft:redstone_lamp[lit=false]")
        return s

    before = circuit()
    world = nu.MchprsWorld.create(circuit())
    world.on_use_block(0, 1, 0)   # flip the lever
    world.tick(2)
    world.flush()
    world.sync_to_schematic()
    after = world.get_schematic()

    tmp = tempfile.mkdtemp(prefix="nuc-redstone-")
    try:
        panels = []
        for s, name in [(before, "before"), (after, "after")]:
            p = os.path.join(tmp, f"{name}.png")
            cfg = nu.RenderConfig.create(560, 420)
            cfg.set_isometric(); cfg.set_yaw(210.0); cfg.set_pitch(28.0)
            cfg.set_zoom(1.1)
            cfg.set_background(0, 0, 0, 0)
            nu.Renderer.render_to_file(s, pack, cfg, p)
            panels.append(p)
        hstack(panels, os.path.join(OUT, "redstone.png"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return after


def scene_diff(pack):
    """Triptych: before | after | Diff.compute changes over a glass ghost."""
    def cottage():
        s = nu.Schematic.create("cottage")
        fill = nu.BuildingTool.fill
        planks = nu.Brush.solid("minecraft:oak_planks")
        bricks = nu.Brush.solid("minecraft:stone_bricks")
        fill(s, nu.Shape.cuboid(0, 0, 0, 8, 0, 6), bricks)            # floor
        fill(s, nu.Shape.cuboid(0, 1, 0, 8, 3, 6).hollow(1), planks)  # walls
        fill(s, nu.Shape.cuboid(0, 4, 0, 8, 4, 6), bricks)            # roof
        for y in (1, 2):                                              # doorway
            s.set_block(4, y, 0, "minecraft:air")
        # garden lantern post
        fill(s, nu.Shape.cuboid(11, 0, 1, 11, 5, 1), nu.Brush.solid("minecraft:oak_log"))
        s.set_block(11, 6, 1, "minecraft:glowstone")
        return s

    def renovate(s):
        nu.BuildingTool.fill(s, nu.Shape.cuboid(7, 5, 5, 7, 6, 5),
                             nu.Brush.solid("minecraft:bricks"))       # chimney
        for x in (2, 6):
            s.set_block(x, 2, 0, "minecraft:air")                      # cut windows

    before = cottage()
    after = cottage()
    renovate(after)
    diff = json.loads(nu.Diff.compute(before, after, "exact").to_json())

    # changes view: glass ghost of the union, lime = added, red = removed
    changes = cottage()
    renovate(changes)
    for b in json.loads(changes.get_all_blocks_json()):
        if b["name"] != "minecraft:air":
            changes.set_block(b["x"], b["y"], b["z"],
                              "minecraft:light_gray_stained_glass")
    for e in diff["added"]:
        changes.set_block(*e["pos"], "minecraft:lime_concrete")
    for e in diff["removed"]:
        changes.set_block(*e["pos"], "minecraft:red_concrete")

    tmp = tempfile.mkdtemp(prefix="nuc-diff-")
    try:
        panels = []
        for s, name in [(before, "a"), (after, "b"), (changes, "c")]:
            p = os.path.join(tmp, f"{name}.png")
            cfg = nu.RenderConfig.create(460, 400)
            cfg.set_isometric(); cfg.set_yaw(215.0); cfg.set_pitch(26.0)
            cfg.set_zoom(1.1)
            cfg.set_background(0, 0, 0, 0)
            nu.Renderer.render_to_file(s, pack, cfg, p)
            panels.append(p)
        hstack(panels, os.path.join(OUT, "diff-engine.png"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return changes


def scene_autostack(pack):
    """Before/after: detect the repeating wall module, restamp it 6 wide."""
    def wall(units):
        s = nu.Schematic.create("wall")
        fill = nu.BuildingTool.fill
        bricks = nu.Brush.solid("minecraft:stone_bricks")
        logs = nu.Brush.solid("minecraft:dark_oak_log")
        for u in range(units):
            x = u * 6
            fill(s, nu.Shape.cuboid(x, 1, 0, x + 5, 5, 0), bricks)   # panel
            fill(s, nu.Shape.cuboid(x, 0, 0, x + 5, 0, 0),
                 nu.Brush.solid("minecraft:polished_andesite"))       # plinth
            fill(s, nu.Shape.cuboid(x, 1, 0, x, 6, 0), logs)          # beam
            fill(s, nu.Shape.cuboid(x + 2, 2, 0, x + 3, 3, 0),
                 nu.Brush.solid("minecraft:air"))                     # window
            fill(s, nu.Shape.cuboid(x + 1, 6, 0, x + 5, 6, 0), logs)  # cap
        return s

    two = wall(2)
    det = json.loads(nu.Autostack.detect_structures(two))
    vx, vy, vz = det[0]["vectors"][0]   # period vector, here [6, 0, 0]
    print(f"  detected: {det[0]['label']} vectors={det[0]['vectors']}")
    six = nu.Autostack.resize_1d(two, vx, vy, vz, 6)

    tmp = tempfile.mkdtemp(prefix="nuc-autostack-")
    try:
        panels = []
        for s, name, w in [(two, "a", 400), (six, "b", 1000)]:
            p = os.path.join(tmp, f"{name}.png")
            cfg = nu.RenderConfig.create(w, 340)
            cfg.set_isometric(); cfg.set_yaw(200.0); cfg.set_pitch(20.0)
            cfg.set_zoom(1.0)
            cfg.set_background(0, 0, 0, 0)
            nu.Renderer.render_to_file(s, pack, cfg, p)
            panels.append(p)
        hstack(panels, os.path.join(OUT, "autostack.png"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return six


ATLAS_BLOCKS = [
    "minecraft:grass_block", "minecraft:oak_planks", "minecraft:stone_bricks",
    "minecraft:bricks", "minecraft:bookshelf", "minecraft:gold_block",
    "minecraft:diamond_ore", "minecraft:glowstone", "minecraft:pumpkin",
    "minecraft:mossy_cobblestone", "minecraft:red_wool", "minecraft:sandstone",
    "minecraft:copper_block", "minecraft:amethyst_block", "minecraft:cherry_planks",
    "minecraft:prismarine", "minecraft:crying_obsidian", "minecraft:melon",
    "minecraft:tnt", "minecraft:crafting_table", "minecraft:note_block",
    "minecraft:target", "minecraft:sea_lantern", "minecraft:redstone_lamp",
    "minecraft:blue_ice", "minecraft:purpur_pillar", "minecraft:quartz_bricks",
    "minecraft:dark_prismarine", "minecraft:warped_planks", "minecraft:end_stone",
    "minecraft:lapis_block", "minecraft:chiseled_sandstone", "minecraft:magma_block",
]


def scene_texture_atlas(pack):
    """The packed RGBA atlas TextureAtlas.build_global returns for a build."""
    s = nu.Schematic.create("atlas-demo")
    for i, block in enumerate(ATLAS_BLOCKS):
        s.set_block(i % 6, 0, i // 6, block)
    rp = nu.ResourcePack.from_bytes(pack)
    atlas = nu.TextureAtlas.build_global(s, rp, nu.MeshConfig.create())
    w, h = atlas.width(), atlas.height()
    rgba = base64.b64decode(atlas.rgba_data_b64())
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo",
                    "-pixel_format", "rgba", "-video_size", f"{w}x{h}", "-i", "-",
                    "-vf", "scale=iw*4:ih*4:flags=neighbor",
                    os.path.join(OUT, "texture-atlas.png")],
                   input=rgba, check=True)
    print(f"  wrote docs/media/texture-atlas.png ({w}x{h} atlas)")
    return s


def scene_basics(pack):
    """Load a schematic, edit it with set_block, render it — the first taste."""
    data = open(os.path.join(ROOT, "test_schematic.schem"), "rb").read()
    s = nu.Schematic.from_data(data)   # 5x5x5 stone/dirt checkerboard
    for i, block in enumerate(["minecraft:gold_block", "minecraft:emerald_block",
                               "minecraft:redstone_block"]):
        s.set_block(5 + i, 0, 0, block)
    s.set_block(2, 5, 2, "minecraft:glowstone")
    render(s, pack, os.path.join(OUT, "basics.png"), w=560, h=460,
           yaw=225, pitch=28, zoom=1.1)
    return s


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
    "basics": scene_basics,
    "hero": scene_hero,
    "torus": scene_gradient_torus,
    "shaded": scene_shaded_sphere,
    "mandelbrot": scene_mandelbrot,
    "shapes": scene_shapes,
    "masked-fill": scene_masked_fill,
    "redstone": scene_redstone,
    "diff": scene_diff,
    "autostack": scene_autostack,
    "atlas": scene_texture_atlas,
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
