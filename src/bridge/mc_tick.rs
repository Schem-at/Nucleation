//! mc-tick: the vanilla-accurate, headless tick engine.
//!
//! New surface — no old `ffi/` counterpart. An opaque [`ffi::TickSimulation`]
//! wraps `mc_tick::Simulation` with the full wiring recipe the engine's own
//! conformance tests use (inventories, behaviours, physics/fluid/rail tables,
//! entities, block-entity tickers), so any schematic that loads runs exactly
//! as the Rust test harness would run it.
//!
//! Design notes:
//! - Everything is headless — no rendering feature involved. Hosts pull
//!   per-tick JSON logs out and compute stats/animations themselves.
//! - Behaviours bind to *interned* states at construction, so any state a
//!   later `place_block` will write (a redstone block, typically) must be
//!   named up front: the constructors take a semicolon-separated
//!   `extra_states` list. `minecraft:redstone_block` and every facing of any
//!   shulker box held as an item are always pre-interned.
//! - Structured data crosses as JSON strings (PORTING.md rule 9).

/// The first data version whose block ids are flattened (1.13).
const FLATTENING_DATA_VERSION: i32 = 1519;

/// Run a pre-flattening build through the dataconverter before the engine
/// ever sees it.
///
/// mc-tick's registry is modern-only, so a 1.12 schematic reaches it holding
/// ids that no longer exist. The failure is not a clean "unknown block"
/// either: `minecraft:slime` is the 1.12 id for the *slime block*, and modern
/// `minecraft:slime` does not exist — so a bore whose whole point is sticky
/// movement loads with 59 inert cells and simulates as a machine that cannot
/// fly. Likewise `stone_slab` (now `smooth_stone_slab`), `leaves`, and
/// `fence_gate`. Converting forward is lossless, so this is unconditional
/// for anything older than the flattening.
///
/// Returns `None` when the build is already modern — callers then use their
/// borrow and clone nothing.
fn modernized(schematic: &crate::UniversalSchematic) -> Option<crate::UniversalSchematic> {
    let from = schematic.metadata.source_data_version.or(schematic.metadata.mc_version)?;
    if from >= FLATTENING_DATA_VERSION {
        return None;
    }
    let mut converted = schematic.clone();
    converted.convert_to_data_version(crate::dataconverter::CANONICAL_DATA_VERSION);
    Some(converted)
}

/// Whether a block's simulated behaviour depends on block-entity data.
///
/// Only the ones whose *absence changes the run*: a comparator without
/// `OutputSignal` reads 0, a container without `Items` is empty and so reads
/// 0 through a comparator and has nothing to transfer. Signs, banners and
/// heads carry block entities too, and losing them changes nothing that
/// ticks — listing them would bury the two that matter.
fn needs_block_entity(name: &str) -> bool {
    let short = name.strip_prefix("minecraft:").unwrap_or(name);
    matches!(
        short,
        "comparator"
            | "chest"
            | "trapped_chest"
            | "barrel"
            | "hopper"
            | "dropper"
            | "dispenser"
            | "furnace"
            | "blast_furnace"
            | "smoker"
            | "brewing_stand"
            | "crafter"
            | "chiseled_bookshelf"
            | "jukebox"
            | "lectern"
            | "decorated_pot"
    ) || short.ends_with("shulker_box")
}

/// See [`ffi::TickSimulation::block_entity_audit_json`].
fn block_entity_audit(schematic: &crate::UniversalSchematic) -> String {
    use std::collections::{HashMap, HashSet};
    use std::fmt::Write as _;

    let have: HashSet<(i32, i32, i32)> =
        schematic.get_block_entities_as_list().into_iter().map(|be| be.position).collect();

    let mut missing: HashMap<String, u32> = HashMap::new();
    for (pos, state) in schematic.iter_blocks() {
        if !needs_block_entity(&state.name) {
            continue;
        }
        if !have.contains(&(pos.x, pos.y, pos.z)) {
            *missing.entry(state.name.to_string()).or_default() += 1;
        }
    }

    let mut rows: Vec<(String, u32)> = missing.into_iter().collect();
    // Descending count, then name — a stable order so two runs of the same
    // file produce byte-identical JSON.
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let total: u32 = rows.iter().map(|(_, n)| *n).sum();

    let mut json = String::from("{\"present\":");
    let _ = write!(json, "{}", have.len());
    let _ = write!(json, ",\"missing_total\":{total},\"missing\":[");
    for (i, (name, count)) in rows.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let _ = write!(json, "{{\"name\":\"{name}\",\"count\":{count}}}");
    }
    json.push_str("],\"summary\":\"");
    if total > 0 {
        let named: Vec<String> = rows
            .iter()
            .take(3)
            .map(|(name, count)| {
                let short = name.strip_prefix("minecraft:").unwrap_or(name);
                let plural = if *count == 1 { "" } else { "s" };
                format!("{count} {short}{plural}")
            })
            .collect();
        let more = if rows.len() > 3 { ", and others" } else { "" };
        let _ = write!(
            json,
            "This schematic contains {}{} with no block-entity data. \
             Comparator outputs and container contents are simulated as empty, \
             so results may not reflect the original build.",
            named.join(", "),
            more
        );
    }
    json.push_str("\"}");
    json
}

/// One double, written the way mc-tick's `float` reader reads it.
///
/// Finite values are written in full rather than with an exponent: `{}` and not
/// `{:?}`, because Rust's `Display` for `f64` never emits an exponent (it
/// spells `4.3e-59` out in full) while `Debug` does. The parser understands
/// exponents now, so this is belt and braces rather than a requirement — but it
/// keeps the output readable by any stricter SNBT reader, and an exponent is
/// unforgiving when it goes wrong: `4.3e-59` read without exponent support
/// becomes `4.3` followed by a *second* number `-59`, silently turning a
/// three-element `Motion` into four. Display drops the fractional part of
/// integral values, so `.0` goes back on to keep the tag a double.
///
/// `NaN` and `±Infinity` are written out as themselves, and **must not be
/// sanitised**. They are not corrupt data — they are the mechanism. The record
/// 3x3 door is glued together by *nan carts*: minecarts whose velocity was
/// deliberately overflowed to ±Infinity on sloped rails, then collided so that
/// `+Inf + -Inf` = NaN. A NaN velocity is dead physics — the cart does not fall
/// when the block under it goes, and nothing but a piston can move it — which
/// is exactly why the builders use them to pin villagers and other carts at
/// exact positions. Rewriting one as `0.0` turns it back into an ordinary cart
/// that moves, falls, and is shoved by its neighbours, and the door comes apart
/// with no error anywhere. This world really does carry six of them; see
/// `crates/mc-tick/docs/entity-abuse-in-record-doors.md`.
///
/// The spelling is Java's `Double.toString` — `NaN`, `Infinity`, `-Infinity` —
/// which is what vanilla's own NBT writer emits, and what mc-tick's `float`
/// reader was taught to accept. The `d` suffix is dropped for these three: no
/// other tag type can hold them, so there is nothing to disambiguate.
fn snbt_double(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() { "Infinity" } else { "-Infinity" }.to_string();
    }
    let mut text = format!("{value}");
    if !text.contains('.') {
        text.push_str(".0");
    }
    text.push('d');
    text
}

/// Any numeric NBT tag as `f64`, whatever width the file happened to use.
fn nbt_number(value: &crate::entity::NbtValue) -> Option<f64> {
    use crate::entity::NbtValue as V;
    match value {
        V::Double(v) => Some(*v),
        V::Float(v) => Some(f64::from(*v)),
        V::Int(v) => Some(f64::from(*v)),
        V::Short(v) => Some(f64::from(*v)),
        V::Byte(v) => Some(f64::from(*v)),
        V::Long(v) => Some(*v as f64),
        _ => None,
    }
}

/// A three-element numeric NBT list — `Motion`, in practice.
///
/// A missing or malformed one is zero rather than an error: vanilla omits
/// `Motion` on a resting entity, and that is exactly what zero means.
fn nbt_vec3(value: Option<&crate::entity::NbtValue>) -> [f64; 3] {
    if let Some(crate::entity::NbtValue::List(items)) = value {
        if items.len() == 3 {
            let parsed: Vec<f64> = items.iter().filter_map(nbt_number).collect();
            if parsed.len() == 3 {
                return [parsed[0], parsed[1], parsed[2]];
            }
        }
    }
    [0.0; 3]
}

