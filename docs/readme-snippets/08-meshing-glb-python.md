# Meshing to GLB (Python)

```python
import base64
from nucleation import Schematic, ResourcePack, MeshConfig, MeshResult

pack = ResourcePack.from_bytes(open("pack.zip", "rb").read())  # any vanilla-format resource pack
schem = Schematic.load_from_file("simple_cube.litematic")
mesh = MeshResult.create(schem, pack, MeshConfig.create())

glb = base64.b64decode(mesh.glb_data_b64())
print("vertices:", mesh.vertex_count(), "triangles:", mesh.triangle_count())
print("GLB bytes:", len(glb), "magic:", glb[:4])
```

Output:

```text
vertices: 216 triangles: 108
GLB bytes: 9848 magic: b'glTF'
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64; pack.zip is a vanilla-format resource pack (the schemati pack, 3312 textures)._
