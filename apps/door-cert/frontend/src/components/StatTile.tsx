export function StatTile({
  label,
  value,
  unit,
  sub,
}: {
  label: string;
  value: string | number;
  unit?: string;
  sub?: string;
}) {
  return (
    <div className="tile">
      <p className="tile-label">{label}</p>
      <p className="tile-value">
        {value}
        {unit ? <small>{unit}</small> : null}
      </p>
      {sub ? <p className="tile-sub">{sub}</p> : null}
    </div>
  );
}