/// Render a schematic's mobile entities as the gametest `entities` list.
///
/// The shape is the one `mc_tick::structure`'s `entity_entry` reads:
/// `{pos: [..], blockPos: [..], nbt: {id, Motion, Item, PickupDelay}}`. Only
/// `pos` and `nbt` are consumed; `blockPos` is skipped by the parser but
/// written anyway so the output stays a real structure file rather than a
/// private dialect that only we can read.
///
/// Positions shift by the same `bb.min` the blocks use, because the parser
/// places entities in the structure's own frame, not the schematic's.
///
/// Every entity is emitted, including types the engine cannot model. That is
/// deliberate: the parser refuses those by name, which is the whole point —
/// filtering here would restore the silent drop this replaced.
fn entities_snbt(schematic: &crate::UniversalSchematic, min: (i32, i32, i32)) -> String {
    use crate::entity::NbtValue as V;
    use std::fmt::Write as _;

    let (mx, my, mz) = min;
    let mut out = String::new();
    for entity in schematic.get_entities_as_list() {
        // The parser matches ids exactly (`minecraft:item`), and some writers
        // store the short form. Namespace it here so a bare `item` is not
        // mistaken for a type we cannot simulate.
        let id = if entity.id.contains(':') {
            entity.id.clone()
        } else {
            format!("minecraft:{}", entity.id)
        };
        let pos = [
            entity.position.0 - f64::from(mx),
            entity.position.1 - f64::from(my),
            entity.position.2 - f64::from(mz),
        ];
        let motion = nbt_vec3(entity.nbt.get("Motion"));

        if !out.is_empty() {
            out.push_str(",\n    ");
        }
        let _ = write!(
            out,
            "{{pos: [{}, {}, {}], blockPos: [{}, {}, {}], nbt: {{id: \"{id}\"",
            snbt_double(pos[0]),
            snbt_double(pos[1]),
            snbt_double(pos[2]),
            pos[0].floor() as i32,
            pos[1].floor() as i32,
            pos[2].floor() as i32,
        );
        let _ = write!(
            out,
            ", Motion: [{}, {}, {}]",
            snbt_double(motion[0]),
            snbt_double(motion[1]),
            snbt_double(motion[2])
        );
        // An item entity is nothing without its stack, and the engine refuses
        // one that arrives without an `Item`.
        if let Some(V::Compound(stack)) = entity.nbt.get("Item") {
            if let Some(V::String(item_id)) = stack.get("id") {
                // Both spellings are in the wild — `Count` before the item
                // components rework, `count` after — and the engine's reader
                // takes either, so normalise to one rather than guess.
                let count = stack
                    .get("count")
                    .or_else(|| stack.get("Count"))
                    .and_then(nbt_number)
                    .unwrap_or(1.0);
                let _ = write!(out, ", Item: {{id: \"{item_id}\", count: {}b}}", count as i64);
            }
        }
        if let Some(delay) = entity.nbt.get("PickupDelay").and_then(nbt_number) {
            let _ = write!(out, ", PickupDelay: {}s", delay as i64);
        }
        // `Rotation` is mechanism, not decoration. A cart's yaw gates whether
        // it can push a neighbour at all, so dropping the tag here silently
        // turned every parked cart in a loaded world into one facing +X. In the
        // record 3x3 door that is the difference between a motionless top row
        // and a row that shoves itself apart on tick 2 — see
        // `mc_tick::structure::SpawnedFurnaceMinecart::yaw`.
        if let Some(V::List(rotation)) = entity.nbt.get("Rotation") {
            if let Some(yaw) = rotation.first().and_then(nbt_number) {
                // `Rotation` is a float list, so the suffix is `f` — not the
                // `d` `snbt_double` appends, which the reader rejects outright.
                let mut text = format!("{yaw}");
                if !text.contains('.') {
                    text.push_str(".0");
                }
                let _ = write!(out, ", Rotation: [{text}f, 0.0f]");
            }
        }
        // A furnace cart's self-drive. Every one in that door reads zero, but
        // the engine *refuses* a fuelled cart rather than running it as an
        // unfuelled one, and a writer that never emits `Fuel` makes that
        // refusal unreachable — the exact shape of silent wrongness the
        // refusal exists to prevent.
        for (tag, key) in [("Fuel", "Fuel"), ("PushX", "PushX"), ("PushZ", "PushZ")] {
            if let Some(value) = entity.nbt.get(key).and_then(nbt_number) {
                if value != 0.0 {
                    if tag == "Fuel" {
                        let _ = write!(out, ", Fuel: {}s", value as i64);
                    } else {
                        let _ = write!(out, ", {tag}: {}", snbt_double(value));
                    }
                }
            }
        }
        out.push_str("}}");
    }
    out
}

/// Render a schematic as vanilla gametest structure SNBT — the flavor
/// `mc_tick::Structure::parse` reads (`palette` + indexed `blocks` +
/// bracketless `Properties`, block-entity `nbt` inline). The
/// `formats::structure_snbt` exporter emits the *data-flavor* instead
/// (inline `state:"id{k:v}"` strings), which mc-tick rejects — so this
/// builds the gametest flavor directly and keeps mc-tick's proven parser
/// as the single reader.
///
/// Pre-flattening builds are converted first; see [`modernized`].
fn to_gametest_snbt(schematic: &crate::UniversalSchematic) -> String {
    if let Some(modern) = modernized(schematic) {
        return to_gametest_snbt(&modern);
    }
    use std::collections::HashMap;
    use std::fmt::Write as _;

    let bb = schematic.get_bounding_box();
    let (mx, my, mz) = bb.min;
    let size = (bb.max.0 - mx + 1, bb.max.1 - my + 1, bb.max.2 - mz + 1);

    let mut nbt_at: HashMap<(i32, i32, i32), String> = HashMap::new();
    for be in schematic.get_block_entities_as_list() {
        let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
        nbt_at.insert(be.position, snbt);
    }

    let mut palette: Vec<String> = Vec::new();
    let mut palette_index: HashMap<String, usize> = HashMap::new();
    let mut blocks = String::new();
    for (pos, state) in schematic.iter_blocks() {
        if state.name == "minecraft:air" {
            continue;
        }
        let mut props: Vec<(&str, &str)> =
            state.properties.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        props.sort();
        let mut entry = format!("{{Name:\"{}\"", state.name);
        if !props.is_empty() {
            entry.push_str(", Properties:{");
            for (i, (k, v)) in props.iter().enumerate() {
                if i > 0 {
                    entry.push_str(", ");
                }
                let _ = write!(entry, "{k}: \"{v}\"");
            }
            entry.push('}');
        }
        entry.push('}');
        let index = *palette_index.entry(entry.clone()).or_insert_with(|| {
            palette.push(entry);
            palette.len() - 1
        });
        if !blocks.is_empty() {
            blocks.push_str(",\n    ");
        }
        let _ = write!(
            blocks,
            "{{pos: [{}, {}, {}], state: {}",
            pos.x - mx,
            pos.y - my,
            pos.z - mz
        , index);
        if let Some(nbt) = nbt_at.get(&(pos.x, pos.y, pos.z)) {
            let _ = write!(blocks, ", nbt: {nbt}");
        }
        blocks.push('}');
    }

    format!(
        "{{\n  DataVersion: 4903,\n  size: [{}, {}, {}],\n  palette: [\n    {}\n  ],\n  blocks: [\n    {}\n  ],\n  entities: [\n    {}\n  ]\n}}\n",
        size.0,
        size.1,
        size.2,
        palette.join(",\n    "),
        blocks,
        entities_snbt(schematic, (mx, my, mz))
    )
}


/// The sentence shown to whoever is holding a structure that will not load.
///
/// Two very different failures reach the same `parse` call and they need
/// opposite answers. An unsupported entity means their *build* names something
/// the engine cannot model — nothing is wrong with the file, so it is reported
/// by name and without blame. Anything else means the text itself is bad;
/// when we generated that text (`converted`), that is our bug and saying so
/// keeps us from accusing a perfectly good upload.
fn structure_parse_detail(error: &mc_tick::structure::StructureError, converted: bool) -> String {
    if let mc_tick::structure::StructureError::UnsupportedEntity { entity_type, .. } = error {
        return format!(
            "this build contains a `{entity_type}` entity, which the engine cannot simulate \
             yet — loading it would mean dropping the entity, and a run without it would not \
             match the real build"
        );
    }
    if converted {
        format!(
            "converted structure did not parse: {error:?} — this is an engine fault, \
             not a problem with the uploaded file"
        )
    } else {
        format!("structure SNBT did not parse: {error:?}")
    }
}

/// Serialise recorded updates for ticks in `[from, to)`.
///
/// Shared by the whole-log and per-tick-range accessors so both emit exactly
/// one schema. `state` is the block at dispatch time, not at the tick boundary.
fn updates_json_range(sim: &mc_tick::Simulation, from: u64, to: u64) -> String {
    use std::fmt::Write as _;
    let mut json = String::from("[");
    let mut first = true;
    for update in sim.recorded_updates() {
        if update.tick < from || update.tick >= to {
            continue;
        }
        if !first {
            json.push(',');
        }
        first = false;
        let state = sim.registry().descriptor(update.state).unwrap_or("minecraft:air");
        let kind = match update.kind {
            mc_tick::UpdateKind::Neighbor => "neighbor",
            mc_tick::UpdateKind::Shape => "shape",
        };
        // No phase means a boundary dispatch: placement, a click, a break —
        // the server loop rather than a phase of the tick.
        let phase = update.phase.map_or("boundary", |p| p.name());
        let _ = write!(
            json,
            "{{\"tick\":{},\"seq\":{},\"pos\":[{},{},{}],\"from\":\"{:?}\",\"kind\":\"{}\",\"phase\":\"{}\",\"state\":\"{}\"}}",
            update.tick,
            update.seq,
            update.pos.x,
            update.pos.y,
            update.pos.z,
            update.from,
            kind,
            phase,
            state
        );
    }
    json.push(']');
    json
}

/// The phase legend shared by the compact update views: index 0 is a boundary
/// dispatch (outside the phase walk), then [`mc_tick::PHASE_ORDER`].
fn phase_legend() -> Vec<&'static str> {
    let mut names = vec!["boundary"];
    names.extend(mc_tick::PHASE_ORDER.iter().map(|p| p.name()));
    names
}

/// A record's index into [`phase_legend`].
fn phase_code(update: &mc_tick::UpdateRecord) -> usize {
    match update.phase {
        None => 0,
        Some(phase) => {
            mc_tick::PHASE_ORDER.iter().position(|p| *p == phase).map_or(0, |i| i + 1)
        }
    }
}

/// A direction's index into [`mc_tick::ALL_DIRS`].
fn dir_code(dir: mc_tick::Dir) -> usize {
    mc_tick::ALL_DIRS.iter().position(|d| *d == dir).unwrap_or(0)
}

/// Per-tick, per-cell update counts — the resolution playback runs at.
///
/// The raw log is unusable for a UI: one tick of a 6x6 door is ~20k updates and
/// megabytes of JSON, and twenty thousand individual flares are not legible
/// anyway. Collapsing to "which cells lit up this tick, and how hot" turns that
/// into a few hundred rows while keeping the two breakdowns worth colouring by.
fn updates_heat_range(sim: &mc_tick::Simulation, from: u64, to: u64) -> String {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let phases = phase_legend();
    // BTreeMap so ticks and cells come out in a stable, sorted order.
    let mut per_tick: BTreeMap<u64, BTreeMap<(i32, i32, i32), (u32, u32, u32, Vec<u32>)>> =
        BTreeMap::new();
    for update in sim.recorded_updates() {
        if update.tick < from || update.tick >= to {
            continue;
        }
        let cells = per_tick.entry(update.tick).or_default();
        let cell = cells
            .entry((update.pos.x, update.pos.y, update.pos.z))
            .or_insert_with(|| (0, 0, 0, vec![0; phases.len()]));
        cell.0 += 1;
        match update.kind {
            mc_tick::UpdateKind::Neighbor => cell.1 += 1,
            mc_tick::UpdateKind::Shape => cell.2 += 1,
        }
        cell.3[phase_code(update)] += 1;
    }

    let mut json = String::from("{\"phases\":[");
    for (i, name) in phases.iter().enumerate() {
        let _ = write!(json, "{}\"{name}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"ticks\":[");
    for (i, (tick, cells)) in per_tick.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let total: u32 = cells.values().map(|c| c.0).sum();
        let _ = write!(json, "{{\"tick\":{tick},\"total\":{total},\"cells\":[");
        for (j, ((x, y, z), (n, nb, sh, ph))) in cells.iter().enumerate() {
            if j > 0 {
                json.push(',');
            }
            let _ = write!(
                json,
                "{{\"p\":[{x},{y},{z}],\"n\":{n},\"nb\":{nb},\"sh\":{sh},\"ph\":["
            );
            for (k, count) in ph.iter().enumerate() {
                let _ = write!(json, "{}{count}", if k > 0 { "," } else { "" });
            }
            json.push_str("]}");
        }
        json.push_str("]}");
    }
    json.push_str("]}");
    json
}

