//! Bounded, observational simulation assessment, not a door/vanilla pass.
//! check_plot_simulation INPUT SOURCE_SHA256 [TICKS=320] [IDLE_SETUP=80]
//! Run through tools/check_tick_plot_simulation.py for the external timeout.
use mc_test::{try_build_sim, SettleMode};
use mc_tick::{Pos, Simulation, StopReason, Structure};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

const CELL_BUDGET: u64 = 8_000_000;
const RECORD_BUDGET: usize = 500_000;
const SCHEMA: u32 = 1;

fn emit(value: Value) {
    println!("{value}");
    std::io::stdout().flush().unwrap();
}

fn digest(value: &Value) -> String {
    blake3::hash(&serde_json::to_vec(value).unwrap())
        .to_hex()
        .to_string()
}

fn absolute(p: Pos, origin: Pos) -> [i32; 3] {
    [p.x + origin.x, p.y + origin.y, p.z + origin.z]
}

fn name(d: &str) -> &str {
    d.split('[').next().unwrap_or(d)
}

fn clickable(d: &str) -> bool {
    name(d) == "minecraft:lever" || name(d).ends_with("_button")
}

fn descriptor(sim: &Simulation, p: Pos) -> &str {
    sim.registry()
        .descriptor(sim.world().get(p))
        .unwrap_or("<unknown-state>")
}

fn stack_value(stack: &mc_tick::ItemStack) -> Value {
    json!({"slot":stack.slot,"id":stack.id,"count":stack.count,
        "contents":stack.contents.as_ref().map(|v| v.iter().map(stack_value).collect::<Vec<_>>())})
}

struct Snapshot {
    blocks: BTreeMap<Pos, String>,
    block_hash: String,
    inventory_hash: String,
    state_hash: String,
    summary: Value,
}

