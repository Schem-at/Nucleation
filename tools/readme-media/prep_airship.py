#!/usr/bin/env python3
"""Download + prepare the MKDS Airship Fortress course as an opaque GLB.

Same recipe as generate.py's _mariokart_glb / _koopa_glb:
  * drop the materials that are not the course itself,
  * strip MTL transparency (d/Tr/Tf/map_d) so the voxelizer's palette snap
    cannot land in stained glass,
  * rebuild as a self-contained binary GLB via `npx obj2gltf`.

Needs Node on PATH. Output: target/readme-models/mkds-airship-fortress.glb
"""

import os
import subprocess

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
MODELS_DIR = os.path.join(ROOT, "target", "readme-models")

# Airship Fortress (Mario Kart DS), Courses section of The Models Resource.
AF_ZIP_URL = "https://models.spriters-resource.com/media/assets/297/299991.zip"
AF_UA = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
         "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")

# What to drop is a question about *position*, not material. lambert376 looks
# like one big skybox material (21 faces spanning 903x166x844) but is really 15
# unconnected billboard quads: most are the distant flying-airship sprites
# scattered out to x=-466..438 / z=-815..29, yet three of them (15 tris) sit
# inside the course. A name-based drop list throws those away with the rest.
#
# So: split the mesh into connected pieces, take the largest (87% of all tris,
# the fortress itself), and drop only pieces lying wholly outside its bounds.
# That keeps every in-course detail and still removes the far-flung sprites,
# which otherwise blow the fit() scale by ~2.2x and wrap the build in a wall.
OUTSIDE_TOLERANCE = 0.02  # fraction of the main piece's size to allow beyond it


def _connected_pieces(tris):
    """Union-find over quantized vertex positions -> list of face-index lists."""
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

    for _, keys in tris:
        for k in keys:
            parent.setdefault(k, k)
        for k in keys[1:]:
            union(keys[0], k)
    groups = {}
    for fi, keys in tris:
        groups.setdefault(find(keys[0]), []).append(fi)
    return list(groups.values())


def prep():
    glb = os.path.join(MODELS_DIR, "mkds-airship-fortress.glb")
    if os.path.exists(glb):
        return glb
    os.makedirs(MODELS_DIR, exist_ok=True)

    zip_path = os.path.join(MODELS_DIR, "mkds-airship.zip")
    if not os.path.exists(zip_path):
        subprocess.run(["curl", "-sL", "-A", AF_UA, "-o", zip_path, AF_ZIP_URL],
                       check=True)
    src = os.path.join(MODELS_DIR, "mkds-airship")
    subprocess.run(["unzip", "-oq", zip_path, "-d", src], check=True)
    src = os.path.join(src, "airship_course")

    raw = open(os.path.join(src, "airship_course.obj"), errors="ignore").readlines()
    verts, faces = [], []
    for i, line in enumerate(raw):
        p = line.split()
        if not p:
            continue
        if p[0] == "v":
            verts.append(tuple(map(float, p[1:4])))
        elif p[0] == "f":
            faces.append((i, [int(t.split("/")[0]) - 1 for t in p[1:]]))

    def key(vi):
        return tuple(round(c, 1) for c in verts[vi])

    pieces = _connected_pieces([(i, [key(v) for v in idx]) for i, idx in faces])
    face_verts = {i: idx for i, idx in faces}

    def bounds(face_ids):
        pts = [verts[v] for fi in face_ids for v in face_verts[fi]]
        return ([min(p[k] for p in pts) for k in range(3)],
                [max(p[k] for p in pts) for k in range(3)])

    pieces.sort(key=len, reverse=True)
    lo, hi = bounds(pieces[0])
    pad = [OUTSIDE_TOLERANCE * (hi[k] - lo[k]) for k in range(3)]

    drop_faces = set()
    for piece in pieces[1:]:
        plo, phi = bounds(piece)
        # Drop only if the piece misses the fortress's bounds on some axis
        # entirely -- an in-course detail always overlaps on all three.
        if any(phi[k] < lo[k] - pad[k] or plo[k] > hi[k] + pad[k] for k in range(3)):
            drop_faces.update(piece)

    lines = [line.replace("airship_course.mtl", "course.mtl")
             for i, line in enumerate(raw) if i not in drop_faces]
    open(os.path.join(src, "course.obj"), "w").writelines(lines)
    print(f"  {len(pieces)} connected pieces; dropped {len(drop_faces)} of "
          f"{len(faces)} faces as out-of-course")

    # Every material in this rip carries `d 1 / Tr 1 / map_d <same png>`. The
    # map_d alpha channel is what pushes obj2gltf into an alpha-blended
    # material, which snaps to stained glass at palette time.
    mtl = [line for line in open(os.path.join(src, "airship_course.mtl"),
                                 errors="ignore")
           if not line.split() or line.split()[0] not in ("d", "Tr", "Tf", "map_d")]
    open(os.path.join(src, "course.mtl"), "w").writelines(mtl)

    subprocess.run(["npx", "-y", "obj2gltf",
                    "-i", os.path.join(src, "course.obj"),
                    "-o", glb, "--binary"], check=True)
    return glb


if __name__ == "__main__":
    print(prep())
