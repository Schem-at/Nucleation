// The door read against MYuen222, "Door Pattern Definitions v1.1" — the
// competitive community's formal spec. The name reads like a spec line and
// the pattern matrix sits beside it, drawn in the door's own orientation so
// you can hold the two against each other.
import type { Classification } from "../lib/types";

/** Layer colours run one lime ramp, front layer lightest. Six steps is more
 *  than any real door needs; deeper cells clamp to the last. */
const RAMP = ["var(--cell-0)", "var(--cell-1)", "var(--cell-2)", "var(--cell-3)", "var(--cell-4)", "var(--cell-5)"];

function MatrixFigure({ cls }: { cls: Classification }) {
  const { matrix, depth, m, n, layers } = cls;
  // Colour only carries depth when depth actually varies; a pattern extruded
  // straight through the wall is one flat shape.
  const steps = new Set(depth.flat().filter((k) => k >= 0));
  const varies = steps.size > 1;
  // The grid should read at a glance without dominating the block.
  const cell = Math.max(9, Math.min(20, Math.round(150 / Math.max(m, n))));
  const gap = 2;
  const w = m * (cell + gap) - gap;
  const h = n * (cell + gap) - gap;
  const rounded = Math.max(1, Math.round(cell / 6));

  return (
    <figure className="matrix-fig">
      <svg
        width={w}
        height={h}
        viewBox={`0 0 ${w} ${h}`}
        role="img"
        aria-label={`Pattern matrix, ${m} columns by ${n} rows. ${matrix
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
            const k = varies ? Math.max(0, Math.min(RAMP.length - 1, depth[r][c])) : 2;
            return (
              <rect
                key={`${r}-${c}`}
                x={x}
                y={y}
                width={cell}
                height={cell}
                rx={rounded}
                fill={RAMP[k]}
              />
            );
          }),
        )}
      </svg>
      <figcaption className="matrix-cap">
        {m} × {n} pattern matrix — filled where a door block sits when closed.
        {varies ? (
          <>
            {" "}
            Layers front
            <span className="matrix-ramp" aria-hidden>
              {RAMP.slice(0, Math.min(steps.size, RAMP.length)).map((c, i) => (
                <i key={i} style={{ background: c }} />
              ))}
            </span>
            back.
          </>
        ) : layers > 1 ? (
          <> One flat shape, extruded {layers} blocks deep.</>
        ) : null}
      </figcaption>
    </figure>
  );
}

export function ClassificationBlock({ cls }: { cls: Classification }) {
  const rows: { term: string; detail: string }[] = [
    { term: "Orientation", detail: cls.orientation },
    { term: "Pattern size", detail: `${cls.m} × ${cls.n}` },
    { term: "Pattern depth", detail: `${cls.layers} ${cls.layers === 1 ? "layer" : "layers"}` },
  ];
  if (cls.qualifiers.length || cls.frameNote)
    rows.push({
      term: "Frame",
      detail: cls.qualifiers.length ? cls.qualifiers.join(" · ") : cls.frameNote!,
    });

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
              {cls.name} — <span className="unknown">unclassified pattern</span>
            </>
          ) : (
            cls.name
          )}
        </h2>

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
            The door blocks do not form any pattern in the standard, under any rotation or
            mirror. The matrix is printed as measured rather than rounded to a near match.
          </p>
        ) : cls.transform ? (
          <p className="classify-note">
            Matched {cls.transform} — the same pattern, authored in a different orientation.
          </p>
        ) : null}

        <p className="classify-cite">
          Patterns per MYuen222, <i>Door Pattern Definitions v1.1</i>. Section numbers refer to
          that document.
        </p>
      </div>

      <MatrixFigure cls={cls} />
    </div>
  );
}
