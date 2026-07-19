#!/usr/bin/env python3
"""Generate the README's images and GIFs using nucleation itself.

Every picture in the README is produced by this script through the Python
binding — the same API the README documents. Regenerate after visual changes:

    pip install nucleation  # or a locally built wheel (bridge-full)
    python3 tools/readme-media/generate.py --pack /path/to/resource-pack.zip

The resource pack is any vanilla-format pack zip (assets/minecraft/...);
it is NOT committed to the repo. Output lands in docs/media/.
GIF assembly and image compositing need ffmpeg on PATH; there are no other
Python dependencies (no Pillow/numpy). The mariokart scene additionally needs
Node on PATH (npx obj2gltf converts the ripped OBJ course to GLB).

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
    """EXACTLY the README basics snippet: load, one set_block, render."""
    s = nu.Schematic.load_from_file(os.path.join(ROOT, "simple_cube.litematic"))
    s.set_block(1, 3, 1, "minecraft:glowstone")
    render(s, pack, os.path.join(OUT, "basics.png"), w=560, h=480,
           pitch=28, zoom=1.15)
    return s


def scene_palette_ramps(pack):
    """A literal picture of Palette ramps: one row per palette, light-sorted."""
    s = nu.Schematic.create("ramps")
    rows = [json.loads(p.sorted_by_lightness().block_ids_json())
            for p in (nu.Palette.wool(), nu.Palette.concrete(),
                      nu.Palette.terracotta(), nu.Palette.wood())]
    # Bottom row: ramp_ids picks the 24 best DISTINCT blocks for a pure
    # white -> pure black ramp out of the grayscale palette.
    rows.append(json.loads(
        nu.Palette.grayscale().ramp_ids_json(255, 255, 255, 0, 0, 0, 24)))
    widest = max(len(r) for r in rows)
    for row, ids in enumerate(rows):
        x0 = (widest - len(ids)) // 2
        y0 = 3 * (len(rows) - 1 - row)
        for col, block_id in enumerate(ids):
            s.set_block(x0 + col, y0, 0, block_id)
            s.set_block(x0 + col, y0 + 1, 0, block_id)
    render(s, pack, os.path.join(OUT, "palette-ramps.png"), w=1500, h=1000,
           yaw=0.0, pitch=0.0, ortho=True)
    return s




# The smoothest survival-obtainable white->black ladder, ordered by measured
# Oklab lightness (see Blocks.get_json colors); tinted/noisy outliers
# (obsidian, sculk, gravel, quartz warmth) deliberately excluded.
GRAY_RAMP = [
    "minecraft:snow_block", "minecraft:white_wool", "minecraft:calcite",
    "minecraft:white_concrete", "minecraft:polished_diorite", "minecraft:smooth_stone",
    "minecraft:light_gray_wool", "minecraft:stone", "minecraft:tuff",
    "minecraft:cyan_terracotta", "minecraft:deepslate", "minecraft:polished_deepslate",
    "minecraft:gray_wool", "minecraft:gray_concrete", "minecraft:polished_blackstone",
    "minecraft:blackstone", "minecraft:black_wool", "minecraft:coal_block",
    "minecraft:black_concrete",
]


def _metaballs_sdf(t):
    """Three spheres orbiting a common center, smooth-unioned into one mass."""
    balls = []
    for i in range(3):
        ph = t * 2 * math.pi + i * 2 * math.pi / 3
        x = 12 * math.cos(ph) * (0.75 + 0.25 * math.cos(2 * ph))
        z = 12 * math.sin(ph) * (0.75 + 0.25 * math.cos(2 * ph))
        y = 7 + 3 * math.sin(2 * ph + i)
        balls.append({"type": "translate", "offset": [round(x, 2), round(y, 2), round(z, 2)],
                      "child": {"type": "sphere", "radius": 9 - i}})
    a, b, c = balls
    return {"type": "smoothUnion", "k": 10.0,
            "a": {"type": "smoothUnion", "k": 10.0, "a": a, "b": b}, "b": c}


_METABALL_RULES = {"fill": [{"gradient": {
    "palette": {"ids": GRAY_RAMP},
    "from": [8, 10, 14], "to": [250, 252, 252],   # black floor of the mass -> white crowns
    "axis": "y", "range": [4, 17]}}]}


def scene_metaballs(pack):
    """Looping metaball animation: smooth-union spheres wearing the smoothest
    survival-block white->black gradient, floating over a dark plate."""
    tmp = tempfile.mkdtemp(prefix="nuc-metaballs-")
    frames = 48
    try:
        for i in range(frames):
            s = nu.Sdf.schematic_from_sdf(
                json.dumps(_metaballs_sdf(i / frames)), json.dumps(_METABALL_RULES),
                True, -25, -6, -25, 25, 21, 25)
            # Static plate: grounds the composition and pins the sphere-fit
            # framing so the orbit cannot pulse.
            s.fill_cuboid(-25, -9, -25, 25, -9, 25, "minecraft:black_concrete")
            cfg = nu.RenderConfig.create(620, 500)
            cfg.set_isometric()
            cfg.set_pitch(14.0)   # near eye-level: the full gradient band stays visible
            cfg.set_sphere_fit(True)
            cfg.set_zoom(1.35)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(s, pack, cfg, os.path.join(tmp, f"f{i:03}.png"))
        assemble_gif(tmp, os.path.join(OUT, "metaballs.gif"), fps=frames / 4.8,
                     max_colors=128)
        print(f"  wrote docs/media/metaballs.gif ({frames} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)




MODELS_DIR = os.path.join(ROOT, "target", "readme-models")
TEAPOT_URL = "https://casual-effects.com/g3d/data10/common/model/teapot/teapot.obj"
DUCK_URL = ("https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/"
            "main/Models/Duck/glTF-Binary/Duck.glb")


def _fetch_model(name, url):
    os.makedirs(MODELS_DIR, exist_ok=True)
    path = os.path.join(MODELS_DIR, name)
    if not os.path.exists(path):
        subprocess.run(["curl", "-sL", "-o", path, url], check=True)
    return path


def scene_teapot(pack):
    """Utah teapot under a single spotlight through the grayscale ladder —
    voxelized from the classic OBJ, lit by the spotlight brush."""
    obj = open(_fetch_model("teapot.obj", TEAPOT_URL)).read()
    # shell=0.75: the canonical teapot is a genuinely hollow, double-walled
    # vessel; the shell closes its quarter-voxel ceramic walls.
    teapot = None
    s = nu.Schematic.create("teapot")
    shape = nu.Voxelizer.shape_from_obj(obj, 56.0, 0.75)
    light_pos = (25.0, 58.0, -55.0)           # high right of the camera
    center = (0.0, 12.0, 0.0)
    d = [c - p for p, c in zip(light_pos, center)]
    n = math.sqrt(sum(v * v for v in d))
    brush = nu.Brush.spotlight(*light_pos, *(v / n for v in d), 50.0, 245, 242, 235)
    # dithered ramp: the falloff blends between neighboring grays per voxel
    brush.set_palette(nu.Palette.from_block_ids(json.dumps(GRAY_RAMP)).dithered())
    nu.BuildingTool.fill(s, shape, brush)
    cfg = nu.RenderConfig.create(880, 620)
    cfg.set_isometric(); cfg.set_yaw(185.0); cfg.set_pitch(10.0); cfg.set_zoom(1.1)
    cfg.set_background(0.086, 0.098, 0.149, 1.0)
    nu.Renderer.render_to_file(s, pack, cfg, os.path.join(OUT, "teapot-spotlight.png"))
    print("  wrote docs/media/teapot-spotlight.png")
    return s


MK64_ZIP_URL = ("https://models.spriters-resource.com/media/assets/309/"
                "311926.zip?updated=1755503213")   # Rainbow Road (Mario Kart 64)
MK64_UA = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
           "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")

# 32 saturated wools/concretes + two "glow" golds: matches the track's neon
# rainbow far better than Palette.all(), which wanders into glass and ores.
MK64_PALETTE = [f"minecraft:{c}_{kind}" for kind in ("concrete", "wool")
                for c in ("white", "orange", "magenta", "light_blue", "yellow",
                          "lime", "pink", "gray", "light_gray", "cyan", "purple",
                          "blue", "brown", "green", "red", "black")]
MK64_PALETTE += ["minecraft:gold_block", "minecraft:glowstone"]


def _mariokart_glb():
    """Download + prepare the MK64 Rainbow Road track as an opaque GLB.

    Needs Node on PATH: the OBJ+MTL+PNG rip is converted with `npx obj2gltf`.
    Preparation is deterministic:
      * keep only the two materials that span the whole course (road ribbon +
        rainbow rails); everything else is trackside billboards/banner,
      * composite the road's star texture (pure yellow + alpha) over the dark
        indigo it blends against in-game, and strip the MTL transparency —
        otherwise the voxelizer's palette snap lands in stained glass,
      * rebuild as a self-contained binary GLB.
    """
    glb = os.path.join(MODELS_DIR, "mk64-rainbow-road.glb")
    if os.path.exists(glb):
        return glb
    zip_path = os.path.join(MODELS_DIR, "mk64-rainbow-road.zip")
    if not os.path.exists(zip_path):
        os.makedirs(MODELS_DIR, exist_ok=True)
        subprocess.run(["curl", "-sL", "-A", MK64_UA, "-o", zip_path,
                        MK64_ZIP_URL], check=True)
    src = os.path.join(MODELS_DIR, "mk64-rainbow-road")
    subprocess.run(["unzip", "-oq", zip_path, "-d", src], check=True)

    keep = {"Material__992", "Material__975"}    # road surface, rainbow rails
    cur, lines = None, []
    for line in open(os.path.join(src, "rainbow.obj")):
        if line.startswith("usemtl"):
            cur = line.split()[1]
        if line.startswith(("f ", "g ", "usemtl", "s ")) and cur is not None \
                and cur not in keep:
            continue
        lines.append(line.replace("rainbow.mtl", "track.mtl"))
    open(os.path.join(src, "track.obj"), "w").writelines(lines)
    mtl = [line for line in open(os.path.join(src, "rainbow.mtl"))
           if not line.split() or line.split()[0] not in ("d", "Tr", "Tf")]
    open(os.path.join(src, "track.mtl"), "w").writelines(
        line.replace("31A2D889_c.png", "road_dark.png") for line in mtl)
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-f", "lavfi", "-i", "color=0x1e1a46:s=32x32,format=rgba",
                    "-i", os.path.join(src, "31A2D889_c.png"),
                    "-filter_complex", "[0][1]overlay=format=auto:shortest=1,format=rgb24",
                    os.path.join(src, "road_dark.png")], check=True)
    subprocess.run(["npx", "-y", "obj2gltf", "-i", os.path.join(src, "track.obj"),
                    "-o", glb, "--binary"], check=True)
    return glb


def scene_mariokart(pack):
    """MK64 Rainbow Road, voxelized from the ripped course model.

    target_size=515 calibrates the road ribbon to 8-9 blocks wide (measured
    as the mode of cross-road block runs on straight segments). shell=1.0 is
    essential: the track is an open ribbon surface, so parity-based interior
    tests fail — the shell claims every voxel within one voxel of the
    surface instead.
    """
    glb = open(_mariokart_glb(), "rb").read()
    pal = nu.Palette.from_block_ids(json.dumps(MK64_PALETTE))
    s = nu.Voxelizer.schematic_from_glb_textured(glb, 515.0, 1.0, pal,
                                                 "rainbow_road")

    # The two kept materials still carry a few floating trackside scraps;
    # drop every connected component except the course itself. Blocks are
    # streamed in z-slabs: one get_all_blocks_json on the full ~500^3-bounded
    # volume enumerates every air voxel too and swallows gigabytes.
    mn, mx = s.tight_bounds_min(), s.tight_bounds_max()
    occ = set()
    for zs in range(mn.z, mx.z + 1, 16):
        chunk = json.loads(s.get_chunk_blocks_json(
            mn.x, mn.y, zs, mx.x - mn.x + 1, mx.y - mn.y + 1,
            min(16, mx.z + 1 - zs)))
        occ.update((b["x"], b["y"], b["z"]) for b in chunk
                   if b["name"] != "minecraft:air")
    seen, comps = set(), []
    for p in occ:
        if p in seen:
            continue
        stack, comp = [p], [p]
        seen.add(p)
        while stack:
            x, y, z = stack.pop()
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    for dz in (-1, 0, 1):
                        q = (x + dx, y + dy, z + dz)
                        if q in occ and q not in seen:
                            seen.add(q)
                            stack.append(q)
                            comp.append(q)
        comps.append(comp)
    comps.sort(key=len, reverse=True)
    for comp in comps[1:]:
        for (x, y, z) in comp:
            s.set_block(x, y, z, "minecraft:air")

    # Hero: three-quarter aerial framing the whole course.
    render(s, pack, os.path.join(OUT, "mariokart-track.png"), w=1200, h=800,
           yaw=250, pitch=45, zoom=1.05, background=NAVY)
    # Closeup: low-angle over the crossing straights; the ~9-block road and
    # its projected star/rail textures fill the frame (sphere_fit off, so
    # zoom>1 dives into the scene instead of re-fitting it).
    cfg = nu.RenderConfig.create(1100, 700)
    cfg.set_isometric(); cfg.set_yaw(205.0); cfg.set_pitch(12.0)
    cfg.set_zoom(4.0); cfg.set_sphere_fit(False)
    cfg.set_background(*NAVY)
    nu.Renderer.render_to_file(s, pack, cfg,
                               os.path.join(OUT, "mariokart-closeup.png"))
    print("  wrote docs/media/mariokart-closeup.png")
    turntable_gif(s, pack, os.path.join(OUT, "mariokart-turntable.gif"),
                  frames=36, pitch=35, zoom=1.75)
    return s


def scene_duck(pack):
    """The classic COLLADA duck, texture-projected onto blocks."""
    glb = open(_fetch_model("Duck.glb", DUCK_URL), "rb").read()
    duck = nu.Voxelizer.schematic_from_glb_textured(glb, 44.0, 0.7,
                                                    nu.Palette.solid(), "duck")
    render(duck, pack, os.path.join(OUT, "textured-duck.png"), w=720, h=600,
           yaw=130, pitch=18, zoom=1.15)
    return duck




def scene_dither(pack):
    """Hard-snap vs dithered palette on the same shaded sphere, hstacked."""
    def sphere(dither):
        s = nu.Schematic.create("d")
        # sparse 6-step ramp: hard snapping bands hard, dithering dissolves it
        pal = nu.Palette.from_block_ids(json.dumps(GRAY_RAMP[::3]))
        brush = nu.Brush.shaded(235, 232, 228, -1.0, 0.7, -0.3)
        brush.set_palette(pal.dithered() if dither else pal)
        nu.BuildingTool.fill(s, nu.Shape.sphere(0, 0, 0, 15), brush)
        return s
    tmp = tempfile.mkdtemp(prefix="nuc-dither-")
    try:
        for tag, d in (("a", False), ("b", True)):
            cfg = nu.RenderConfig.create(560, 520)
            cfg.set_isometric(); cfg.set_yaw(45.0); cfg.set_pitch(22.0); cfg.set_zoom(1.15)
            cfg.set_background(0, 0, 0, 0)
            nu.Renderer.render_to_file(sphere(d == True), pack, cfg,
                                       os.path.join(tmp, f"{tag}.png"))
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                        "-i", os.path.join(tmp, "a.png"), "-i", os.path.join(tmp, "b.png"),
                        "-filter_complex", "hstack", os.path.join(OUT, "dither-compare.png")],
                       check=True)
        print("  wrote docs/media/dither-compare.png")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def scene_scripting(pack):
    """The Lua sine-wave wall from the scripting guide, rendered."""
    lua = """
