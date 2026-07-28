/** Single source of truth for the evolutionary alphabet.
 *
 * Ported from apps/flying-ga/backend/ga.py and extended: normal pistons
 * (push-only), note blocks and white concrete give the GA inert mass and
 * one-way thrust to play with. Every entry with `enabled: true` becomes a
 * genome state index (in listed order, air first); disabled entries are
 * documented future material and cost nothing.
 */

export interface AlphabetEntry {
  /** Full blockstate string as the engine understands it. */
  state: string;
  /** Coarse family, used for kick finding and viewer colors. */
  kind:
    | "air"
    | "slime_block"
    | "honey_block"
    | "sticky_piston"
    | "piston"
    | "observer"
    | "note_block"
    | "white_concrete"
    | "oak_trapdoor";
  enabled: boolean;
}

const FACINGS = ["east", "west", "north", "south", "up", "down"] as const;

export const ALPHABET_ENTRIES: AlphabetEntry[] = [
  { state: "minecraft:air", kind: "air", enabled: true },
  { state: "minecraft:slime_block", kind: "slime_block", enabled: true },
  { state: "minecraft:honey_block", kind: "honey_block", enabled: true },
  ...FACINGS.map(
    (f): AlphabetEntry => ({
      state: `minecraft:sticky_piston[extended=false,facing=${f}]`,
      kind: "sticky_piston",
      enabled: true,
    }),
  ),
  // Normal pistons: they push and never pull — a one-way engine part.
  ...FACINGS.map(
    (f): AlphabetEntry => ({
      state: `minecraft:piston[extended=false,facing=${f}]`,
      kind: "piston",
      enabled: true,
    }),
  ),
  ...FACINGS.map(
    (f): AlphabetEntry => ({
      state: `minecraft:observer[facing=${f},powered=false]`,
      kind: "observer",
      enabled: true,
    }),
  ),
  { state: "minecraft:note_block", kind: "note_block", enabled: true },
  { state: "minecraft:white_concrete", kind: "white_concrete", enabled: true },
  // Trapdoors: engine support landed (bytecode-faithful; unpowered trapdoors
  // hold their open state), so they are live in the genome.
  ...FACINGS.slice(0, 4).map(
    (f): AlphabetEntry => ({
      state: `minecraft:oak_trapdoor[facing=${f},half=bottom,open=true,powered=false,waterlogged=false]`,
      kind: "oak_trapdoor",
      enabled: true,
    }),
  ),
];

/** The live alphabet: blockstate strings indexed by genome state index. */
export const ALPHABET: string[] = ALPHABET_ENTRIES.filter((e) => e.enabled).map(
  (e) => e.state,
);

export const AIR = 0;

const kindIdxs = (kind: AlphabetEntry["kind"]): Set<number> => {
  const s = new Set<number>();
  ALPHABET_ENTRIES.filter((e) => e.enabled).forEach((e, i) => {
    if (e.kind === kind) s.add(i);
  });
  return s;
};

/** Kind of each live genome state index (parallel to ALPHABET). */
export const KIND_OF_INDEX: AlphabetEntry["kind"][] = ALPHABET_ENTRIES.filter(
  (e) => e.enabled,
).map((e) => e.kind);

/** Every kind the GA can place (excluding air) — constraint chip options. */
export const PLACEABLE_KINDS = [
  ...new Set(KIND_OF_INDEX.filter((k) => k !== "air")),
];

/** Kinds that are pure inert mass — the cargo objective counts these. */
export const INERT_KINDS = new Set<AlphabetEntry["kind"]>([
  "note_block",
  "white_concrete",
]);

/** Sticky pistons only — the preferred kick targets (they hold the engine). */
export const STICKY_PISTON_IDXS = kindIdxs("sticky_piston");
/** Every piston (sticky + normal) — any of them responds to a kick. */
export const PISTON_IDXS = new Set<number>([
  ...STICKY_PISTON_IDXS,
  ...kindIdxs("piston"),
]);

/** Index of the east-facing sticky piston (the reference kick target). */
export const PISTON_EAST = ALPHABET.indexOf(
  "minecraft:sticky_piston[extended=false,facing=east]",
);

/** Headroom under the 12-push limit per piston (ga.py MAX_BLOCKS). */
export const MAX_BLOCKS = 14;

/** Everything place_block may write, pre-interned at sim construction. */
export const EXTRA_STATES = ["minecraft:redstone_block", ...ALPHABET.slice(1)].join(
  ";",
);

/** Strip "minecraft:" and blockstate props: viewer + legend identity. */
export function blockKind(state: string): string {
  return state.replace(/^minecraft:/, "").replace(/\[.*$/, "");
}
