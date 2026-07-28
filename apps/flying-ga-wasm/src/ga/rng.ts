/** Small deterministic PRNG (mulberry32). The Python backend uses
 * random.Random(seed); we only need reproducibility within this app, not
 * bit-compatibility with CPython. */
export class Rng {
  private s: number;

  constructor(seed: number) {
    this.s = seed >>> 0;
    // A few warm-up spins so tiny seeds decorrelate.
    for (let i = 0; i < 4; i++) this.next();
  }

  /** Uniform float in [0, 1). */
  next(): number {
    let t = (this.s += 0x6d2b79f5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }

  /** Integer in [0, n). */
  randrange(n: number): number {
    return Math.floor(this.next() * n);
  }

  /** Integer in [a, b) — mirrors Python randrange(a, b). */
  randrangeAB(a: number, b: number): number {
    return a + this.randrange(b - a);
  }

  /** k distinct indices out of [0, n) — mirrors random.sample(range(n), k). */
  sample(n: number, k: number): number[] {
    const idx = Array.from({ length: n }, (_, i) => i);
    for (let i = 0; i < k; i++) {
      const j = i + this.randrange(n - i);
      [idx[i], idx[j]] = [idx[j], idx[i]];
    }
    return idx.slice(0, k);
  }
}
