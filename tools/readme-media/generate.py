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
           ortho=False, background=(0, 0, 0, 0), sphere_fit=False):
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
    if sphere_fit:
        cfg.set_sphere_fit(True)
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
    # shell=1.0: the canonical teapot is a genuinely hollow, double-walled
    # vessel; a full-voxel shell closes its ceramic walls without pinholes.
    teapot = None
    s = nu.Schematic.create("teapot")
    shape = nu.Voxelizer.shape_from_obj(obj, 56.0, 1.0)
    light_pos = (-40.0, 88.0, -46.0)          # high three-quarter key, camera right
    center = (5.0, 10.0, 0.0)                 # aimed just spout-of-center
    d = [c - p for p, c in zip(light_pos, center)]
    n = math.sqrt(sum(v * v for v in d))
    brush = nu.Brush.spotlight(*light_pos, *(v / n for v in d), 58.0, 248, 242, 232)
    # dithered ramp: the falloff blends between neighboring grays per voxel
    brush.set_palette(nu.Palette.from_block_ids(json.dumps(GRAY_RAMP)).dithered())
    nu.BuildingTool.fill(s, shape, brush)
    cfg = nu.RenderConfig.create(880, 620)
    cfg.set_isometric(); cfg.set_yaw(185.0); cfg.set_pitch(14.0); cfg.set_zoom(1.1)
    cfg.set_background(0.105, 0.115, 0.168, 1.0)
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

    # Keep ONLY the rainbow ribbon (__975). Material __992 is the track's
    # near-vertical side-skirt geometry (100% of its faces point sideways):
    # each material voxelizes cleanly on its own, but together the skirt walls
    # give the scanline parity solver extra ray crossings, so it marks the
    # gaps between the skirt and the ribbon as "interior" and fills them into
    # blue "sails". Dropping the skirt leaves a clean open rainbow ribbon that
    # the shell handles perfectly — which is what Rainbow Road actually is.
    keep = {"Material__975"}
    cur, lines = None, []
    for line in open(os.path.join(src, "rainbow.obj")):
        if line.startswith("usemtl"):
            cur = line.split()[1]
        if line.startswith(("f ", "g ", "usemtl", "s ")) and cur is not None \
                and cur not in keep:
            continue
        lines.append(line.replace("rainbow.mtl", "track.mtl"))
    open(os.path.join(src, "track.obj"), "w").writelines(lines)
    # Strip MTL transparency so the palette snap can't land in stained glass.
    mtl = [line for line in open(os.path.join(src, "rainbow.mtl"))
           if not line.split() or line.split()[0] not in ("d", "Tr", "Tf")]
    open(os.path.join(src, "track.mtl"), "w").writelines(mtl)
    subprocess.run(["npx", "-y", "obj2gltf", "-i", os.path.join(src, "track.obj"),
                    "-o", glb, "--binary"], check=True)
    return glb


def scene_mariokart(pack):
    """MK64 Rainbow Road, voxelized from the ripped course model.

    target_size=515 calibrates the road ribbon to 8-9 blocks wide (measured
    as the mode of cross-road block runs on straight segments). shell=-1.0
    asks for surface-only voxelization: the track is an open ribbon that dips
    and crosses over itself, so a parity interior test fills those arcs and
    overlaps as enclosed volume. A negative shell skips parity and keeps just
    a one-block skin of the ribbon.
    """
    glb = open(_mariokart_glb(), "rb").read()
    pal = nu.Palette.from_block_ids(json.dumps(MK64_PALETTE))
    s = nu.Voxelizer.schematic_from_glb_textured(glb, 515.0, -1.0, pal,
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
           yaw=250, pitch=52, zoom=1.05, background=NAVY)
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


KTB_ZIP_URL = ("https://models.spriters-resource.com/media/assets/297/"
               "300000.zip")                       # Koopa Troopa Beach (MK64)

# Daytime beach palette: sands/sandstones for the shore, browns for the dirt
# road and cliffs, planks for the bridges, greens for grass and palm fronds,
# blues for the sea. MK64_PALETTE's neon wools are wrong for this course.
KTB_PALETTE = [
    "minecraft:sand", "minecraft:sandstone", "minecraft:smooth_sandstone",
    "minecraft:cut_sandstone", "minecraft:end_stone",
    "minecraft:white_terracotta", "minecraft:yellow_terracotta",
    "minecraft:terracotta", "minecraft:brown_terracotta",
    "minecraft:packed_mud", "minecraft:brown_concrete", "minecraft:brown_wool",
    "minecraft:oak_planks", "minecraft:spruce_planks",
    "minecraft:dark_oak_planks",
    "minecraft:green_concrete", "minecraft:green_wool", "minecraft:moss_block",
    "minecraft:blue_concrete", "minecraft:blue_wool",
    "minecraft:light_blue_concrete", "minecraft:light_blue_wool",
    "minecraft:cyan_concrete",
    "minecraft:white_concrete", "minecraft:black_concrete",
]


