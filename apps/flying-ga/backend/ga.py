"""Genetic-algorithm core for the flying-machine lab.

Genome: one state index per cell of the configured bbox (default 5x3x3).
Alphabet: air, slime, honey, sticky_piston x6 facings, observer x6 facings.
Evaluation: build a gametest-style structure SNBT (machine at x-offset 1 in a
corridor sized bbox + [26, 2, 2]), quiet-settle it in nucleation's
TickSimulation, kick it with the redstone-block protocol pinned by
crates/mc-tick/tests/cases/flying_machine.test.json (place
minecraft:redstone_block beside a sticky piston at tick 2, remove at tick 4 —
observers cannot see placed blocks in this engine build, so redstone-block
kicks are the only reliable starter), run eval_ticks, and score +x travel.

Fitness = center-of-mass displacement along +x of all non-air blocks
(moving_piston / piston_head count as blocks), minus 0.3 per "left behind"
block. Left-behind heuristic: after the run, any block whose x is still below
start_min_x + displacement/2 didn't come along for the ride (the machine
itself has fully crossed that line once it has travelled its own length);
each such block costs 0.3 fitness so debris-shedding "machines" rank below
clean fliers of the same displacement. Only applied when displacement > 0.5
so resting builds aren't penalised for existing. Sim exceptions, pistonless
genomes, and empty end-states all score 0.
"""

from __future__ import annotations

import json
import random

# ---------------------------------------------------------------- alphabet

AIR = 0

ALPHABET: list[str] = [
    "minecraft:air",
    "minecraft:slime_block",
    "minecraft:honey_block",
    "minecraft:sticky_piston[extended=false,facing=east]",
    "minecraft:sticky_piston[extended=false,facing=west]",
    "minecraft:sticky_piston[extended=false,facing=north]",
    "minecraft:sticky_piston[extended=false,facing=south]",
    "minecraft:sticky_piston[extended=false,facing=up]",
    "minecraft:sticky_piston[extended=false,facing=down]",
    "minecraft:observer[facing=east,powered=false]",
    "minecraft:observer[facing=west,powered=false]",
    "minecraft:observer[facing=north,powered=false]",
    "minecraft:observer[facing=south,powered=false]",
    "minecraft:observer[facing=up,powered=false]",
    "minecraft:observer[facing=down,powered=false]",
]

PISTON_EAST = 3
PISTON_IDXS = {3, 4, 5, 6, 7, 8}

MAX_BLOCKS = 14  # headroom under the 12-push limit per piston

# Everything place_block may write, pre-interned at construction.
EXTRA_STATES = ";".join(["minecraft:redstone_block"] + ALPHABET[1:])

# ------------------------------------------------------------------ genome

Genome = tuple[int, ...]  # len == bx*by*bz, indices into ALPHABET


def cell_index(x: int, y: int, z: int, bbox: tuple[int, int, int]) -> int:
    bx, by, bz = bbox
    return (y * bz + z) * bx + x


def genome_blocks(genome: Genome, bbox: tuple[int, int, int]):
    """Yield (x, y, z, state_idx) for non-air cells."""
    bx, by, bz = bbox
    for y in range(by):
        for z in range(bz):
            for x in range(bx):
                s = genome[cell_index(x, y, z, bbox)]
                if s != AIR:
                    yield x, y, z, s


def block_count(genome: Genome) -> int:
    return sum(1 for s in genome if s != AIR)


def cap_blocks(genome: list[int], rng: random.Random) -> list[int]:
    """Turn random non-air cells to air until <= MAX_BLOCKS remain."""
    filled = [i for i, s in enumerate(genome) if s != AIR]
    while len(filled) > MAX_BLOCKS:
        i = filled.pop(rng.randrange(len(filled)))
        genome[i] = AIR
    return genome


# Engine B (verified this session): travels +x at 1 block / 10 ticks.
ENGINE_B = [
    (1, 0, 0, 10),  # observer[facing=west]
    (2, 0, 0, 1),   # slime_block
    (3, 0, 0, 4),   # sticky_piston[facing=west]
    (2, 0, 1, 3),   # sticky_piston[facing=east]
    (3, 0, 1, 1),   # slime_block
    (4, 0, 1, 9),   # observer[facing=east]
]


def engine_b_genome(bbox: tuple[int, int, int], mirror_z: bool) -> Genome | None:
    bx, by, bz = bbox
    if bx < 5 or bz < 2:
        return None
    g = [AIR] * (bx * by * bz)
    for x, y, z, s in ENGINE_B:
        zz = 1 - z if mirror_z else z
        g[cell_index(x, y, zz, bbox)] = s
    return tuple(g)


def random_genome(bbox: tuple[int, int, int], rng: random.Random) -> Genome:
    bx, by, bz = bbox
    n = bx * by * bz
    density = min(0.9, 8.0 / n)  # ~8 blocks expected
    g = [
        rng.randrange(1, len(ALPHABET)) if rng.random() < density else AIR
        for _ in range(n)
    ]
    return tuple(cap_blocks(g, rng))


def mutate(genome: Genome, rate: float, rng: random.Random) -> Genome:
    g = list(genome)
    for i in range(len(g)):
        if rng.random() < rate:
            # 40% of mutations clear the cell; the rest pick any state.
            g[i] = AIR if rng.random() < 0.4 else rng.randrange(1, len(ALPHABET))
    return tuple(cap_blocks(g, rng))


