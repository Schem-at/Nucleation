/** The bill of materials as an inventory, not a spreadsheet.
 *
 * Each block gets a beveled slot holding its real pack texture (resolved and
 * composited by lib/textures — honey and slime arrive pre-layered) with the
 * count stamped bottom-right in the classic style. The caption carries the
 * name and the stack math a builder actually needs when gathering: 132 is
 * "2×64 + 4", not a number to divide in your head at the chest. */

import { useEffect, useReducer } from "react";
import { ensureTextureIndex, onTexturesChanged, textureURL } from "../lib/textures";
import type { Material } from "../lib/types";

const STACK = 64;

/** "132" -> "2×64 + 4". Under a stack, there is no math to show. */
export function stackMath(n: number): string | null {
  if (n < STACK) return null;
  const full = Math.floor(n / STACK);
  const rem = n % STACK;
  return `${full}×${STACK}${rem ? ` + ${rem}` : ""}`;
}

/** "minecraft:sticky_piston" -> "Sticky piston". */
function label(id: string): string {
  const bare = id.replace(/^minecraft:/, "").replace(/_/g, " ");
  return bare.charAt(0).toUpperCase() + bare.slice(1);
}

/** Re-render when the texture index loads or an image finishes decoding. */
function useTextureRevision(): void {
  const [, bump] = useReducer((n: number) => n + 1, 0);
  useEffect(() => {
    ensureTextureIndex();
    return onTexturesChanged(bump);
  }, []);
}

export function MaterialsGrid({ materials }: { materials: Material[] }) {
  useTextureRevision();
  const sorted = [...materials].sort((a, b) => b.count - a.count || (a.id < b.id ? -1 : 1));

  return (
    <ul className="stacks" role="list">
      {sorted.map((m) => {
        const url = textureURL(m.id, "side");
        const name = label(m.id);
        const math = stackMath(m.count);
        return (
          <li className="stack" key={m.id}>
            <div
              className="stack-slot"
              title={`${name} — ${m.count}`}
              aria-label={`${name}, ${m.count} blocks${math ? ` (${math})` : ""}`}
              role="img"
            >
              {url ? (
                <img className="stack-tex" src={url} alt="" aria-hidden width={40} height={40} />
              ) : (
                <span className="stack-tex stack-tex-blank" aria-hidden />
              )}
              <b className="stack-count">{m.count}</b>
            </div>
            <p className="stack-name" aria-hidden>
              {name}
            </p>
            <p className="stack-math" aria-hidden>
              {math ?? `${m.count}`}
            </p>
          </li>
        );
      })}
    </ul>
  );
}
