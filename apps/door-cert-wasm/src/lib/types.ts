import type { XrayData } from "./xray";

export type TickEvents = {
  tick: number;
  piston: number;
  redstone: number;
  /** Item-movement events. The engine does not report these yet, so this is
   *  zero everywhere; the stacked trace already carries the series. */
  items: number;
  changes: number;
};

export type Material = { id: string; count: number };

/** `INCONCLUSIVE` is not a failure of the door — it is a refusal by the tool.
 *  It is issued when the machine ran fine but the run cannot say WHICH door it
 *  measured: the state the file was saved in and the state the machine settles
 *  into disagree about where the doorway is. Better a loud refusal than a
 *  confident wrong number. */
export type Verdict = "CERTIFIED" | "DID NOT RESET" | "INCONCLUSIVE";

/** The doorway itself: cells that are solid at rest and air once open. */
export type Aperture = {
  /** How many cells open up. This is the MEASUREMENT; `w × h` is only the
   *  envelope it sits in, and the two are equal only when `rectangular`. */
  cells: number;
  /** Horizontal span of the opening, in blocks. */
  w: number;
  /** Vertical span of the opening, in blocks. */
  h: number;
  /** Depth of the opening along the remaining axis. */
  depth: number;
  /** True when the opening exactly fills its own `w × h` envelope. False for an
   *  L, a ring, a cross or any ragged edge — and then `cells < w × h`, which
   *  the sheet has to say out loud rather than let the spans imply a count that
   *  was never measured. */
  rectangular: boolean;
  /** Set when the measurement needed a caveat — a non-rectangular opening,
   *  several separate openings, or no walkable passage at all. Null when the
   *  doorway read cleanly. */
  note?: string | null;
};

/** Which control drives the door, and how hard the evidence for it is.
 *
 *  Nothing here is guessed from the palette. Every candidate was actuated in a
 *  throwaway copy of the world and the build was watched for movement; this is
 *  the one that moved it. See `detectInputs()` in the worker. */
export type InputControl = {
  pos: Vec3;
  /** Full block state as found, e.g. `minecraft:lever[face=floor,...]`. */
  state: string;
  kind: "lever" | "button" | "note block" | "pressure plate";
  /** Cells that physically moved at the peak of its trial. 0 disqualifies. */
  moved: number;
  /** True for a control that springs back on its own — a button, a plate you
   *  step off. The door's second stroke is then the control's own release, not
   *  a second actuation. */
  momentary: boolean;
};

/** One reading of the doorway, for the two-readings-disagree case. */
export type ApertureReading = {
  w: number;
  h: number;
  cells: number;
  depth: number;
  /** The pattern name this reading classifies as, when it classifies at all. */
  name: string | null;
};

/** Raised when the saved state and the settled cycle describe different doors.
 *  Both readings are kept; neither is presented as the answer. */
export type ApertureConflict = {
  /** What the doorway measures in the state the file was saved in. */
  saved: ApertureReading;
  /** What it measures once the machine is on its own repeating cycle. */
  settled: ApertureReading;
  /** Cells that never came back — the size of the disagreement. */
  drift: number;
};

/** The cells the doorway timing is measured over, in world coordinates.
 *  `aperture()` already knows both of these exactly; timing them is the whole
 *  point of extracting them. */
export type ApertureGeometry = {
  /** Every cell of the walkable passage, across the wall's full depth. The
   *  door is OPEN when all of these are air. */
  passage: Vec3[];
  /** The passage cells that hold a door block in the closed reference state.
   *  The door is SHUT when all of these are solid again; a sissy bar or a
   *  checkerboard never fills the rest. */
  closed: Vec3[];
  /** The subset of `closed` you can actually SEE — the first solid cell cast
   *  in from either face. This is the pattern. */
  visible: Vec3[];
  /** The rest: cells that are solid when shut but hidden behind another block
   *  from both faces. The slime blocks pushing a vault's centre panel are
   *  here, not in the pattern — they are carriers. */
  carriers: Vec3[];
  /** True when the first snapshot handed to `aperture()` is the closed one,
   *  i.e. the first lever click opens the door. A file saved open measures
   *  its own closing first, and the strokes swap. */
  restIsClosed: boolean;
};

/** One door block, in the standard's matrix coordinates: row counted down
 *  from the top of the pattern, column left to right, layer front to back. */
export type PatternCell = { r: number; c: number; k: number; id: string };

/** What one face of the door SHOWS.
 *
 *  Cast inward from a face and stop at the first solid cell: that cell is the
 *  pattern at that (row, column), and everything behind it is machinery. See
 *  the header of `aperture.ts` for why this — not the filled volume — is what
 *  the standard's pattern definitions are about. */
