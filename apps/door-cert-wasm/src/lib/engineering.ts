/** Engineering insights — the measurements that need the update log.
 *
 * Everything in `aperture.ts` and the timing code answers "what does this door
 * DO". This module answers "what does it COST, and what of it is doing any
 * work" — questions that need the engine's recorded update stream, not just the
 * block-change log. That stream exists because the engine's update ORDER is
 * verified against real Minecraft; without that, none of these numbers would
 * mean anything to compare between doors.
 *
 * Four of the five readings here are cheap set arithmetic over data the
 * certificate already carries. The fifth — the first-movement trace — is
 * deliberately weaker than it looks, and the type comments say exactly how.
 *
 * ------------------------------------------------------------- honesty ----
 *
 * Two claims this module refuses to make:
 *
 *  - **It does not predict TPS.** `ServerCost.updates` is a count of update
 *    dispatches the engine delivered over one cycle. A dispatch is not a
 *    millisecond; a dust update and a block-entity tick cost wildly different
 *    amounts of real server time. The count is comparable BETWEEN doors —
 *    same engine, same cycle definition, same seed — and that is all it is for.
 *
 *  - **It does not claim causality.** The update log is ordered, not
 *    parented: the engine records the sequence in which updates were
 *    delivered, never which update caused which. So `FirstMovement` reports
 *    the ordered sequence of components touched before the first block moved,
 *    and is named for that rather than for a critical path it cannot prove.
 */

import type {
  ApertureGeometry,
  Badges,
  Census,
  DeadWeight,
  Engineering,
  FirstMovement,
  InputControl,
  ReplayBlock,
  ReplayChange,
  ServerCost,
  Symmetry,
  Vec3,
} from "./types";
import type { XrayData } from "./xray";

const baseName = (state: string) => state.split("[", 1)[0];
const shortName = (state: string) => baseName(state).replace(/^minecraft:/, "");
const isAir = (state: string) => baseName(state).endsWith("air");
const posKey = (p: Vec3) => `${p[0]},${p[1]},${p[2]}`;

/** A change that moved MASS, as opposed to one that only re-powered a cell.
 *
 *  Shared with the worker's peak-in-flight counter so the two cannot drift:
 *  a cell whose contents changed is mass in flight, a cell that merely
 *  re-powered is the signal racing ahead of it. Piston bodies count as
 *  movement — `extended=false -> true` is the same block name but is the
 *  stroke itself. */
export function isMovement(c: ReplayChange): boolean {
  const from = baseName(c.from);
  const to = baseName(c.to);
  if (from !== to) return true;
  return from.endsWith("piston") || from.endsWith("piston_head");
}

/** True for a change that is a piston STARTING to extend. Retractions and
 *  re-powers are not fires; only the outward stroke is. */
function isPistonFire(c: ReplayChange): boolean {
  const n = shortName(c.to);
  if (n !== "piston" && n !== "sticky_piston") return false;
  return c.to.includes("extended=true") && !c.from.includes("extended=true");
}

/* ------------------------------------------------------------ server cost -- */

/** Total dispatches, the block-events share, the per-tick peak, and the
 *  normalisations that let doors of different sizes be compared.
 *
 *  Every number is read off the same heat view the x-ray draws, so the panel
 *  and the sheet are two readings of one recording. */
export function serverCost(
  xray: XrayData,
  opts: {
    passageCells: number | null;
    movedCells: number;
    cycleTicks: number | null;
  },
): ServerCost {
  const P = xray.phases.length;
  const byPhase = new Array<number>(P).fill(0);
  let peak = 0;
  let peakTick = 0;
  for (const t of xray.ticks) {
    if (t.total > peak) {
      peak = t.total;
      peakTick = t.tick;
    }
    for (let i = 0; i < t.cell.length; i++)
      for (let p = 0; p < P; p++) byPhase[p] += t.ph[i * P + p];
  }
  const phases = xray.phases
    .map((phase, i) => ({ phase, n: byPhase[i] }))
    .filter((r) => r.n > 0)
    .sort((a, b) => b.n - a.n);
  const events = phases.find((r) => r.phase === "block_events")?.n ?? 0;
  const total = xray.totalUpdates;
  const per = (d: number | null) => (d && d > 0 ? total / d : null);
  return {
    updates: total,
    block_events: events,
    by_phase: phases,
    peak,
    peak_tick: peakTick,
    per_passage_cell: per(opts.passageCells),
    per_moved_cell: per(opts.movedCells),
    // What it costs to hold the door on a loop: one cycle's dispatches over
    // the cycle's own length, at Minecraft's 20 ticks a second.
    per_second: opts.cycleTicks && opts.cycleTicks > 0 ? (total * 20) / opts.cycleTicks : null,
  };
}

