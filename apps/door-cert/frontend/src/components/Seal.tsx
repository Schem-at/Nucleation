// The signature element: a pixel-serrated certification seal, stamped with
// the door's opening time. Serrations are axis-aligned squares — voxels, not
// scalloped lace — so the seal belongs to Minecraft, not a diploma mill.
import type { ReactNode } from "react";

export function Seal({ openTicks }: { openTicks: number }) {
  const size = 190;
  const c = size / 2;
  const teeth: ReactNode[] = [];
  const n = 26;
  const rTeeth = 82;
  const px = 11;
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    const x = Math.round((c + rTeeth * Math.cos(a) - px / 2) / 2) * 2;
    const y = Math.round((c + rTeeth * Math.sin(a) - px / 2) / 2) * 2;
    teeth.push(<rect key={i} x={x} y={y} width={px} height={px} fill="var(--seal)" />);
  }
  const seconds = (openTicks / 20).toFixed(2);
  const rText = 62;
  return (
    <svg
      className="seal"
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      role="img"
      aria-label={`Certified: opens in ${openTicks} ticks (${seconds} seconds)`}
    >
      {teeth}
      <circle cx={c} cy={c} r={73} fill="var(--surface)" stroke="var(--seal)" strokeWidth="2" />
      <circle cx={c} cy={c} r={49} fill="none" stroke="var(--seal)" strokeWidth="1" />
      <defs>
        <path
          id="seal-arc"
          d={`M ${c - rText},${c} a ${rText},${rText} 0 1,1 ${rText * 2},0 a ${rText},${rText} 0 1,1 ${-rText * 2},0`}
        />
      </defs>
      <text fontSize="9.5" letterSpacing="1.7" fill="var(--seal-ink)">
        <textPath href="#seal-arc" startOffset="0%">
          DOOR CERTIFICATION BUREAU · HEADLESS VERIFIED ·
        </textPath>
      </text>
      <text x={c} y={c - 22} textAnchor="middle" fontSize="9" letterSpacing="1.5" fill="var(--seal-ink)">
        OPENS IN
      </text>
      <text x={c} y={c + 16} textAnchor="middle" fontSize="42" fontWeight="700" fill="var(--seal-ink)">
        {openTicks}
      </text>
      <text x={c} y={c + 30} textAnchor="middle" fontSize="8" letterSpacing="0.8" fill="var(--seal-ink)">
        TICKS · {seconds} S
      </text>
    </svg>
  );
}
