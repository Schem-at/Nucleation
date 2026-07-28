// The verdict stamp: a pixel-serrated seal carrying the door's opening time.
// Serrations are axis-aligned squares — voxels, not scalloped lace — so it
// belongs to Minecraft. It takes the lime of a pass and the alarm red of a
// failure, never both.
import type { ReactNode } from "react";
import type { Verdict } from "../lib/types";

export function Seal({ openTicks, verdict }: { openTicks: number; verdict: Verdict }) {
  const size = 190;
  const c = size / 2;
  const ok = verdict === "CERTIFIED";
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
  const seconds = (openTicks / 20).toFixed(2);
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
        ok
          ? `Certified: opens in ${openTicks} ticks (${seconds} seconds)`
          : `Not certified: door did not reset after its cycle`
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
            : "SCHEMAT.IO DOOR VALIDATOR · DID NOT RESET ·"}
        </textPath>
      </text>
      <text x={c} y={c - 22} textAnchor="middle" fontSize="9" letterSpacing="1.5" fill={ink}>
        OPENS IN
      </text>
      <text x={c} y={c + 16} textAnchor="middle" fontSize="42" fontWeight="700" fill={ink}>
        {openTicks}
      </text>
      <text x={c} y={c + 30} textAnchor="middle" fontSize="8" letterSpacing="0.8" fill={ink}>
        TICKS · {seconds} S
      </text>
    </svg>
  );
}
