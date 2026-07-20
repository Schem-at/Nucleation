# Regions, transforms, and stamping

## Regions, transforms, and stamping


A schematic is multi-region in the Litematica sense: many named sub-volumes,
each with its own palette and bounds, and both whole builds and single regions
transform in place. Here a keep and two wings are three separate named regions;
`rotate_region_y` turns the copper wing 90° and leaves the keep and the
prismarine wing exactly where they were:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/regions.png" width="760" alt="Before and after: a quartz keep with copper and prismarine wings as three named regions, with the copper wing rotated 90 degrees in place">

```python
# Address independent named regions in one schematic:
s.set_block_in_region("keep",  0, 0, 0, "minecraft:quartz_block")
s.set_block_in_region("gate", 10, 0, 0, "minecraft:blackstone")
s.region_names_json()                 # ["Main", "keep", "gate"]
s.rotate_region_y("gate", 90)         # turn one region, leave the rest

# Transform the whole build with rotate_x/y/z (degrees) and flip_x/y/z:
s.rotate_y(90)                        # a bar's +x tip at (9,0,0) lands at (0,0,0)

# Stamp a sub-volume of one schematic into another:
dst.copy_region(src, 0, 0, 0,  9, 0, 0,   100, 0, 0,  "[]")
#               source   min-corner    max-corner    target    exclude
```

Those two operations compose into symmetry. This mandala is one asymmetric
petal, built once, then `flip_x` and `flip_z` mirror four `copy_region` stamps
into the quadrants of a canvas so the domes meet at the center:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/transforms.png" width="520" alt="A four-fold symmetric mandala built by mirroring one petal into each quadrant">
</div>