def _koopa_glb():
    """Download + prepare the MK64 Koopa Troopa Beach course as an opaque GLB.

    Same recipe as _mariokart_glb: keep only course geometry (terrain, dirt
    road, cliffs/arches, wooden bridges, water, finish line, palm trunks and
    fronds), dropping the 2-face trackside banners, arrow signs and flag
    billboards; flatten the two RGBA textures — the palm frond over its
    in-game leaf green, the second water sheet is already fully opaque —
    and strip the MTL transparency so the palette snap stays out of glass.
    """
    glb = os.path.join(MODELS_DIR, "mk64-koopa-beach.glb")
    if os.path.exists(glb):
        return glb
    zip_path = os.path.join(MODELS_DIR, "mk64-koopa-beach.zip")
    if not os.path.exists(zip_path):
        os.makedirs(MODELS_DIR, exist_ok=True)
        subprocess.run(["curl", "-sL", "-A", MK64_UA, "-o", zip_path,
                        KTB_ZIP_URL], check=True)
    src = os.path.join(MODELS_DIR, "mk64-koopa-beach")
    subprocess.run(["unzip", "-oq", zip_path, "-d", src], check=True)

    keep = {"_Rip1VMtl019",   # beach sand
            "_Rip1VMtl033",   # dirt road
            "_Rip1VMtl027",   # deep ocean floor (retextured as deep water)
            "_Rip1VMtl025",   # tide flats (retextured as shallow water)
            "_Rip1VMtl040",   # gravel cliff walls / rock arch
            "_Rip1VMtl041",   # grass clifftops
            "_Rip1VMtl020",   # start/finish line
            "_Rip1VMtl003",   # palm trunks
            "_Rip3VMtl058",   # wooden bridge planks
            "_Rip3VMtl059",   # palm fronds
            "water1", "water2"}
    cur, verts, lines = None, [], []
    for line in open(os.path.join(src, "KoopaTroopaBeach.obj")):
        if line.startswith("v "):
            verts.append(tuple(float(v) for v in line.split()[1:4]))
        if line.startswith("usemtl"):
            cur = line.split()[1]
        if line.startswith(("f ", "g ", "usemtl", "s ")) and cur is not None \
                and cur not in keep:
            continue
        # Frame-ripped courses carry big near-vertical "curtain" faces
        # (capture-frustum skirts radiating across the map). On the road they
        # streak the track; across the sea they voxelize into thin lines. Drop
        # any *large* near-vertical face (|normal.y| small AND a long edge);
        # genuine vertical detail — cliff walls, palm trunks — is finely
        # tessellated with short edges and survives. Applied to every material,
        # not just the road, which is what clears the lines over the water.
        if line.startswith("f "):
            p = [verts[int(t.split("/")[0]) - 1] for t in line.split()[1:4]]
            u = [p[1][i] - p[0][i] for i in range(3)]
            v = [p[2][i] - p[0][i] for i in range(3)]
            n = (u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2],
                 u[0] * v[1] - u[1] * v[0])
            length = math.sqrt(sum(c * c for c in n)) or 1.0
            edge = max(math.dist(p[a], p[b]) for a, b in ((0, 1), (1, 2), (2, 0)))
            if abs(n[1]) / length < 0.35 and edge > 16.0:
                continue
        lines.append(line.replace("KoopaTroopaBeach.mtl", "track.mtl"))
    # Close the model from below: every rip surface is single-sided, so the
    # voxelizer's parity sweeps see one crossing along +y and vote "inside"
    # for the whole sky, teaming up with stray horizontal-parity wedges into
    # radiating phantom walls above the sea. A floor quad one unit under the
    # deep-ocean sheet makes the y-parity even everywhere above the map (and
    # harmlessly fills the hidden interior below the surfaces).
    nv = len(verts)
    lines.append("usemtl _Rip1VMtl027\n")
    for fx, fz in ((-278.0, -241.0), (157.0, -241.0),
                   (157.0, 418.0), (-278.0, 418.0)):
        lines.append(f"v {fx} -7.0 {fz}\n")
    lines.append(f"f {nv+1}/1/1 {nv+2}/1/1 {nv+3}/1/1\n")
    lines.append(f"f {nv+1}/1/1 {nv+3}/1/1 {nv+4}/1/1\n")
    open(os.path.join(src, "track.obj"), "w").writelines(lines)
    mtl = [line for line in open(os.path.join(src, "KoopaTroopaBeach.mtl"))
           if not line.split() or line.split()[0] not in ("d", "Tr", "Tf")]
    open(os.path.join(src, "track.mtl"), "w").writelines(
        line.replace("E27C62B_c.png", "frond_flat.png")
            .replace("46E1B53_c.png", "water_flat.png")
            .replace("236F857A_c.png", "ocean_deep.png")
            .replace("43EFF121_c.png", "ocean_shallow.png")
            .replace("20F371EB_c.png", "road_warm.png")
            .replace("7CF42C55_c.png", "cliff_warm.png") for line in mtl)
    # Palm fronds: composite the alpha leaf over its in-game green.
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-f", "lavfi", "-i", "color=0x2e8b3a:s=64x32,format=rgba",
                    "-i", os.path.join(src, "E27C62B_c.png"),
                    "-filter_complex", "[0][1]overlay=format=auto:shortest=1,format=rgb24",
                    os.path.join(src, "frond_flat.png")], check=True)
    # Shallow-crossing water sheet: fully opaque already, just drop the
    # alpha channel so obj2gltf can't mark it blended.
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-i", os.path.join(src, "46E1B53_c.png"),
                    "-vf", "format=rgb24",
                    os.path.join(src, "water_flat.png")], check=True)
    # The rip exports the sea as bare seabed (the N64 water plane is an
    # animated transparent layer the ripper never captured): 236F857A is the
    # deep floor, 43EFF121 the tide flats — both entirely below sea level.
    # Retexture them as flat lagoon blues (any real water texture tiles into
    # sparkle noise at these UV scales) so the sea voxelizes as clean water.
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-f", "lavfi", "-i", "color=0x2d2f8f:s=32x32", "-frames:v", "1",
                    "-vf", "format=rgb24",
                    os.path.join(src, "ocean_deep.png")], check=True)
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-f", "lavfi", "-i", "color=0x2489c7:s=32x32", "-frames:v", "1",
                    "-vf", "format=rgb24",
                    os.path.join(src, "ocean_shallow.png")], check=True)
    # The dirt-road gravel rips very dark and slightly blue; sepia-tone it
    # (plus a small lift) so the palette snap lands in browns, never in
    # gray or blue concrete mush.
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-i", os.path.join(src, "20F371EB_c.png"),
                    "-vf", "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0"
                           ":.272:.534:.131:0,eq=brightness=0.08,format=rgb24",
                    os.path.join(src, "road_warm.png")], check=True)
    # Same treatment for the blue-gray gravel cliff walls (they snap to
    # blue concrete otherwise, with no grays in the beach palette).
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                    "-i", os.path.join(src, "7CF42C55_c.png"),
                    "-vf", "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0"
                           ":.272:.534:.131:0,eq=brightness=0.05,format=rgb24",
                    os.path.join(src, "cliff_warm.png")], check=True)
    subprocess.run(["npx", "-y", "obj2gltf", "-i", os.path.join(src, "track.obj"),
                    "-o", glb, "--binary"], check=True)
    return glb


def _largest_component(s):
    """Keep only the largest 26-connected component of a voxelized schematic
    (same cleanup as scene_mariokart: drops floating trackside scraps).
    Blocks are streamed in z-slabs to avoid enumerating the air voxels."""
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


def scene_mk64_track2(pack):
    """MK64 Koopa Troopa Beach, voxelized from the ripped course model.

    target_size=500 calibrates the dirt road to ~8-10 blocks wide (measured
    as x-runs of road blocks on straight segments; the course map is mostly
    sea, so the island itself stays compact). shell=1.0 for the same reason
    as scene_mariokart: open surfaces defeat parity interior tests.
    """
    glb = open(_koopa_glb(), "rb").read()
    pal = nu.Palette.from_block_ids(json.dumps(KTB_PALETTE))
    s = nu.Voxelizer.schematic_from_glb_textured(glb, 500.0, 1.0, pal,
                                                 "koopa_troopa_beach")
    _largest_component(s)
    # Hero: three-quarter aerial over the island. Background is the deep-water
    # block's *rendered* navy pushed back through sRGB into linear light, so it
    # matches the synthesized ocean floor exactly and the floor's rectangular
    # edge dissolves into an endless sea.
    render(s, pack, os.path.join(OUT, "mk64-koopa-beach.png"), w=1200, h=800,
           yaw=250, pitch=45, zoom=1.7, background=(0.024, 0.026, 0.255, 1.0))
    return s


def scene_duck(pack):
    """The classic COLLADA duck, texture-projected onto blocks."""
    glb = open(_fetch_model("Duck.glb", DUCK_URL), "rb").read()
    duck = nu.Voxelizer.schematic_from_glb_textured(glb, 44.0, 0.7,
                                                    nu.Palette.solid(), "duck")
    render(duck, pack, os.path.join(OUT, "textured-duck.png"), w=720, h=600,
           yaw=130, pitch=18, zoom=1.15)
    return duck




# ── Real-world geodata ───────────────────────────────────────────────────────

TERRARIUM_URL = "https://s3.amazonaws.com/elevation-tiles-prod/terrarium/{z}/{x}/{y}.png"
MATTERHORN_LATLON = (45.976, 7.658)
TERRAIN_ZOOM = 12        # 26.5 m/px at this latitude
TERRAIN_WINDOW = 600     # px cropped around the peak from the 3x3 tile block
TERRAIN_GRID = 300       # columns after 2x2 box-averaging -> 53 m per block
TERRAIN_PEAK_Y = 108     # blocks from the lowest valley floor to the summit


