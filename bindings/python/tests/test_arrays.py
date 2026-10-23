import nucleation as n
import pytest

np = pytest.importorskip("numpy")


def test_set_blocks_array_round_trips_through_region_array():
    s = n.Schematic.create("arrays")
    rng = np.random.default_rng(7)
    positions = rng.integers(-5, 20, size=(500, 3))
    palette = ["minecraft:stone", "minecraft:oak_log[axis=x]", "minecraft:air"]
    indices = rng.integers(0, len(palette), size=len(positions))
    assert n.set_blocks_array(s, positions, palette, indices) == len(positions)

    blocks, region_palette, origin = n.region_array(s)
    expected = {}
    for p, i in zip(map(tuple, positions), indices):
        expected[p] = palette[i]  # later positions win
    for (x, y, z), block in expected.items():
        got = region_palette[blocks[x - origin[0], y - origin[1], z - origin[2]]]
        assert got == block

    assert n.set_blocks_array(s, positions[:3], "minecraft:gold_block") == 3
    assert s.get_block_string(*map(int, positions[0])) == "minecraft:gold_block"