/* -------------------------------------------------------------- dead weight -- */

/** Blocks that neither moved nor received a single update across the cycle.
 *
 *  A set difference over two complete logs: the change log names every cell
 *  whose contents or state changed, and the update log names every cell an
 *  update was DELIVERED to. A block in neither did nothing at all this cycle.
 *
 *  "Did nothing" is not "can be removed", and the certificate says so: a block
 *  that only holds another one up is load-bearing precisely because it never
 *  has to do anything. What the number is good for is the opposite reading —
 *  it is an upper bound on how much of the build is decoration or redundancy,
 *  and a builder trying to compact a design knows which cells to interrogate. */
export function deadWeight(
  rest: ReplayBlock[],
  changes: ReplayChange[],
  xray: XrayData | null,
): DeadWeight | null {
  if (!xray) return null;
  const touched = new Set<string>();
  for (const c of changes) touched.add(posKey(c.pos));
  for (let i = 0; i < xray.cells.length; i += 3)
    touched.add(`${xray.cells[i]},${xray.cells[i + 1]},${xray.cells[i + 2]}`);

  const cells: Vec3[] = [];
  const byId = new Map<string, number>();
  let total = 0;
  for (const b of rest) {
    if (isAir(b.state)) continue;
    total++;
    if (touched.has(posKey(b.pos))) continue;
    cells.push(b.pos);
    const id = baseName(b.state);
    byId.set(id, (byId.get(id) ?? 0) + 1);
  }
  // Deterministic order, and a ceiling: the overlay only needs to draw them,
  // and a pathological build should not blow the certificate's storage quota.
  cells.sort((a, b) => a[1] - b[1] || a[0] - b[0] || a[2] - b[2]);
  const CAP = 3000;
  return {
    total,
    idle: cells.length,
    cells: cells.slice(0, CAP),
    truncated: cells.length > CAP,
    by_id: [...byId.entries()]
      .map(([id, count]) => ({ id, count }))
      .sort((a, b) => b.count - a.count || (a.id < b.id ? -1 : 1)),
  };
}

/* ---------------------------------------------------------- first movement -- */

/** Components worth naming in the chain. Everything else an update lands on —
 *  the stone a torch is attached to, the block a piston pushes — is a
 *  neighbour, not a step, and listing it would bury the signal path in wall. */
const COMPONENT: Record<string, string> = {
  lever: "lever",
  note_block: "note block",
  redstone_wire: "dust",
  repeater: "repeater",
  comparator: "comparator",
  redstone_torch: "torch",
  redstone_wall_torch: "torch",
  observer: "observer",
  piston: "piston",
  sticky_piston: "sticky piston",
  piston_head: "piston head",
  moving_piston: "moving piston",
  redstone_block: "redstone block",
  target: "target",
  daylight_detector: "daylight detector",
  dispenser: "dispenser",
  dropper: "dropper",
  tripwire_hook: "tripwire hook",
};

function componentLabel(name: string): string | null {
  if (name in COMPONENT) return COMPONENT[name];
  if (name.endsWith("_button")) return "button";
  if (name.endsWith("_pressure_plate")) return "pressure plate";
  return null;
}

/** The ordered sequence of components the engine touched between the input
 *  click and the first block that moved.
 *
 *  **This is not a causal trace and is not presented as one.** The engine
 *  records update ORDER — tick, sequence, position, kind, phase and the block
 *  state at dispatch time — but not parentage: nothing in the log says which
 *  update scheduled which. A true critical path would need that edge.
 *
 *  What order alone supports is still worth having, because it is still a fact
 *  about this machine that no tool without a verified-order engine can state:
 *  every update the engine delivered before the first block moved, in the
 *  order it delivered them, collapsed to the distinct components they landed
 *  on. A chain of `lever → dust → repeater → piston` really does mean the
 *  signal reached the dust before the repeater and the repeater before the
 *  piston — it just does not mean the dust is why the repeater fired. */