def crossover(a: Genome, b: Genome, rng: random.Random) -> Genome:
    g = [ai if rng.random() < 0.5 else bi for ai, bi in zip(a, b)]
    return tuple(cap_blocks(g, rng))


def tournament(pop, fits, rng: random.Random, k: int = 3) -> Genome:
    best = max(rng.sample(range(len(pop)), k), key=lambda i: fits[i])
    return pop[best]


# -------------------------------------------------------------------- SNBT

X_OFF = 1  # machine sits at x-offset 1 in the corridor


def genome_to_snbt(genome: Genome, bbox: tuple[int, int, int]) -> str:
    """Structure SNBT shaped like flying_machine_east.snbt: machine at
    x-offset 1, corridor of air sized bbox + [26, 2, 2] for travel room."""
    bx, by, bz = bbox
    size = (bx + 26, by + 2, bz + 2)
    palette: list[str] = []
    pal_of: dict[int, int] = {}
    blocks = []
    for x, y, z, s in genome_blocks(genome, bbox):
        if s not in pal_of:
            pal_of[s] = len(palette)
            palette.append(ALPHABET[s])
        blocks.append(f"{{pos: [{x + X_OFF}, {y}, {z}], state: {pal_of[s]}}}")

    def pal_entry(state: str) -> str:
        if "[" not in state:
            return f'{{Name: "{state}"}}'
        name, props = state[:-1].split("[", 1)
        kv = ", ".join(
            f'{k}: "{v}"' for k, v in (p.split("=", 1) for p in props.split(","))
        )
        return f'{{Name: "{name}", Properties: {{{kv}}}}}'

    return (
        "{\n  DataVersion: 4903,\n"
        f"  size: [{size[0]}, {size[1]}, {size[2]}],\n"
        "  palette: [" + ", ".join(pal_entry(p) for p in palette) + "],\n"
        "  blocks: [" + ", ".join(blocks) + "],\n"
        "  entities: []\n}"
    )


def kick_pos(genome: Genome, bbox: tuple[int, int, int]) -> tuple[int, int, int] | None:
    """Structure-space cell to drop the redstone block into: beside the first
    east-facing sticky piston, else beside any piston. Must be air."""
    cells = list(genome_blocks(genome, bbox))
    occupied = {(x + X_OFF, y, z) for x, y, z, _ in cells}
    pistons = [c for c in cells if c[3] == PISTON_EAST] or [
        c for c in cells if c[3] in PISTON_IDXS
    ]
    for x, y, z, _ in pistons:
        sx = x + X_OFF
        # Above first (the reference kick), then sideways in z, then behind.
        for cand in ((sx, y + 1, z), (sx, y, z + 1), (sx, y, z - 1), (sx - 1, y, z)):
            if cand not in occupied and cand[1] >= 0 and cand[2] >= 0:
                return cand
    return None


# -------------------------------------------------------------- evaluation

_nucleation = None


def _engine():
    global _nucleation
    if _nucleation is None:
        import nucleation

        _nucleation = nucleation
    return _nucleation


def evaluate(args) -> float:
    """(genome, bbox, eval_ticks, seed) -> fitness. Top-level for mp.Pool."""
    genome, bbox, eval_ticks, seed = args
    try:
        return _evaluate(tuple(genome), tuple(bbox), eval_ticks, seed)
    except Exception:
        return 0.0


def _evaluate(genome: Genome, bbox, eval_ticks: int, seed: int) -> float:
    if block_count(genome) == 0:
        return 0.0
    kick = kick_pos(genome, bbox)
    if kick is None:
        return 0.0  # no piston, nothing to kick

    nuc = _engine()
    sim = nuc.TickSimulation.from_snbt(
        genome_to_snbt(genome, bbox), nuc.TickSettleMode.Quiet, 0, 0, 0, EXTRA_STATES
    )
    sim.set_rng_seed(seed)

    start = _solid_blocks(sim)
    if not start:
        return 0.0
    start_com_x = sum(x for x, _, _ in start) / len(start)
    start_min_x = min(x for x, _, _ in start)

    # Kick protocol: quiet settle happened at construction; redstone block
    # beside the piston at tick 2, removed at tick 4.
    kx, ky, kz = kick
    sim.run(2)
    sim.place_block(kx, ky, kz, "minecraft:redstone_block")
    sim.run(2)
    sim.place_block(kx, ky, kz, "minecraft:air")
    if eval_ticks > 4:
        sim.run(eval_ticks - 4)

    end = _solid_blocks(sim)
    if not end:
        return 0.0
    disp = sum(x for x, _, _ in end) / len(end) - start_com_x

    penalty = 0.0
    if disp > 0.5:
        left_behind = sum(1 for x, _, _ in end if x < start_min_x + disp / 2)
        penalty = 0.3 * left_behind
    return max(0.0, disp - penalty)


def _solid_blocks(sim) -> list[tuple[int, int, int]]:
    out = []
    for b in json.loads(sim.world_snapshot_json()):
        s = b["state"]
        if s.startswith("minecraft:air") or s.startswith("minecraft:redstone_block"):
            continue
        x, y, z = b["pos"]
        out.append((x, y, z))
    return out