export type SurfaceReading = {
  /** Which face this was cast from. `front` is the shallower layer of the
   *  wall along the door's own depth axis; `back` the deeper one. */
  side: "front" | "back";
  m: number;
  n: number;
  /** Distinct depths the visible surface reaches — 1 for a flat face. */
  layers: number;
  /** How many cells of the m × n cross-section show a block at all. */
  cells: number;
  /** Matched pattern, or null when this face matches no definition. */
  pattern: string | null;
  patternRef: string | null;
  transform: string | null;
  /** Section 5.1 variant (Iris / Onion) when one matched. */
  variant: { label: string; ref: string } | null;
  /** 1 where the face shows a block. */
  matrix: number[][];
  /** Depth of that block below this face; -1 where the column is empty. */
  depth: number[][];
  /** The depth matrix as text, for reading the door out loud. */
  ascii: string;
};

/** What sits around the doorway — needed for the Section 5 compositions. */
export type PatternSurroundings = {
  /** Blocks in the ring immediately around the pattern, door layers, closed. */
  frameIds: string[];
  /** The ring outside that one. */
  outerIds: string[];
  /** What the doorway stands on once it is open. */
  sillIds: string[];
};

/** The door read against MYuen222, "Door Pattern Definitions v1.1". */
export type Classification = {
  /** The formal name, e.g. "6 × 6 Flush Regular Door". */
  name: string;
  /** Pattern length and height (Def 2.9). */
  m: number;
  n: number;
  /** Pattern depth: parallel planes the door blocks occupy. */
  layers: number;
  /** Def 2.4. */
  orientation: "Door" | "Skydoor" | "Ceiling Skydoor";
  /** Flush / Deluxe / Trapdoor (Defs 2.6-2.8), when one of them applies. */
  qualifiers: string[];
  /** Where the outermost door layer sits relative to the frame face, in
   *  plain language — the standard has no term for every case. */
  frameNote: string | null;
  /** True when every door block spans the same run of layers: a flat pattern
   *  extruded through the wall rather than a depth-varying one. */
  extruded: boolean;
  /** The door's pattern, read off what its faces SHOW. Null when nothing
   *  matched exactly. For a two-sided door this is the dual name — "Vault" for
   *  the dual funnel of §7.5. */
  pattern: string | null;
  /** Section of the standard the pattern is defined in. */
  patternRef: string | null;
  /** Which member of the symmetry group matched, when not the identity. */
  transform: string | null;
  /** One reading per visible face: one for a one-sided door, two for a door
   *  you can see the pattern from either way. */
  surfaces: SurfaceReading[];
  /** Set when both faces show the same pattern (Definition 2.24). */
  dual: {
    pattern: string;
    patternRef: string;
    name: string;
    ref: string;
    symmetric: boolean;
  } | null;
  /** What the FILLED VOLUME matches, carriers and all. Reported beside the
   *  surface reading rather than as the answer: on a door with carriers inside
   *  its own passage the two differ, and the difference is the point. */
  volumePattern: string | null;
  /** Section 5 block compositions found. */
  composition: { label: string; ref: string }[];
  /** Binary pattern matrix: 1 where a door block sits when closed. */
  matrix: number[][];
  /** Layer index per cell; -1 where empty. */
  depth: number[][];
  unclassified: boolean;
};

/** Counts of the parts that make the door move. */
export type Census = {
  sticky_piston: number;
  piston: number;
  observer: number;
  repeater: number;
  /** Distinct repeater delays present, ascending. */
  repeater_delays: number[];
  comparator: number;
  redstone_block: number;
  redstone_torch: number;
  redstone_wire: number;
  slime_block: number;
  honey_block: number;
};

/** ------------------------------------------------------------ engineering --
 *
 *  What the door COSTS and what of it is doing any work — the readings that
 *  need the engine's recorded update stream rather than just the block-change
 *  log. See `lib/engineering.ts` for the derivations and for the two claims
 *  this group deliberately refuses to make. */

/** Update dispatches over the measured cycle.
 *
 *  Not milliseconds and not a TPS prediction: a dispatch is one delivery of
 *  one update to one cell, and a dust update and a block-entity tick cost the
 *  real server wildly different amounts. What it IS good for is comparing
 *  doors — same engine, same seed, same cycle definition. */
export type ServerCost = {
  /** Every update the engine delivered across one open + close. */
  updates: number;
  /** …of which these were dispatched in the block-events phase, the one that
   *  actually moves pistons. */
  block_events: number;
  /** The full split, biggest first. Phases that never fired are omitted. */
  by_phase: { phase: string; n: number }[];
  /** Busiest single tick, and when. */
  peak: number;
  peak_tick: number;
  /** Normalised so doors of different sizes compare: dispatches per cell of
   *  walkable doorway, and per cell of travelling mass. Null when there is no
   *  doorway or nothing moved. */
  per_passage_cell: number | null;
  per_moved_cell: number | null;
  /** Dispatches per real-time second if the door were held on a loop at its
   *  own measured cycle length. */
  per_second: number | null;
};

