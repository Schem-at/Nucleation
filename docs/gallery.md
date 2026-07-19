# The Nucleation Gallery

Ten builds that show what composes. Every one is a real recipe in
[`tools/readme-media/generate.py`](../tools/readme-media/generate.py), rendered
by nucleation itself, and small enough to read in a sitting. Back to the
[README](../README.md).

Snippets below assume `from nucleation import *`, an empty schematic `s`, and a
little `hsv(t)` helper that returns an `(r, g, b)` for a hue in `[0, 1)`.

## Rainbow DNA

Two phase-shifted strands sweeping up an axis, with base-pair rungs between them,
each bead colored by its turn around the helix.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-dna.png" width="320" alt="A rainbow DNA double helix built from blocks, with white base-pair rungs">
</div>

```python
pal, fill = Palette.concrete(), BuildingTool.fill
for i in range(260):
    t, y = i / 24, round(i * 0.42)
    bead = pal.closest_block(*hsv(i / 260))          # rainbow up the axis
    for phase in (0.0, math.pi):                     # two strands, half a turn apart
        x, z = round(11 * math.cos(t + phase)), round(11 * math.sin(t + phase))
        fill(s, Shape.sphere(x, y, z, 2), Brush.solid(bead))
    if i % 11 == 0:                                  # a base-pair rung across the axis
        ax, az = round(11 * math.cos(t)), round(11 * math.sin(t))
        fill(s, Shape.cylinder_between(ax, y, az, -ax, y, -az, 1),
             Brush.solid("minecraft:white_concrete"))
```

## Trefoil knot

A parametric knot stamped as a fat tube of overlapping spheres, hue running
along its length so the over-and-under reads at a glance.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-knot.png" width="460" alt="A trefoil knot rendered as a thick rainbow tube of blocks">
</div>

```python
pal = Palette.concrete()
for i in range(480):
    t = i / 480 * 2 * math.pi
    x = 11 * (math.sin(t) + 2 * math.sin(2 * t))
    y = 11 * (math.cos(t) - 2 * math.cos(2 * t))
    z = 11 * (-math.sin(3 * t))
    BuildingTool.fill(s, Shape.sphere(round(x), round(y), round(z), 3),
                      Brush.solid(pal.closest_block(*hsv(i / 480))))
```

## Menger sponge

The classic recursive fractal, four levels deep. Every cell is kept or carved by
the same `mod 3` test, and the whole 81³ volume is one triple loop.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-menger.png" width="520" alt="A level-4 Menger sponge fractal in blue and white blocks">
</div>

```python
def in_menger(x, y, z, level):
    for _ in range(level):
        if [x % 3, y % 3, z % 3].count(1) >= 2:      # carve the mod-3 centers
            return False
        x, y, z = x // 3, y // 3, z // 3
    return True

pal = flat_art_palette().dithered()              # a rich palette, dithered for smooth ramps
for x in range(81):
    for y in range(81):
        for z in range(81):
            if in_menger(x, y, z, 4):
                s.set_block(x, y, z, pal.closest_block_dithered(*teal_to_ice(y / 81), x, y, z))
```

## Fractal tree

A recursive grower: each branch is a tapering `cylinder_between`, splitting into
three tilted children until the tips bloom into autumn foliage banded by height.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-tree.png" width="440" alt="A voxel fractal tree with a full autumn canopy fading from green to red">
</div>

```python
def grow(p, d, length, radius, depth, twist):
    q = p + d * length
    fill(s, Shape.cylinder_between(*p, *q, max(1, radius)), Brush.solid("minecraft:spruce_log"))
    if depth == 0:
        fill(s, Shape.sphere(*q, 4), Brush.solid(autumn(q.y)))      # foliage at the tip
        return
    for k in range(3):                                             # three tilted children
        child = rotate(rotate(d, perp(d), radians(34)), d, twist + k * tau / 3)
        grow(q, child, length * 0.74, radius * 0.68, depth - 1, twist + 0.7)

grow(origin, up, length=15, radius=3, depth=7, twist=0.4)
```

## Gyroid

The gyroid is a triply-periodic minimal surface: the set where
`sin·cos + sin·cos + sin·cos` crosses zero. Keep the voxels near that zero and
you get its endless interlocking labyrinth, here tinted iridescent along the
cube's diagonal.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-gyroid.png" width="520" alt="A gyroid minimal surface in blocks, an interlocking labyrinth tinted rainbow along its diagonal">
</div>

