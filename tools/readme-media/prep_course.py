#!/usr/bin/env python3
"""Download + prepare a Models Resource course rip as an opaque GLB.

Generalized from prep_airship.py. For each course:
  * download + unzip the asset,
  * find the OBJ (folder names vary per rip),
  * split into connected pieces and drop small ones lying wholly outside the
    main piece's bounds -- skyboxes, backdrops and distant billboard sprites,
  * strip MTL transparency (d/Tr/Tf/map_d) so the palette snap cannot land in
    glass,
  * rebuild as a self-contained binary GLB via `npx obj2gltf`.

    python3 tools/readme-media/prep_course.py            # all courses
    python3 tools/readme-media/prep_course.py dk-pass    # a subset, by slug

Needs Node on PATH.
"""

import json
import os
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
MODELS_DIR = os.path.join(ROOT, "target", "readme-models", "courses")
MANIFEST = os.path.join(os.path.dirname(os.path.abspath(__file__)), "courses.json")

UA = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
      "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
# The zip's bucket directory is NOT derivable from the asset id (299991 lives
# under 297) -- assets are bucketed by insertion index. courses.json carries the
# bucket scraped from each listing icon's URL, which shares it.
ASSET_URL = "https://models.spriters-resource.com/media/assets/{bucket}/{id}.zip"

# A piece is discarded only if it is BOTH outside the main piece's bounds and
# small. Size alone is not enough (in-course details are small too) and
# outside-ness alone is not enough either: a legitimately far-flung section of
# a long course would qualify. Skyboxes and billboard sprites are both.
OUTSIDE_TOLERANCE = 0.02   # slack on the main bounds, as a fraction of its size
MAX_DROP_FRACTION = 0.05   # a piece over this share of all faces is never junk


def _connected_pieces(faces):
    parent = {}

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb

    for _, keys in faces:
        for k in keys:
            parent.setdefault(k, k)
        for k in keys[1:]:
            union(keys[0], k)
    groups = {}
    for fi, keys in faces:
        groups.setdefault(find(keys[0]), []).append(fi)
    return list(groups.values())


def _filter_obj(obj_path, out_obj, mtl_from, mtl_to):
    raw = open(obj_path, errors="ignore").readlines()
    verts, faces = [], []
    for i, line in enumerate(raw):
        p = line.split()
        if not p:
            continue
        if p[0] == "v":
            verts.append(tuple(map(float, p[1:4])))
        elif p[0] == "f":
            try:
                faces.append((i, [int(t.split("/")[0]) - 1 for t in p[1:]]))
            except ValueError:
                pass
    if not faces:
        raise RuntimeError(f"{obj_path}: no faces")

    def key(vi):
        return tuple(round(c, 1) for c in verts[vi])

    pieces = _connected_pieces([(i, [key(v) for v in idx]) for i, idx in faces])
    face_verts = {i: idx for i, idx in faces}

    def bounds(ids):
        pts = [verts[v] for fi in ids for v in face_verts[fi]]
        return ([min(p[k] for p in pts) for k in range(3)],
                [max(p[k] for p in pts) for k in range(3)])

    pieces.sort(key=len, reverse=True)
    lo, hi = bounds(pieces[0])
    pad = [OUTSIDE_TOLERANCE * (hi[k] - lo[k]) for k in range(3)]

    drop = set()
    for piece in pieces[1:]:
        if len(piece) > MAX_DROP_FRACTION * len(faces):
            continue
        plo, phi = bounds(piece)
        if any(phi[k] < lo[k] - pad[k] or plo[k] > hi[k] + pad[k] for k in range(3)):
            drop.update(piece)

    lines = [line.replace(mtl_from, mtl_to)
             for i, line in enumerate(raw) if i not in drop]
    open(out_obj, "w").writelines(lines)
    return len(pieces), len(drop), len(faces)


def prep(slug, asset_id, bucket, force=False):
    os.makedirs(MODELS_DIR, exist_ok=True)
    glb = os.path.join(MODELS_DIR, f"{slug}.glb")
    if os.path.exists(glb) and not force:
        return glb, "cached"

    zip_path = os.path.join(MODELS_DIR, f"{slug}.zip")
    if not os.path.exists(zip_path):
        url = ASSET_URL.format(bucket=bucket, id=asset_id)
        subprocess.run(["curl", "-sL", "-A", UA, "-o", zip_path, url], check=True)
    src = os.path.join(MODELS_DIR, slug)
    subprocess.run(["unzip", "-oq", zip_path, "-d", src], check=True)

    # Exclude only our own generated file, not the rip's -- several rips name
    # their model `course_model.obj`. Skip `*_V`/`*_v` variants too: those are
    # vertex-colour twins of the textured model and voxelize to grey.
    objs = [os.path.join(r, f) for r, _, fs in os.walk(src)
            for f in fs if f.lower().endswith(".obj")
            and f != "course_filtered.obj"
            and not f.lower().endswith("_v.obj")]
    if not objs:
        raise RuntimeError(f"{slug}: no .obj in asset {asset_id}")
    obj = max(objs, key=os.path.getsize)          # the course, not a prop
    base = os.path.dirname(obj)
    mtl_name = os.path.splitext(os.path.basename(obj))[0] + ".mtl"

    n_pieces, n_drop, n_faces = _filter_obj(
        obj, os.path.join(base, "course_filtered.obj"), mtl_name, "course_filtered.mtl")

    mtl_path = os.path.join(base, mtl_name)
    if os.path.exists(mtl_path):
        mtl = [line for line in open(mtl_path, errors="ignore")
               if not line.split() or line.split()[0] not in ("d", "Tr", "Tf", "map_d")]
        open(os.path.join(base, "course_filtered.mtl"), "w").writelines(mtl)

    subprocess.run(["npx", "-y", "obj2gltf",
                    "-i", os.path.join(base, "course_filtered.obj"),
                    "-o", glb, "--binary"],
                   check=True, stdout=subprocess.DEVNULL)
    return glb, f"{n_faces} faces, {n_pieces} pieces, dropped {n_drop}"


def main():
    courses = json.load(open(MANIFEST))
    wanted = sys.argv[1:]
    if wanted:
        courses = [c for c in courses if c["slug"] in wanted]
    for c in courses:
        try:
            glb, note = prep(c["slug"], c["asset"], c["bucket"])
            print(f"OK   {c['slug']:<26} {note}", flush=True)
        except Exception as e:
            print(f"FAIL {c['slug']:<26} {e}", flush=True)


if __name__ == "__main__":
    main()