/** Blocks that neither moved nor received a single update all cycle.
 *
 *  An upper bound on decoration and redundancy — NOT a removal list. A block
 *  that only holds another one up never has to do anything, and is
 *  load-bearing for exactly that reason. */
export type DeadWeight = {
  /** Non-air blocks at rest. */
  total: number;
  /** …of which this many did nothing this cycle. */
  idle: number;
  /** Their positions, for the replay overlay. Capped; `truncated` says so. */
  cells: Vec3[];
  truncated: boolean;
  /** What they are, most common first. */
  by_id: { id: string; count: number }[];
};

/** The components touched between the input click and the first block that
 *  moved, in the order the engine delivered updates to them.
 *
 *  **Order, not causality.** The log records the sequence of dispatches, never
 *  which dispatch scheduled which, so this is a sequence and is named as one.
 *  See the note on `firstMovement()`. */
export type FirstMovement = {
  /** Updates delivered before the first movement. */
  hops: number;
  /** Ticks it took. */
  ticks: number;
  /** Distinct components in first-touch order, with how many cells of each. */
  chain: { id: string; cells: number }[];
  /** What moved first, and where. */
  block: string;
  pos: Vec3;
};

/** Mirror symmetry. The machine test compares BASE block names only — a
 *  mirrored piston faces the other way, and demanding the state match would
 *  report every symmetric door as asymmetric. */
export type Symmetry = {
  /** The door blocks' silhouette, mirrored inside the passage bbox. Null when
   *  no passage was extracted. */
  pattern: { horizontal: boolean; vertical: boolean } | null;
  /** The whole build, per axis, named relative to the doorway when one is
   *  known and `x`/`y`/`z` otherwise. `share` is the fraction of blocks with a
   *  mirrored twin — the number that carries the signal, because a door with a
   *  lever on one side is 90-something per cent symmetric and never 100. */
  machine: { axis: string; mirror: boolean; share: number }[];
};

/** The four qualifiers pro door makers compete on (ROADMAP §4). */
export type Badges = {
  observerless: boolean;
  dustless: boolean;
  /** Honey counts as slime — it is the standard substitute, and a door that
   *  swapped one for the other has not earned the tag. */
  slimeless: boolean;
  /** True when no piston fires repeatedly at a steady period. */
  cycleless: boolean;
  /** Set when one does: how many pistons run a tape, the busiest one's fire
   *  count, and the period it runs at. */
  tape: { pistons: number; fires: number; period: number | null } | null;
  /** Pistons that fired at all, for context under the badge. */
  pistons: number;
  /** Most fires any single piston managed — the evidence the badge turned on,
   *  present whether or not it did. */
  busiest: number;
};

export type Engineering = {
  /** Null when the update recorder failed — the door still certifies, it just
   *  has no cost reading. */
  cost: ServerCost | null;
  dead: DeadWeight | null;
  first: FirstMovement | null;
  symmetry: Symmetry;
  badges: Badges;
};

/** Purplers' reset measurement: the shortest delay after a lever click that
 *  still lets the input be used again without breaking the machine. There is
 *  no closed form — every value here comes from a trial. */
export type ResetTime = {
  /** Shortest delay that worked, in ticks. Null when nothing under `searched`
   *  did — reported as such rather than guessed at. */
  ticks: number | null;
  /** How far the search actually went, so a null is honest about its reach. */
  searched: number;
  /** The stroke this reset re-triggers into, for the comparison below. */
  stroke_ticks: number | null;
  /** True when the reset lands before the stroke it interrupts has finished:
   *  the input can be used again mid-stroke. Rare. */
  negative: boolean;
  /** Set when the measurement was skipped or qualified. */
  note: string | null;
};