/// One tick's updates in delivery order, as parallel arrays.
///
/// The wavefront resolution: everything the raw log has for a single tick, but
/// without repeating a field name per record. `seq` is the array index; every
/// small enum is an integer code with its legend in the payload; and the
/// dispatch-time state is an index into a deduplicated table, which is where
/// most of the saving comes from — a tick touches thousands of cells but only
/// tens of distinct states.
fn updates_wave(sim: &mc_tick::Simulation, tick: u64) -> String {
    use std::collections::HashMap;
    use std::fmt::Write as _;

    let mut pos = String::new();
    let mut kinds = String::new();
    let mut phases_arr = String::new();
    let mut froms = String::new();
    let mut states_arr = String::new();
    let mut table: Vec<&str> = Vec::new();
    let mut seen: HashMap<mc_tick::StateId, usize> = HashMap::new();
    let mut n = 0usize;

    for update in sim.recorded_updates() {
        if update.tick != tick {
            continue;
        }
        let sep = if n > 0 { "," } else { "" };
        let _ = write!(pos, "{sep}{},{},{}", update.pos.x, update.pos.y, update.pos.z);
        let _ = write!(
            kinds,
            "{sep}{}",
            match update.kind {
                mc_tick::UpdateKind::Neighbor => 0,
                mc_tick::UpdateKind::Shape => 1,
            }
        );
        let _ = write!(phases_arr, "{sep}{}", phase_code(update));
        let _ = write!(froms, "{sep}{}", dir_code(update.from));
        let index = *seen.entry(update.state).or_insert_with(|| {
            table.push(sim.registry().descriptor(update.state).unwrap_or("minecraft:air"));
            table.len() - 1
        });
        let _ = write!(states_arr, "{sep}{index}");
        n += 1;
    }

    let mut json = String::new();
    let _ = write!(json, "{{\"tick\":{tick},\"n\":{n},\"pos\":[{pos}],\"kind\":[{kinds}],");
    let _ = write!(json, "\"phase\":[{phases_arr}],\"from\":[{froms}],\"state\":[{states_arr}],");
    json.push_str("\"states\":[");
    for (i, descriptor) in table.iter().enumerate() {
        let _ = write!(json, "{}\"{descriptor}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"phases\":[");
    for (i, name) in phase_legend().iter().enumerate() {
        let _ = write!(json, "{}\"{name}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"dirs\":[");
    for (i, dir) in mc_tick::ALL_DIRS.iter().enumerate() {
        let _ = write!(json, "{}\"{dir:?}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"kinds\":[\"neighbor\",\"shape\"]}");
    json
}

/// Largest build the simulator will accept, in cells.
///
/// `IRIS_B.schem` is 499x379x442 — 83.5 million cells, a saved world rather
/// than a door. Loading one exhausts the wasm heap, and a Rust OOM in wasm is
/// an `unreachable` trap that poisons the whole instance: every later call on
/// it traps too, so one oversized upload takes down every door after it. Eight
/// million cells is roughly a 200-cube, far past any real door and well inside
/// what the heap survives.
const MAX_VOLUME: usize = 8_000_000;

thread_local! {
    /// Why the last constructor failed. Set on every failure path, cleared on
    /// success, read back through `TickSimulation::last_error_detail`.
    static LAST_ERROR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn set_last_error(detail: impl Into<String>) {
    LAST_ERROR.with(|e| *e.borrow_mut() = detail.into());
}

fn clear_last_error() {
    LAST_ERROR.with(|e| e.borrow_mut().clear());
}

/// Refuse a build too large to load before allocating for it.
fn check_volume(size: (i32, i32, i32)) -> Result<(), String> {
    let volume = (size.0 as i64) * (size.1 as i64) * (size.2 as i64);
    if volume > MAX_VOLUME as i64 {
        return Err(format!(
            "build is {} x {} x {} = {volume} cells, over the {MAX_VOLUME}-cell limit — \
             this looks like a saved world rather than a contraption",
            size.0, size.1, size.2
        ));
    }
    Ok(())
}

fn wire_simulation(
    structure: &mc_tick::Structure,
    hash_origin: mc_tick::Pos,
    settle: ffi::TickSettleMode,
    extra_states: &[&str],
    source_data_version: Option<i32>,
) -> Result<mc_tick::Simulation, String> {
    use mc_tick::{Pos, Simulation};
    const MARGIN: i32 = 4;

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // Which game's `Entity.load` these entities came through.
    //
    // The blocks in `structure` have been converted to the canonical data
    // version; the *entities* have not been reinterpreted, and whether a NaN
    // velocity survives being read is decided by the version of the save it
    // was read from. `Motion` handling changed at 1.21.11, and the record 3x3
    // door is 1.21.3 — under the new rules its nan carts do not exist. Only
    // the caller knows the source version, which is why it is a parameter and
    // not something the engine tries to infer. See [`mc_tick::MotionSemantics`].
    if let Some(version) = source_data_version {
        sim.set_motion_semantics(mc_tick::MotionSemantics::for_data_version(version));
    }
    // The universal actuator, plus anything the caller names.
    let mut wanted: Vec<String> = vec!["minecraft:redstone_block".to_string()];
    wanted.extend(extra_states.iter().map(|s| s.to_string()));
    // A dispenser can *place* a shulker box it holds as an item; behaviours
    // bind only to interned states, so intern every facing up front.
    for (_, stacks) in &structure.inventories {
        for stack in stacks {
            let base = stack.id.split('[').next().unwrap_or(&stack.id);
            if base.ends_with("_shulker_box") || base == "minecraft:shulker_box" {
                for facing in ["up", "down", "north", "south", "west", "east"] {
                    wanted.push(format!("{base}[facing={facing}]"));
                }
            }
        }
    }
    for descriptor in &wanted {
        sim.registry_mut()
            .intern(descriptor)
            .map_err(|e| format!("interning {descriptor}: {e:?}"))?;
    }
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, strength) in &structure.comparator_outputs {
        sim.set_comparator_output(*pos, *strength);
    }
    for (pos, stacks) in &structure.inventories {
        let entry = structure
            .blocks
            .iter()
            .find(|(p, _)| p == pos)
            .map(|(_, e)| *e)
            .ok_or_else(|| format!("inventory at {pos:?} with no block"))?;
        let name = structure.palette[entry].split('[').next().unwrap_or_default().to_string();
        let slots = mc_tick::vanilla::container_slots(&name)
            .ok_or_else(|| format!("{name} has an inventory but no slot count"))?;
        sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
        *sim.behaviours_mut() = table;
    }
    if let Some(report) = sim.unknown_report() {
        return Err(format!("blocks without behaviour: {report}"));
    }
    {
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
    }
    // Entities the parser can read but the engine cannot yet run.
    //
    // This is the entity half of the `unknown_report` gate above: a build is
    // refused whole rather than simulated with pieces missing. It lives here,
    // at construction, rather than in the parser, because "can this be read"
    // and "is there a behaviour for it" are different questions — the parser
    // answering both meant the behaviours agent could not even load a villager
    // to develop against.
    //
    // The match is deliberately exhaustive with no catch-all: a new
    // `SpawnedEntity` variant will fail to compile here until someone decides
    // whether it can be simulated. That is the whole point of the split — the
    // gate cannot silently fall out of date.
    // Every arm below either spawns or records a refusal. The match has no
    // catch-all on purpose: a new `SpawnedEntity` variant fails to compile here
    // until someone decides whether it can be simulated, so the gate cannot
    // silently fall out of date.
    let mut refused: Vec<String> = Vec::new();
    for spawned in &structure.entities {
        match spawned {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                sim.spawn_item(item.item.clone(), item.pos, item.motion, item.pickup_delay);
            }
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                sim.spawn_authored_minecart(cart, None);
            }
            // A furnace cart is dimensionally an ordinary cart, so an
            // *unfuelled* one needs nothing more than being a cart. A fuelled
            // one drives itself and is refused rather than run as a passenger.
            mc_tick::structure::SpawnedEntity::FurnaceMinecart(cart) => {
                if let Err(why) = sim.spawn_authored_furnace_minecart(cart, None) {
                    refused.push(why);
                }
            }
            // Fireballs and villagers exist in the record doors as scaffolding:
            // a hitbox a pressure plate can see and a piston can shove. That is
            // all that is implemented, and anything that would need more — a
            // fireball with velocity, a villager that should walk — refuses by
            // name rather than being quietly frozen.
            mc_tick::structure::SpawnedEntity::Fireball(ball) => {
                if let Err(why) = sim.spawn_authored_fireball(ball) {
                    refused.push(why);
                }
            }
            mc_tick::structure::SpawnedEntity::Villager(villager) => {
                if let Err(why) = sim.spawn_authored_villager(villager) {
                    refused.push(why);
                }
            }
        }
    }
    if !refused.is_empty() {
        return Err(format!(
            "{} entit{} in this build need behaviour that is not implemented, and the \
             build is refused rather than simulated with them standing still:\n  - {}",
            refused.len(),
            if refused.len() == 1 { "y" } else { "ies" },
            refused.join("\n  - ")
        ));
    }
    for (pos, entry) in &structure.blocks {
        let state = sim.registry().get(&structure.palette[*entry]);
        let is_ticker = state
            .and_then(|s| sim.behaviours().get(s))
            .is_some_and(|b| b.ticks_as_block_entity());
        if is_ticker {
            sim.add_block_entity_ticker(*pos);
        }
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    if settle != ffi::TickSettleMode::InWorld {
        sim.place_on_place(&order);
    }
    if settle == ffi::TickSettleMode::Placement {
        sim.settle_with_order(&order);
    }
    sim.record();
    Ok(sim)
}

fn is_named(descriptor: &str, needle: &str) -> bool {
    descriptor
        .split('[')
        .next()
        .unwrap_or(descriptor)
        .contains(needle)
}

/// One pass over the world: (non-air count, center-of-mass x, min x, max x).
fn non_air_stats(sim: &mc_tick::Simulation) -> (u32, f64, i32, i32) {
    let mut n = 0u32;
    let mut sum = 0.0;
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    for (pos, _) in sim.world().iter_non_air() {
        n += 1;
        sum += f64::from(pos.x);
        if pos.x < min {
            min = pos.x;
        }
        if pos.x > max {
            max = pos.x;
        }
    }
    (n, if n == 0 { f64::NAN } else { sum / f64::from(n) }, min, max)
}

/// Build a Structure directly from a flat genome-cell array — the GA fast
/// path, no SNBT text. Layout mirrors the flying-ga corridor: machine at
/// `x_off`, world size `[bx + travel, by + 2, bz + 2]`, cells flattened as
/// `((y * bz) + z) * bx + x`, `air` the palette index meaning empty. The
/// full palette rides along (indices are alphabet indices verbatim), so
/// behaviours bind to every alphabet state exactly as the SNBT path did via
/// its EXTRA_STATES list.
fn structure_from_blocks(
    bx: i32,
    by: i32,
    bz: i32,
    travel: i32,
    x_off: i32,
    palette: &[String],
    cells: &[u16],
    air: u16,
) -> Result<mc_tick::Structure, String> {
    let volume = (bx.max(0) as usize) * (by.max(0) as usize) * (bz.max(0) as usize);
    if cells.len() != volume {
        return Err(format!("cells len {} != bbox volume {volume}", cells.len()));
    }
    let mut blocks = Vec::new();
    let mut i = 0usize;
    for _y in 0..by {
        for _z in 0..bz {
            for _x in 0..bx {
                let s = cells[i];
                let (x, y, z) = (_x, _y, _z);
                i += 1;
                if s == air {
                    continue;
                }
                if s as usize >= palette.len() {
                    return Err(format!("palette index {s} out of range"));
                }
                blocks.push((mc_tick::Pos::new(x + x_off, y, z), s as usize));
            }
        }
    }
    Ok(mc_tick::Structure {
        size: (bx + travel, by + 2, bz + 2),
        palette: palette.to_vec(),
        blocks,
        inventories: Vec::new(),
        comparator_outputs: Vec::new(),
        block_entities: Vec::new(),
        entities: Vec::new(),
        item_entities: Vec::new(),
    })
}

/// The modal gait period from min-x rise gaps — bit-identical port of the
/// app's `modalGap` (mode, ties to the smaller gap, modal share >= 0.6).
fn modal_gap(gaps: &[u32]) -> u32 {
    if gaps.len() < 3 {
        return 0;
    }
    let mut order: Vec<u32> = Vec::new();
    let mut counts: Vec<u32> = Vec::new();
    for &g in gaps {
        match order.iter().position(|&o| o == g) {
            Some(i) => counts[i] += 1,
            None => {
                order.push(g);
                counts.push(1);
            }
        }
    }
    let mut best = 0u32;
    let mut best_gap: Option<u32> = None;
    for (i, &g) in order.iter().enumerate() {
        let n = counts[i];
        if n > best || (n == best && best_gap.is_some_and(|b| g < b)) {
            best = n;
            best_gap = Some(g);
        }
    }
    match best_gap {
        Some(g) if (best as f64) / (gaps.len() as f64) >= 0.6 => g,
        _ => 0,
    }
}

/// One kicked flight, mirroring the app's `evalCore.fly` exactly: quiet
/// settle at construction, redstone-block kick at tick 2 removed at tick 4,
/// the same probe schedule (must-move deadline, mid-window centre of mass),
/// optional in-eval gait detection over the last 120 ticks, and an optional
/// early exit for machines that are provably frozen — quiescent and unmoved
/// at tick 40, where every later scalar equals the tick-40 scalar, so the
/// shortcut changes wall time and nothing else.
///
/// Row layout: `[n0, startCom, startMinX, startMaxX, comAtMoveCheck(NaN =
/// no deadline), comAtMid, period, n1, endCom, endMinX, endMaxX]`.
#[allow(clippy::too_many_arguments)]
fn fly_metrics(
    structure: &mc_tick::Structure,
    extras: &[&str],
    kick: (i32, i32, i32),
    eval_ticks: u32,
    seed: i64,
    must_move_by_tick: i32,
    need_period: bool,
    early_exit: bool,
) -> Result<[f64; 11], String> {
    let mut sim = wire_simulation(
        structure,
        mc_tick::Pos::new(0, 0, 0),
        ffi::TickSettleMode::Quiet,
        extras,
        // SNBT in, and the bridge emits the canonical DataVersion — there is no
        // source version to read here. `None` keeps the engine's default,
        // which is the version every captured trace came from.
        None,
    )?;
    fly_on(&mut sim, kick, eval_ticks, seed, must_move_by_tick, need_period, early_exit)
}

/// The flight itself, on an already-wired sim (fresh, quiet-settled).
#[allow(clippy::too_many_arguments)]
fn fly_on(
    sim: &mut mc_tick::Simulation,
    kick: (i32, i32, i32),
    eval_ticks: u32,
    seed: i64,
    must_move_by_tick: i32,
    need_period: bool,
    early_exit: bool,
) -> Result<[f64; 11], String> {
    const PERIOD_WINDOW: u32 = 120;
    const EARLY_TICK: u32 = 40;
    sim.set_rng_seed(seed);

    let (n0, start_com, start_min, start_max) = non_air_stats(sim);
    let mut row = [f64::NAN; 11];
    row[0] = f64::from(n0);
    row[1] = start_com;
    row[2] = f64::from(start_min);
    row[3] = f64::from(start_max);
    if n0 == 0 {
        return Ok(row); // the caller short-circuits on n0 before reading on
    }

    let redstone = sim
        .registry()
        .get("minecraft:redstone_block")
        .ok_or("redstone_block not interned")?;
    let kick_pos = mc_tick::Pos::new(kick.0, kick.1, kick.2);
    sim.run(2);
    sim.place_block(kick_pos, redstone);
    sim.run(2);
    sim.place_block(kick_pos, mc_tick::StateId::AIR);
    let mut elapsed: u32 = 4;

    let mid_tick = eval_ticks.min((eval_ticks / 2).max(elapsed));
    let move_check: Option<u32> = if must_move_by_tick >= 0 {
        Some((must_move_by_tick as u32).max(elapsed).min(eval_ticks))
    } else {
        None
    };
    let mut probes: Vec<u32> = Vec::new();
    if let Some(mc) = move_check {
        probes.push(mc);
    }
    probes.push(mid_tick);
    if early_exit && eval_ticks > EARLY_TICK {
        probes.push(EARLY_TICK.max(elapsed));
    }
    probes.sort_unstable();
    probes.dedup();

    let mut com_mid = start_com;
    let mut com_move = f64::NAN;
    let mut frozen = false;
    for &t in &probes {
        if t > elapsed {
            sim.run(u64::from(t - elapsed));
            elapsed = t;
        }
        let (_, com, _, _) = non_air_stats(&sim);
        if Some(t) == move_check {
            com_move = com;
        }
        if t == mid_tick {
            com_mid = com;
        }
        if early_exit
            && t == EARLY_TICK
            && sim.is_quiescent()
            && (com - start_com).abs() < 0.25
        {
            frozen = true;
            if mid_tick > t {
                com_mid = com;
            }
            if let Some(mc) = move_check {
                if mc > t && com_move.is_nan() {
                    com_move = com;
                }
            }
            break;
        }
    }

    let mut period = 0u32;
    if need_period && !frozen {
        let win_start = elapsed.max(eval_ticks.saturating_sub(PERIOD_WINDOW));
        if win_start > elapsed {
            sim.run(u64::from(win_start - elapsed));
            elapsed = win_start;
        }
        let mut gaps: Vec<u32> = Vec::new();
        let (_, _, mut prev_min, _) = non_air_stats(&sim);
        let mut last_rise: i64 = -1;
        while elapsed < eval_ticks {
            sim.run(1);
            elapsed += 1;
            let (_, _, mx, _) = non_air_stats(&sim);
            if mx > prev_min {
                if last_rise >= 0 {
                    gaps.push(elapsed - last_rise as u32);
                }
                last_rise = i64::from(elapsed);
            }
            prev_min = mx;
        }
        period = modal_gap(&gaps);
    }
    if !frozen && eval_ticks > elapsed {
        sim.run(u64::from(eval_ticks - elapsed));
    }

    let (n1, end_com, end_min, end_max) = non_air_stats(&sim);
    row[4] = com_move;
    row[5] = com_mid;
    row[6] = f64::from(period);
    row[7] = f64::from(n1);
    row[8] = end_com;
    row[9] = f64::from(end_min);
    row[10] = f64::from(end_max);
    Ok(row)
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};
    use std::fmt::Write;

    /// How the loaded structure is settled before tick 0.
    #[derive(PartialEq, Eq)]
    pub enum TickSettleMode {
        /// Vanilla placement pass + ordered settle — a build saved at rest.
        Placement,
        /// `onPlace` only, no settle — a knownShape capture.
        Quiet,
        /// Neither — a build recorded mid-state in the world it stood in.
        InWorld,
    }

    /// A headless, vanilla-accurate tick simulation of one structure.
    #[diplomat::opaque_mut]
    pub struct TickSimulation {
        pub(crate) sim: mc_tick::Simulation,
        pub(crate) checkpoints: Vec<mc_tick::sim::Checkpoint>,
    }

    impl TickSimulation {
        /// Why the last constructor on this thread failed, in words.
        ///
        /// The enum cannot carry a message, and "Simulation" is useless to
        /// someone holding a door that will not load: the engine already knows
        /// it is `minecraft:waxed_copper_bulb` at (4,2,1) and says so here.
        /// Empty when the last construction succeeded.
        pub fn last_error_detail(out: &mut DiplomatWrite) {
            super::LAST_ERROR.with(|e| {
                let _ = write!(out, "{}", e.borrow());
            });
        }

        /// Largest build this will attempt, in cells.
        ///
        /// A 500x379x442 "door" is a saved world, and loading one exhausts the
        /// wasm heap — after which every later call on that instance traps,
        /// not just the one that overflowed. Refused up front instead.
        pub fn max_volume() -> u32 {
            super::MAX_VOLUME as u32
        }

        /// Load from Java structure SNBT text.
        ///
        /// `extra_states`: semicolon-separated block-state descriptors that
        /// later `place_block` calls may write (behaviours bind at
        /// construction). `minecraft:redstone_block` is always available.
        /// `origin_*`: where the build's (0,0,0) sits in world coordinates —
        /// wire update order hashes absolute positions.
        pub fn from_snbt(
            snbt: &DiplomatStr,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
            extra_states: &DiplomatStr,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            let snbt =
                std::str::from_utf8(snbt).map_err(|_| NucleationError::InvalidArgument)?;
            let extra =
                std::str::from_utf8(extra_states).map_err(|_| NucleationError::InvalidArgument)?;
            super::clear_last_error();
            let structure = mc_tick::Structure::parse(snbt).map_err(|e| {
                super::set_last_error(super::structure_parse_detail(&e, false));
                // An entity we cannot model is not a malformed file, so it
                // reports as a simulator limit rather than a parse failure.
                if matches!(e, mc_tick::structure::StructureError::UnsupportedEntity { .. }) {
                    NucleationError::Simulation
                } else {
                    NucleationError::Parse
                }
            })?;
            super::check_volume(structure.size).map_err(|e| {
                super::set_last_error(e);
                NucleationError::InvalidArgument
            })?;
            let extras: Vec<&str> =
                extra.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
                None,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
        }

        /// Load from a schematic (any format nucleation can read), rendered
        /// to gametest-flavor structure SNBT for mc-tick's parser.
        pub fn from_schematic(
            schematic: &Schematic,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
            extra_states: &DiplomatStr,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            super::clear_last_error();
            let extra =
                std::str::from_utf8(extra_states).map_err(|_| NucleationError::InvalidArgument)?;
            // Before rendering anything: the SNBT for a world-sized build is
            // what exhausts the heap, and the trap it raises poisons the
            // instance for every door after it.
            let bb = schematic.0.get_bounding_box();
            super::check_volume((
                bb.max.0 - bb.min.0 + 1,
                bb.max.1 - bb.min.1 + 1,
                bb.max.2 - bb.min.2 + 1,
            ))
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::InvalidArgument
            })?;
            let snbt = super::to_gametest_snbt(&schematic.0);
            let structure = mc_tick::Structure::parse(&snbt).map_err(|e| {
                // The schematic loaded; either it names an entity we cannot
                // model, or our own rendering of it was rejected. Saying
                // "Parse" here blames the user's file for our bug, so both
                // report as an engine failure and the detail says which.
                super::set_last_error(super::structure_parse_detail(&e, true));
                NucleationError::Simulation
            })?;
            let extras: Vec<&str> =
                extra.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
                // The schematic remembers which game wrote it, and that
                // decides whether a non-finite `Motion` survives
                // `Entity.load` — the mechanism of the record nan-cart
                // doors. Passed through rather than defaulted: a 1.21.3
                // door and a 1.21.11 door are different machines, and
                // only this layer knows which one this is.
                schematic.0.metadata.source_data_version,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
        }

        /// GA fast path: construct from a flat genome-cell array — no SNBT
        /// text built or parsed. Corridor layout matches the flying-ga app:
        /// machine at `x_off`, world size `[bx + travel, by + 2, bz + 2]`,
        /// cells flattened `((y * bz) + z) * bx + x`, `air_index` = empty
        /// cell. `palette` is the run's alphabet, semicolon-separated; every
        /// entry is pre-interned so behaviours bind exactly as the SNBT
        /// path's EXTRA_STATES did.
        #[allow(clippy::too_many_arguments)]
        pub fn from_blocks(
            bx: i32,
            by: i32,
            bz: i32,
            travel: i32,
            x_off: i32,
            palette: &DiplomatStr,
            cells: &[u16],
            air_index: u16,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            let palette =
                std::str::from_utf8(palette).map_err(|_| NucleationError::InvalidArgument)?;
            let pal: Vec<String> = palette
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let structure =
                super::structure_from_blocks(bx, by, bz, travel, x_off, &pal, cells, air_index)
                    .map_err(|_| NucleationError::InvalidArgument)?;
            let extras: Vec<&str> = pal.iter().map(String::as_str).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
                None,
            )
            .map_err(|_| NucleationError::Simulation)?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
        }

        /// Evaluate a whole batch of kicked flights inside the engine — one
        /// wasm call per generation chunk instead of a dozen boundary calls
        /// per machine. `cells` holds N genomes concatenated (each
        /// `bx*by*bz` entries), `kicks` N structure-space `[x,y,z]` triples.
        /// The flight protocol, probe schedule and gait detection mirror the
        /// app's evalCore exactly; `early_exit` stops provably-frozen
        /// machines at tick 40 without changing any reported value. Writes
        /// JSON rows `[n0, startCom, startMinX, startMaxX, comAtMoveCheck |
        /// null, comAtMid, period, n1, endCom, endMinX, endMaxX]`.
        #[allow(clippy::too_many_arguments)]
        pub fn eval_flight_batch(
            bx: i32,
            by: i32,
            bz: i32,
            travel: i32,
            x_off: i32,
            palette: &DiplomatStr,
            cells: &[u16],
            air_index: u16,
            kicks: &[i32],
            eval_ticks: u32,
            seed: i64,
            must_move_by_tick: i32,
            need_period: bool,
            early_exit: bool,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let palette =
                std::str::from_utf8(palette).map_err(|_| NucleationError::InvalidArgument)?;
            let pal: Vec<String> = palette
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let extras: Vec<&str> = pal.iter().map(String::as_str).collect();
            let volume = (bx.max(0) as usize) * (by.max(0) as usize) * (bz.max(0) as usize);
            if volume == 0
                || cells.len() % volume != 0
                || kicks.len() != (cells.len() / volume) * 3
            {
                return Err(NucleationError::InvalidArgument);
            }
            let n_genomes = cells.len() / volume;
            // Wire ONE empty-corridor sim (registry, behaviours, physics
            // tables — the expensive part), checkpoint it pristine, and per
            // genome restore + place. Construction cost is paid once per
            // batch instead of once per machine.
            let empty = vec![air_index; volume];
            let empty_structure = super::structure_from_blocks(
                bx, by, bz, travel, x_off, &pal, &empty, air_index,
            )
            .map_err(|_| NucleationError::InvalidArgument)?;
            let mut sim = super::wire_simulation(
                &empty_structure,
                mc_tick::Pos::new(0, 0, 0),
                TickSettleMode::Quiet,
                &extras,
                None,
            )
            .map_err(|_| NucleationError::Simulation)?;
            let pristine = sim.checkpoint();
            let mut json = String::from("[");
            for g in 0..n_genomes {
                let slice = &cells[g * volume..(g + 1) * volume];
                let structure = super::structure_from_blocks(
                    bx, by, bz, travel, x_off, &pal, slice, air_index,
                )
                .map_err(|_| NucleationError::InvalidArgument)?;
                sim.restore(&pristine);
                {
                    let (registry, world) = sim.registry_and_world_mut();
                    structure.place(world, registry, mc_tick::Pos::new(0, 0, 0));
                }
                // Quiet settle for the placed genome, exactly as
                // wire_simulation would have done for a fresh sim.
                let order = structure.placement_order(
                    mc_tick::vanilla::is_collision_full_cube,
                    mc_tick::vanilla::has_dynamic_shape,
                );
                sim.place_on_place(&order);
                sim.record();
                let kick = (kicks[g * 3], kicks[g * 3 + 1], kicks[g * 3 + 2]);
                let row = super::fly_on(
                    &mut sim,
                    kick,
                    eval_ticks,
                    seed,
                    must_move_by_tick,
                    need_period,
                    early_exit,
                )
                .map_err(|_| NucleationError::Simulation)?;
                if g > 0 {
                    json.push(',');
                }
                json.push('[');
                for (i, v) in row.iter().enumerate() {
                    if i > 0 {
                        json.push(',');
                    }
                    if v.is_nan() {
                        json.push_str("null");
                    } else {
                        let _ = write!(json, "{v:?}");
                    }
                }
                json.push(']');
            }
            json.push(']');
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// Seed the vanilla random source (`java.util.Random`'s LCG,
        /// bit-for-bit). Unseeded, jittering behaviours use each
        /// distribution's mean — fully deterministic, no noise.
        pub fn set_rng_seed(&mut self, seed: i64) {
            self.sim.set_rng_seed(seed);
        }

        /// Advance one game tick.
        pub fn step(&mut self) {
            self.sim.step();
        }

        /// Advance `ticks` game ticks.
        pub fn run(&mut self, ticks: u32) {
            self.sim.run(u64::from(ticks));
        }

        /// Run until nothing is scheduled or `budget` ticks pass. Returns
        /// whether the world went quiet.
        pub fn run_until_quiescent(&mut self, budget: u32) -> bool {
            self.sim.run_until_quiescent(u64::from(budget));
            self.sim.is_quiescent()
        }

        /// Game ticks elapsed since settle.
        pub fn tick_count(&self) -> u32 {
            self.sim.tick_count() as u32
        }

        /// Whether nothing is scheduled or queued.
        pub fn is_quiescent(&self) -> bool {
            self.sim.is_quiescent()
        }

        /// Right-click a block with an empty hand (lever, button, note block).
        pub fn use_block(&mut self, x: i32, y: i32, z: i32) {
            self.sim.use_block(mc_tick::Pos::new(x, y, z));
        }

        /// Write a block state (`minecraft:air` breaks). The state must be in
        /// the structure, in `extra_states`, or `minecraft:redstone_block`.
        pub fn place_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            state: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let state =
                std::str::from_utf8(state).map_err(|_| NucleationError::InvalidArgument)?;
            let id = self
                .sim
                .registry()
                .get(state)
                .ok_or(NucleationError::NotFound)?;
            self.sim.place_block(mc_tick::Pos::new(x, y, z), id);
            Ok(())
        }

        /// The block state descriptor at a position (`minecraft:air` for empty).
        pub fn get_block(&self, x: i32, y: i32, z: i32, out: &mut DiplomatWrite) {
            let id = self.sim.world().get(mc_tick::Pos::new(x, y, z));
            let descriptor = self.sim.registry().descriptor(id).unwrap_or("minecraft:air");
            let _ = write!(out, "{descriptor}");
        }

        /// Snapshot the entire simulation; returns a checkpoint id.
        pub fn checkpoint(&mut self) -> u32 {
            self.checkpoints.push(self.sim.checkpoint());
            (self.checkpoints.len() - 1) as u32
        }

        /// Restore a checkpoint taken earlier on this simulation.
        pub fn restore(&mut self, id: u32) -> Result<(), NucleationError> {
            let checkpoint = self
                .checkpoints
                .get(id as usize)
                .ok_or(NucleationError::NotFound)?;
            self.sim.restore(checkpoint);
            Ok(())
        }

        /// Render a schematic as gametest-flavor structure SNBT — the text
        /// `from_snbt` and the corpus/render tooling consume. Lets hosts hand
        /// a converted `.litematic`/`.schem` to the video renderer.
        pub fn gametest_snbt(schematic: &Schematic, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::to_gametest_snbt(&schematic.0));
        }

        /// Report blocks whose behaviour is defined by block-entity data the
        /// file does not carry.
        ///
        /// Some exporters write the blocks and drop the block entities. The
        /// build then loads clean and simulates *wrongly but plausibly*: a
        /// comparator with no `OutputSignal` reads 0, a barrel holding the
        /// item that latched a repeater reads empty, and the door quietly
        /// fails to reset. Two files with identical block arrays get
        /// different verdicts and nothing says why. `0.45_4x4_funnel.schem`
        /// is exactly this — 4 comparators, 2 furnaces, `BlockEntities` of
        /// length 0, while its `.litematic` twin carries all 9.
        ///
        /// This does not refuse the build; it names the doubt so a host can.
        /// JSON: `{"present":N,"missing_total":N,"missing":[{"name":..,
        /// "count":N}],"summary":"..."}` — `summary` is empty when nothing
        /// is missing, and otherwise a sentence fit to show as-is.
        pub fn block_entity_audit_json(schematic: &Schematic, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::block_entity_audit(&schematic.0));
        }

        /// Start (or stop) recording every delivered redstone update.
        ///
        /// Off by default and much larger than the block-change log — a door's
        /// cycle runs several updates per change — so a propagation view asks
        /// for it explicitly and pages with
        /// [`TickSimulation::updates_json_between`].
        ///
        /// Switching it off keeps what was recorded; use
        /// [`TickSimulation::clear_updates`] to free it.
        pub fn record_updates(&mut self, on: bool) {
            self.sim.record_updates(on);
        }

        /// Drop the recorded updates without changing whether recording is on.
        ///
        /// A cycle of a 6x6 door is tens of megabytes of log, so a page that
        /// certifies several builds on one instance needs to release one
        /// before recording the next.
        pub fn clear_updates(&mut self) {
            self.sim.clear_updates();
        }

        /// How many updates have been recorded — page before pulling them.
        pub fn updates_count(&self) -> u32 {
            self.sim.recorded_updates().len() as u32
        }

        /// Every recorded update, in delivery order.
        ///
        /// `seq` counts from 0 within each tick: that is the sub-tick axis, and
        /// `(tick, seq)` is the order the engine actually delivered them in.
        /// `state` is the block as it stood **at dispatch time**, which is what
        /// makes intra-tick order legible — a snapshot cannot show it.
        pub fn updates_json(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::updates_json_range(&self.sim, 0, u64::MAX));
        }

        /// The recorded updates for ticks in `[from_tick, to_tick)`.
        ///
        /// The whole log for a 6x6 door's cycle is megabytes; a scrubber only
        /// ever shows one tick, so it should ask for one tick.
        pub fn updates_json_between(
            &self,
            from_tick: u32,
            to_tick: u32,
            out: &mut DiplomatWrite,
        ) {
            let _ = write!(
                out,
                "{}",
                super::updates_json_range(&self.sim, u64::from(from_tick), u64::from(to_tick))
            );
        }

        /// Per-tick, per-cell update counts for ticks in `[from_tick, to_tick)`.
        ///
        /// The resolution playback should run at: `{phases, ticks:[{tick, total,
        /// cells:[{p:[x,y,z], n, nb, sh, ph:[…]}]}]}`, where `nb`/`sh` split
        /// neighbour from shape and `ph` indexes the `phases` legend. Collapses
        /// a tick's tens of thousands of updates into a few hundred cells.
        pub fn updates_heat_json(
            &self,
            from_tick: u32,
            to_tick: u32,
            out: &mut DiplomatWrite,
        ) {
            let _ = write!(
                out,
                "{}",
                super::updates_heat_range(&self.sim, u64::from(from_tick), u64::from(to_tick))
            );
        }

        /// One tick's updates in delivery order, as parallel arrays.
        ///
        /// For stepping *within* a tick: `seq` is the array index, `pos` is flat
        /// x,y,z triples, `kind`/`phase`/`from` are integer codes with legends
        /// in the payload, and `state` indexes a deduplicated `states` table.
        pub fn updates_wave_json(&self, tick: u32, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::updates_wave(&self.sim, u64::from(tick)));
        }

        pub fn changes_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("[");
            for (i, change) in self.sim.recorded().iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let from = self.sim.registry().descriptor(change.from).unwrap_or("?");
                let to = self.sim.registry().descriptor(change.to).unwrap_or("?");
                let _ = write!(
                    json,
                    "{{\"tick\":{},\"pos\":[{},{},{}],\"from\":\"{}\",\"to\":\"{}\"}}",
                    change.tick, change.pos.x, change.pos.y, change.pos.z, from, to
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }

        /// Live item entities and minecarts, as JSON:
        /// `{"items":[{"id":N,"item":"...","count":N,"pos":[..],"vel":[..],
        ///   "on_ground":bool,"contents":[{"id":"...","count":N}]}],
        ///  "minecarts":[{"id":N,"kind":"...","pos":[..],"vel":[..]}]}`.
        pub fn item_entities_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("{\"items\":[");
            let mut first = true;
            for entity in self.sim.item_entities() {
                if entity.removed {
                    continue;
                }
                if !first {
                    json.push(',');
                }
                first = false;
                let _ = write!(
                    json,
                    "{{\"id\":{},\"item\":\"{}\",\"count\":{},\"pos\":[{},{},{}],\"vel\":[{},{},{}],\"on_ground\":{}",
                    entity.id,
                    entity.item.0,
                    entity.item.1,
                    entity.pos[0], entity.pos[1], entity.pos[2],
                    entity.vel[0], entity.vel[1], entity.vel[2],
                    entity.on_ground,
                );
                json.push_str(",\"contents\":[");
                let contents = self.sim.item_contents(entity.id).unwrap_or(&[]);
                for (i, stack) in contents.iter().enumerate() {
                    if i > 0 {
                        json.push(',');
                    }
                    let _ = write!(
                        json,
                        "{{\"id\":\"{}\",\"count\":{}}}",
                        stack.id, stack.count
                    );
                }
                json.push_str("]}");
            }
            json.push_str("],\"minecarts\":[");
            let mut first = true;
            for cart in self.sim.minecarts() {
                if cart.removed {
                    continue;
                }
                if !first {
                    json.push(',');
                }
                first = false;
                let _ = write!(
                    json,
                    "{{\"id\":{},\"kind\":\"{}\",\"pos\":[{},{},{}],\"vel\":[{},{},{}]}}",
                    cart.id,
                    cart.kind,
                    cart.pos[0], cart.pos[1], cart.pos[2],
                    cart.vel[0], cart.vel[1], cart.vel[2],
                );
            }
            json.push_str("],\"frozen\":[");
            let mut first = true;
            for body in self.sim.entity_bodies() {
                if body.is_minecart {
                    continue;
                }
                if !first {
                    json.push(',');
                }
                first = false;
                let _ = write!(
                    json,
                    "{{\"id\":{},\"kind\":\"{}\",\"pos\":[{},{},{}]}}",
                    body.id,
                    body.kind,
                    (body.min[0] + body.max[0]) / 2.0,
                    body.min[1],
                    (body.min[2] + body.max[2]) / 2.0,
                );
            }
            json.push_str("]}");
            let _ = write!(out, "{json}");
        }

        /// Which `Entity.load` Motion semantics this run uses:
        /// `"clamp_abs_ten"` (DataVersion <= 4556 — NaN survives a cold load)
        /// or `"drop_non_finite"` (>= 4671 — it does not).
        ///
        /// Exposed because a door built on nan carts is a *different machine*
        /// under the two, and a caller that cannot tell them apart cannot
        /// report why it came apart.
        pub fn motion_semantics(&self, out: &mut DiplomatWrite) {
            let name = match self.sim.motion_semantics() {
                mc_tick::MotionSemantics::ClampAbsTen => "clamp_abs_ten",
                mc_tick::MotionSemantics::DropNonFinite => "drop_non_finite",
            };
            let _ = write!(out, "{name}");
        }

        /// How many times an entity stood in a **retracting** piston's sweep.
        ///
        /// Piston extension displacement is measured and implemented;
        /// retraction is not — `tools/gametest/captures/piston_pull.entities.log`
        /// records sub-0.03 movements that are not uniformly backwards and
        /// that no model here reproduces. Non-zero means this run leaned on
        /// unimplemented behaviour and its result is not trustworthy. Zero
        /// means no entity was ever in a retracting arm's way: a real answer,
        /// not a missing instrument.
        pub fn piston_retract_contacts(&self) -> u32 {
            self.sim.piston_retract_contacts().len() as u32
        }

        /// Per-tick aggregates over the recorded changes, as JSON:
        /// `[{"tick":N,"changes":N,"piston":N,"redstone":N}]` — `piston`
        /// counts changes touching piston blocks (base, head, moving), and
        /// `redstone` changes touching wire/torch/repeater/comparator/
        /// observer/lamp/lever/button/pressure-plate states.
        pub fn events_summary_json(&self, out: &mut DiplomatWrite) {
            use std::collections::BTreeMap;
            #[derive(Default)]
            struct Row {
                changes: u32,
                piston: u32,
                redstone: u32,
            }
            let mut rows: BTreeMap<u64, Row> = BTreeMap::new();
            for change in self.sim.recorded() {
                let from = self.sim.registry().descriptor(change.from).unwrap_or("");
                let to = self.sim.registry().descriptor(change.to).unwrap_or("");
                let row = rows.entry(change.tick).or_default();
                row.changes += 1;
                let named = |needle: &str| {
                    super::is_named(from, needle) || super::is_named(to, needle)
                };
                if named("piston") {
                    row.piston += 1;
                }
                if named("redstone")
                    || named("repeater")
                    || named("comparator")
                    || named("observer")
                    || named("lever")
                    || named("button")
                    || named("pressure_plate")
                    || named("lamp")
                {
                    row.redstone += 1;
                }
            }
            let mut json = String::from("[");
            for (i, (tick, row)) in rows.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let _ = write!(
                    json,
                    "{{\"tick\":{},\"changes\":{},\"piston\":{},\"redstone\":{}}}",
                    tick, row.changes, row.piston, row.redstone
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }

        /// Every non-air block, as JSON:
        /// `[{"pos":[x,y,z],"state":"..."}]`.
        /// How many non-air blocks stand in the world right now.
        pub fn non_air_count(&self) -> u32 {
            self.sim.world().non_air_count() as u32
        }

        /// Center of mass (x) of every non-air block — the GA's displacement
        /// metric without a JSON round-trip. NaN when the world is empty.
        pub fn non_air_center_x(&self) -> f64 {
            let mut sum = 0.0;
            let mut n = 0u32;
            for (pos, _) in self.sim.world().iter_non_air() {
                sum += f64::from(pos.x);
                n += 1;
            }
            if n == 0 {
                f64::NAN
            } else {
                sum / f64::from(n)
            }
        }

        /// Smallest x holding a non-air block; `i32::MAX` when empty.
        pub fn non_air_min_x(&self) -> i32 {
            self.sim
                .world()
                .iter_non_air()
                .map(|(pos, _)| pos.x)
                .min()
                .unwrap_or(i32::MAX)
        }

        /// Largest x holding a non-air block; `i32::MIN` when empty.
        pub fn non_air_max_x(&self) -> i32 {
            self.sim
                .world()
                .iter_non_air()
                .map(|(pos, _)| pos.x)
                .max()
                .unwrap_or(i32::MIN)
        }

        /// How many block changes recording has captured so far.
        pub fn changes_count(&self) -> u32 {
            self.sim.recorded().len() as u32
        }

        pub fn world_snapshot_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("[");
            let mut first = true;
            for (pos, id) in self.sim.world().iter_non_air() {
                if !first {
                    json.push(',');
                }
                first = false;
                let state = self.sim.registry().descriptor(id).unwrap_or("?");
                let _ = write!(
                    json,
                    "{{\"pos\":[{},{},{}],\"state\":\"{}\"}}",
                    pos.x, pos.y, pos.z, state
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{block_entity_audit, needs_block_entity, to_gametest_snbt};
    use crate::{BlockState, UniversalSchematic};

    /// A 1.12 build must reach the engine flattened.
    ///
    /// The trap this pins is `minecraft:slime`: in 1.12 that is the *slime
    /// block*, and no modern block has the id — so an unconverted bore loads
    /// with every sticky cell inert and simulates as a machine that cannot
    /// fly, without erroring anywhere.
    #[test]
    fn pre_flattening_ids_are_converted_before_the_engine_sees_them() {
        let mut schem = UniversalSchematic::new("legacy".into());
        schem.metadata.source_data_version = Some(1343); // 1.12.2
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:slime"));
        // 1.12 stored the sub-type as a property; the flattening rules key on
        // it, so a realistic file carries `variant` and a bare id does not
        // convert. Real .litematic/.schem imports always have it.
        schem.set_block(
            1,
            0,
            0,
            &BlockState::new("minecraft:stonebrick").with_property("variant", "stonebrick"),
        );

        let snbt = to_gametest_snbt(&schem);
        assert!(snbt.contains("minecraft:slime_block"), "slime block not flattened: {snbt}");
        assert!(snbt.contains("minecraft:stone_bricks"), "stone brick not flattened: {snbt}");
        assert!(
            !snbt.contains("\"minecraft:slime\""),
            "the 1.12 id survived into the engine's input: {snbt}"
        );
    }


    #[test]
    fn modern_builds_are_passed_through_untouched() {
        let mut schem = UniversalSchematic::new("modern".into());
        schem.metadata.source_data_version = Some(3955);
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:slime_block"));
        assert!(to_gametest_snbt(&schem).contains("minecraft:slime_block"));
    }

    #[test]
    fn audit_names_blocks_whose_block_entity_is_missing() {
        let mut schem = UniversalSchematic::new("stripped".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:comparator"));
        schem.set_block(1, 0, 0, &BlockState::new("minecraft:comparator"));
        schem.set_block(2, 0, 0, &BlockState::new("minecraft:furnace"));
        // Carries no block-entity state that ticks; must not be reported.
        schem.set_block(3, 0, 0, &BlockState::new("minecraft:stone"));

        let json = block_entity_audit(&schem);
        assert!(json.contains("\"missing_total\":3"), "{json}");
        assert!(json.contains("\"name\":\"minecraft:comparator\",\"count\":2"), "{json}");
        assert!(json.contains("\"name\":\"minecraft:furnace\",\"count\":1"), "{json}");
        assert!(!json.contains("stone"), "a block with no ticking NBT was reported: {json}");
        assert!(json.contains("2 comparators"), "summary not written: {json}");
    }

    #[test]
    fn audit_is_silent_when_nothing_is_missing() {
        let mut schem = UniversalSchematic::new("plain".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone"));
        let json = block_entity_audit(&schem);
        assert!(json.contains("\"missing_total\":0"), "{json}");
        assert!(json.contains("\"summary\":\"\""), "{json}");
    }

    #[test]
    fn every_colour_of_shulker_box_counts_as_a_container() {
        assert!(needs_block_entity("minecraft:shulker_box"));
        assert!(needs_block_entity("minecraft:lime_shulker_box"));
        assert!(!needs_block_entity("minecraft:oak_sign"));
    }

    /// Entities survive the trip from schematic to engine input.
    ///
    /// The converter used to write a hardcoded `entities: []`, so a build whose
    /// mechanism depends on entities loaded clean and simulated as though they
    /// were not there. This asserts the whole path: emitted, re-read by the
    /// engine's own parser, with the fields that change a run intact.
    #[test]
    fn entities_round_trip_from_schematic_into_the_engines_parser() {
        use crate::entity::{Entity, NbtValue};
        use std::collections::HashMap;

        let mut schem = UniversalSchematic::new("carts".into());
        // Away from the origin on purpose: entity positions are absolute in
        // the schematic and structure-relative in the SNBT, so a missing
        // `bb.min` shift would sail through a build placed at 0,0,0.
        schem.set_block(10, 0, 5, &BlockState::new("minecraft:rail"));
        schem.set_block(12, 2, 7, &BlockState::new("minecraft:stone"));

        let mut cart = Entity::new("minecraft:minecart".into(), (10.5, 0.0625, 5.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(0.25),
                NbtValue::Double(0.0),
                NbtValue::Double(-0.5),
            ]),
        );
        assert!(schem.add_entity(cart));

        let mut stack = HashMap::new();
        stack.insert("id".to_string(), NbtValue::String("minecraft:redstone".into()));
        stack.insert("count".to_string(), NbtValue::Byte(7));
        let mut item = Entity::new("minecraft:item".into(), (11.5, 1.0, 6.5));
        item.nbt.insert("Item".into(), NbtValue::Compound(stack));
        item.nbt.insert("PickupDelay".into(), NbtValue::Short(40));
        assert!(schem.add_entity(item));

        let snbt = to_gametest_snbt(&schem);
        let parsed = mc_tick::Structure::parse(&snbt)
            .unwrap_or_else(|e| panic!("engine rejected our own output: {e}\n{snbt}"));

        assert_eq!(parsed.entities.len(), 2, "entities dropped: {snbt}");
        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.kind, "minecraft:minecart");
                assert_eq!(cart.pos, [0.5, 0.0625, 0.5], "position not shifted into structure space");
                assert_eq!(cart.motion, [0.25, 0.0, -0.5]);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
        match &parsed.entities[1] {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                assert_eq!(item.pos, [1.5, 1.0, 1.5]);
                assert_eq!(item.item, ("minecraft:redstone".to_string(), 7));
                assert_eq!(item.pickup_delay, 40);
            }
            other => panic!("expected an item, got {other:?}"),
        }
        // The same list also reaches the item-entity view the simulator spawns from.
        assert_eq!(parsed.item_entities.len(), 1);
    }

    /// A denormal motion must not corrupt the entity it belongs to.
    ///
    /// Real furnace minecarts in the 55_3x3 door carry motions like 4.3e-59.
    /// Written with an exponent, the engine's reader takes `4.3` and `-59` as
    /// two numbers, turning a three-element `Motion` into four — which it then
    /// silently discards. The value has to be spelled out in full.
    #[test]
    fn tiny_motions_are_written_without_an_exponent() {
        use crate::entity::{Entity, NbtValue};

        let mut schem = UniversalSchematic::new("denormal".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        let mut cart = Entity::new("minecraft:minecart".into(), (0.5, 0.0, 0.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(4.27987680632209e-59),
                NbtValue::Double(0.0),
                NbtValue::Double(0.0),
            ]),
        );
        assert!(schem.add_entity(cart));

        let snbt = to_gametest_snbt(&schem);
        let (_, entities) = snbt.split_once("entities:").expect("an entities section");
        assert!(
            !entities.contains("e-") && !entities.contains("e+"),
            "an exponent reached the engine's input: {entities}"
        );

        let parsed = mc_tick::Structure::parse(&snbt).expect("parse");
        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.motion, [4.27987680632209e-59, 0.0, 0.0]);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// A nan cart's velocity must reach the engine as NaN, not as zero.
    ///
    /// These exact numbers are lifted from `55_3x3.zip`: a furnace minecart
    /// whose `Motion` is `[4.27987680632209e-59, 0.0, NaN]`. The NaN is the
    /// mechanism — it is what makes the cart's physics dead so it can be used
    /// as glue — so rewriting it to 0.0 turns the cart back into an ordinary
    /// one that moves, and the door quietly falls apart. This pins both halves
    /// at once: the denormal must not become an exponent, and the NaN must not
    /// become a number.
    #[test]
    fn a_nan_cart_velocity_survives_the_round_trip() {
        use crate::entity::{Entity, NbtValue};

        let mut schem = UniversalSchematic::new("nan cart".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        let mut cart = Entity::new("minecraft:minecart".into(), (0.5, 0.0, 0.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(4.27987680632209e-59),
                NbtValue::Double(0.0),
                NbtValue::Double(f64::NAN),
            ]),
        );
        assert!(schem.add_entity(cart));

        let snbt = to_gametest_snbt(&schem);
        let parsed = mc_tick::Structure::parse(&snbt)
            .unwrap_or_else(|e| panic!("a NaN motion must parse, not error: {e}\n{snbt}"));

        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.motion[0], 4.27987680632209e-59, "denormal mangled: {snbt}");
                assert_eq!(cart.motion[1], 0.0);
                assert!(
                    cart.motion[2].is_nan(),
                    "the NaN was sanitised to {} — this un-glues the door: {snbt}",
                    cart.motion[2]
                );
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// The ±Infinity that a nan cart is made from round-trips too.
    ///
    /// Overflowed-but-not-yet-collided carts hold these, and a build captured
    /// mid-sequence carries them. Signed, because `+Inf` and `-Inf` are what
    /// collide to produce the NaN in the first place.
    #[test]
    fn infinite_velocities_survive_the_round_trip() {
        use crate::entity::{Entity, NbtValue};

        let mut schem = UniversalSchematic::new("overflowed".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        let mut cart = Entity::new("minecraft:minecart".into(), (0.5, 0.0, 0.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(f64::INFINITY),
                NbtValue::Double(f64::NEG_INFINITY),
                NbtValue::Double(0.0),
            ]),
        );
        assert!(schem.add_entity(cart));

        let snbt = to_gametest_snbt(&schem);
        let parsed = mc_tick::Structure::parse(&snbt).expect("infinities must parse");
        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.motion[0], f64::INFINITY);
                assert_eq!(cart.motion[1], f64::NEG_INFINITY, "the sign was lost: {snbt}");
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// Load the record 3x3 door sample and wire it under `settle`.
    ///
    /// This goes through the product path — world zip, schematic, gametest
    /// SNBT, `wire_simulation` — because the question is about that path and a
    /// test that reached past it would answer a different one.
    fn wire_record_door(settle: super::ffi::TickSettleMode) -> mc_tick::Simulation {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/samples/55_3x3.zip");
        let bytes = std::fs::read(&path).expect("the record-door sample must be present");
        let schematic = crate::formats::world::from_world_zip(&bytes).expect("the sample loads");
        let snbt = to_gametest_snbt(&schematic);
        let structure = mc_tick::Structure::parse(&snbt).expect("the sample parses");
        super::wire_simulation(
            &structure,
            mc_tick::Pos::new(0, 0, 0),
            settle,
            &[],
            schematic.metadata.source_data_version,
        )
        .expect("the engine must accept the record door")
    }

    /// A build cut out of a running world must load into that world's state.
    ///
    /// `--in-world` capture of this same save records **zero** block changes:
    /// the door is at rest in the game, so it must be at rest in us. The mode
    /// that means "the build *is* the world" is `InWorld`, and this pins that
    /// it genuinely places nothing and settles nothing.
    ///
    /// The `Quiet` half is the negative control, and it is not decoration —
    /// it is the entire reason this test is trustworthy. `Quiet` runs
    /// [`Simulation::place_on_place`], which blanks the region to air and
    /// re-writes every block one at a time, handing each landing block's
    /// already-placed neighbours a shape update. Every observer in the build
    /// therefore watches its facing neighbour *appear*, and pulses. That is
    /// correct for a paste and catastrophic for a load, and it is what made
    /// this door look like it actuated itself: the diagnostic was asking for
    /// `Quiet`. If the two modes ever stop differing here, one of them has
    /// silently become the other and this assertion says so.
    #[test]
    fn the_record_door_is_at_rest_under_in_world_and_disturbed_under_quiet() {
        let mut at_rest = wire_record_door(super::ffi::TickSettleMode::InWorld);
        at_rest.run(200);
        assert_eq!(
            at_rest.recorded().len(),
            0,
            "nobody touched this door: vanilla changes no block ticking the same save in \
             place, so neither may we. First few: {:?}",
            at_rest.recorded().iter().take(4).collect::<Vec<_>>()
        );
        assert!(at_rest.is_quiescent(), "a build at rest has nothing pending");

        // The control. Placement *should* perturb this build, and if it does
        // not then the assertion above is passing for the wrong reason.
        let mut placed = wire_record_door(super::ffi::TickSettleMode::Quiet);
        placed.run(200);
        assert!(
            !placed.recorded().is_empty(),
            "placing this build must disturb it — an observer whose neighbour just \
             appeared pulses. If this is empty, `InWorld` proves nothing."
        );
    }

    /// Wire a one-rail structure carrying `entities`, as the app would.
    fn wire_with_entities(entities: &str) -> Result<mc_tick::Simulation, String> {
        let snbt = format!(
            "{{DataVersion: 4903, size: [1, 1, 1], \
              palette: [{{Name: \"minecraft:rail\"}}], \
              blocks: [{{pos: [0, 0, 0], state: 0}}], entities: [{entities}]}}"
        );
        let structure = mc_tick::Structure::parse(&snbt)
            .unwrap_or_else(|e| panic!("the parser must accept this: {e}\n{snbt}"));
        super::wire_simulation(
            &structure,
            mc_tick::Pos::new(0, 0, 0),
            super::ffi::TickSettleMode::InWorld,
            &[],
            None,
        )
    }

    /// The refusal moved from the parser to the simulator, and still names the
    /// type — but it now fires on *capability*, not on the type existing.
    ///
    /// Furnace carts, fireballs and villagers all load today, as the mass and
    /// hitboxes the record doors use them for. What is refused is any of them
    /// that would need behaviour nobody has implemented: a cart with fuel to
    /// drive itself, a fireball with velocity to fly, a villager with velocity
    /// to walk. That distinction is the whole gate — running one of those as a
    /// stationary box is a confident wrong answer, and the wrongness is
    /// invisible unless it is refused here.
    #[test]
    fn entities_needing_unimplemented_behaviour_are_refused_by_name() {
        for (entity, expected) in [
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:furnace_minecart", Fuel: 3600}}"#,
                "minecraft:furnace_minecart",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:furnace_minecart", PushX: 1.0d}}"#,
                "minecraft:furnace_minecart",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:dragon_fireball", Motion: [0.5d, 0.0d, 0.0d]}}"#,
                "minecraft:dragon_fireball",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:small_fireball", Motion: [0.0d, -0.1d, 0.0d]}}"#,
                "minecraft:small_fireball",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:villager", Motion: [0.0d, 0.0d, 0.2d]}}"#,
                "minecraft:villager",
            ),
        ] {
            let error = wire_with_entities(entity)
                .err()
                .unwrap_or_else(|| panic!("{expected} needs behaviour that does not exist"));
            assert!(error.contains(expected), "refusal does not name the type: {error}");
        }
    }

    /// The scaffolding the record doors actually contain does load.
    ///
    /// An unfuelled furnace cart, a frozen fireball of each size and a
    /// motionless villager: mass and hitboxes, which is all those builds ask
    /// of them. Without this, a gate that refused every one of these types
    /// would pass the test above and look correct.
    #[test]
    fn frozen_scaffolding_entities_load_as_hitboxes() {
        let sim = wire_with_entities(
            r#"{pos: [0.5d, 0.0625d, 0.5d], nbt: {id: "minecraft:furnace_minecart", Fuel: 0}},
               {pos: [1.5d, 1.0d, 0.5d], nbt: {id: "minecraft:dragon_fireball"}},
               {pos: [2.5d, 1.0d, 0.5d], nbt: {id: "minecraft:small_fireball"}},
               {pos: [3.5d, 1.0d, 0.5d], nbt: {id: "minecraft:villager"}}"#,
        )
        .expect("the record doors' scaffolding is exactly what this supports");
        assert_eq!(sim.minecarts().len(), 1, "a furnace cart is a cart");
        let frozen: Vec<&str> = sim
            .entity_bodies()
            .iter()
            .filter(|b| !b.is_minecart)
            .map(|b| b.kind.as_str())
            .collect();
        assert_eq!(
            frozen,
            ["minecraft:dragon_fireball", "minecraft:small_fireball", "minecraft:villager"],
            "each keeps its own identity, because each has its own hitbox"
        );
    }

    /// Negative control: the gate refuses the unmodelled, not everything.
    ///
    /// Without this, a gate that rejected every build in existence would pass
    /// the test above and look correct.
    #[test]
    fn entities_that_do_have_behaviour_still_load() {
        let sim = wire_with_entities(
            r#"{pos: [0.5d, 0.0625d, 0.5d], nbt: {id: "minecraft:minecart", Motion: [0.0d, 0.0d, 0.0d]}},
               {pos: [0.5d, 1.0d, 0.5d], nbt: {id: "minecraft:item", Item: {id: "minecraft:redstone", count: 1b}}}"#,
        )
        .expect("a plain cart and an item are both simulated today");
        assert_eq!(sim.minecarts().len(), 1, "the cart should be live");
        assert_eq!(sim.item_entities().len(), 1, "the item should be live");
    }

    /// A type the reader cannot even represent is still refused by name.
    ///
    /// This is the *other* refusal — not "no behaviour yet" but "no idea what
    /// this is". A creeper has no `SpawnedEntity` variant, so it cannot be
    /// carried at all, and saying so beats inventing a shape for it.
    #[test]
    fn an_unrepresentable_entity_is_refused_and_named() {
        use crate::entity::Entity;

        let mut schem = UniversalSchematic::new("creeper".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        assert!(schem.add_entity(Entity::new("minecraft:creeper".into(), (0.5, 0.0, 0.5))));

        let snbt = to_gametest_snbt(&schem);
        let err = mc_tick::Structure::parse(&snbt).expect_err("should refuse the creeper");
        assert!(
            matches!(
                &err,
                mc_tick::structure::StructureError::UnsupportedEntity { entity_type, .. }
                    if entity_type == "minecraft:creeper"
            ),
            "wrong error: {err}"
        );
        // And the sentence the app shows blames the build, not the converter.
        let detail = super::structure_parse_detail(&err, true);
        assert!(detail.contains("minecraft:creeper"), "{detail}");
        assert!(!detail.contains("engine fault"), "{detail}");
    }
}