fn snapshot(sim: &Simulation, origin: Pos, known_inventories: &BTreeSet<Pos>) -> Snapshot {
    let blocks = sim
        .world()
        .iter_non_air()
        .map(|(p, s)| {
            (
                p,
                sim.registry()
                    .descriptor(s)
                    .unwrap_or("<unknown-state>")
                    .to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let block_rows = blocks
        .iter()
        .map(|(p, d)| json!([absolute(*p, origin), d]))
        .collect::<Vec<_>>();
    // Include original inventory positions even after their block disappears,
    // as well as runtime containers at every currently occupied cell.
    let positions = known_inventories
        .iter()
        .copied()
        .chain(blocks.keys().copied())
        .collect::<BTreeSet<_>>();
    let mut inventory_rows = Vec::new();
    let mut occupied_slots = 0;
    let mut item_count = 0u64;
    for p in positions {
        if let Some(inv) = sim.inventory(p) {
            let mut stacks = inv.stacks.iter().collect::<Vec<_>>();
            stacks.sort_by_key(|s| s.slot);
            occupied_slots += stacks.iter().filter(|s| s.count > 0).count();
            item_count += stacks.iter().map(|s| u64::from(s.count)).sum::<u64>();
            inventory_rows.push(json!([
                absolute(p, origin),
                inv.slots,
                inv.blocked_slots,
                stacks.into_iter().map(stack_value).collect::<Vec<_>>()
            ]));
        }
    }
    let mut comparators = sim
        .comparator_outputs()
        .iter()
        .map(|(p, v)| (absolute(*p, origin), *v))
        .collect::<Vec<_>>();
    comparators.sort();
    let pending = sim
        .pending_ticks()
        .into_iter()
        .map(|(tick, p)| {
            (
                i128::from(tick) - i128::from(sim.tick_count()),
                absolute(p, origin),
            )
        })
        .collect::<Vec<_>>();
    let events = sim
        .pending_events()
        .into_iter()
        .map(|(p, id)| (absolute(p, origin), id))
        .collect::<Vec<_>>();
    // Debug text retains floating-point NaNs without JSON turning them into null.
    // This is a versioned harness fingerprint, not a stable cross-version wire format.
    let entities = format!(
        "{:?}|{:?}|{:?}|{:?}",
        sim.item_entities(),
        sim.minecarts(),
        sim.entity_bodies(),
        sim.riders()
    );
    let block_hash = digest(&json!(block_rows));
    let inventory_hash = digest(&json!(inventory_rows));
    let state_hash = digest(&json!({"blocks":block_hash,"inventories":inventory_hash,
        "comparator_outputs":comparators,"entities_local":entities,"pending_ticks_relative":pending,
        "pending_events":events,"moving_blocks_local":format!("{:?}",sim.moving_blocks())}));
    let bounds = sim.world().bounds();
    let summary = json!({"tick":sim.tick_count(),"non_air_blocks":blocks.len(),
        "block_fingerprint":block_hash,"inventory_fingerprint":inventory_hash,
        "observable_state_fingerprint":state_hash,"inventory_count":inventory_rows.len(),
        "occupied_slots":occupied_slots,"item_count":item_count,
        "item_entities":sim.item_entities().len(),"minecarts":sim.minecarts().len(),
        "bodies":sim.entity_bodies().len(),"riders":sim.riders().len(),
        "pending_ticks":pending.len(),"pending_events":events.len(),
        "moving_blocks":sim.moving_blocks().len(),"scheduler_quiescent":sim.is_quiescent(),
        "world_bounds":{"min":absolute(bounds.min,origin),"max":absolute(bounds.max,origin)}});
    Snapshot {
        blocks,
        block_hash,
        inventory_hash,
        state_hash,
        summary,
    }
}

fn delta(before: &Snapshot, after: &Snapshot, origin: Pos) -> Value {
    let positions = before
        .blocks
        .keys()
        .chain(after.blocks.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let changed = positions
        .into_iter()
        .filter(|p| before.blocks.get(p) != after.blocks.get(p))
        .collect::<Vec<_>>();
    let removed = changed
        .iter()
        .filter(|p| before.blocks.contains_key(*p) && !after.blocks.contains_key(*p))
        .count();
    let added = changed
        .iter()
        .filter(|p| !before.blocks.contains_key(*p) && after.blocks.contains_key(*p))
        .count();
    json!({"changed_cells":changed.len(),"non_air_to_air":removed,"air_to_non_air":added,
        "sample":changed.iter().take(32).map(|p| json!({"pos":absolute(*p,origin),
            "from":before.blocks.get(p),"to":after.blocks.get(p)})).collect::<Vec<_>>(),
        "interpretation":"Net cell changes; removals can be motion, not proof of damage."})
}

fn run_trial(
    sim: &mut Simulation,
    origin: Pos,
    known_inventories: &BTreeSet<Pos>,
    ticks: u64,
    control: Option<Pos>,
    label: &str,
) -> Value {
    emit(json!({"type":"stage","stage":label,"control":control.map(|p|absolute(p,origin))}));
    sim.record();
    let before = snapshot(sim, origin, known_inventories);
    let start = sim.tick_count();
    let mut hash = blake3::Hasher::new();
    let mut inv_cursor = 0;
    let mut ent_cursor = 0;
    let mut block_changes = 0;
    let mut changes_away_from_control = 0;
    let mut piston_changes = 0;
    let mut active_ticks = 0;
    let mut last_active = None;
    let mut checkpoints = vec![before.summary.clone()];
    let mut endpoint_hashes = vec![(before.block_hash.clone(), before.inventory_hash.clone())];
    let mut actions = Vec::new();
    let mut status = "completed";
    let mut stop_detail = None;
    for elapsed in 0..ticks {
        if let Some(p) = control.filter(|_| elapsed % 80 == 0) {
            let old = descriptor(sim, p).to_owned();
            if clickable(&old) {
                sim.use_block(p);
                actions.push(json!({"before_elapsed_tick":elapsed,"before_engine_tick":start+elapsed,
                    "position":absolute(p,origin),"status":"used","from":old,"to":descriptor(sim,p)}));
            } else {
                actions.push(
                    json!({"before_elapsed_tick":elapsed,"position":absolute(p,origin),
                    "status":"control_missing_or_replaced","state":old}),
                );
            }
        }
        let stop = sim.step();
        hash.update(&elapsed.to_le_bytes());
        for change in sim.recorded() {
            let from = sim
                .registry()
                .descriptor(change.from)
                .unwrap_or("<unknown-state>");
            let to = sim
                .registry()
                .descriptor(change.to)
                .unwrap_or("<unknown-state>");
            hash.update(
                &serde_json::to_vec(&json!([
                    change.tick - start,
                    absolute(change.pos, origin),
                    from,
                    to
                ]))
                .unwrap(),
            );
            block_changes += 1;
            changes_away_from_control += usize::from(Some(change.pos) != control);
            piston_changes +=
                usize::from(name(from).contains("piston") || name(to).contains("piston"));
        }
        let inv = &sim.recorded_inventory()[inv_cursor..];
        for change in inv {
            hash.update(
                &serde_json::to_vec(&json!([
                    change.tick - start,
                    absolute(change.pos, origin),
                    change.slot,
                    change.from,
                    change.to
                ]))
                .unwrap(),
            );
        }
        let ent = &sim.recorded_entities()[ent_cursor..];
        for (tick, event) in ent {
            hash.update(format!("{}:{event:?}", tick - start).as_bytes());
        }
        let active = !sim.recorded().is_empty() || !inv.is_empty() || !ent.is_empty();
        if active {
            active_ticks += 1;
            last_active = Some(elapsed + 1);
        }
        inv_cursor = sim.recorded_inventory().len();
        ent_cursor = sim.recorded_entities().len();
        // Pending queues and the entity states expose stalls which block logs miss.
        hash.update(
            format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}",
                sim.pending_ticks(),
                sim.pending_events(),
                sim.item_entities(),
                sim.minecarts(),
                sim.entity_bodies()
            )
            .as_bytes(),
        );
        sim.clear_recorded();
        if stop != StopReason::Completed {
            status = "engine_stop";
            stop_detail = Some(format!("{stop:?}"));
        } else if block_changes + inv_cursor + ent_cursor > RECORD_BUDGET {
            status = "record_budget_exhausted";
            stop_detail = Some(format!("more than {RECORD_BUDGET} records"));
        }
        if (elapsed + 1) % 80 == 0 || elapsed + 1 == ticks || status != "completed" {
            let s = snapshot(sim, origin, known_inventories);
            endpoint_hashes.push((s.block_hash.clone(), s.inventory_hash.clone()));
            checkpoints.push(s.summary);
        }
        if status != "completed" {
            break;
        }
    }
    let after = snapshot(sim, origin, known_inventories);
    let unknown = sim.unknown_report();
    if unknown.is_some() {
        status = "unsupported_runtime_block";
    }
    let completed_ticks = sim.tick_count() - start;
    let trace_hash = hash.finalize().to_hex().to_string();
    let periodic_layout = endpoint_hashes.len() >= 5
        && endpoint_hashes[0].0 == endpoint_hashes[2].0
        && endpoint_hashes[2].0 == endpoint_hashes[4].0;
    let periodic_inventory = endpoint_hashes.len() >= 5
        && endpoint_hashes[0].1 == endpoint_hashes[2].1
        && endpoint_hashes[2].1 == endpoint_hashes[4].1;
    json!({"label":label,"status":status,"stop_detail":stop_detail,"unknown_report":unknown,
        "requested_ticks":ticks,"completed_ticks":completed_ticks,"actions":actions,
        "block_changes":block_changes,"changes_away_from_control":changes_away_from_control,
        "piston_related_changes":piston_changes,"inventory_changes":inv_cursor,"entity_changes":ent_cursor,
        "active_ticks":active_ticks,"last_active_elapsed_tick":last_active,"trace_fingerprint":trace_hash,
        "before":before.summary,"after":after.summary,"endpoints":checkpoints,
        "net_change":delta(&before,&after,origin),"returned_to_initial_block_layout":before.block_hash==after.block_hash,
        "returned_to_initial_inventory":before.inventory_hash==after.inventory_hash,
        "block_layout_equal_at_0_160_320":periodic_layout,"inventory_equal_at_0_160_320":periodic_inventory,
        "observable_state_fingerprint":after.state_hash})
}

fn error(status: &str, detail: impl ToString) -> Value {
    json!({"status":status,"detail":detail.to_string(),"functional_status":"unproven","vanilla_status":"not_checked"})
}

fn assess(input: &str, ticks: u64, setup: u64) -> Value {
    emit(json!({"type":"stage","stage":"production_import"}));
    let bytes = match std::fs::read(input) {
        Ok(v) => v,
        Err(e) => return error("input_error", e),
    };
    let schematic = match nucleation::formats::manager::get_manager()
        .lock()
        .unwrap()
        .read(&bytes)
    {
        Ok(v) => v,
        Err(e) => return error("import_rejected", e),
    };
    let source_version = schematic
        .metadata
        .source_data_version
        .or(schematic.metadata.mc_version);
    if source_version != Some(4671) {
        return error(
            "unsupported_source_version",
            format!("Expected native 4671, got {source_version:?}"),
        );
    }
    let bb = schematic.get_bounding_box();
    let origin = Pos::new(bb.min.0, bb.min.1, bb.min.2);
    let size = [
        bb.max.0 - bb.min.0 + 1,
        bb.max.1 - bb.min.1 + 1,
        bb.max.2 - bb.min.2 + 1,
    ];
    let volume = size.iter().try_fold(1u64, |v, n| {
        if *n > 0 {
            v.checked_mul((*n + 2 * mc_test::MARGIN) as u64)
        } else {
            None
        }
    });
    if volume.is_none_or(|v| v > CELL_BUDGET) {
        return error(
            "volume_budget_exhausted",
            format!("size={size:?}, cells with margin={volume:?}"),
        );
    }
    let exported = nucleation::formats::gametest::to_gametest_snbt(&schematic);
    emit(
        json!({"type":"stage","stage":"engine_structure_parse","source_data_version":source_version,
        "origin":[origin.x,origin.y,origin.z],"size":size,"input_blake3":blake3::hash(&bytes).to_hex().to_string(),
        "source_entities":schematic.get_entities_as_list().len(),"source_block_entities":schematic.get_block_entities_as_list().len()}),
    );
    let structure = match Structure::parse(&exported) {
        Ok(v) => v,
        Err(e) => return error("structure_rejected", format!("{e:?}")),
    };
    if structure.data_version != source_version {
        return error(
            "source_version_not_preserved",
            "production export changed DataVersion",
        );
    }
    if structure.entities.len() != schematic.get_entities_as_list().len()
        || structure.block_entities.len() != schematic.get_block_entities_as_list().len()
    {
        return error(
            "import_count_mismatch",
            "entity/block-entity count changed during production export/engine parse",
        );
    }
    let mut controls = structure
        .blocks
        .iter()
        .filter(|(_, i)| clickable(&structure.palette[*i]))
        .map(|(p, i)| (*p, structure.palette[*i].clone()))
        .collect::<Vec<_>>();
    controls.sort_by_key(|(p, _)| *p);
    let untrialled_controls = structure
        .blocks
        .iter()
        .filter(|(_, i)| {
            let n = name(&structure.palette[*i]);
            n.ends_with("_pressure_plate") || n == "minecraft:tripwire_hook"
        })
        .map(|(p, i)| {
            json!({"position":absolute(*p,origin),"state":structure.palette[*i],
        "reason":"requires entity contact; empty-hand clicks would not be a valid stimulus"})
        })
        .collect::<Vec<_>>();
    let limitations = structure
        .entities
        .iter()
        .filter_map(|e| match e {
            mc_tick::structure::SpawnedEntity::Body(b) => {
                Some(json!({"kind":b.kind,"limitation":"frozen body model; no general mob AI"}))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let ignored_commands = structure
        .commands
        .iter()
        .filter(|(_, c)| mc_tick::vanilla::parse_command(c).is_none())
        .map(|(p, c)| json!({"position":absolute(*p,origin),"command":c}))
        .collect::<Vec<_>>();
    let metadata = json!({"source_data_version":source_version,"origin":[origin.x,origin.y,origin.z],
        "coordinate_system":"engine-local cells + origin; original origin also supplies Java dust hash ordering",
        "size":size,"source_blocks":structure.blocks.len(),"source_entities":structure.entities.len(),
        "source_block_entities":structure.block_entities.len(),"source_inventory_count":structure.inventories.len(),
        "source_occupied_slots":structure.inventories.iter().map(|(_,v)|v.iter().filter(|s|s.count>0).count()).sum::<usize>(),
        "source_item_count":structure.inventories.iter().flat_map(|(_,v)|v).map(|s|u64::from(s.count)).sum::<u64>(),
        "clickable_controls":controls.iter().map(|(p,d)|json!({"position":absolute(*p,origin),"state":d})).collect::<Vec<_>>(),
        "untrialled_contact_controls":untrialled_controls,"entity_model_limitations":limitations,
        "unsupported_commands":ignored_commands,"input_blake3":blake3::hash(&bytes).to_hex().to_string(),
        "export_snbt_blake3":blake3::hash(exported.as_bytes()).to_hex().to_string()});
    emit(json!({"type":"metadata","metadata":metadata}));
    if !ignored_commands.is_empty() {
        return error(
            "unsupported_commands",
            "refusing to silently execute no-op commands",
        );
    }
    // The shared fixture helper currently panics if an inventory-bearing block
    // has no known slot count. Surface this as an explicit unsupported input;
    // inventing a slot count would not implement that block's mechanics.
    for (p, _) in &structure.inventories {
        let block = structure
            .blocks
            .iter()
            .find(|(q, _)| q == p)
            .map(|(_, i)| name(&structure.palette[*i]));
        if block.and_then(mc_tick::vanilla::container_slots).is_none() {
            return error(
                "unsupported_inventory_container",
                format!(
                    "Inventory at {:?} belongs to {:?}, with no supported container slot count",
                    absolute(*p, origin),
                    block
                ),
            );
        }
    }
    emit(json!({"type":"stage","stage":"engine_build"}));
    let mut sim = match try_build_sim(
        &structure,
        origin,
        SettleMode::InWorld,
        &[],
        &[],
        structure.data_version,
        input,
    ) {
        Ok(v) => v,
        Err(e) => return error("build_rejected", e),
    };
    sim.world_mut().set_growth_limit(CELL_BUDGET);
    sim.set_rng_seed(0);
    sim.set_random_ticks(0);
    let known_inventories = structure
        .inventories
        .iter()
        .map(|(p, _)| *p)
        .collect::<BTreeSet<_>>();
    let initial = snapshot(&sim, origin, &known_inventories);
    let settling = run_trial(
        &mut sim,
        origin,
        &known_inventories,
        setup,
        None,
        "initial_idle_observation",
    );
    emit(json!({"type":"trial","trial":settling}));
    if settling["status"] != "completed" {
        return json!({"status":"initial_idle_failed","metadata":metadata,"initial":initial.summary,
            "settling":settling,"functional_status":"unproven","vanilla_status":"not_checked"});
    }
    let checkpoint = sim.checkpoint();
    let mut trials = Vec::new();
    let cases = std::iter::once((None, "idle_control".to_string())).chain(
        controls
            .iter()
            .enumerate()
            .map(|(i, (p, _))| (Some(*p), format!("control_{i}"))),
    );
    for (control, label) in cases {
        sim.restore(&checkpoint);
        sim.set_rng_seed(0);
        let first = run_trial(&mut sim, origin, &known_inventories, ticks, control, &label);
        emit(json!({"type":"trial","trial":first}));
        let mut repeat = Value::Null;
        let mut deterministic = Value::Null;
        if first["status"] == "completed" {
            sim.restore(&checkpoint);
            sim.set_rng_seed(0);
            repeat = run_trial(
                &mut sim,
                origin,
                &known_inventories,
                ticks,
                control,
                &format!("{label}_repeat"),
            );
            deterministic = json!(
                repeat["status"] == "completed"
                    && first["trace_fingerprint"] == repeat["trace_fingerprint"]
                    && first["observable_state_fingerprint"]
                        == repeat["observable_state_fingerprint"]
            );
            emit(json!({"type":"trial","trial":repeat}));
        }
        trials.push(
            json!({"control":control.map(|p|absolute(p,origin)),"first":first,"repeat":repeat,
            "deterministic_after_restore_and_reseed":deterministic}),
        );
    }
    let non_deterministic = trials
        .iter()
        .any(|t| t["deterministic_after_restore_and_reseed"] == false);
    let failed = trials.iter().any(|t| {
        t["first"]["status"] != "completed"
            || (!t["repeat"].is_null() && t["repeat"]["status"] != "completed")
    });
    let responsive = trials
        .iter()
        .skip(1)
        .filter(|t| {
            t["first"]["changes_away_from_control"]
                .as_u64()
                .unwrap_or(0)
                > 0
        })
        .count();
    json!({"status":if non_deterministic {"non_deterministic"} else if failed {"trial_failed"} else {"assessed"},
        "functional_status":"unproven","vanilla_status":"not_checked","metadata":metadata,
        "initial":initial.summary,"settling":settling,"trials":trials,
        "controls_trialled":controls.len(),"controls_with_changes_away_from_control":responsive,
        "qualification":"Activity and repeatable layouts are observations, not proof of a complete door or vanilla parity."})
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert!(
        (3..=5).contains(&args.len()),
        "INPUT SOURCE_SHA256 [TICKS=320] [IDLE_SETUP=80]"
    );
    assert!(
        args[2].len() == 64 && args[2].bytes().all(|c| c.is_ascii_hexdigit()),
        "source SHA-256"
    );
    let ticks = args
        .get(3)
        .map(|v| v.parse::<u64>().unwrap())
        .unwrap_or(320);
    let setup = args.get(4).map(|v| v.parse::<u64>().unwrap()).unwrap_or(80);
    assert!(
        (320..=640).contains(&ticks) && ticks % 160 == 0 && setup <= 320,
        "bounded protocol"
    );
    let result = std::panic::catch_unwind(|| assess(&args[1], ticks, setup)).unwrap_or_else(|e| {
        let detail = e
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_owned()))
            .unwrap_or_else(|| "non-string panic".into());
        error("panic", detail)
    });
    emit(
        json!({"type":"result","schema_version":SCHEMA,"source_sha256":args[2],"source_input":args[1],
        "config":{"ticks":ticks,"idle_setup_ticks":setup,"rng_seed":0,"random_tick_speed":0,
            "settle":"in_world","click_interval":80,"repeat_from_checkpoint":true,
            "action_boundary":"at elapsed 0,80,160,240,... immediately before the next simulation step",
            "cell_budget":CELL_BUDGET,"record_budget_per_trial":RECORD_BUDGET,
            "fingerprint_scope":"ordered block/inventory/entity records, every-tick pending queues and entity state, full visible snapshots every 80 ticks; no ordered update-delivery recording"},
        "assessment":result}),
    );
}
