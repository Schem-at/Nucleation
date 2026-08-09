/** Engine reasons, turned into sentences a person can act on.
 *
 *  The router's refusals are already excellent DIAGNOSTICS — `design_corridor::
 *  diagnose` names the blocking layer, the coordinate, the bounded search it
 *  ran and the cross-level probe it tried — but one of them is 500 characters
 *  of prose, and a panel that prints it verbatim reads as noise. This module
 *  splits each one into the three things a UI needs:
 *
 *    headline  one clause: WHAT failed and WHERE
 *    fix       one clause: what to do about it, naming the thing to move
 *    at        the coordinate to fly the camera to
 *
 *  ...and keeps the engine's own words as `detail`, because when the summary is
 *  wrong the original is the only thing that can be trusted. Nothing is
 *  invented here: every clause is lifted from the string.
 */
import type { Vec3 } from "./studio";

export interface HumanReason {
  /** One clause: what failed, and where. */
  headline: string;
  /** One clause: what to do, naming what to move. */
  fix: string;
  /** Where to look, if the reason names a coordinate. */
  at: Vec3 | null;
  /** Layers/instances the user can actually move (`u2`, `bus1`, ...). */
  blame: string[];
  /** The engine's own text, verbatim. */
  detail: string;
  /** Which parser matched — `"corridor"`, `"level"`, ... `"raw"` if none did. */
  kind: string;
}

/** `(12, 3, 40)` / `(12,3,40)` — the engine prints Rust tuple debug. */
const COORD = /\((-?\d+),\s*(-?\d+),\s*(-?\d+)\)/;
const COORD_G = /\((-?\d+),\s*(-?\d+),\s*(-?\d+)\)/g;

