/** `disputed` is for a reading the run took but will not stand behind — the
 *  settled-cycle half of a door whose file and machine disagree. The number is
 *  still shown, because deleting a measurement is its own kind of dishonesty,
 *  but it is set at body size in the muted ink and carries the word "disputed"
 *  where the eye lands first. A tile is the most liftable thing on the sheet;
 *  at 30px it reads as the answer no matter what the paragraph above it says. */
export function StatTile({
  label,
  value,
  unit,
  sub,
  disputed = false,
}: {
  label: string;
  value: string | number;
  unit?: string;
  sub?: string;
  disputed?: boolean;
}) {
  return (
    <div className={disputed ? "tile tile-disputed" : "tile"}>
      <p className="tile-label">
        {label}
        {disputed ? <span className="tile-tag">disputed</span> : null}
      </p>
      <p className="tile-value">
        {value}
        {unit ? <small>{unit}</small> : null}
      </p>
      {sub ? <p className="tile-sub">{sub}</p> : null}
    </div>
  );
}
