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