local wall = Schematic.new("sine_wall")
local ramp = palette_gradient_ids("concrete", 200, 60, 40, 255, 220, 80, 8)
local width, base, amp = 48, 6, 5
for x = 0, width - 1 do
  local h = base + math.floor(amp * math.sin(x * 2 * math.pi / 24) + 0.5)
  for y = 0, h - 1 do
    wall:set_block(x, y, 0, ramp[math.floor(y * (#ramp - 1) / (base + amp - 1)) + 1])
  end
end
result = wall
"""
    tmp = tempfile.mkdtemp(prefix="nuc-lua-")
    try:
        path = os.path.join(tmp, "wall.lua")
        open(path, "w").write(lua)
        wall = nu.Scripting.run_lua_script(path)
        render(wall, pack, os.path.join(OUT, "scripting-wall.png"), w=1100, h=320,
               yaw=0.0, pitch=0.0, ortho=True)
        return wall
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def scene_simulation(pack):
    """A byte on a lamp bus: eight lever->wire->lamp lines, some levers
    flipped through the simulator — the lamps display the binary pattern."""
    s = nu.Schematic.create("byte")
    for i in range(8):
        z = i * 2
        for x in range(3):
            s.set_block(x, 0, z, "minecraft:gray_concrete")
        s.set_block_from_string(0, 1, z, "minecraft:lever[facing=east,face=floor,powered=false]")
        s.set_block_from_string(1, 1, z, "minecraft:redstone_wire[power=0,east=side,west=side]")
        s.set_block_from_string(2, 1, z, "minecraft:redstone_lamp[lit=false]")
    world = nu.MchprsWorld.create(s)
    for i, bit in enumerate("10110010"):
        if bit == "1":
            world.on_use_block(0, 1, i * 2)
    world.tick(2)
    world.flush()
    world.sync_to_schematic()
    lit = world.get_schematic()
    render(lit, pack, os.path.join(OUT, "simulation-byte.png"), w=900, h=460,
           yaw=150, pitch=35, zoom=1.1)
    return lit




# Public-domain paintings for the pixel-art gallery (Wikimedia Commons).
PAINTINGS = {
    "starry-night": ("Van Gogh - Starry Night - Google Art Project.jpg", 128),
    "sunflowers": ("Vincent Willem van Gogh 127.jpg", 96),
    "great-wave": ("Great Wave off Kanagawa2.jpg", 128),
    "pearl-earring": ("1665 Girl with a Pearl Earring.jpg", 96),
}

# Blocks whose textures are too patterned for pixel art (faces, dots,
# machinery) — the map-art exclusion list, applied on top of the
# full-cube/opaque/no-tile-entity builder filters.
NOISY_BLOCKS = [
    "prismarine", "carved", "jack_o", "command", "structure", "loom",
    "cartography", "crafting", "smithing", "fletching", "barrel", "jukebox",
    "note_block", "tnt", "target", "piston", "observer", "dispenser", "dropper",
    "furnace", "smoker", "sculk", "chiseled", "pumpkin", "melon", "mycelium",
    "podzol", "glazed", "shroomlight", "froglight", "_ore", "raw_", "bookshelf",
    "hay_block", "dried", "magma", "sponge", "cake", "spawner", "respawn",
    "ancient", "reinforced", "suspicious", "infested", "mushroom", "coral",
    "sulfur", "cinnabar", "copper_grate", "bulb", "daylight", "composter",
    "beehive", "bee_nest", "lodestone", "bedrock", "grass_block", "creaking",
    "pale_moss", "resin_clump", "amethyst", "budding", "wart", "nylium",
    "oxidized", "weathered", "exposed", "pillar", "lantern", "blue_ice",
    "packed_ice", "frosted", "crying", "gilded", "redstone_block", "slime",
    "honey", "scaffolding", "kelp", "bamboo_block", "muddy", "root",
]


def flat_art_palette():
    """A flat-texture palette for pixel art, built from filters + excludes."""
    b = nu.PaletteBuilder.create()
    b.full_blocks_only()
    b.exclude_tile_entities()
    b.exclude_transparent()
    for kw in NOISY_BLOCKS:
        b.exclude_keyword(kw)
    return b.build()


def _boost(r, g, b, sat=1.35, contrast=1.06):
    """Chroma + contrast exaggeration before palette matching, so muted
    paint pigments land on saturated blocks instead of gray clays."""
    lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
    out = []
    for c in (r, g, b):
        v = lum + (c - lum) * sat
        v = 128.0 + (v - 128.0) * contrast
        out.append(max(0, min(255, round(v))))
    return out


def _image_pixels(path, width):
    probe = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0",
                            "-show_entries", "stream=width,height", "-of", "csv=p=0", path],
                           capture_output=True, text=True).stdout.strip().split(",")
    height = round(width * int(probe[1]) / int(probe[0]))
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path,
                          "-vf", f"scale={width}:{height}", "-f", "rawvideo",
                          "-pix_fmt", "rgb24", "-"], capture_output=True).stdout
    return width, height, raw


def _commons_url(title):
    import urllib.parse, urllib.request
    q = urllib.parse.urlencode({
        "action": "query", "titles": f"File:{title}", "prop": "imageinfo",
        "iiprop": "url", "iiurlwidth": 1280, "format": "json"})
    req = urllib.request.Request(
        f"https://commons.wikimedia.org/w/api.php?{q}",
        headers={"User-Agent": "nucleation-readme-media/1.0"})
    d = json.load(urllib.request.urlopen(req))
    return list(d["query"]["pages"].values())[0]["imageinfo"][0]["thumburl"]


def scene_paintings(pack):
    """Public-domain paintings as block pixel art: flat-texture palette from
    color-logic filters, chroma-boosted matching, ordered dithering."""
    import urllib.request
    pal = flat_art_palette().dithered()
    tiles = {}
    for name, (title, width) in PAINTINGS.items():
        path = os.path.join(MODELS_DIR, f"{name}.jpg")
        if not os.path.exists(path):
            os.makedirs(MODELS_DIR, exist_ok=True)
            req = urllib.request.Request(
                _commons_url(title), headers={"User-Agent": "nucleation-readme-media/1.0"})
            open(path, "wb").write(urllib.request.urlopen(req).read())
        w, h, raw = _image_pixels(path, width)
        s = nu.Schematic.create(name)
        for py in range(h):
            for px in range(w):
                i = (py * w + px) * 3
                r, g, b = _boost(raw[i], raw[i + 1], raw[i + 2])
                s.set_block(px, 0, py, pal.closest_block_dithered(r, g, b, px, 0, py))
        out_w = 1100 if width == 128 else 760
        cfg = nu.RenderConfig.create(out_w, round(out_w * h / w))
        cfg.set_isometric(); cfg.set_yaw(0.0); cfg.set_pitch(89.9)
        cfg.set_orthographic(True); cfg.set_background(0, 0, 0, 0)
        tile = os.path.join(MODELS_DIR, f"px-{name}.png")
        nu.Renderer.render_to_file(s, pack, cfg, tile)
        tiles[name] = tile

    shutil.copy(tiles["starry-night"], os.path.join(OUT, "painting-starry-night.png"))
    # gallery strip: sunflowers | great wave | pearl earring at equal height
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-i", tiles["sunflowers"], "-i", tiles["great-wave"],
                    "-i", tiles["pearl-earring"],
                    "-filter_complex",
                    "[0]scale=-2:760[a];[1]scale=-2:760[b];[2]scale=-2:760[c];[a][b][c]hstack=3",
                    os.path.join(OUT, "painting-gallery.png")], check=True)
    print("  wrote docs/media/painting-starry-night.png + painting-gallery.png")


SCENES = {
    "paintings": scene_paintings,
    "dither": scene_dither,
    "scripting": scene_scripting,
    "simulation": scene_simulation,
    "teapot": scene_teapot,
    "duck": scene_duck,
    "mariokart": scene_mariokart,
    "metaballs": scene_metaballs,
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
