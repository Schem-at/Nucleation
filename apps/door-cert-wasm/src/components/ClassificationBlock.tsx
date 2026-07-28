// The door read against MYuen222, "Door Pattern Definitions v1.1" — the
// competitive community's formal spec.
//
// The headline is the PATTERN. That is the door's name, the thing a builder
// would call it, and the only line here that answers "what is this?" Flush,
// Deluxe and Trapdoor are frame properties (Definitions 2.6–2.8) — they say
// where the door sits in its wall, not what it looks like — so they sit with
// orientation and pattern size in the attribute row below. An earlier version
// spent the headline on "4 × 4 Flush Door", which read as the door's name and
// was not one.
//
// Beside it, the surfaces: one matrix per face, drawn in the door's own
// orientation, so the reader can hold the classifier's claim against the
// picture. A vault gets two, and its two funnels are the whole argument for
// the name.
import type { Classification, SurfaceReading } from "../lib/types";

/** Layer colours run one lime ramp, front layer lightest — depth is a
 *  magnitude, so it gets a sequential scale rather than a categorical one.
 *  Six steps is more than any real door needs; deeper cells clamp to the last. */
const RAMP = [
  "var(--cell-0)",
  "var(--cell-1)",
  "var(--cell-2)",
  "var(--cell-3)",
  "var(--cell-4)",
  "var(--cell-5)",
];

type Fig = {
  matrix: number[][];
  depth: number[][];
  m: number;
  n: number;
  label: string;
  caption: React.ReactNode;
  testid?: string;
};

/** One pattern matrix. Depth is never carried by colour alone: where it
 *  varies, each filled cell also prints its own depth digit, which is the same
 *  number the ASCII matrix reports. */
function Matrix({ matrix, depth, m, n, label, caption, testid }: Fig) {
  const steps = new Set(depth.flat().filter((k) => k >= 0));
  const varies = steps.size > 1;
  const cell = Math.max(11, Math.min(22, Math.round(150 / Math.max(m, n))));
  const gap = 2;
  const w = m * (cell + gap) - gap;
  const h = n * (cell + gap) - gap;
  const rounded = Math.max(1, Math.round(cell / 6));
  const fontSize = Math.round(cell * 0.52);

  return (
    <figure className="matrix-fig" data-testid={testid}>
      <figcaption className="matrix-label">{label}</figcaption>
      <svg
        width={w}
        height={h}
        viewBox={`0 0 ${w} ${h}`}
        role="img"
        aria-label={`${label}. ${m} columns by ${n} rows. ${matrix
          .map((row, r) => `Row ${r + 1}: ${row.join(" ")}`)
          .join(". ")}`}
      >
        {matrix.map((row, r) =>
          row.map((v, c) => {
            const x = c * (cell + gap);
            const y = r * (cell + gap);
            if (!v)
              return (
                <rect
                  key={`${r}-${c}`}
                  x={x + 0.5}
                  y={y + 0.5}
                  width={cell - 1}
                  height={cell - 1}
                  rx={rounded}
                  fill="none"
                  stroke="var(--baseline)"
                  strokeWidth="1"
                />
              );
            const k = Math.max(0, Math.min(RAMP.length - 1, depth[r][c]));
            return (
              <g key={`${r}-${c}`}>
                <rect
                  x={x}
                  y={y}
                  width={cell}
                  height={cell}
                  rx={rounded}
                  fill={varies ? RAMP[k] : RAMP[2]}
                />
                {varies && (
                  <text
                    x={x + cell / 2}
                    y={y + cell / 2}
                    textAnchor="middle"
                    dominantBaseline="central"
                    fontSize={fontSize}
                    fontWeight="700"
                    fill={k >= 3 ? "var(--cell-0)" : "var(--cell-5)"}
                  >
                    {k}
                  </text>
                )}
              </g>
            );
          }),
        )}
      </svg>
      <figcaption className="matrix-cap">{caption}</figcaption>
    </figure>
  );
}

/** How a surface reads, in one line. */
function surfaceVerdict(s: SurfaceReading) {
  if (!s.pattern) return <span className="unknown">no matching pattern</span>;
  return (
    <>
      <b>{s.pattern}</b> <sup>{s.patternRef}</sup>
      {s.transform ? `, ${s.transform}` : ""}
    </>
  );
}

