//! Vanilla gametest structure SNBT — the flavor `mc_tick::Structure::parse` reads.
//!
//! `formats::structure_snbt` emits the *data flavor* instead (inline
//! `state:"id{k:v}"` strings), which the tick engine rejects, so this module
//! emits the gametest flavor directly and keeps mc-tick's proven parser as the
//! single reader.
//!
//! This lives in `formats` rather than in `bridge` because it is the only path
//! from a `UniversalSchematic` into the tick engine, and tests that drive a
//! saved build through the engine need it whether or not the generated-bindings
//! bridge is compiled in.

#![allow(clippy::needless_range_loop)]

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
/// `crates/mc-tick/docs/history/entity-abuse-in-record-doors.md`.
///
/// The spelling is Java's `Double.toString` — `NaN`, `Infinity`, `-Infinity` —
/// which is what vanilla's own NBT writer emits, and what mc-tick's `float`
/// reader was taught to accept. The `d` suffix is dropped for these three: no
/// other tag type can hold them, so there is nothing to disambiguate.
/// Serialise a block entity's NBT as SNBT with *correct* string quoting.
///
/// quartz_nbt's `to_snbt` leaves a string like `summon tnt ~ 10000 ~` bare —
/// its `should_quote` misses interior spaces — and no SNBT parser accepts
/// that back, which cost every command-block build its round trip. String
/// values are therefore always quoted here; keys are quoted only when they
/// need it, and everything else follows vanilla's suffix spelling.
fn compound_snbt(map: &crate::utils::NbtMap) -> String {
    use std::fmt::Write as _;
    let mut entries: Vec<(&String, &crate::nbt::NbtValue)> = map.iter().collect();
    // Deterministic output: the same build must emit the same file.
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::from("{");
    for (index, (key, value)) in entries.into_iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let bare = !key.is_empty()
            && key.chars().all(|c| c.is_ascii_alphanumeric() || "_-.+".contains(c));
        if bare {
            out.push_str(key);
        } else {
            let _ = write!(out, "{}", quoted_snbt(key));
        }
        out.push_str(": ");
        out.push_str(&value_snbt(value));
    }
    out.push('}');
    out
}

fn value_snbt(value: &crate::nbt::NbtValue) -> String {
    use crate::nbt::NbtValue as V;
    use std::fmt::Write as _;
    match value {
        V::Byte(v) => format!("{v}b"),
        V::Short(v) => format!("{v}s"),
        V::Int(v) => v.to_string(),
        V::Long(v) => format!("{v}L"),
        V::Float(v) => format!("{v}f"),
        V::Double(v) => format!("{}", snbt_double(*v)),
        V::String(v) => quoted_snbt(v),
        V::ByteArray(values) => {
            let body: Vec<String> = values.iter().map(|v| format!("{v}b")).collect();
            format!("[B; {}]", body.join(", "))
        }
        V::IntArray(values) => {
            let body: Vec<String> = values.iter().map(i32::to_string).collect();
            format!("[I; {}]", body.join(", "))
        }
        V::LongArray(values) => {
            let body: Vec<String> = values.iter().map(|v| format!("{v}L")).collect();
            format!("[L; {}]", body.join(", "))
        }
        V::List(values) => {
            let body: Vec<String> = values.iter().map(value_snbt).collect();
            format!("[{}]", body.join(", "))
        }
        V::Compound(map) => {
            let mut out = String::new();
            let _ = write!(out, "{}", compound_snbt(map));
            out
        }
    }
}

