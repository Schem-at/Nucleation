import type { Material } from "../lib/types";

export function MaterialsTable({ materials }: { materials: Material[] }) {
  const sorted = [...materials].sort((a, b) => b.count - a.count);
  const max = Math.max(1, ...sorted.map((m) => m.count));
  return (
    <table className="materials">
      <thead>
        <tr>
          <th>Block</th>
          <th className="num">Count</th>
          <th aria-hidden />
        </tr>
      </thead>
      <tbody>
        {sorted.map((m) => {
          const [ns, name] = m.id.includes(":") ? m.id.split(":") : ["", m.id];
          return (
            <tr key={m.id}>
              <td className="mono">
                {ns ? <span className="ns">{ns}:</span> : null}
                {name}
              </td>
              <td className="num">{m.count}</td>
              <td className="bar-cell">
                <div className="bar" style={{ width: `${(m.count / max) * 100}%` }} />
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