export type Certificate = {
  name: string;
  dims: [number, number, number];
  /** Position of the control that drives the door. Named `lever` because that
   *  is what it is on almost every door; `input` carries what it actually is. */
  lever: [number, number, number];
  /** The control the brute-force search proved drives this door. */
  input: InputControl | null;
  /** Other controls that also moved the build on their own. A door with two
   *  independent inputs is real; the measured cycle used `input`, and these are
   *  named so nobody has to wonder which one was thrown. */
  input_alternatives: InputControl[];
  /** How the input was found, when that is worth saying — an ambiguity, or a
   *  control that is not a lever. Null when a single lever drove it. */
  input_note: string | null;
  /** Ticks from the click until every passage cell is clear — the doorway is
   *  walkable. Null when the passage never fully cleared. */
  open_ticks: number | null;
  /** Ticks from the click until every cell of the closed pattern is solid
   *  again. Null when the doorway never fully re-filled. */
  close_ticks: number | null;
  /** Ticks from the click until the whole machine goes quiet. Always >= the
   *  doorway time above: the tape can still be shuffling long after you can
   *  walk through. This is the number the certificate used to call "opens in". */
  open_settle_ticks: number | null;
  close_settle_ticks: number | null;
  /** Set when a doorway time is missing, saying why. */
  timing_note: string | null;
  /** Ticks from the lever click to the first block that moves. */
  open_latency: number | null;
  close_latency: number | null;
  /** Shortest wait after the opening click before the lever may be thrown
   *  back, and the same after the closing click. */
  reset_open: ResetTime | null;
  reset_close: ResetTime | null;
  materials: Material[];
  events_per_tick: TickEvents[];
  heatmap: { w: number; h: number; values: number[][] };
  /** Length of the measured cycle, in ticks. */
  sim_ticks: number;
  seed: number;
  verdict: Verdict;
  /** Cells that travel on one stroke — the door's working mass. */
  moved_cells: number;
  /** The doorway the machine actually opens. Null if nothing opened. */
  aperture: Aperture | null;
  /** The doorway's cells, in world coordinates — the exact set `open_ticks`
   *  and `close_ticks` were measured over. It rides on the certificate so the
   *  replay can DRAW what the classifier derived; picture and numbers come
   *  from one source and cannot drift apart. ~3 KB for a 6 × 6 door.
   *  Null when no walkable passage was found, same as `aperture`. */
  aperture_geometry: ApertureGeometry | null;
  /** The doorway read against the community pattern standard. */
  classification: Classification | null;
  /** Busiest tick of the cycle: how many cells changed, and when. */
  peak_changes: number;
  peak_tick: number;
  /** Busiest tick for pure signal traffic — the redstone cascade, which
   *  peaks on the click tick and moves nothing. */
  peak_signal: number;
  peak_signal_tick: number;
  /** Parts census, taken at rest. */
  census: Census;
  /** Fastest safe re-trigger, as cycles per minute. */
  cycles_per_minute: number;
  /** Total cells inside the scanned bounds. */
  volume: number;
  /** True when the door was saved mid-cycle and had to be run to its
   *  steady state before the measured cycle could start. */
  needed_priming: boolean;
  /** Cycles needed before the machine repeated itself. */
  priming_cycles: number;
  /** Cells between the saved state and the cycle it settles into. */
  saved_state_drift: number;
  /** Set when priming moved the doorway itself — the file and the settled
   *  machine describe different doors. While this is set the verdict is
   *  INCONCLUSIVE and `classification` is withheld: the run knows it measured
   *  A door and cannot prove it measured THIS one. */
  aperture_conflict: ApertureConflict | null;
  /** False when the file was saved with its doorway already standing open —
   *  the first lever click then closes it, and the two strokes swap. */
  rest_is_closed: boolean;
  /** Cost, dead weight, first movement, symmetry and qualifier badges. Null on
   *  records certified before these were measured. */
  engineering: Engineering | null;
};

export type Vec3 = [number, number, number];

export type ReplayBlock = { pos: Vec3; state: string };
export type ReplayChange = { tick: number; pos: Vec3; from: string; to: string };
export type LeverFlip = { tick: number; label: string; measured: boolean };

/** Everything the voxel replay needs: t=0 world + the recorded change log. */
export type Replay = {
  blocks: ReplayBlock[];
  changes: ReplayChange[];
  simTicks: number;
  flips: LeverFlip[];
};

export type CertRecord = { certificate: Certificate; replay: Replay };

/** Worker → page protocol. */
export type WorkerProgress = { type: "progress"; step: string };
/** `xray` travels beside the record rather than inside it: it is ~1.5 MB of
 *  transferable typed arrays, which neither `JSON.stringify` nor a
 *  localStorage quota survives. The certificate persists; the x-ray is a
 *  this-session artefact and says so when it is missing. */
export type WorkerDone = { type: "done"; record: CertRecord; xray: XrayData | null };
/** `error` is a sentence for the person at the keyboard. `code` is the raw
 *  engine enum, kept as small print so we can still triage from a screenshot,
 *  and `detail` is whatever the engine could name — the offending block, say. */
export type WorkerError = {
  type: "error";
  error: string;
  code?: string | null;
  detail?: string | null;
};
export type WorkerMessage = WorkerProgress | WorkerDone | WorkerError;

/** Page → worker payload. */
export type WorkerJob = { name: string; ext: string; buffer: ArrayBuffer };