export function firstMovement(
  changes: ReplayChange[],
  xray: XrayData | null,
  input: InputControl | null,
): FirstMovement | null {
  if (!xray || xray.waves.length === 0) return null;
  const first = changes.find(isMovement);
  if (!first) return null;
  const tick = first.tick;
  const onTick = changes.filter((c) => c.tick === tick && isMovement(c));
  // Every cell that moves on that tick — a piston stroke writes its base, its
  // head and the block it carries, and the update that drove it landed on one
  // of them.
  const moving = new Set(onTick.map((c) => posKey(c.pos)));

  // What to NAME as the thing that moved. A stroke writes three cells and two
  // of them read badly: `air -> moving_piston` would report "air moved", and
  // `redstone_block -> moving_piston` names the cargo rather than the machine.
  // The piston BASE going `extended=false -> true` is the stroke itself, so it
  // is preferred whenever one fired on this tick, and the count of pistons
  // that fired with it goes on the chain's last link.
  const fired = onTick.filter(isPistonFire);
  const mover = fired[0] ?? first;
  const moverName = isPistonFire(mover)
    ? shortName(mover.to)
    : shortName(isAir(mover.from) ? mover.to : mover.from);
  const endLabel = componentLabel(moverName);
  const endCells = fired.length > 0 ? new Set(fired.map((c) => posKey(c.pos))).size : 1;

  const order: string[] = [];
  const cellsOf = new Map<string, Set<string>>();
  const note = (label: string, key: string) => {
    let set = cellsOf.get(label);
    if (!set) {
      set = new Set();
      cellsOf.set(label, set);
      order.push(label);
    }
    set.add(key);
  };
  // The click is the known start, so the chain is seeded with it rather than
  // inferred. Left to delivery order alone the control lands mid-chain — a
  // lever's own shape update arrives after the updates it sent its neighbours,
  // which is true and reads as nonsense.
  if (input) note(input.kind, posKey(input.pos));

  let hops = 0;
  let stopped = false;
  for (let t = 0; t <= tick && t < xray.waves.length && !stopped; t++) {
    const w = xray.waves[t];
    for (let i = 0; i < w.n; i++) {
      const ci = w.cell[i] * 3;
      const key = `${xray.cells[ci]},${xray.cells[ci + 1]},${xray.cells[ci + 2]}`;
      // The boundary: within the tick the door moves on, stop at the first
      // update delivered to a cell that moved. Everything after it in that
      // tick is the consequence, not the run-up.
      if (t === tick && moving.has(key)) {
        stopped = true;
        break;
      }
      hops++;
      const label = componentLabel(shortName(w.states[w.state[i]] ?? ""));
      if (label) note(label, key);
    }
  }
  // …and the chain ends where the door does: on the part that moved. It is
  // excluded from `hops` on purpose — the count is what it took to GET there.
  const chain = order.map((id) => ({ id, cells: cellsOf.get(id)!.size }));
  if (endLabel && chain[chain.length - 1]?.id !== endLabel)
    chain.push({ id: endLabel, cells: endCells });

  return {
    hops,
    ticks: tick,
    chain,
    block: moverName.replace(/_/g, " "),
    pos: mover.pos,
  };
}

/* ------------------------------------------------------------------ symmetry -- */

/** Which axis of the passage is the wall's thickness. Same rule as
 *  `doorwayFacts`: the shallowest span, ties broken x → z → y. */
function axesOf(cells: Vec3[]): { depth: number; w: number; h: number } | null {
  if (cells.length === 0) return null;
  const min: Vec3 = [Infinity, Infinity, Infinity];
  const max: Vec3 = [-Infinity, -Infinity, -Infinity];
  for (const p of cells)
    for (let a = 0; a < 3; a++) {
      if (p[a] < min[a]) min[a] = p[a];
      if (p[a] > max[a]) max[a] = p[a];
    }
  const span = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
  const order = [0, 2, 1];
  let depth = order[0];
  for (const a of order) if (span[a] < span[depth]) depth = a;
  const rest = [0, 1, 2].filter((a) => a !== depth);
  const h = rest.includes(1) ? 1 : rest[1];
  const w = rest.find((a) => a !== h) as number;
  return { depth, w, h };
}