/// Always-quoted SNBT string: double quotes unless the content prefers single.
fn quoted_snbt(text: &str) -> String {
    if text.contains('"') && !text.contains('\'') {
        format!("'{text}'")
    } else {
        let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    }
}

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
        // A container cart's inventory — a hopper or chest minecart parked on
        // a detector rail *is* a comparator input, and a cart emitted without
        // its `Items` reads as empty on the other side.
        if let Some(V::List(items)) = entity.nbt.get("Items") {
            let mut body = String::new();
            for item in items {
                let V::Compound(fields) = item else { continue };
                let Some(V::String(id)) = fields.get("id") else { continue };
                let slot = match fields.get("Slot") {
                    Some(V::Byte(v)) => i64::from(*v),
                    Some(V::Int(v)) => i64::from(*v),
                    _ => 0,
                };
                let count = match fields.get("count").or_else(|| fields.get("Count")) {
                    Some(V::Byte(v)) => i64::from(*v),
                    Some(V::Int(v)) => i64::from(*v),
                    _ => 1,
                };
                if !body.is_empty() {
                    body.push_str(", ");
                }
                let _ = write!(body, "{{Slot: {slot}b, count: {count}, id: \"{id}\"}}");
            }
            if !body.is_empty() {
                let _ = write!(out, ", Items: [{body}]");
            }
        }
        // `Passengers` — riders nested inside their vehicle's compound rather
        // than listed beside it. Dropping this tag is a *silent under-report of
        // the world*: `55_3x3.zip` holds 22 top-level entities and vanilla's own
        // capture of the same save counts 24, and the two missing bodies are
        // blazes riding two of its nan carts. Whether they are there decides
        // whether two hitboxes the build is glued together with exist at all.
        //
        // The rider's `Pos` is shifted like every other position so the file
        // stays coherent, though the engine does not use it — a rider's seat is
        // re-derived from its vehicle every tick. See
        // `mc_tick::entity::passenger_attachment`.
        if let Some(V::List(passengers)) = entity.nbt.get("Passengers") {
            let mut riders = String::new();
            for rider in passengers {
                let V::Compound(fields) = rider else { continue };
                let Some(V::String(rider_id)) = fields.get("id") else { continue };
                let rider_id = if rider_id.contains(':') {
                    rider_id.clone()
                } else {
                    format!("minecraft:{rider_id}")
                };
                // The engine refuses a passenger with no `Pos`, which is the
                // right outcome — leave the tag off rather than invent one.
                let seat: Vec<f64> = match fields.get("Pos") {
                    Some(V::List(values)) if values.len() == 3 => {
                        values.iter().map(|v| nbt_number(v).unwrap_or(0.0)).collect()
                    }
                    _ => Vec::new(),
                };
                let motion = nbt_vec3(fields.get("Motion"));
                if !riders.is_empty() {
                    riders.push_str(", ");
                }
                let _ = write!(riders, "{{id: \"{rider_id}\"");
                if seat.len() == 3 {
                    let _ = write!(
                        riders,
                        ", Pos: [{}, {}, {}]",
                        snbt_double(seat[0] - f64::from(mx)),
                        snbt_double(seat[1] - f64::from(my)),
                        snbt_double(seat[2] - f64::from(mz)),
                    );
                }
                let _ = write!(
                    riders,
                    ", Motion: [{}, {}, {}]}}",
                    snbt_double(motion[0]),
                    snbt_double(motion[1]),
                    snbt_double(motion[2])
                );
            }
            if !riders.is_empty() {
                let _ = write!(out, ", Passengers: [{riders}]");
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
pub fn to_gametest_snbt(schematic: &crate::UniversalSchematic) -> String {
    // Decided *before* any block conversion, and threaded through it.
    //
    // `Entity.load`'s handling of a non-finite `Motion` changed at 1.21.11, so
    // the stamped version is what tells a reader whether this build's NaN
    // velocities are real. Modernising blocks does not reinterpret entities, so
    // the answer is the version the *save* was written at either way — which is
    // why this is read here rather than off the converted clone below, whose
    // `source_data_version` has been rewritten to the canonical one.
    let data_version = source_data_version(schematic);
    if let Some(modern) = modernized(schematic) {
        return render(&modern, data_version);
    }
    render(schematic, data_version)
}

/// The version to stamp: what the file says, else the canonical one.
///
/// A build that states nothing gets today's canonical version, which selects the
/// modern NaN-dropping rule. That is the right default — a schematic with no
/// provenance is not evidence of an old save — but it is a default, and a file
/// that does state a version always wins over it.
fn source_data_version(schematic: &crate::UniversalSchematic) -> i32 {
    schematic
        .metadata
        .source_data_version
        .or(schematic.metadata.mc_version)
        .unwrap_or(crate::dataconverter::CANONICAL_DATA_VERSION)
}

/// Render `schematic`, stamping `data_version`, with no further conversion.
fn render(schematic: &crate::UniversalSchematic, data_version: i32) -> String {
    use std::collections::HashMap;
    use std::fmt::Write as _;

    let bb = schematic.get_bounding_box();
    let (mx, my, mz) = bb.min;
    let size = (bb.max.0 - mx + 1, bb.max.1 - my + 1, bb.max.2 - mz + 1);

    let mut nbt_at: HashMap<(i32, i32, i32), String> = HashMap::new();
    for be in schematic.get_block_entities_as_list() {
        nbt_at.insert(be.position, compound_snbt(&be.nbt));
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
        "{{\n  DataVersion: {},\n  size: [{}, {}, {}],\n  palette: [\n    {}\n  ],\n  blocks: [\n    {}\n  ],\n  entities: [\n    {}\n  ]\n}}\n",
        data_version,
        size.0,
        size.1,
        size.2,
        palette.join(",\n    "),
        blocks,
        entities_snbt(schematic, (mx, my, mz))
    )
}
