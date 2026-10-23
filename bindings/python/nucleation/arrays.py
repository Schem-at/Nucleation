"""NumPy in and out of a schematic in a few calls instead of one per block.

Each bridge call costs a few hundred nanoseconds, many times the work of
placing one block, so bulk edits should go through these (or the batch
methods they wrap: ``Schematic.set_blocks`` / ``set_blocks_indexed``).
"""

import json

_WINDOW = 65_536  # cells per region_block_indices call


def set_blocks_array(schematic, positions, palette, indices=None):
    """Place blocks at ``positions`` (an ``(N, 3)`` int array).

    ``palette`` is one block string for every position, or a list of block
    strings with ``indices`` (``N`` ints) choosing one per position. Later
    positions win over earlier ones. Returns the number of blocks set.
    """
    import numpy as np

    flat = np.ascontiguousarray(positions, dtype=np.int32).reshape(-1)
    if isinstance(palette, str):
        return schematic.set_blocks(flat, palette)
    indices = np.ascontiguousarray(indices, dtype=np.uint32).reshape(-1)
    return schematic.set_blocks_indexed(flat, indices, json.dumps(list(palette)))


def region_array(schematic, region="Main"):
    """A region's blocks as ``(indices, palette, origin)``.

    ``indices[x, y, z]`` indexes ``palette`` (block strings) for the block at
    ``origin + (x, y, z)``. The array covers the region's allocated storage,
    so it can extend past the placed blocks (as air).
    """
    import numpy as np

    info = next(r for r in json.loads(schematic.render_regions_json()) if r["name"] == region)
    sx, sy, sz = info["size"]
    flat = np.empty(info["length"], dtype=np.uint32)
    for start in range(0, len(flat), _WINDOW):
        window = schematic.region_block_indices(region, start, min(_WINDOW, len(flat) - start))
        flat[start : start + len(window)] = window
    palette = [_block_string(b) for b in info["palette"]]
    # Storage is x fastest, then z, then y.
    return flat.reshape(sy, sz, sx).transpose(2, 0, 1), palette, tuple(info["min"])


def _block_string(block):
    props = block.get("properties") or []
    pairs = props.items() if isinstance(props, dict) else props
    inner = ",".join(f"{k}={v}" for k, v in pairs)
    return f"{block['name']}[{inner}]" if inner else block["name"]