/** Mirror symmetry, of the pattern and of the whole machine.
 *
 *  A real signal in the door community and cheap from geometry we already
 *  have — but only under one stated simplification: **the machine test
 *  compares base block names, not block states.** A piston mirrored across
 *  the doorway faces the other way, and demanding `facing=east` match
 *  `facing=west` would report every symmetric door as asymmetric. So the
 *  question asked is "is the same PART in the mirrored cell", which is what
 *  a builder means by a symmetric build. */
export function symmetry(
  rest: ReplayBlock[],
  geometry: ApertureGeometry | null,
): Symmetry {
  // --- the pattern: the door blocks' silhouette in the doorway plane -------
  let pattern: Symmetry["pattern"] = null;
  const axes = geometry ? axesOf(geometry.passage) : null;
  if (geometry && axes) {
    const face = geometry.visible.length > 0 ? geometry.visible : geometry.closed;
    if (face.length > 0) {
      let c0 = Infinity, c1 = -Infinity, r0 = Infinity, r1 = -Infinity;
      const flat = new Set<string>();
      for (const p of face) {
        const c = p[axes.w];
        const r = p[axes.h];
        flat.add(`${c},${r}`);
        if (c < c0) c0 = c;
        if (c > c1) c1 = c;
        if (r < r0) r0 = r;
        if (r > r1) r1 = r;
      }
      // Mirrored inside the PASSAGE bbox, not the silhouette's own: a pattern
      // that hugs one side of its doorway is asymmetric, and measuring it
      // inside its own tight box would call it symmetric.
      let pc0 = Infinity, pc1 = -Infinity, pr0 = Infinity, pr1 = -Infinity;
      for (const p of geometry.passage) {
        const c = p[axes.w];
        const r = p[axes.h];
        if (c < pc0) pc0 = c;
        if (c > pc1) pc1 = c;
        if (r < pr0) pr0 = r;
        if (r > pr1) pr1 = r;
      }
      const mirror = (flipC: boolean) => {
        for (const k of flat) {
          const [c, r] = k.split(",").map(Number);
          const mk = flipC ? `${pc0 + pc1 - c},${r}` : `${c},${pr0 + pr1 - r}`;
          if (!flat.has(mk)) return false;
        }
        return true;
      };
      pattern = { horizontal: mirror(true), vertical: mirror(false) };
    }
  }

  // --- the machine: base names over the build's own bounding box ----------
  const solid = rest.filter((b) => !isAir(b.state));
  const names = new Map<string, string>();
  const min: Vec3 = [Infinity, Infinity, Infinity];
  const max: Vec3 = [-Infinity, -Infinity, -Infinity];
  for (const b of solid) {
    names.set(posKey(b.pos), baseName(b.state));
    for (let a = 0; a < 3; a++) {
      if (b.pos[a] < min[a]) min[a] = b.pos[a];
      if (b.pos[a] > max[a]) max[a] = b.pos[a];
    }
  }
  const AXIS_NAME = ["x", "y", "z"];
  const doorName = (a: number) =>
    !axes
      ? AXIS_NAME[a]
      : a === axes.depth
        ? "front–back"
        : a === axes.h
          ? "top–bottom"
          : "left–right";
  // A bare yes/no is a dud here and the first run proved it: all three test
  // doors answered "no" on all three axes, which is true and says nothing. A
  // real door is symmetric APART FROM its control and the wiring that feeds
  // it, so the number worth reporting is how much of the build has a mirrored
  // twin — 94% names a symmetric machine with a lever on one side, and 40%
  // names one that genuinely is not.
  const machine = [0, 1, 2].map((a) => {
    let matched = 0;
    for (const [k, id] of names) {
      const p = k.split(",").map(Number) as Vec3;
      const q: Vec3 = [p[0], p[1], p[2]];
      q[a] = min[a] + max[a] - p[a];
      if (names.get(posKey(q)) === id) matched++;
    }
    return {
      axis: doorName(a),
      mirror: solid.length > 0 && matched === solid.length,
      share: solid.length > 0 ? matched / solid.length : 0,
    };
  });
  return { pattern, machine };
}

/* -------------------------------------------------------------------- badges -- */

/** The longest run of CONSECUTIVE equal gaps between fires, and the gap it
 *  runs at.
 *
 *  Not the modal gap. The flight-loop detector this is modelled on can use a
 *  modal gap because a flying machine repeats forever; a door's piston series
 *  is two short bursts with a long wait between them, and the modal gap
 *  cheerfully reports a period from that. The first run of this detector proved
 *  it: a 6 × 6 sliding door whose pistons fire twice per stroke has gaps
 *  [3, 12, 3], the mode is 3 at 67% agreement, and the door was labelled "runs
 *  a tape of period 3". It does not run a tape. A CONSECUTIVE run is the thing
 *  a tape actually produces and a two-burst door never does. */
