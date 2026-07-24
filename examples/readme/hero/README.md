# README hero

This generator rebuilds every frame of the root README's scorched 3x7 knot as a
separate Minecraft schematic, then renders the 120-frame loop with a fixed
camera.

The knot's corrugation phase animates its braided geometry. A periodic cellular
field is evaluated in intrinsic curve coordinates—arc length and angle around
the tube—so the raised plate texture travels with the strands rather than
through world space.

## Generate

Build/install the current Python binding with the `bridge-full` feature, install
NumPy, and provide a Java resource-pack ZIP:

```bash
uv pip install numpy
NUCLEATION_PACK=/path/to/pack.zip python examples/readme/hero/generate_intrinsic_animation.py --workers 3
```

Generated frames, schematics, manifests, poster, and GIF are written under the
ignored `render_work/scorched-3x7-intrinsic/` tree. The excluded endpoint is
generated separately and must have the same canonical block-map hash as frame
zero.

`generate_intrinsic_approval_frame.py` generates only the phase-zero poster and
schematic for fast composition checks.
