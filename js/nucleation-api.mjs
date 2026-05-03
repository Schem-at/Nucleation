// Polished JS API for Nucleation, layered over the wasm-bindgen output.
//
// The default `nucleation` import still exposes `init`, `SchematicWrapper`,
// `SchematicBuilderWrapper`, etc. — that surface is unchanged. This sub-module
// (`nucleation/api`) adds an ergonomic facade analogous to the Python API
// described in api_upgrade.md.
//
// Usage:
//   import init from 'nucleation';
//   import { Schematic, Block, UseBlock } from 'nucleation/api';
//   await init();
//
//   const schem = Schematic.new('test');
//   schem.setBlock([0, 0, 0], 'minecraft:repeater', { state: { delay: 4 } });
//   schem.save('out.litematic'); // browser: returns Uint8Array if no fs.

import init, * as raw from './nucleation.js';

export { init };
export const _raw = raw;

// --- Block ----------------------------------------------------------------

const SNBT_RAW = '__snbt__';

export class Block {
  /**
   * @param {string} id
   * @param {object} [opts]
   * @param {Record<string, string|number|boolean>} [opts.state]
   * @param {Record<string, any>} [opts.nbt]
   */
  constructor(id, { state = null, nbt = null } = {}) {
    this.id = id;
    this.state = state;
    this.nbt = nbt;
    Object.freeze(this);
  }

  /** Parse "id[k=v,...]{snbt}" form. */
  static parse(s) {
    let rest = String(s).trim();
    let snbt = null;
    if (rest.endsWith('}')) {
      let depth = 0;
      let cut = -1;
      for (let i = rest.length - 1; i >= 0; i--) {
        const ch = rest[i];
        if (ch === '}') depth++;
        else if (ch === '{') {
          depth--;
          if (depth === 0) {
            cut = i;
            break;
          }
        }
      }
      if (cut >= 0) {
        snbt = rest.slice(cut + 1, -1);
        rest = rest.slice(0, cut);
      }
    }
    let state = null;
    if (rest.endsWith(']')) {
      const cut = rest.lastIndexOf('[');
      if (cut >= 0) {
        const inner = rest.slice(cut + 1, -1);
        rest = rest.slice(0, cut);
        state = {};
        for (const kv of inner.split(',')) {
          if (!kv.trim()) continue;
          const eq = kv.indexOf('=');
          if (eq < 0) continue;
          const k = kv.slice(0, eq).trim();
          const v = kv.slice(eq + 1).trim();
          state[k] = coerceStateValue(v);
        }
      }
    }
    const nbt = snbt !== null ? { [SNBT_RAW]: snbt } : null;
    return new Block(rest, { state, nbt });
  }

  withState(extra) {
    return new Block(this.id, {
      state: { ...(this.state || {}), ...extra },
      nbt: this.nbt,
    });
  }

  withNbt(extra) {
    return new Block(this.id, {
      state: this.state,
      nbt: { ...(this.nbt || {}), ...extra },
    });
  }

  toString() {
    let out = this.id;
    if (this.state) {
      const parts = Object.entries(this.state).map(
        ([k, v]) => `${k}=${stateToStr(v)}`
      );
      if (parts.length) out += `[${parts.join(',')}]`;
    }
    if (this.nbt) {
      if (SNBT_RAW in this.nbt && Object.keys(this.nbt).length === 1) {
        out += `{${this.nbt[SNBT_RAW]}}`;
      } else {
        out += `{${dictToSnbt(this.nbt)}}`;
      }
    }
    return out;
  }
}

function coerceStateValue(v) {
  if (v === 'true') return true;
  if (v === 'false') return false;
  if (/^-?\d+$/.test(v)) return parseInt(v, 10);
  return v;
}

function stateToStr(v) {
  if (typeof v === 'boolean') return v ? 'true' : 'false';
  return String(v);
}