function longestRun(ticks: number[]): { gap: number; run: number } | null {
  if (ticks.length < 2) return null;
  const gaps = ticks.slice(1).map((t, i) => t - ticks[i]);
  let best = { gap: gaps[0], run: 1 };
  let run = 1;
  for (let i = 1; i < gaps.length; i++) {
    run = gaps[i] === gaps[i - 1] ? run + 1 : 1;
    if (run > best.run) best = { gap: gaps[i], run };
  }
  return best;
}

/** A piston is running a tape when it fires repeatedly AT A STEADY PERIOD.
 *  Four fires with three equal gaps between them is the floor: it is one more
 *  repetition than the "extends twice per stroke" pattern that a plain
 *  sequential door produces, and it is what separates a loop from a sequence. */
const TAPE_FIRES = 4;
const TAPE_RUN = 3;

/** The four qualifiers pro door makers compete on.
 *
 *  The first three are set membership on the census, with one trap: **slimeless
 *  counts honey too.** Honey is the standard substitute for slime and a door
 *  that swaps one for the other has not earned the tag; checking `slime_block`
 *  alone would hand it out for free.
 *
 *  Cycle-less is the one that needs measuring, and TWO obvious tests are both
 *  wrong. Looking for a periodic beat in the whole piston-event series flags
 *  every sequential door whose stages happen to fire on an even cadence.
 *  Counting per-piston fires alone is not enough either — a sliding door
 *  regularly extends the same piston twice per stroke, which is repetition
 *  without a loop, and the first run of this detector duly labelled a 6 × 6
 *  sliding door "runs a tape of period 3" on exactly that.
 *
 *  What a tape actually produces is one piston firing repeatedly at a STEADY
 *  period, so both conditions are demanded of the same piston: at least
 *  `TAPE_FIRES` fires, with a run of `TAPE_RUN` consecutive equal gaps between
 *  them. A door that never clears that bar is cycle-less, and the badge says
 *  how close it came so the threshold is inspectable rather than magic. */
export function badges(census: Census, changes: ReplayChange[]): Badges {
  const fires = new Map<string, number[]>();
  for (const c of changes) {
    if (!isPistonFire(c)) continue;
    const k = posKey(c.pos);
    const arr = fires.get(k);
    if (arr) arr.push(c.tick);
    else fires.set(k, [c.tick]);
  }
  let busiest: number[] = [];
  let repeats = 0;
  let period: number | null = null;
  for (const arr of fires.values()) {
    const ticks = [...arr].sort((a, b) => a - b);
    const run = longestRun(ticks);
    if (ticks.length >= TAPE_FIRES && run && run.run >= TAPE_RUN) {
      repeats++;
      if (period === null) period = run.gap;
    }
    if (ticks.length > busiest.length) busiest = ticks;
  }
  return {
    observerless: census.observer === 0,
    dustless: census.redstone_wire === 0,
    slimeless: census.slime_block + census.honey_block === 0,
    cycleless: repeats === 0,
    tape: repeats > 0 ? { pistons: repeats, fires: busiest.length, period } : null,
    pistons: fires.size,
    /** The busiest piston's own fire count, tape or not — the evidence the
     *  badge was decided on. */
    busiest: busiest.length,
  };
}

/* --------------------------------------------------------------------- all -- */

export function engineering(args: {
  rest: ReplayBlock[];
  changes: ReplayChange[];
  xray: XrayData | null;
  geometry: ApertureGeometry | null;
  census: Census;
  movedCells: number;
  cycleTicks: number | null;
  input: InputControl | null;
}): Engineering {
  const { rest, changes, xray, geometry, census, movedCells, cycleTicks, input } = args;
  return {
    cost: xray
      ? serverCost(xray, {
          passageCells: geometry?.passage.length ?? null,
          movedCells,
          cycleTicks,
        })
      : null,
    dead: deadWeight(rest, changes, xray),
    first: firstMovement(changes, xray, input),
    symmetry: symmetry(rest, geometry),
    badges: badges(census, changes),
  };
}