```python
n, k = 64, 2 * math.pi / 16                 # a 16-block period
for x in range(n):
    for y in range(n):
        for z in range(n):
            f = (math.sin(x*k) * math.cos(y*k) + math.sin(y*k) * math.cos(z*k)
                 + math.sin(z*k) * math.cos(x*k))
            if abs(f) < 0.55:               # thicken the zero-surface into a shell
                s.set_block(x, y, z, pal.closest_block(*hsv((x + y + z) / (2 * n))))
```

## Mandelbulb

The power-8 Mandelbulb, escape-tested per voxel: raise each point to the eighth
power in spherical coordinates, add its own position, and keep the ones that
never fly off to infinity. Tinted like an ember.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-mandelbulb.png" width="520" alt="A Mandelbulb 3D fractal in blocks with an ember-red to gold gradient">
</div>

```python
for cx, cy, cz in grid(n=132, span=2.5):        # c = each voxel's position
    x = y = z = 0.0
    for _ in range(7):
        r = math.sqrt(x*x + y*y + z*z)
        if r > 2.0:
            break                                # escaped, so outside
        t, p = 8 * math.acos(z / r), 8 * math.atan2(y, x)     # z -> z^8 ...
        rp = r ** 8
        x, y, z = (rp*math.sin(t)*math.cos(p) + cx,
                   rp*math.sin(t)*math.sin(p) + cy,
                   rp*math.cos(t) + cz)          # ... + c
    else:
        s.set_block(gx, gy, gz, pal.closest_block_dithered(*ember(gy), gx, gy, gz))
```

## Voxelized fox

Any GLB or OBJ becomes a `Shape`, and `schematic_from_glb_textured` projects its
texture onto the blocks. Here the Khronos low-poly Fox turns into a chunky
character, orange fur, white tail-tip, and dark legs all carried by the texture.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-fox.png" width="520" alt="The Khronos low-poly Fox model voxelized, its texture giving it orange fur, a white-tipped tail, and dark legs">
</div>

```python
glb = open("Fox.glb", "rb").read()
fox = Voxelizer.schematic_from_glb_textured(glb, 84.0, 0.7, Palette.solid(), "fox")
```

## Supershape

The superformula draws a closed profile with any number of lobes; a spherical
product of two profiles sweeps it into a 3D solid. This one is seven-lobed and
colored by longitude.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-supershape.png" width="520" alt="A seven-lobed 3D supershape in rainbow blocks, like a mathematical starburst">
</div>

```python
def superformula(a, m, n1, n2, n3):
    return (abs(math.cos(m*a/4))**n2 + abs(math.sin(m*a/4))**n3) ** (-1/n1)

for theta in linspace(-pi, pi, 200):
    r1 = superformula(theta, 7, 0.2, 1.7, 1.7)
    for phi in linspace(-pi/2, pi/2, 100):
        r2 = superformula(phi, 7, 0.2, 1.7, 1.7)          # spherical product
        x = 26 * r1 * math.cos(theta) * r2 * math.cos(phi)
        y = 26 * r2 * math.sin(phi)
        z = 26 * r1 * math.sin(theta) * r2 * math.cos(phi)
        fill(s, Shape.sphere(round(x), round(y), round(z), 1),
             Brush.solid(pal.closest_block(*hsv((theta + pi) / (2 * pi)))))
```

## Wave interference

Two circular sources rippling across a heightfield, animated. Each column rises
to the summed wave and takes the color of its crest, so the interference pattern
reads straight off the surface.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-wave.gif" width="560" alt="Two interfering circular wave sources rippling across a blocky heightfield, animated">
</div>

```python
for frame in range(40):
    ph = frame / 40 * tau
    for gx in range(88):
        for gz in range(88):
            h = (math.sin(dist(gx, gz, src_a) * 0.5 - 2 * ph)
                 + math.sin(dist(gx, gz, src_b) * 0.5 - 2 * ph))
            s.fill_cuboid(gx, 0, gz, gx, round(10 + h * 4), gz, blue_to_white(h))
    frames.append(render(s))       # fixed camera
```

## Type in blocks

Draw a word to an image, then extrude every letter pixel into a short prism and
sweep a rainbow across it. Any font, any word.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-logo.png" width="760" alt="The word NUCLEATION set in 3D extruded blocks with a rainbow gradient across the letters">
</div>

```python
img = drawtext("NUCLEATION", font="Impact")     # ffmpeg draws the text
w, h, px = image_pixels(img, 230)
for y in range(h):
    for x in range(w):
        if px[y][x] > 128:                       # a letter pixel
            for d in range(7):                   # extrude toward the camera
                s.set_block(x, h - 1 - y, d, pal.closest_block(*hsv(x / w)))
```

---

That's ten. Every one is a few dozen lines that lean on the same handful of
primitives: shapes, an SDF string, a brush, a palette, a voxelizer. Pick one,
change a number, and it's yours. Back to the [README](../README.md).