def _terrarium_heightfield():
    """TERRAIN_GRID^2 elevations (meters ASL) around the Matterhorn.

    Fetches the 3x3 block of AWS terrarium tiles whose center tile contains
    the peak, decodes each PNG to raw rgb24 via ffmpeg (the PNG-input twin of
    _image_pixels), merges, crops a TERRAIN_WINDOW px window centered on the
    peak and 2x2 box-averages it down. Terrarium encoding:
    elevation_m = R*256 + G + B/256 - 32768.
    """
    lat, lon = MATTERHORN_LATLON
    n = 2 ** TERRAIN_ZOOM
    fx = (lon + 180.0) / 360.0 * n
    fy = (1.0 - math.asinh(math.tan(math.radians(lat))) / math.pi) / 2.0 * n
    tx, ty = int(fx), int(fy)
    merged = [[0.0] * 768 for _ in range(768)]
    for j, yt in enumerate((ty - 1, ty, ty + 1)):
        for i, xt in enumerate((tx - 1, tx, tx + 1)):
            path = _fetch_model(f"terrarium-{TERRAIN_ZOOM}-{xt}-{yt}.png",
                                TERRARIUM_URL.format(z=TERRAIN_ZOOM, x=xt, y=yt))
            raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path,
                                  "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
                                 capture_output=True).stdout
            for py in range(256):
                row = merged[j * 256 + py]
                base = py * 256 * 3
                for px in range(256):
                    k = base + px * 3
                    row[i * 256 + px] = (raw[k] * 256 + raw[k + 1]
                                         + raw[k + 2] / 256 - 32768)
    cx = int((fx - tx + 1) * 256)
    cy = int((fy - ty + 1) * 256)
    x0 = min(max(cx - TERRAIN_WINDOW // 2, 0), 768 - TERRAIN_WINDOW)
    y0 = min(max(cy - TERRAIN_WINDOW // 2, 0), 768 - TERRAIN_WINDOW)
    step = TERRAIN_WINDOW // TERRAIN_GRID
    field = []
    for gz in range(TERRAIN_GRID):
        row = []
        for gx in range(TERRAIN_GRID):
            acc = 0.0
            for dy in range(step):
                for dx in range(step):
                    acc += merged[y0 + gz * step + dy][x0 + gx * step + dx]
            row.append(acc / (step * step))
        field.append(row)
    return field


def _band_jitter(x, z):
    """Deterministic +-80 m elevation jitter so palette bands dissolve into
    each other instead of drawing contour lines."""
    h = ((x * 73856093) ^ (z * 19349663)) & 0xFFFF
    return (h / 65535.0 - 0.5) * 160.0


def _alp_surface(elev, slope):
    """Surface block from elevation (meters ASL) + slope (blocks per column
    step): rock wherever it is too steep for snow, then altitude bands."""
    if slope > 2.4:                                    # cliff faces: bare rock
        return "minecraft:deepslate" if elev > 3800 else "minecraft:stone"
    if elev > 3000:
        return "minecraft:snow_block"                  # glaciers + summit snow
    if elev > 2650:
        return "minecraft:gravel"                      # scree fields
    if elev > 2100:
        return "minecraft:moss_block"                  # alpine meadows
    return "minecraft:grass_block"                     # valley floor


def _mountains_schematic():
    field = _terrarium_heightfield()
    lo = min(min(r) for r in field)
    hi = max(max(r) for r in field)
    scale = (hi - lo) / float(TERRAIN_PEAK_Y)          # meters per block of y
    g = TERRAIN_GRID
    hgrid = [[max(1, round((field[z][x] - lo) / scale)) for x in range(g)]
             for z in range(g)]
    # Flat row-major heights + a per-column surface band (elevation + slope),
    # extruded by the library. surface_depth=3 caps the top three blocks of
    # each column with its band block over a stone core.
    heights, surfaces = [], []
    for z in range(g):
        for x in range(g):
            h = hgrid[z][x]
            slope = max(abs(h - hgrid[z2][x2])
                        for x2, z2 in ((x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1))
                        if 0 <= x2 < g and 0 <= z2 < g)
            heights.append(h)
            surfaces.append(_alp_surface(field[z][x] + _band_jitter(x, z), slope))
    return nu.Geo.heightmap_terrain(json.dumps(heights), g, json.dumps(surfaces),
                                    "minecraft:stone", 3, "matterhorn")


def scene_mountains(pack):
    """The Matterhorn from real elevation data: AWS terrarium tiles ->
    300x300 columns, palette banded by altitude + slope."""
    s = _mountains_schematic()
    # Hero: straight up the axis at the pyramid, Zermatt's valley to the right.
    render(s, pack, os.path.join(OUT, "geo-mountains.png"), w=1100, h=660,
           yaw=180, pitch=32, zoom=2.15, background=NAVY, sphere_fit=True)
    # Aerial: the whole 16 km window from the southwest corner.
    render(s, pack, os.path.join(OUT, "geo-mountains-aerial.png"), w=1100,
           h=700, yaw=225, pitch=30, zoom=1.5, background=NAVY, sphere_fit=True)
    return s


OVERPASS_URLS = [   # public instances 504 under load; try in order
    "https://overpass-api.de/api/interpreter",
    "https://overpass.kumi.systems/api/interpreter",
    "https://maps.mail.ru/osm/tools/overpass/api/interpreter",
]
FIDI_BBOX = (40.7035, -74.0130, 40.7090, -74.0060)   # S, W, N, E: Wall St core
CITY_BLOCK_M = 2.0       # 1 block = 2 m -> a ~296x304 column grid
CITY_MAX_H = 320.0       # sanity cap, meters (28 Liberty tops out at 248 m)

# Height-banded massing palette: warm brick low-rise -> pale stone mid-rise ->
# white/quartz towers. Two choices per band, picked by OSM id hash, so
# adjacent same-band buildings do not fuse into one slab.
CITY_BANDS = [           # (min height in meters, block choices)
    (150.0, ["minecraft:quartz_block", "minecraft:quartz_block"]),
    (80.0, ["minecraft:white_concrete", "minecraft:smooth_stone"]),
    (40.0, ["minecraft:sandstone", "minecraft:smooth_stone"]),
    (15.0, ["minecraft:terracotta", "minecraft:bricks"]),
    (0.0, ["minecraft:red_terracotta", "minecraft:bricks"]),
]


def _fetch_fidi():
    path = os.path.join(MODELS_DIR, "fidi-buildings.json")
    if not os.path.exists(path):
        os.makedirs(MODELS_DIR, exist_ok=True)
        b = "(%s,%s,%s,%s)" % FIDI_BBOX
        q = ("[out:json][timeout:60]; ("
             f'way["building"]{b}; relation["building"]{b}; '
             f'way["building:part"]{b}; relation["building:part"]{b};'
             "); out geom;")
        for url in OVERPASS_URLS:
            r = subprocess.run(["curl", "-sf", "-m", "90",
                                "-A", "nucleation-readme-media/1.0",
                                "-o", path, "--data-urlencode", "data=" + q,
                                url])
            if r.returncode == 0:
                break
        else:
            raise RuntimeError("all Overpass instances failed")
    return json.load(open(path))


def _osm_height_m(tags, default):
    try:
        return min(float(str(tags["height"]).replace("m", "").strip()),
                   CITY_MAX_H)
    except (KeyError, ValueError):
        pass
    try:
        return min(float(tags["building:levels"]) * 3.2, CITY_MAX_H)
    except (KeyError, ValueError):
        return default


def _rings(el):
    if el["type"] == "way":
        return [el["geometry"]] if el.get("geometry") else []
    return [m["geometry"] for m in el.get("members", [])
            if m.get("geometry") and m.get("role") in ("outer", "inner")]


def _city_block(h_m, osm_id):
    for floor, choices in CITY_BANDS:
        if h_m >= floor:
            return choices[osm_id % len(choices)]
    return CITY_BANDS[-1][1][0]


def _city_schematic():
    data = _fetch_fidi()
    s_, w_, n_, e_ = FIDI_BBOX
    latc = math.radians((s_ + n_) / 2)

    def mx(lon):
        return (lon - w_) * 111320.0 * math.cos(latc) / CITY_BLOCK_M

    def mz(lat):
        return (n_ - lat) * 110574.0 / CITY_BLOCK_M

    # Build the footprint list and hand it to Geo.extrude_footprints — the
    # library rasterizes, extrudes, and does the tallest-wins stacking.
    buildings = []
    n_buildings, tallest = 0, 0.0
    for el in data["elements"]:
        tags = el.get("tags", {})
        is_part = tags.get("building:part") not in (None, "no")
        # Outlines with no data get a 10 m default; untagged parts defer to
        # the base outline that is already stamped underneath them.
        h_m = _osm_height_m(tags, None if is_part else 10.0)
        if h_m is None:
            continue
        rings = _rings(el)
        if not rings:
            continue
        # Towers straddling the bbox edge leave floating spire slivers behind;
        # keep an element only if its centroid is inside the district.
        pts = [p for ring in rings for p in ring]
        clat = sum(p["lat"] for p in pts) / len(pts)
        clon = sum(p["lon"] for p in pts) / len(pts)
        if not (s_ <= clat <= n_ and w_ <= clon <= e_):
            continue
        if not is_part:
            n_buildings += 1
            tallest = max(tallest, h_m)
        block = _city_block(h_m, el["id"])
        top = 1 + max(1, round(h_m / CITY_BLOCK_M))
        for ring in rings:
            buildings.append({
                "polygon": [[mx(p["lon"]), mz(p["lat"])] for p in ring],
                "min_y": 1, "height": top, "block": block,
            })
    print(f"  {n_buildings} buildings, tallest {tallest:.0f} m")
    return nu.Geo.extrude_footprints(json.dumps(buildings),
                                     "minecraft:gray_concrete", "fidi")


def scene_city(pack):
    """Lower Manhattan's Financial District from OSM building footprints,
    extruded by Geo.extrude_footprints and banded by height."""
    s = _city_schematic()
    # Aerial massing.
    render(s, pack, os.path.join(OUT, "geo-city.png"), w=1100, h=740,
           yaw=315, pitch=30, zoom=1.5, background=NAVY, sphere_fit=True)
    # Low-angle skyline: the 2.4M-block district as a silhouette, the same
    # schematic that streams straight out to a playable world.
    render(s, pack, os.path.join(OUT, "geo-city-skyline.png"), w=1200, h=620,
           yaw=300, pitch=10, zoom=1.9, background=NAVY, sphere_fit=True)
    return s


def scene_worldgen(pack):
    """The OSM city generated chunk by chunk — the streaming world generator,
    animated. The full city is built once (the one-shot path), then its 16×16
    chunk columns are revealed in a diagonal wavefront into a fixed frame, the
    way `from_schematic` streams them out one chunk at a time."""
    full = _city_schematic()
    mn, mx = full.tight_bounds_min(), full.tight_bounds_max()
    chunks = [(cx, cz)
              for cx in range(mn.x // 16, mx.x // 16 + 1)
              for cz in range(mn.z // 16, mx.z // 16 + 1)]
    chunks.sort(key=lambda c: (c[0] + c[1], c[0]))     # diagonal wavefront
    frames = 30
    per = max(1, math.ceil(len(chunks) / frames))
    accum = nu.Schematic.create("worldgen")
    tmp = tempfile.mkdtemp(prefix="nuc-wg-")
    try:
        fi = 0
        for start in range(0, len(chunks), per):
            for cx, cz in chunks[start:start + per]:
                x0, z0 = cx * 16, cz * 16
                accum.copy_region(full, x0, mn.y, z0, x0 + 15, mx.y, z0 + 15,
                                  x0, mn.y, z0, "[]")
            cfg = nu.RenderConfig.create(900, 520)
            cfg.set_isometric(); cfg.set_yaw(300.0); cfg.set_pitch(12.0)
            cfg.set_zoom(0.8); cfg.set_sphere_fit(False)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(accum, pack, cfg,
                                       os.path.join(tmp, f"f{fi:03}.png"))
            fi += 1
        # hold the finished skyline for a beat before the loop restarts
        last = os.path.join(tmp, f"f{fi - 1:03}.png")
        for _ in range(8):
            shutil.copy(last, os.path.join(tmp, f"f{fi:03}.png"))
            fi += 1
        assemble_gif(tmp, os.path.join(OUT, "worldgen-osm.gif"),
                     fps=10, max_colors=96)
        print(f"  wrote docs/media/worldgen-osm.gif ({fi} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return None


def scene_worldgen_sdf(pack):
    """A different source through the same generator: an SDF island filled one
    chunk at a time (the field only ever evaluated inside the chunk being
    written — the true streaming generator, not a reveal), accumulated
    center-outward into a fixed frame."""
    island = ('{"type": "displace", "amplitude": 7, "frequency": 0.06, "seed": 9,'
              ' "child": {"type": "ellipsoid", "radii": [64, 13, 64]}}')
    sdf = nu.Shape.sdf(island)
    brush = nu.Brush.solid("minecraft:moss_block")
    y0, y1 = -20, 20
    chunks = [(cx, cz) for cx in range(-5, 5) for cz in range(-5, 5)]
    chunks.sort(key=lambda c: c[0] * c[0] + c[1] * c[1] + c[0] * 0.01)   # center-outward
    accum = nu.Schematic.create("wg-sdf")
    tmp = tempfile.mkdtemp(prefix="nuc-wgs-")
    try:
        fi, since = 0, 0
        for cx, cz in chunks:
            box = nu.Shape.cuboid(cx * 16, y0, cz * 16, cx * 16 + 15, y1, cz * 16 + 15)
            nu.BuildingTool.fill(accum, sdf.intersection_with(box), brush)
            since += 1
            if since >= 2 and accum.block_count() > 0:      # a frame every 2 chunks
                since = 0
                cfg = nu.RenderConfig.create(760, 620)
                cfg.set_isometric(); cfg.set_yaw(225.0); cfg.set_pitch(26.0)
                cfg.set_zoom(1.15); cfg.set_sphere_fit(False)
                cfg.set_background(*NAVY)
                nu.Renderer.render_to_file(accum, pack, cfg,
                                           os.path.join(tmp, f"f{fi:03}.png"))
                fi += 1
        last = os.path.join(tmp, f"f{fi - 1:03}.png")
        for _ in range(8):
            shutil.copy(last, os.path.join(tmp, f"f{fi:03}.png"))
            fi += 1
        assemble_gif(tmp, os.path.join(OUT, "worldgen-sdf.gif"),
                     fps=10, max_colors=80)
        print(f"  wrote docs/media/worldgen-sdf.gif ({fi} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return None


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


def flat_art_palette(extra_exclude=()):
    """A flat-texture palette for pixel art, built from filters + excludes."""
    b = nu.PaletteBuilder.create()
    b.full_blocks_only()
    b.exclude_tile_entities()
    b.exclude_transparent()
    for kw in (*NOISY_BLOCKS, *extra_exclude):
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


def _starry_night_path():
    import urllib.request
    path = os.path.join(MODELS_DIR, "starry-night.jpg")
    if not os.path.exists(path):
        os.makedirs(MODELS_DIR, exist_ok=True)
        req = urllib.request.Request(_commons_url(PAINTINGS["starry-night"][0]),
                                     headers={"User-Agent": "nucleation-readme-media/1.0"})
        open(path, "wb").write(urllib.request.urlopen(req).read())
    return path


# The composition: a torus SDF, a lattice of spheres subtracted for holes, a
# noise warp to deform it, then Starry Night texture-projected around the ring
# and tube through the flat-art palette. Five features stacking in one build.
COMPOSE_SDF = json.dumps({
    "type": "warp", "amplitude": 3.0, "frequency": 0.045, "seed": 11,
    "child": {
        "type": "smoothSubtract", "k": 1.5,
        "a": {"type": "torus", "majorRadius": 26, "minorRadius": 9},
        "b": {"type": "repeat", "spacing": [11, 11, 11],
              "child": {"type": "sphere", "radius": 3.5}},
    },
})
COMPOSE_MAJOR = 26.0


def scene_compose(pack):
    """Everything at once: an SDF torus with lattice holes, warp-deformed, wearing
    Van Gogh's Starry Night wrapped around it (UVs from the torus geometry, colors
    through the dithered flat-art palette). Rendered as a turntable."""
    w, h, raw = _image_pixels(_starry_night_path(), 200)

    def sample(u, v):
        px = min(w - 1, max(0, int(u * w)))
        py = min(h - 1, max(0, int((1.0 - v) * h)))
        i = (py * w + px) * 3
        return raw[i], raw[i + 1], raw[i + 2]

    s = nu.Schematic.create("compose")
    nu.BuildingTool.fill(s, nu.Shape.sdf(COMPOSE_SDF), nu.Brush.solid("minecraft:stone"))
    pal = flat_art_palette().dithered()
    two_pi = 2.0 * math.pi
    r = COMPOSE_MAJOR
    for b in json.loads(s.get_all_blocks_json()):
        if b["name"] != "minecraft:stone":
            continue
        x, y, z = b["x"], b["y"], b["z"]
        phi = math.atan2(z, x)
        u = (phi / two_pi + 0.5) % 1.0                 # around the ring
        cphi, sphi = math.cos(phi), math.sin(phi)
        radial = (x - r * cphi) * cphi + (z - r * sphi) * sphi
        v = (math.atan2(y, radial) / two_pi + 0.5) % 1.0   # around the tube
        cr, cg, cb = _boost(*sample(u, v))
        s.set_block(x, y, z, pal.closest_block_dithered(cr, cg, cb, x, y, z))

    turntable_gif(s, pack, os.path.join(OUT, "compose-torus.gif"),
                  frames=36, w=680, h=560, pitch=36, zoom=1.2)
    return s


# NASA Blue Marble (land/ocean/ice composite), public domain — the globe's
# equirectangular texture, mirrored on Wikimedia Commons.
GLOBE_TEXTURE = "Land ocean ice 2048.jpg"
GLOBE_RADIUS = 120.0
GLOBE_FRAMES = 48
# Fixed sun, world space (camera at yaw 0 looks along -z, so +z faces the
# viewer): from the camera's upper right, pulled far enough sideways that the
# terminator crosses the visible disc as the texture spins underneath it.
GLOBE_SUN = (0.75, 0.40, 0.55)
GLOBE_AMBIENT = 0.26
GLOBE_GAMMA = 0.55   # lifts Blue Marble's near-black oceans into deep blues

# Speckle that reads as noise at one-block-per-"pixel" globe scale, on top of
# the map-art exclusions: mono-color concretes/wools should carry the scene.
GLOBE_EXTRA_NOISE = ["granite", "diorite", "andesite", "netherrack", "basalt",
                     "bone_block", "purpur", "quartz", "snow"]


def _globe_texture():
    import urllib.request
    path = os.path.join(MODELS_DIR, "blue-marble.jpg")
    if not os.path.exists(path):
        os.makedirs(MODELS_DIR, exist_ok=True)
        req = urllib.request.Request(
            _commons_url(GLOBE_TEXTURE),
            headers={"User-Agent": "nucleation-readme-media/1.0"})
        open(path, "wb").write(urllib.request.urlopen(req).read())
    return _image_pixels(path, 1024)


def _globe_surface(r=GLOBE_RADIUS):
    """The sphere's one-voxel surface shell with unit normals, computed once —
    the frame loop only re-shades it."""
    surf = []
    n = int(r) + 1
    for x in range(-n, n + 1):
        for y in range(-n, n + 1):
            for z in range(-n, n + 1):
                d = math.sqrt(x * x + y * y + z * z)
                if r - 0.6 <= d <= r + 0.4:
                    surf.append((x, y, z, x / d, y / d, z / d))
    return surf


def _globe_frame(phase, surf, tex, pal):
    """One globe schematic: texture spun by `phase` turns, sun fixed, so the
    day/night cycle emerges as continents cross the terminator."""
    tw, th, raw = tex
    m = math.sqrt(sum(c * c for c in GLOBE_SUN))
    sx, sy, sz = (c / m for c in GLOBE_SUN)
    amb = GLOBE_AMBIENT
    s = nu.Schematic.create("globe")
    # solid interior so no surface gap ever sees through to the far side
    nu.BuildingTool.fill(s, nu.Shape.sphere(0, 0, 0, GLOBE_RADIUS),
                         nu.Brush.solid("minecraft:black_concrete"))
    two_pi = 2.0 * math.pi
    for x, y, z, nx, ny, nz in surf:
        # -atan2 so longitude runs west→east left→right across the visible
        # face (the camera looks down -Z, +X on the right); -phase spins the
        # globe eastward (prograde), features drifting to the right.
        u = (-math.atan2(nz, nx) / two_pi - phase) % 1.0
        v = 0.5 - math.asin(max(-1.0, min(1.0, ny))) / math.pi
        i = (min(th - 1, max(0, int(v * th))) * tw + min(tw - 1, int(u * tw))) * 3
        light = max(0.0, nx * sx + ny * sy + nz * sz)
        light = light * light * (3.0 - 2.0 * light)     # soft terminator
        lum = amb + (1.0 - amb) * light
        r_, g_, b_ = _boost(
            *(255.0 * (c / 255.0) ** GLOBE_GAMMA * lum
              for c in (raw[i], raw[i + 1], raw[i + 2])),
            sat=1.3, contrast=1.0)
        s.set_block(x, y, z, pal.closest_block_dithered(r_, g_, b_, x, y, z))
    return s


def scene_globe(pack):
    """Day/night Earth: Blue Marble sampled per surface voxel, spun under a
    fixed sun, per-voxel Lambert + ambient, dithered flat-block palette."""
    tex = _globe_texture()
    surf = _globe_surface()
    pal = flat_art_palette(GLOBE_EXTRA_NOISE).dithered()
    tmp = tempfile.mkdtemp(prefix="nuc-globe-")
    try:
        for i in range(GLOBE_FRAMES):
            s = _globe_frame(i / GLOBE_FRAMES, surf, tex, pal)
            cfg = nu.RenderConfig.create(840, 840)
            cfg.set_isometric(); cfg.set_yaw(0.0); cfg.set_pitch(15.0)
            cfg.set_zoom(1.6); cfg.set_sphere_fit(True)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(s, pack, cfg,
                                       os.path.join(tmp, f"f{i:03}.png"))
        assemble_gif(tmp, os.path.join(OUT, "globe-day-night.gif"),
                     fps=GLOBE_FRAMES / 4.8, max_colors=80)
        print(f"  wrote docs/media/globe-day-night.gif ({GLOBE_FRAMES} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ── Chunked iteration / streaming ────────────────────────────────────────────

# Plasma control points (perceptually smooth, warm) — sampled per chunk so the
# tint contrasts hard against the green/stone terrain still waiting to stream.
_PLASMA = [(13, 8, 135), (126, 3, 168), (204, 71, 120),
           (248, 149, 64), (240, 249, 33)]


def _plasma(t):
    t = max(0.0, min(1.0, t)) * (len(_PLASMA) - 1)
    i = min(len(_PLASMA) - 2, int(t))
    f = t - i
    a, b = _PLASMA[i], _PLASMA[i + 1]
    return tuple(a[k] + (b[k] - a[k]) * f for k in range(3))


def scene_streaming(pack):
    """Chunked streaming caught mid-sweep: a rolling terrain walked 16×16
    column by column with the same chunk iterator the world streamer uses.
    The chunks already visited are tinted by *when* they came out of a
    center-outward traversal; the rest are still natural terrain — so the
    strategy's wavefront reads straight off the map as the frontier between
    processed and pending."""
    island = ('{"type": "displace", "amplitude": 6, "frequency": 0.085, "seed": 5,'
              ' "child": {"type": "ellipsoid", "radii": [66, 8, 66]}}')
    rules = ('{"fill": ['
             '{"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},'
             '{"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},'
             '{"block": "minecraft:stone"}]}')
    s = nu.Sdf.schematic_from_sdf_auto(island, rules)

    mn, mx = s.tight_bounds_min(), s.tight_bounds_max()
    cx, cz = (mn.x + mx.x) / 2.0, (mn.z + mx.z) / 2.0
    # One chunk per 16×16 column (chunk height spans the whole build), walked
    # center-outward: chunks[0] is the middle column, the four corners last.
    chunks = json.loads(s.get_chunks_with_strategy_json(
        16, 512, 16, "center_outward", cx, 0.0, cz))
    pal = nu.Palette.concrete()
    natural = '["minecraft:grass_block", "minecraft:dirt", "minecraft:stone"]'
    processed = int(round(len(chunks) * 0.6))     # freeze the sweep 60% in
    for i, ch in enumerate(chunks[:processed]):
        block = pal.closest_block(
            *(int(round(v)) for v in _plasma(i / (processed - 1))))
        x0, z0 = ch["chunk_x"] * 16, ch["chunk_z"] * 16
        nu.BuildingTool.fill_replacing(
            s, nu.Shape.cuboid(x0, mn.y, z0, x0 + 15, mx.y, z0 + 15),
            nu.Brush.solid(block), natural)

    render(s, pack, os.path.join(OUT, "streaming-chunks.png"), w=1100, h=720,
           yaw=235, pitch=44, zoom=1.12, background=NAVY, sphere_fit=True)
    return s


# ── Regions, transforms & stamping ───────────────────────────────────────────

def _fort_regions():
    """A keep with two wings, each a *separate named region* in one schematic —
    distinct blocks so the regions read apart in the render."""
    s = nu.Schematic.create("fort")

    def box(region, x0, y0, z0, x1, y1, z1, block, hollow=False):
        for x in range(x0, x1 + 1):
            for y in range(y0, y1 + 1):
                for z in range(z0, z1 + 1):
                    if hollow and x0 < x < x1 and z0 < z < z1 and y < y1:
                        continue
                    s.set_block_in_region(region, x, y, z, block)

    # keep: central quartz tower with a crenellated crown
    box("keep", -3, 0, -3, 3, 11, 3, "minecraft:quartz_block", hollow=True)
    for x in range(-3, 4, 2):
        for z in (-3, 3):
            s.set_block_in_region("keep", x, 12, z, "minecraft:quartz_block")
    for z in range(-3, 4, 2):
        for x in (-3, 3):
            s.set_block_in_region("keep", x, 12, z, "minecraft:quartz_block")
    # east_wing: a copper hall reaching out along +x
    box("east_wing", 4, 0, -2, 13, 4, 2, "minecraft:cut_copper", hollow=True)
    box("east_wing", 4, 5, -2, 13, 5, 2, "minecraft:copper_block")
    # west_wing: a prismarine hall reaching out along -x
    box("west_wing", -13, 0, -2, -4, 4, 2, "minecraft:prismarine_bricks", hollow=True)
    box("west_wing", -13, 5, -2, -4, 5, 2, "minecraft:dark_prismarine")
    return s


def scene_regions(pack):
    """Before/after: three named regions, then one wing rotated in place."""
    tmp = tempfile.mkdtemp(prefix="nuc-regions-")
    try:
        before = os.path.join(tmp, "a.png")
        after = os.path.join(tmp, "b.png")
        s = _fort_regions()
        render(s, pack, before, w=760, h=620, yaw=210, pitch=32, zoom=1.05,
               background=NAVY, sphere_fit=True)
        s.rotate_region_y("east_wing", 90)      # turn just the copper wing
        render(s, pack, after, w=760, h=620, yaw=210, pitch=32, zoom=1.05,
               background=NAVY, sphere_fit=True)
        hstack([before, after], os.path.join(OUT, "regions.png"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return s


# ── Block entities, entities & NBT ───────────────────────────────────────────

def scene_blockentities(pack):
    """A loot vault: a room of block-entity blocks, each a carrier of NBT —
    chests set with real Item SNBT, a caged spawner, furnaces, brewing, and
    stacked dyed shulker boxes."""
    s = nu.Schematic.create("vault")
    fill = nu.BuildingTool.fill
    # stone-brick floor + low kerb walls, NO ceiling (open for the aerial look)
    fill(s, nu.Shape.cuboid(-7, 0, -7, 7, 0, 7), nu.Brush.solid("minecraft:stone_bricks"))
    for (a, b, c, d) in ((-7, -7, 7, -7), (-7, 7, 7, 7), (-7, -7, -7, 7), (7, -7, 7, 7)):
        fill(s, nu.Shape.cuboid(a, 1, b, c, 2, d),
             nu.Brush.solid("minecraft:chiseled_stone_bricks"))

    # chests along the back wall, each carrying real loot NBT (place the block,
    # then attach the block entity — set_block_entity only writes the NBT)
    for i, x in enumerate(range(-6, 7, 2)):
        s.set_block(x, 1, -6, "minecraft:chest[facing=south]")
        s.set_block_entity(x, 1, -6, "minecraft:chest",
            '{Items:[{Slot:0b,id:"minecraft:diamond",Count:%db}]}' % (i + 1))
    # barrels along the front, a second data-bearing container
    for x in range(-6, 7, 2):
        s.set_block(x, 1, 6, "minecraft:barrel[facing=up]")
    # furnaces / blast furnace / smoker down the left wall
    for i, z in enumerate(range(-5, 6, 2)):
        blk = ("minecraft:furnace", "minecraft:blast_furnace", "minecraft:smoker",
               "minecraft:furnace", "minecraft:blast_furnace", "minecraft:smoker")[i]
        s.set_block(-6, 1, z, blk + "[facing=east]")
    # dyed shulker boxes stacked in the right corner — the color accent
    for i, c in enumerate(("red", "orange", "yellow", "lime", "cyan", "purple",
                           "magenta", "light_blue", "green")):
        s.set_block(6, 1 + i % 3, 5 - (i // 3) * 2, f"minecraft:{c}_shulker_box")
    # a mob spawner in an iron cage, dead center
    s.set_block(0, 1, 0, "minecraft:spawner")
    for dx in (-1, 1):
        s.set_block(dx, 1, 0, "minecraft:iron_bars")
        s.set_block(0, 1, dx, "minecraft:iron_bars")
    # brewing + enchanting corners
    s.set_block(-5, 1, 5, "minecraft:brewing_stand")
    s.set_block(-6, 1, 5, "minecraft:cauldron")
    s.set_block(4, 1, -5, "minecraft:enchanting_table")
    s.set_block(5, 1, -5, "minecraft:lectern[facing=west]")
    s.set_block(3, 1, -5, "minecraft:bell[attachment=floor]")

    render(s, pack, os.path.join(OUT, "block-entities.png"), w=900, h=720,
           yaw=225, pitch=58, zoom=1.2, background=NAVY, sphere_fit=True)
    return s


def scene_storage(pack):
    """A little library of saved builds — storage's illustration: four distinct
    schematics, each the kind of thing you persist and reload through one URI."""
    def sphere():
        s = nu.Schematic.create("sphere")
        b = nu.Brush.shaded(206, 120, 78, -1.0, 0.7, -0.3)
        b.set_palette(nu.Palette.terracotta())
        nu.BuildingTool.fill(s, nu.Shape.sphere(0, 0, 0, 11), b)
        return s

    def torus():
        s = nu.Schematic.create("torus")
        stops = [i / 6 for i in range(7)]
        colors = [255, 40, 40, 255, 180, 0, 60, 200, 60, 40, 180, 220,
                  60, 70, 230, 200, 60, 220, 255, 40, 40]
        b = nu.Brush.curve_gradient(stops, bytes(colors), nu.InterpolationSpace.Oklab)
        b.set_palette(nu.Palette.wool())
        nu.BuildingTool.fill(s, nu.Shape.torus(0, 0, 0, 13, 5, 0, 1, 0), b)
        return s

    def tree():
        s = nu.Schematic.create("tree")
        nu.BuildingTool.fill(s, nu.Shape.cylinder(0, 0, 0, 0, 1, 0, 1.4, 11),
                             nu.Brush.solid("minecraft:oak_log"))
        nu.BuildingTool.fill(s, nu.Shape.sphere(0, 13, 0, 7),
                             nu.Brush.solid("minecraft:oak_leaves"))
        return s

    def pyramid():
        s = nu.Schematic.create("pyramid")
        nu.BuildingTool.fill(s, nu.Shape.pyramid(0, 0, 0, 11, 11, 15, 0, 1, 0),
                             nu.Brush.solid("minecraft:sandstone"))
        return s

    tmp = tempfile.mkdtemp(prefix="nuc-store-")
    try:
        parts = []
        for name, mk, yaw, pitch, zoom in (
                ("sphere", sphere, 45, 22, 1.2),
                ("torus", torus, 0, 34, 1.25),
                ("tree", tree, 30, 18, 1.05),
                ("pyramid", pyramid, 35, 22, 1.1)):
            p = os.path.join(tmp, name + ".png")
            cfg = nu.RenderConfig.create(380, 380)
            cfg.set_isometric(); cfg.set_yaw(yaw); cfg.set_pitch(pitch)
            cfg.set_zoom(zoom); cfg.set_sphere_fit(True)
            cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(mk(), pack, cfg, p)
            parts.append(p)
        hstack(parts, os.path.join(OUT, "storage-gallery.png"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return None


# ── Gallery: ten cool builds ─────────────────────────────────────────────────

import colorsys as _colorsys


def _spectrum(n):
    """n concrete blocks evenly around the hue wheel, for closest_block lookups."""
    pal = nu.Palette.concrete()
    return pal


def _hue_block(pal, hue, sat=0.85, val=1.0):
    r, g, b = (round(c * 255) for c in _colorsys.hsv_to_rgb(hue % 1.0, sat, val))
    return pal.closest_block(r, g, b)


def scene_g_dna(pack):
    """A rainbow DNA double helix: two phase-shifted strands with base-pair rungs,
    each strand bead colored by its turn around the axis."""
    s = nu.Schematic.create("dna")
    pal = nu.Palette.concrete()
    fill, sphere = nu.BuildingTool.fill, nu.Shape.sphere
    radius, steps, rise = 11.0, 260, 0.42
    for i in range(steps):
        t = i / 24.0
        y = round(i * rise)
        block = _hue_block(pal, i / steps)
        for offset in (0.0, math.pi):
            x = round(radius * math.cos(t + offset))
            z = round(radius * math.sin(t + offset))
            fill(s, sphere(x, y, z, 2), nu.Brush.solid(block))
        if i % 11 == 0:                       # a base-pair rung
            ax, az = round(radius * math.cos(t)), round(radius * math.sin(t))
            bx, bz = round(radius * math.cos(t + math.pi)), round(radius * math.sin(t + math.pi))
            rung = ("minecraft:white_concrete" if (i // 11) % 2 == 0
                    else "minecraft:light_gray_concrete")
            fill(s, nu.Shape.cylinder_between(ax, y, az, bx, y, bz, 1.0), nu.Brush.solid(rung))
    render(s, pack, os.path.join(OUT, "gallery-dna.png"), w=520, h=820,
           yaw=35, pitch=8, zoom=1.25, background=NAVY, sphere_fit=True)
    return s


def scene_g_knot(pack):
    """A trefoil knot drawn as a fat rainbow tube: a parametric curve stamped as
    overlapping spheres, hue running along its length."""
    s = nu.Schematic.create("knot")
    pal = nu.Palette.concrete()
    fill, sphere = nu.BuildingTool.fill, nu.Shape.sphere
    steps, scale = 480, 11.0
    for i in range(steps):
        t = i / steps * 2 * math.pi
        x = round(scale * (math.sin(t) + 2 * math.sin(2 * t)))
        y = round(scale * (math.cos(t) - 2 * math.cos(2 * t)))
        z = round(scale * (-math.sin(3 * t)))
        fill(s, sphere(x, y, z, 3), nu.Brush.solid(_hue_block(pal, i / steps)))
    render(s, pack, os.path.join(OUT, "gallery-knot.png"), w=720, h=680,
           yaw=30, pitch=30, zoom=1.2, background=NAVY, sphere_fit=True)
    return s


def _in_menger(x, y, z, level):
    for _ in range(level):
        if [x % 3, y % 3, z % 3].count(1) >= 2:
            return False
        x //= 3; y //= 3; z //= 3
    return True


def scene_g_menger(pack):
    """A level-4 Menger sponge: the classic recursive fractal, every cell kept
    or carved by the same mod-3 test, tinted by height."""
    s = nu.Schematic.create("menger")
    pal = flat_art_palette().dithered()                        # rich + dithered = smooth
    n = 81
    for x in range(n):
        for y in range(n):
            for z in range(n):
                if _in_menger(x, y, z, 4):
                    f = y / n                                  # deep teal up to pale ice
                    block = pal.closest_block_dithered(
                        int(30 + f * 185), int(120 + f * 125), int(150 + f * 105), x, y, z)
                    s.set_block(x, y, z, block)
    render(s, pack, os.path.join(OUT, "gallery-menger.png"), w=720, h=680,
           yaw=35, pitch=28, zoom=1.15, background=NAVY, sphere_fit=True)
    return s


def _rodrigues(v, axis, ang):
    # Rodrigues rotation of vector v about `axis` by angle `ang`.
    import math as _m
    ux, uy, uz = axis
    n = _m.sqrt(ux * ux + uy * uy + uz * uz) or 1.0
    ux, uy, uz = ux / n, uy / n, uz / n
    c, sn = _m.cos(ang), _m.sin(ang)
    vx, vy, vz = v
    dot = ux * vx + uy * vy + uz * vz
    return (
        vx * c + (uy * vz - uz * vy) * sn + ux * dot * (1 - c),
        vy * c + (uz * vx - ux * vz) * sn + uy * dot * (1 - c),
        vz * c + (ux * vy - uy * vx) * sn + uz * dot * (1 - c),
    )


def scene_g_tree(pack):
    """A recursive fractal tree: each branch splits into three tilted children,
    autumn foliage at the tips."""
    s = nu.Schematic.create("tree")
    fill = nu.BuildingTool.fill
    autumn = ["minecraft:green_concrete", "minecraft:lime_concrete", "minecraft:yellow_concrete",
              "minecraft:orange_concrete", "minecraft:red_concrete"]

    def perp(d):
        a = (1.0, 0.0, 0.0) if abs(d[0]) < 0.9 else (0.0, 1.0, 0.0)
        cx = d[1] * a[2] - d[2] * a[1]
        cy = d[2] * a[0] - d[0] * a[2]
        cz = d[0] * a[1] - d[1] * a[0]
        return (cx, cy, cz)

    def grow(x, y, z, d, length, radius, depth, twist):
        x2, y2, z2 = x + d[0] * length, y + d[1] * length, z + d[2] * length
        fill(s, nu.Shape.cylinder_between(round(x), round(y), round(z),
                                          round(x2), round(y2), round(z2), max(1.0, radius)),
             nu.Brush.solid("minecraft:spruce_log"))
        if depth == 0:
            leaf = autumn[min(len(autumn) - 1, int((y2 / 55.0) * len(autumn)))]
            fill(s, nu.Shape.sphere(round(x2), round(y2), round(z2), 4), nu.Brush.solid(leaf))
            return
        p = perp(d)
        for k in range(3):
            tilted = _rodrigues(d, p, math.radians(34))
            child = _rodrigues(tilted, d, twist + k * 2 * math.pi / 3)
            grow(x2, y2, z2, child, length * 0.74, radius * 0.68, depth - 1, twist + 0.7)

    grow(0, 0, 0, (0.0, 1.0, 0.0), 15.0, 3.0, 7, 0.4)
    render(s, pack, os.path.join(OUT, "gallery-tree.png"), w=640, h=760,
           yaw=40, pitch=18, zoom=1.2, background=NAVY, sphere_fit=True)
    return s


def scene_g_gyroid(pack):
    """A gyroid: the triply-periodic minimal surface where
    sin·cos + sin·cos + sin·cos crosses zero, thickened into a shell and tinted
    iridescent along its diagonal."""
    s = nu.Schematic.create("gyroid")
    pal = nu.Palette.concrete()
    n, k = 64, 2 * math.pi / 16.0
    for x in range(n):
        for y in range(n):
            for z in range(n):
                sx, sy, sz = x * k, y * k, z * k
                f = (math.sin(sx) * math.cos(sy) + math.sin(sy) * math.cos(sz)
                     + math.sin(sz) * math.cos(sx))
                if abs(f) < 0.55:
                    s.set_block(x, y, z, _hue_block(pal, (x + y + z) / (2.0 * n), 0.85, 1.0))
    render(s, pack, os.path.join(OUT, "gallery-gyroid.png"), w=720, h=680,
           yaw=30, pitch=30, zoom=1.2, background=NAVY, sphere_fit=True)
    return s


def scene_g_mandelbulb(pack):
    """The power-8 Mandelbulb: a spherical z → z^8 + c escape test per voxel,
    the surface tinted by a fiery vertical gradient."""
    s = nu.Schematic.create("bulb")
    pal = flat_art_palette().dithered()                        # dithered ember gradient
    n, power, span = 132, 8, 2.5
    for ix in range(n):
        for iy in range(n):
            for iz in range(n):
                cx, cy, cz = ((ix / n - 0.5) * span, (iy / n - 0.5) * span,
                              (iz / n - 0.5) * span)
                x = y = z = 0.0
                inside = True
                for _ in range(7):
                    r = math.sqrt(x * x + y * y + z * z)
                    if r > 2.0:
                        inside = False
                        break
                    r = r or 1e-9
                    theta = power * math.acos(max(-1.0, min(1.0, z / r)))
                    phi = power * math.atan2(y, x)
                    rp = r ** power
                    x = rp * math.sin(theta) * math.cos(phi) + cx
                    y = rp * math.sin(theta) * math.sin(phi) + cy
                    z = rp * math.cos(theta) + cz
                if inside:
                    f = iy / n                                   # ember red up to gold
                    s.set_block(ix, iy, iz, pal.closest_block_dithered(
                        int(90 + f * 165), int(15 + f * 210), int(8 + f * 70), ix, iy, iz))
    render(s, pack, os.path.join(OUT, "gallery-mandelbulb.png"), w=720, h=700,
           yaw=25, pitch=26, zoom=1.3, background=NAVY, sphere_fit=True)
    return s


FOX_URL = ("https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Assets/"
           "main/Models/Fox/glTF-Binary/Fox.glb")


def scene_g_fox(pack):
    """The Khronos low-poly Fox, voxelized with its texture projected onto the
    blocks — a real 3D model becomes a chunky character."""
    glb = open(_fetch_model("Fox.glb", FOX_URL), "rb").read()
    fox = nu.Voxelizer.schematic_from_glb_textured(glb, 84.0, 0.7,
                                                   nu.Palette.solid(), "fox")
    render(fox, pack, os.path.join(OUT, "gallery-fox.png"), w=760, h=620,
           yaw=30, pitch=14, zoom=1.15, background=NAVY, sphere_fit=True)
    return fox


def _superformula(a, m, n1, n2, n3):
    t1 = abs(math.cos(m * a / 4.0)) ** n2
    t2 = abs(math.sin(m * a / 4.0)) ** n3
    return (t1 + t2) ** (-1.0 / n1)


def scene_g_supershape(pack):
    """A 3D supershape from the superformula: a spherical product of two profiles,
    sampled onto a surface of blocks and colored by longitude."""
    s = nu.Schematic.create("supershape")
    pal = nu.Palette.concrete()
    fill, sphere = nu.BuildingTool.fill, nu.Shape.sphere
    scale, steps = 26.0, 200
    for i in range(steps):
        theta = -math.pi + i / steps * 2 * math.pi
        r1 = _superformula(theta, 7, 0.2, 1.7, 1.7)
        for j in range(steps // 2):
            phi = -math.pi / 2 + j / (steps // 2) * math.pi
            r2 = _superformula(phi, 7, 0.2, 1.7, 1.7)
            x = round(scale * r1 * math.cos(theta) * r2 * math.cos(phi))
            y = round(scale * r2 * math.sin(phi))
            z = round(scale * r1 * math.sin(theta) * r2 * math.cos(phi))
            fill(s, sphere(x, y, z, 1), nu.Brush.solid(_hue_block(pal, i / steps)))
    render(s, pack, os.path.join(OUT, "gallery-supershape.png"), w=720, h=680,
           yaw=30, pitch=26, zoom=1.2, background=NAVY, sphere_fit=True)
    return s


def scene_g_wave(pack):
    """Two circular wave sources interfering on a heightfield, animated: each
    column rises to the summed wave and is colored by its crest, deep blue in
    the troughs to white on the peaks."""
    frames, n = 40, 88
    pal = nu.Palette.concrete()
    s1, s2 = (n * 0.32, n * 0.34), (n * 0.70, n * 0.62)
    tmp = tempfile.mkdtemp(prefix="nuc-wave-")
    try:
        for f in range(frames):
            s = nu.Schematic.create("wave")
            ph = f / frames * 2 * math.pi
            for gx in range(n):
                for gz in range(n):
                    d1 = math.hypot(gx - s1[0], gz - s1[1])
                    d2 = math.hypot(gx - s2[0], gz - s2[1])
                    hv = math.sin(d1 * 0.5 - 2 * ph) + math.sin(d2 * 0.5 - 2 * ph)
                    y = round(10 + hv * 4)
                    t = (hv + 2) / 4.0
                    block = pal.closest_block(int(30 + t * 190),
                                              int(90 + t * 150), int(205 + t * 50))
                    s.fill_cuboid(gx, 0, gz, gx, y, gz, block)
            cfg = nu.RenderConfig.create(720, 470)
            cfg.set_isometric(); cfg.set_yaw(35.0); cfg.set_pitch(34.0)
            cfg.set_zoom(1.06); cfg.set_sphere_fit(True); cfg.set_background(*NAVY)
            nu.Renderer.render_to_file(s, pack, cfg, os.path.join(tmp, f"f{f:03}.png"))
        assemble_gif(tmp, os.path.join(OUT, "gallery-wave.gif"), fps=14, max_colors=64)
        print(f"  wrote docs/media/gallery-wave.gif ({frames} frames)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return None


LOGO_FONT = "/System/Library/Fonts/Supplemental/Impact.ttf"


def scene_g_logo(pack):
    """Type set in blocks: draw a word to an image, then extrude every letter
    pixel into a short prism, colored by a rainbow sweep across the word."""
    tmp = tempfile.mkdtemp(prefix="nuc-logo-")
    try:
        txt = os.path.join(tmp, "txt.png")
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error",
                        "-f", "lavfi", "-i", "color=c=black:s=1000x170",
                        "-vf", (f"drawtext=fontfile={LOGO_FONT}:text='NUCLEATION':"
                                "fontsize=130:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2"),
                        "-frames:v", "1", txt], check=True)
        w, h, raw = _image_pixels(txt, 230)
        pal = nu.Palette.concrete()
        s = nu.Schematic.create("logo")
        for py in range(h):
            for px in range(w):
                if raw[(py * w + px) * 3] > 128:           # a letter pixel
                    block = _hue_block(pal, px / w)         # rainbow across the word
                    for d in range(7):                     # extrude toward the camera
                        s.set_block(px, h - 1 - py, d, block)
        render(s, pack, os.path.join(OUT, "gallery-logo.png"), w=1000, h=300,
               yaw=24, pitch=20, zoom=1.85, background=NAVY, sphere_fit=True)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return None


SCENES = {
    "g-dna": scene_g_dna,
    "g-knot": scene_g_knot,
    "g-menger": scene_g_menger,
    "g-tree": scene_g_tree,
    "g-gyroid": scene_g_gyroid,
    "g-mandelbulb": scene_g_mandelbulb,
    "g-fox": scene_g_fox,
    "g-supershape": scene_g_supershape,
    "g-wave": scene_g_wave,
    "g-logo": scene_g_logo,
    "streaming": scene_streaming,
    "worldgen": scene_worldgen,
    "worldgen-sdf": scene_worldgen_sdf,
    "storage": scene_storage,
    "regions": scene_regions,
    "blockentities": scene_blockentities,
    "globe": scene_globe,
    "paintings": scene_paintings,
    "compose": scene_compose,
    "dither": scene_dither,
    "scripting": scene_scripting,
    "simulation": scene_simulation,
    "teapot": scene_teapot,
    "duck": scene_duck,
    "mariokart": scene_mariokart,
    "mariokart2": scene_mk64_track2,
    "mountains": scene_mountains,
    "city": scene_city,
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
