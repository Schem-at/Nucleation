import type { CertRecord } from "./types";

// Certificates live in memory first (this session's runs) and are mirrored
// into localStorage so a certificate URL survives a reload. No network.

const PREFIX = "door-cert-wasm:";
const mem = new Map<string, CertRecord>();

export function saveRecord(id: string, rec: CertRecord): void {
  mem.set(id, rec);
  try {
    localStorage.setItem(PREFIX + id, JSON.stringify(rec));
  } catch {
    // Quota exceeded: evict older certificates, then retry once.
    try {
      for (const key of Object.keys(localStorage)) {
        if (key.startsWith(PREFIX) && key !== PREFIX + id) localStorage.removeItem(key);
      }
      localStorage.setItem(PREFIX + id, JSON.stringify(rec));
    } catch {
      /* memory copy still works for this session */
    }
  }
}

export function loadRecord(id: string): CertRecord | null {
  const hit = mem.get(id);
  if (hit) return hit;
  try {
    const raw = localStorage.getItem(PREFIX + id);
    if (raw) {
      const rec = JSON.parse(raw) as CertRecord;
      mem.set(id, rec);
      return rec;
    }
  } catch {
    /* corrupted entry — treat as missing */
  }
  return null;
}

export function newId(): string {
  const bytes = new Uint8Array(4);
  crypto.getRandomValues(bytes);
  return [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
}
