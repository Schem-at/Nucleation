/** Minimal typings + loader for the wasm engine served from /engine/.
 *
 * Per tests/browser_bench/README.md: workers MUST use a dynamic import inside
 * try/catch — a failing top-level import in a module worker fires neither
 * onmessage nor onerror, the page just hangs. The import is annotated
 * vite-ignore so the bundler leaves the runtime URL alone (public/ serves it
 * with a stable URL and the right wasm content type).
 */

export interface TickSimulationT {
  setRngSeed(seed: bigint): void;
  step(): void;
  run(ticks: number): void;
  placeBlock(x: number, y: number, z: number, state: string): void;
  nonAirCount(): number;
  nonAirCenterX(): number;
  nonAirMinX(): number;
  nonAirMaxX(): number;
  changesCount(): number;
  changesJson(): string;
  worldSnapshotJson(): string;
  /** Static structural analysis of the build in this world — sections,
   * edges, and any proof that it cannot move. See
   * `crates/mc-tick/src/machine_graph.rs`. */
  machineGraphJson(): string;
}

export interface EngineModule {
  TickSimulation: {
    fromSnbt(
      snbt: string,
      settle: unknown,
      ox: number,
      oy: number,
      oz: number,
      extraStates: string,
    ): TickSimulationT;
    /** GA fast path: flat palette-index cells, no SNBT built or parsed. */
    fromBlocks(
      bx: number,
      by: number,
      bz: number,
      travel: number,
      xOff: number,
      palette: string,
      cells: number[],
      airIndex: number,
      settle: unknown,
      ox: number,
      oy: number,
      oz: number,
    ): TickSimulationT;
    /** One call per generation chunk: kicked flights for N genomes, JSON
     * rows of raw scalars (see evalCore.FlightRow). */
    evalFlightBatch(
      bx: number,
      by: number,
      bz: number,
      travel: number,
      xOff: number,
      palette: string,
      cells: number[],
      airIndex: number,
      kicks: number[],
      evalTicks: number,
      seed: bigint,
      mustMoveByTick: number,
      needPeriod: boolean,
      earlyExit: boolean,
    ): string;
    /** One call per generation chunk: static verdicts for N genomes, JSON
     * rows (see evalCore.GraphRow). Cheap enough to run before every batch. */
    machineGraphBatchJson(
      bx: number,
      by: number,
      bz: number,
      travel: number,
      xOff: number,
      palette: string,
      cells: number[],
      airIndex: number,
    ): string;
  };
  TickSettleMode: { Quiet: unknown };
}

let modP: Promise<EngineModule> | null = null;

export function loadEngine(): Promise<EngineModule> {
  if (!modP) {
    modP = import(
      /* @vite-ignore */ new URL("/engine/index.mjs", self.location.origin).href
    ) as Promise<EngineModule>;
  }
  return modP;
}
