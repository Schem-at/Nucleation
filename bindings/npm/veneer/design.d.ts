/** Typings for the JS design veneer (`nucleation/design`), the 1:1 mirror
 *  of `bindings/python/nucleation/design.py` over the generated wasm core. */

export type Vec3 = [number, number, number] | number[];

export interface GateObj {
  anchor: Vec3;
  step: Vec3;
  name?: string | null;
}
/** A bus gate; unnamed gates auto-name `g0`, `g1`, ... in order. */
export function Gate(anchor: Vec3, step: Vec3, name?: string | null): GateObj;

export interface StyleOpts {
  busBlock?: string | null;
  transparentBlock?: string | null;
  /** wire-format spellings pass through untouched */
  bus_block?: string;
  transparent_block?: string;
}
/** Per-bus routing style; omitted fields fall back to router defaults. */
export function Style(opts?: StyleOpts): Record<string, string>;

export declare class CheckReport {
  constructor(raw: any);
  raw: any;
  readonly clean: boolean;
  readonly drc: any[];
  readonly lvs: Record<string, any>;
  readonly rules: any[];
  readonly buses: Record<string, any>;
  readonly skew: Record<string, any>;
  get(key: string): any;
}

export declare class DesignCheckError extends Error {
  report: CheckReport;
}

export declare class Bus {
  design: Design;
  name: string;
  gateNames: string[];
  readonly state: string;
  readonly skew: { per_bit_rt: number[]; skew_rt: number; max_rt: number };
  moveGate(gate: number | string, anchor: Vec3): { state: string; rerouted_segments: number };
  addGate(anchor: Vec3, step: Vec3, name?: string | null): string;
  rule(rule?: Record<string, any>): void;
  rip(): void;
}

export declare class Executor {
  cell: any; // core CellExecutor
  static forSchematic(schematic: any, settle?: number): Executor;
  set(name: string, value: number | boolean | string): void;
  get(name: string): number | boolean;
  setInput(name: string, value: number | boolean | string): void;
  readOutput(name: string): any; // core Value
  settle(budget?: number): boolean;
  reset(): void;
}

export declare class Flat {
  constructor(raw: any);
  raw: any; // core Schematic
  executor(settle?: number): Executor;
  toBytes(format?: string): Uint8Array;
  save(path: string): Promise<void>; // Node only
}

export interface PortDecl {
  anchor: Vec3;
  step: Vec3;
  width: number;
  ty?: "uint" | "bool" | string;
}

export declare class Design {
  readonly raw: any; // core Design
  static create(name: string): Design;
  static forSchematic(name: string, schematic: any): Design;
  static fromNucm(data: Uint8Array | number[]): Design;
  static fromLitematic(data: Uint8Array | number[]): Design;
  static loadNucm(path: string): Promise<Design>; // Node only
  static importLitematic(path: string): Promise<Design>; // Node only

  declareInput(name: string, port: PortDecl): void;
  declareOutput(name: string, port: PortDecl): void;

  routeBus(name: string, opts: {
    driver: string; sinks: string | string[];
    gates?: GateObj[]; style?: StyleOpts | Record<string, string> | string | null;
  }): Bus;
  routeBusOr(name: string, opts: {
    drivers: string | string[]; sinks: string | string[];
    gates?: GateObj[]; style?: StyleOpts | Record<string, string> | string | null;
  }): Bus;
  addGate(bus: string, gate: string, anchor: Vec3, step: Vec3): string;
  moveGate(bus: string, gate: string, anchor: Vec3): { state: string; rerouted_segments: number };
  setBusRule(bus: string, rule?: Record<string, any>): void;
  busState(name: string): string;
  busSkew(name: string): { per_bit_rt: number[]; skew_rt: number; max_rt: number };
  rip(name: string): void;

  addCell(name: string, schematic: any): string;
  place(name: string, cell: string, at: Vec3, rot?: number): void;
  moveInstance(name: string, at: Vec3, rot?: number): {
    rerouted: string[]; failed: Record<string, string>;
  };

  setBlock(x: number, y: number, z: number, block: string): void;

  check(opts?: { strict?: boolean }): CheckReport;
  flatten(): Flat;
  bake(budget?: number): Flat;

  toNucmB64(): string;
  toLitematicB64(): string;
  toBytes(pathOrSuffix: string): Uint8Array;
  save(path: string): Promise<void>; // Node only
}

export interface VeneerSurface {
  Design: typeof Design;
  Executor: typeof Executor;
  Flat: typeof Flat;
  Bus: typeof Bus;
  CheckReport: typeof CheckReport;
  DesignCheckError: typeof DesignCheckError;
  Gate: typeof Gate;
  Style: typeof Style;
}

/** Bind the veneer to a loaded wasm core module; memoized on the core. */
export function veneer(core: any): VeneerSurface;
