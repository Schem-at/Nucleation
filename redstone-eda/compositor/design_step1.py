"""Step 1 of the Design API, built bottom-up: place the endpoint hardware
with nothing but the plain Schematic primitives.

Two 8-bit input banks (levers) on perpendicular faces, two 8-bit output banks
(lamps), each endpoint with its single CONNECTION DUST -- the cell a router
will later land on, and the cell an IO port will later name.

Geometry facts in use (all sim-verified earlier):
  * a floor lever sits on a support block and powers adjacent dust
  * a lamp lights when the dust ON TOP of it is powered (dust weakly powers
    the block beneath) -- so the lamp IS the support of its connection dust
  * bits stack vertically at 2y pitch with solid separators (bus8 v2 form)
"""
import nucleation as n

STONE = "minecraft:stone"
DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]"
LAMP = "minecraft:redstone_lamp[lit=false]"
LEVER = "minecraft:lever[face=floor,facing=north,powered=false]"

s = n.Schematic.create("bus_cross_endpoints")

N, Y0, STEP = 8, 2, 2


def lever_bank(x, z, dust_dx, dust_dz):
    """8 levers stacked at 2y pitch; returns the connection-dust cells."""
    cells = []
    for i in range(N):
        y = Y0 + STEP * i
        s.set_block_from_string(x, y - 1, z, STONE)              # lever support
        s.set_block_from_string(x, y, z, LEVER)
        dx, dz = x + dust_dx, z + dust_dz
        s.set_block_from_string(dx, y - 1, dz, STONE)            # dust support
        s.set_block_from_string(dx, y, dz, DUST)                 # connection dust
        cells.append((dx, y, dz))
    return cells


def lamp_bank(x, z):
    """8 lamps stacked at 2y pitch, each lamp supporting its own dust."""
    cells = []
    for i in range(N):
        y = Y0 + STEP * i
        s.set_block_from_string(x, y - 1, z, LAMP)               # the lamp itself
        s.set_block_from_string(x, y, z, DUST)                   # connection dust ON the lamp
        cells.append((x, y, z))
    return cells


# input a: west face, bits toward +x   |  input b: north face, bits toward +z
in_a = lever_bank(0, 8, dust_dx=+1, dust_dz=0)
in_b = lever_bank(8, 0, dust_dx=0, dust_dz=+1)
# outputs on the opposite faces
out_a = lamp_bank(47, 8)
out_b = lamp_bank(8, 47)

print("placed: %d blocks" % s.block_count())
print("in_a  connection dust:", in_a[0], "..", in_a[-1])
print("in_b  connection dust:", in_b[0], "..", in_b[-1])
print("out_a connection dust:", out_a[0], "..", out_a[-1])
print("out_b connection dust:", out_b[0], "..", out_b[-1])

# sanity in the sim: flip bit 0 and bit 7 of `a`, see their dust light up
EXTRA = ";".join(["minecraft:lever[face=floor,facing=north,powered=%s]" % p
                  for p in ("true", "false")]
                 + ["minecraft:redstone_lamp[lit=true]", "minecraft:redstone_lamp[lit=false]"])
sim = n.TickSimulation.from_schematic(s, n.TickSettleMode.Placement, 0, 0, 0, EXTRA)
sim.run_until_quiescent(100)
bmin = s.tight_bounds_min()
ox, oy, oz = bmin.x, bmin.y, bmin.z

def sp(c):  # sim coords are bbox-relative
    return (c[0] - ox, c[1] - oy, c[2] - oz)

for bit in (0, 7):
    lx, ly, lz = 0, Y0 + STEP * bit, 8
    sim.use_block(*sp((lx, ly, lz)))
sim.run_until_quiescent(100)
for bit in (0, 1, 7):
    st = sim.get_block(*sp(in_a[bit]))
    print("a bit %d connection dust: %s" % (bit, st.split("power=")[1][:2]))

s.save("compositor/step1_endpoints.schem")
print("saved step1_endpoints.schem")
