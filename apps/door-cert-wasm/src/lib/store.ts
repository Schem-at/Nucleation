import type { CertRecord } from "./types";
import type { XrayData } from "./xray";

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

// The x-ray payload is ~1.5 MB of typed arrays — it does not survive
// JSON.stringify and would not fit the localStorage quota if it did. It stays
// in memory for the session that recorded it; a certificate opened from a cold
// reload keeps every number and offers no x-ray until the door is re-run.
const xrays = new Map<string, XrayData>();

export function saveXray(id: string, data: XrayData): void {
  xrays.set(id, data);
}

export function loadXray(id: string): XrayData | null {
  return xrays.get(id) ?? null;
}

export function newId(): string {
  const bytes = new Uint8Array(4);
  crypto.getRandomValues(bytes);
  return [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
}
