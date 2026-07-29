// The verdict stamp: a pixel-serrated seal carrying the door's opening time.
// Serrations are axis-aligned squares — voxels, not scalloped lace — so it
// belongs to Minecraft. It takes the lime of a pass and the alarm red of a
// failure, never both.
import type { ReactNode } from "react";
import type { Verdict } from "../lib/types";

/** `openTicks` is the doorway time — the tick the passage becomes walkable,
 *  not the tick the machine goes quiet. Null when the passage never cleared,
 *  which the seal says outright rather than quoting the settle time instead.
 *
 *  An INCONCLUSIVE run has an opening time and must not print it. The number is
 *  a true measurement of a cycle the run cannot prove is this door's, and a
 *  stamp is the most quotable thing on the page — "WALKABLE IN 1 TICKS" would
 *  outlive every caveat around it. So the seal refuses the number outright and
 *  stamps the refusal instead. */
export function Seal({ openTicks, verdict }: { openTicks: number | null; verdict: Verdict }) {
  const size = 190;
  const c = size / 2;
  const ok = verdict === "CERTIFIED";
  const inconclusive = verdict === "INCONCLUSIVE";
  const teeth: ReactNode[] = [];
  const n = 26;
  const rTeeth = 82;
  const px = 11;
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    const x = Math.round((c + rTeeth * Math.cos(a) - px / 2) / 2) * 2;
    const y = Math.round((c + rTeeth * Math.sin(a) - px / 2) / 2) * 2;
    teeth.push(
      <rect key={i} x={x} y={y} width={px} height={px} fill={ok ? "var(--accent)" : "var(--alarm)"} />,
    );
  }
  const timed = openTicks !== null && !inconclusive;
  const seconds = timed ? (openTicks! / 20).toFixed(2) : null;
  const rText = 62;
  const ring = ok ? "var(--accent)" : "var(--alarm)";
  const ink = ok ? "var(--accent-ink)" : "var(--alarm-ink)";
  return (
    <svg
      className={"seal" + (ok ? "" : " seal-fail")}
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      role="img"
      aria-label={
        inconclusive
          ? `Inconclusive: the file and the settled machine open different doorways, so no opening time is stamped`
          : !ok
            ? `Not certified: door did not reset after its cycle`
            : timed
              ? `Certified: the doorway is walkable ${openTicks} ticks after the input (${seconds} seconds)`
              : `Certified: the passage never fully cleared, so no opening time was measured`
      }
    >
      {teeth}
      <circle cx={c} cy={c} r={73} fill="var(--surface)" stroke={ring} strokeWidth="2" />
      <circle cx={c} cy={c} r={49} fill="none" stroke={ring} strokeWidth="1" />
      <defs>
        <path
          id="seal-arc"
          d={`M ${c - rText},${c} a ${rText},${rText} 0 1,1 ${rText * 2},0 a ${rText},${rText} 0 1,1 ${-rText * 2},0`}
        />
      </defs>
      <text fontSize="9.5" letterSpacing="1.7" fill={ink}>
        <textPath href="#seal-arc" startOffset="0%">
          {ok
            ? "SCHEMAT.IO DOOR VALIDATOR · SIMULATED IN BROWSER ·"
            : inconclusive
              ? "SCHEMAT.IO DOOR VALIDATOR · NOT CLASSIFIED ·"
              : "SCHEMAT.IO DOOR VALIDATOR · DID NOT RESET ·"}
        </textPath>
      </text>
      <text x={c} y={c - 22} textAnchor="middle" fontSize="9" letterSpacing="1.5" fill={ink}>
        {inconclusive ? "DOORWAY IS" : "WALKABLE IN"}
      </text>
      <text
        x={c}
        y={c + 16}
        textAnchor="middle"
        fontSize={timed ? 42 : inconclusive ? 21 : 26}
        fontWeight="700"
        fill={ink}
      >
        {timed ? openTicks : inconclusive ? "AMBIGUOUS" : "n/a"}
      </text>
      <text x={c} y={c + 30} textAnchor="middle" fontSize="8" letterSpacing="0.8" fill={ink}>
        {timed
          ? `TICKS · ${seconds} S`
          : inconclusive
            ? "NO TIME STAMPED"
            : "PASSAGE NEVER CLEARED"}
      </text>
    </svg>
  );
}