function coordAt(s: string, which = 0): Vec3 | null {
  const all = [...s.matchAll(COORD_G)];
  const m = all[which];
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

export const fmtAt = (p: Vec3 | null) => (p ? `(${p[0]}, ${p[1]}, ${p[2]})` : "");

/** Backtick-quoted owners the engine names: ``instance `u2` `` -> `u2`. */
function owners(s: string): string[] {
  const out = new Set<string>();
  for (const m of s.matchAll(/(?:instance|bus|cell|layer)\s+`([^`]+)`/g)) out.add(m[1]);
  // `blocked by instance `u2`` also appears as a bare owner in lists.
  return [...out];
}

const joinOr = (xs: string[]) =>
  xs.length === 0 ? "" : xs.length === 1 ? xs[0] : `${xs.slice(0, -1).join(", ")} or ${xs[xs.length - 1]}`;

/** Turn one engine reason into a sentence pair. Never throws, never returns
 *  empty: an unrecognised reason degrades to its own first sentence. */
export function humanReason(raw: string | null | undefined): HumanReason {
  const detail = (raw ?? "").trim();
  const blame = owners(detail);
  const base = { at: null as Vec3 | null, blame, detail };

  if (!detail) {
    return { ...base, kind: "empty", headline: "failed", fix: "re-route it, or move an endpoint" };
  }

  // "driver port `x`: <inner>" / "sink port `x`: <inner>" — the failure is in
  // resolving an ENDPOINT, so recurse and keep which end it was.
  const end = /^(driver|sink) port `([^`]+)`:\s*([\s\S]+)$/.exec(detail);
  if (end) {
    const inner = humanReason(end[3]);
    return {
      ...inner,
      headline: `${end[1]} port ${end[2]}: ${inner.headline}`,
      detail,
      blame: [...new Set([...inner.blame, ...blame])],
    };
  }

  // A bus is a single-level 2y-pitch stack: the two anchors are on different
  // levels. The most common failure a drag produces, and the most fixable.
  if (/SINGLE-LEVEL|level change|share a level/.test(detail)) {
    const ys = /bit-0 dust sits at y=(-?\d+) and y=(-?\d+)/.exec(detail);
    const by = /by (-?\d+) in y/.exec(detail);
    const seg = coordAt(detail, 0);
    const to = coordAt(detail, 1);
    const where = seg ? ` between ${fmtAt(seg)} and ${fmtAt(to)}` : "";
    return {
      ...base,
      kind: "level",
      at: seg,
      headline: ys
        ? `the two ends are on different levels — driver bit 0 at y=${ys[1]}, sink bit 0 at y=${ys[2]}${where}`
        : `the two ends are on different levels${where}`,
      fix: by
        ? `move ${blame.length ? joinOr(blame) : "one endpoint's instance"} by ${by[1]} in y so both ports share a level, or split the run with a gate at the target level`
        : `move one endpoint's instance in y so both ports share a level, or split the run with a gate`,
    };
  }

  // The anchor is walled in: nothing can leave the port at all.
  if (/endpoint approach blocked/.test(detail)) {
    const which = /the (driver|sink) anchor/.exec(detail)?.[1] ?? "endpoint";
    const anchor = coordAt(detail, 0);
    return {
      ...base,
      kind: "walled-in",
      at: anchor,
      headline: `the ${which} port at ${fmtAt(anchor)} is walled in — every column beside it is occupied`,
      fix: blame.length
        ? `move ${joinOr(blame)}, or leave one clear lane beside the port`
        : `move the neighbouring hardware, or leave one clear lane beside the port`,
    };
  }

  // No corridor. The engine already tells us WHICH layer is in the way, WHERE,
  // and whether another level would work — so say those three things and stop.
  if (/no corridor/.test(detail)) {
    const hit = /the direct line is blocked at \((-?\d+),\s*(-?\d+),\s*(-?\d+)\) by ([^.]+)/.exec(detail);
    const from = coordAt(detail, 0);
    const at: Vec3 | null = hit ? [Number(hit[1]), Number(hit[2]), Number(hit[3])] : from;
    const culprit = hit ? hit[4].replace(/`/g, "").trim() : "";
    const level = /A clear corridor DOES exist at (y=[-\d]+(?: or y=[-\d]+)*)/.exec(detail);
    const width = /for a (\d+)-bit bus/.exec(detail)?.[1];
    return {
      ...base,
      kind: "corridor",
      at,
      headline:
        `no corridor at ${fmtAt(at)}` +
        (width ? ` for the ${width}-bit bus` : "") +
        (culprit ? ` — ${culprit} is in the way` : " — the workspace is full along the whole line"),
      fix: level
        ? `this level is the problem, not the space: shift ${blame.length ? joinOr(blame) : "an endpoint's instance"} to ${level[1]} (a bus cannot ramp between levels)`
        : blame.length
          ? `move ${joinOr(blame)}, or give the bus a gate so it routes in two legs`
          : `move one endpoint, or give the bus a gate so it routes in two legs`,
    };
  }

  // A port that no bus can land on: the promotion prompt, not a routing error.
  if (/no dust connection cell|executor-only/.test(detail)) {
    const bit = /bit (\d+)/.exec(detail)?.[1] ?? "0";
    const at = coordAt(detail, 0);
    const holds = /holds `minecraft:([a-z_]+)/.exec(detail)?.[1] ?? "executor hardware";
    const promotable = /PROMOTE it/.test(detail);
    return {
      ...base,
      kind: "executor-only",
      at,
      headline: `executor-only: bit ${bit} lands on a ${holds} at ${fmtAt(at)}, and nothing in redstone drives one`,
      fix: promotable
        ? `switch the port to Bus mode — or just click it as a bus target and it promotes itself (reversible)`
        : `drive it by hand through the baked executor instead`,
    };
  }

  // Width / type / direction refusals, which arrive as thrown errors.
  if (/width/i.test(detail) && /mismatch/i.test(detail)) {
    return { ...base, kind: "width", headline: detail, fix: `bus one bit range at a time, or pick ports of equal width` };
  }

  // Unrecognised: the first sentence is the headline, the rest is detail.
  const first = detail.split(/(?<=\.)\s+/)[0];
  return {
    ...base,
    kind: "raw",
    at: coordAt(detail, 0),
    headline: first.length > 160 ? `${first.slice(0, 157)}…` : first,
    fix: detail.length > first.length ? "see the engine's full reason below" : "",
  };
}

/** `Bus b1 failed: <headline> — <fix>` in one line, for a toast. */
export function busFailureLine(bus: string, raw: string): string {
  const h = humanReason(raw);
  return `Bus ${bus} failed: ${h.headline}${h.fix ? ` — ${h.fix}` : ""}`;
}