export function ClassificationBlock({ cls }: { cls: Classification }) {
  const rows: { term: string; detail: string }[] = [
    { term: "Orientation", detail: cls.orientation },
    { term: "Pattern size", detail: `${cls.m} × ${cls.n}` },
    {
      term: "Pattern depth",
      detail: `${cls.layers} ${cls.layers === 1 ? "layer" : "layers"}`,
    },
  ];
  // Frame properties, named as such. `qualifiers` holds the standard's own
  // terms when one applies; `frameNote` is the plain-language reading for the
  // cases the standard has no word for.
  const frame = [...cls.qualifiers, ...(cls.frameNote ? [cls.frameNote] : [])];
  if (frame.length) rows.push({ term: "Frame", detail: frame.join(" · ") });

  const surfaces = cls.surfaces ?? [];
  const twoSided = surfaces.length > 1;
  // The volume reading is only worth printing when it disagrees with the
  // surfaces — that disagreement IS the carriers.
  const volumeDiffers = cls.volumePattern !== cls.pattern;

  return (
    <div className="classify" data-testid="classification">
      <div>
        <div className="classify-head">
          <p className="eyebrow" style={{ margin: 0 }}>
            Classification
          </p>
          {cls.unclassified ? (
            <span className="badge badge-quiet">No exact match</span>
          ) : (
            <span className="badge">
              {cls.pattern} <sup>{cls.patternRef}</sup>
            </span>
          )}
        </div>

        <h2 className="classify-name" data-testid="classify-name">
          {cls.unclassified ? (
            <>
              {cls.m} × {cls.n} <span className="unknown">unclassified pattern</span>{" "}
              {cls.orientation}
            </>
          ) : (
            cls.name
          )}
        </h2>

        {cls.dual && (
          <p className="classify-lede" data-testid="classify-dual">
            Two-sided: each face reads as a <b>{cls.dual.pattern.toLowerCase()}</b>{" "}
            <sup>{cls.dual.patternRef}</sup>, so the door is that pattern in a dual
            arrangement <sup>Def 2.24</sup>
            {cls.dual.name === "Vault" ? (
              <>
                {" "}
                — which the standard names a <b>Vault</b> <sup>{cls.dual.ref}</sup>.
              </>
            ) : (
              "."
            )}
            {!cls.dual.symmetric &&
              " The two faces match, but the blocks between them do not mirror."}
          </p>
        )}

        <dl className="classify-defs">
          {rows.map((r) => (
            <div key={r.term}>
              <dt>{r.term}</dt>
              <dd>{r.detail}</dd>
            </div>
          ))}
        </dl>

        {cls.composition.length > 0 && (
          <div className="classify-chips">
            {cls.composition.map((t) => (
              <span className="badge badge-quiet" key={t.label + t.ref}>
                {t.label} <sup>{t.ref}</sup>
              </span>
            ))}
          </div>
        )}

        {cls.unclassified ? (
          <p className="classify-note">
            {twoSided ? "Neither visible surface forms" : "The visible surface does not form"}{" "}
            any pattern in the standard, under any rotation, mirror or front/back reversal. The
            matrices are printed as measured rather than rounded to a near match.
          </p>
        ) : cls.transform && !cls.dual ? (
          <p className="classify-note">
            Matched {cls.transform} — the same pattern, authored in a different orientation.
          </p>
        ) : null}

        {volumeDiffers && (
          <p className="classify-note" data-testid="classify-carriers">
            Read as a solid volume instead — counting every block in the doorway, carriers and
            all — the door matches{" "}
            {cls.volumePattern ? <b>{cls.volumePattern}</b> : <span className="unknown">nothing</span>}
            . The pattern is what the door <i>shows</i>: the blocks behind the surface push it,
            they are not part of it.
          </p>
        )}

        <p className="classify-cite">
          Patterns per MYuen222, <i>Door Pattern Definitions v1.1</i>. Section numbers refer to
          that document.
        </p>
      </div>

      <div className="matrix-set" data-testid="surface-matrices">
        {surfaces.map((s) => (
          <Matrix
            key={s.side}
            testid={`surface-${s.side}`}
            matrix={s.matrix}
            depth={s.depth}
            m={s.m}
            n={s.n}
            label={twoSided ? `${s.side} face` : "Visible surface"}
            caption={
              <>
                {surfaceVerdict(s)}
                {s.layers > 1 ? (
                  <>
                    {" "}
                    · digits are depth below this face, {s.layers} deep
                  </>
                ) : (
                  " · one flat face"
                )}
              </>
            }
          />
        ))}
        {surfaces.length === 0 && (
          <Matrix
            matrix={cls.matrix}
            depth={cls.depth}
            m={cls.m}
            n={cls.n}
            label="Pattern matrix"
            caption={`${cls.m} × ${cls.n}, filled where a door block sits when closed.`}
          />
        )}
      </div>
    </div>
  );
}