function valueToSnbt(v) {
  if (typeof v === 'boolean') return v ? '1b' : '0b';
  if (typeof v === 'number') {
    return Number.isInteger(v) ? `${v}` : `${v}f`;
  }
  if (typeof v === 'string') {
    return `"${v.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
  }
  if (Array.isArray(v)) return `[${v.map(valueToSnbt).join(',')}]`;
  if (v && typeof v === 'object') return `{${dictToSnbt(v)}}`;
  throw new TypeError(`Cannot encode ${typeof v} as SNBT`);
}

function dictToSnbt(d) {
  if (SNBT_RAW in d && Object.keys(d).length === 1) return String(d[SNBT_RAW]);
  return Object.entries(d)
    .map(([k, v]) => `${k}:${valueToSnbt(v)}`)
    .join(',');
}

// --- Events ---------------------------------------------------------------

export class UseBlock {
  constructor(pos) {
    this.pos = pos;
    Object.freeze(this);
  }
}

export class ButtonPress {
  constructor(pos) {
    this.pos = pos;
    Object.freeze(this);
  }
}

// --- Cursor ---------------------------------------------------------------

export class Cursor {
  constructor(schem, origin, step) {
    this._schem = schem;
    this._origin = origin;
    this.pos = origin;
    this.step = step;
  }

  place(block, opts = {}) {
    const offset = opts.offset || [0, 0, 0];
    const target = [
      this.pos[0] + offset[0],
      this.pos[1] + offset[1],
      this.pos[2] + offset[2],
    ];
    this._schem.setBlock(target, block, {
      state: opts.state,
      nbt: opts.nbt,
    });
    return this;
  }

  advance(n = 1) {
    this.pos = [
      this.pos[0] + this.step[0] * n,
      this.pos[1] + this.step[1] * n,
      this.pos[2] + this.step[2] * n,
    ];
    return this;
  }

  reset() {
    this.pos = this._origin;
    return this;
  }
}

// --- Schematic ------------------------------------------------------------

const LOAD_EXTS = ['.schem', '.litematic', '.nbt', '.schematic', '.mcstructure'];

function suffixOf(path) {
  const m = String(path).toLowerCase().match(/\.[^.\/\\]+$/);
  return m ? m[0] : '';
}

export class Schematic {
  constructor(rawWrapper, { pack = null } = {}) {
    this.raw = rawWrapper;
    this.pack = pack;
    this._pendingBuilder = null;
  }

  /** Create a blank schematic. */
  static new(name = 'untitled', { pack = null } = {}) {
    const w = new raw.SchematicWrapper();
    if (typeof w.set_name === 'function') {
      try { w.set_name(name); } catch {}
    }
    return new Schematic(w, { pack });
  }

  /**
   * Load from binary data (Uint8Array). Format inferred from the second arg
   * if it ends in a known extension; otherwise from_data() autodetects.
   */
  static open(data, { hint = null, pack = null } = {}) {
    const w = new raw.SchematicWrapper();
    const ext = hint ? suffixOf(hint) : '';
    if (ext === '.litematic') w.from_litematic(data);
    else if (ext === '.schem' || ext === '.schematic') w.from_schematic(data);
    else if (ext === '.mcstructure') w.from_mcstructure(data);
    else w.from_data(data);
    return new Schematic(w, { pack });
  }

  /** Build from an ASCII-art template via SchematicBuilderWrapper. */
  static fromTemplate(template, { name = 'untitled', pack = null } = {}) {
    const builder = raw.SchematicBuilderWrapper.from_template
      ? raw.SchematicBuilderWrapper.from_template(template)
      : new raw.SchematicBuilderWrapper().from_template(template);
    if (typeof builder.name === 'function') builder.name(name);
    const schem = new Schematic(/* placeholder */ null, { pack });
    schem._pendingBuilder = builder;
    schem._pendingName = name;
    return schem;
  }

  _ensureBuilt() {
    if (this._pendingBuilder) {
      this.raw = this._pendingBuilder.build();
      this._pendingBuilder = null;
    }
    return this.raw;
  }

  /**
   * setBlock([x, y, z], "id" | Block, { state?, nbt? })
   * setBlock(x, y, z, "id")           — legacy form
   */
  setBlock(...args) {
    let x, y, z, block, opts;
    if (args.length === 4 && typeof args[0] === 'number') {
      [x, y, z, block] = args;
      opts = {};
    } else if (args.length >= 2 && Array.isArray(args[0])) {
      [[x, y, z], block, opts = {}] = args;
    } else {
      throw new TypeError(
        'setBlock([x,y,z], block, opts?) or setBlock(x,y,z, block)'
      );
    }
    const inner = this._ensureBuilt();
    let blockObj;
    if (block instanceof Block) blockObj = block;
    else if (typeof block === 'string') {
      if (!opts.state && !opts.nbt && (block.includes('[') || block.includes('{'))) {
        blockObj = Block.parse(block);
      } else {
        blockObj = new Block(block, { state: opts.state, nbt: opts.nbt });
      }
    } else {
      throw new TypeError(`block must be string or Block, got ${typeof block}`);
    }
    const effState = { ...(blockObj.state || {}), ...(opts.state || {}) };
    const effNbt = { ...(blockObj.nbt || {}), ...(opts.nbt || {}) };
    const hasState = Object.keys(effState).length > 0;
    const hasNbt = Object.keys(effNbt).length > 0;

    if (hasNbt) {
      const snbt = (SNBT_RAW in effNbt && Object.keys(effNbt).length === 1)
        ? String(effNbt[SNBT_RAW])
        : dictToSnbt(effNbt);
      let payload = blockObj.id;
      if (hasState) {
        payload += '[' + Object.entries(effState).map(([k, v]) => `${k}=${stateToStr(v)}`).join(',') + ']';
      }
      payload += '{' + snbt + '}';
      inner.set_block_from_string(x, y, z, payload);
    } else if (hasState) {
      const props = {};
      for (const [k, v] of Object.entries(effState)) props[k] = stateToStr(v);
      inner.set_block_with_properties(x, y, z, blockObj.id, props);
    } else {
      inner.set_block(x, y, z, blockObj.id);
    }
    return this;
  }

  map(char, block, { state = null, nbt = null } = {}) {
    if (!this._pendingBuilder) {
      throw new Error('map() is only valid on Schematic.fromTemplate()');
    }
    let payload;
    if (block instanceof Block) payload = block.toString();
    else if (state || nbt) payload = new Block(block, { state, nbt }).toString();
    else payload = block;
    this._pendingBuilder.map(char, payload);
    return this;
  }

  fill([p1, p2], block) {
    const blockId = block instanceof Block ? block.toString() : block;
    const inner = this._ensureBuilt();
    inner.fillCuboid(p1[0], p1[1], p1[2], p2[0], p2[1], p2[2], blockId);
    return this;
  }

  cursor({ origin = [0, 0, 0], step = [1, 0, 0] } = {}) {
    this._ensureBuilt();
    return new Cursor(this, origin, step);
  }

  withPack(pack) {
    this.pack = pack;
    return this;
  }

  copy() {
    const inner = this._ensureBuilt();
    const data = inner.to_litematic();
    const w = new raw.SchematicWrapper();
    w.from_litematic(data);
    return new Schematic(w, { pack: this.pack });
  }

  /** Run a redstone simulation, then sync results back into this schematic. */
  simulate({ ticks = 1, events = [] } = {}) {
    const inner = this._ensureBuilt();
    if (typeof inner.create_simulation_world !== 'function') {
      throw new Error('simulation feature is not available in this WASM build');
    }
    const world = inner.create_simulation_world();
    for (const ev of events) {
      if (ev instanceof UseBlock || ev instanceof ButtonPress) {
        const [x, y, z] = ev.pos;
        world.on_use_block(x, y, z);
      } else {
        throw new TypeError('Unsupported event; use UseBlock or ButtonPress');
      }
    }
    world.tick(ticks);
    world.sync_to_schematic();
    if (typeof world.into_schematic === 'function') {
      this.raw = world.into_schematic();
    } else if (typeof world.get_schematic === 'function') {
      this.raw = world.get_schematic();
    }
    return this;
  }

  /**
   * Serialize to bytes by format extension; if a Node fs is provided
   * (the default in Node), write to disk.
   */
  async save(path, { format = null, fs = null } = {}) {
    const inner = this._ensureBuilt();
    const ext = format ? '.' + format : suffixOf(path);
    let bytes;
    if (ext === '.litematic') bytes = inner.to_litematic();
    else if (ext === '.schem' || ext === '.schematic') bytes = inner.to_schematic();
    else if (ext === '.mcstructure') bytes = inner.to_mcstructure();
    else throw new Error(`save: unknown format for ${path}`);
    bytes = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
    if (typeof process !== 'undefined' && process.versions?.node) {
      const nodeFs = fs || (await import('node:fs/promises'));
      await nodeFs.writeFile(path, bytes);
      return null;
    }
    return bytes;
  }
}

// --- SchematicBuilder shim ------------------------------------------------

export class SchematicBuilder {
  constructor() {
    if (typeof console !== 'undefined' && console.warn) {
      console.warn(
        '[nucleation] SchematicBuilder is deprecated; use Schematic.fromTemplate(...)'
      );
    }
    this._raw = new raw.SchematicBuilderWrapper();
  }
  name(n) { this._raw.name(n); return this; }
  fromTemplate(t) { this._raw = raw.SchematicBuilderWrapper.from_template(t); return this; }
  map(c, b) { this._raw.map(c, b); return this; }
  build() { return new Schematic(this._raw.build()); }
}
