//! Compare an external playerless Minecraft 1.21.11 community capture.
//! Usage: check_community_capture --capture PATH
//! Compares raw initial state and every tick; no placement/settling callbacks.
use mc_test::{try_build_sim, SettleMode};
use mc_tick::{Pos, Simulation, Structure};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

type BlockRow = (i32, i32, i32, String);
type InventoryRow = (i32, i32, i32, u8, String, u8);
type ComparatorRow = (i32, i32, i32, u8);
type Result<T> = std::result::Result<T, Value>;

#[derive(Deserialize)]
struct Capture {
    format_version: u32,
    mc_version: String,
    data_version: i32,
    source: Source,
    case: String,
    negative_control: bool,
    origin: [i32; 3],
    rng_seed: i64,
    size: [i32; 3],
    watch: Watch,
    settle: String,
    action_boundary: String,
    capture_ticks: u64,
    fixture: Vec<FixtureBlock>,
    actions: Vec<Action>,
    #[serde(default)]
    entities: Vec<Value>,
    #[serde(default)]
    item_entities: Vec<Value>,
    initial: Snapshot,
    snapshots: Vec<Snapshot>,
}

#[derive(Deserialize)]
struct Source { data_version: i32, sha256: String }

#[derive(Deserialize, Serialize)]
struct Watch { min: [i32; 3], max: [i32; 3] }

impl Watch {
    fn contains(&self, p: [i32; 3]) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }
}

#[derive(Deserialize)]
struct FixtureBlock { pos: [i32; 3], state: String, nbt: Option<String> }

#[derive(Deserialize)]
struct Action {
    tick: u64,
    #[serde(rename = "type")]
    kind: String,
    pos: [i32; 3],
    state: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct Snapshot {
    tick: u64,
    players: u32,
    blocks: Vec<BlockRow>,
    inventories: Vec<InventoryRow>,
    comparator_outputs: Vec<ComparatorRow>,
}

fn fail(tick: Option<u64>, position: Option<[i32; 3]>, field: &str, expected: Value, actual: Value) -> Value {
    json!({"tick":tick,"position":position,"field":field,"expected":expected,"actual":actual})
}

fn schema(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(fail(None, None, "capture_schema", json!(message), json!("invalid capture"))) }
}

fn pos([x, y, z]: [i32; 3]) -> Pos { Pos::new(x, y, z) }
fn quote(text: &str) -> String { serde_json::to_string(text).unwrap() }

fn palette_entry(descriptor: &str) -> Result<String> {
    let Some((name, properties)) = descriptor.split_once('[') else {
        return Ok(format!("{{Name:{}}}", quote(descriptor)));
    };
    let properties = properties.strip_suffix(']').ok_or_else(||
        fail(None,None,"descriptor",json!("closing ]"),json!(descriptor)))?;
    let mut entries = Vec::new();
    for pair in properties.split(',') {
        let (key, value) = pair.split_once('=').ok_or_else(||
            fail(None,None,"descriptor",json!("key=value property"),json!(descriptor)))?;
        entries.push(format!("{}:{}", quote(key), quote(value)));
    }
    Ok(format!("{{Name:{},Properties:{{{}}}}}", quote(name), entries.join(",")))
}

fn validate(c: &Capture) -> Result<()> {
    schema(c.format_version == 1 && c.mc_version == "1.21.11", "format 1 / Minecraft 1.21.11")?;
    schema(c.data_version == 4671 && c.source.data_version == 4671,
        "capture and source must both be native DataVersion 4671; no version substitution")?;
    schema(c.source.sha256.len() == 64 && c.source.sha256.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "source SHA-256 must be 64 lowercase hexadecimal characters")?;
    schema(c.rng_seed == 0 && c.settle == "in_world", "seed zero and raw InWorld loading")?;
    schema(c.action_boundary == "immediately before the numbered tick; initial snapshot is tick zero",
        "known before-tick action boundary")?;
    schema(c.entities.is_empty() && c.item_entities.is_empty(), "authored entities are unsupported by this capture")?;
    schema((1..=5000).contains(&c.capture_ticks), "capture_ticks in 1..=5000")?;
    let mut volume = 1u64;
    for i in 0..3 {
        schema(c.size[i] > 0 && c.watch.min[i] <= 0 && c.watch.max[i] >= c.size[i] - 1,
            "positive size contained by watch box")?;
        let extent = i64::from(c.watch.max[i]) - i64::from(c.watch.min[i]) + 1;
        schema((1..=262144).contains(&extent), "bounded watch extent")?;
        volume = volume.checked_mul(extent as u64).ok_or_else(||
            fail(None,None,"watch",json!("bounded volume"),json!("overflow")))?;
        let low = i64::from(c.origin[i]) + i64::from(c.watch.min[i]);
        let high = i64::from(c.origin[i]) + i64::from(c.watch.max[i]);
        schema(low >= -29999968 && high <= 29999967, "watch inside bounded world coordinates")?;
        if i == 1 { schema(low >= -64 && high < 320, "watch inside overworld build height")?; }
    }
    schema(volume <= 262144 && volume * c.capture_ticks <= 50_000_000, "bounded cell/tick budget")?;
    schema(c.initial.tick == 0 && c.snapshots.len() as u64 == c.capture_ticks, "initial tick zero plus every tick")?;
    schema(!c.fixture.is_empty(), "nonempty fixture")?;
    let mut fixture = BTreeMap::new();
    for b in &c.fixture {
        schema((0..3).all(|i| b.pos[i] >= 0 && b.pos[i] < c.size[i]), "fixture position inside size")?;
        schema(fixture.insert(b.pos, b.state.as_str()).is_none(), "unique fixture positions")?;
    }
    let initial = c.initial.blocks.iter().map(|(x,y,z,s)| ([*x,*y,*z],s.as_str())).collect::<BTreeMap<_,_>>();
    let fixture_nonair = fixture.into_iter().filter(|(_,s)| *s != "minecraft:air").collect::<BTreeMap<_,_>>();
    schema(initial == fixture_nonair, "initial non-air states exactly match raw fixture")?;
    for (index, snapshot) in std::iter::once(&c.initial).chain(&c.snapshots).enumerate() {
        schema(snapshot.tick == index as u64, "consecutive tick boundaries")?;
        if snapshot.players != 0 {
            return Err(fail(Some(snapshot.tick),None,"players",json!(0),json!(snapshot.players)));
        }
        let mut blocks = BTreeMap::new();
        for (x,y,z,state) in &snapshot.blocks {
            let p=[*x,*y,*z];
            schema(c.watch.contains(p) && state != "minecraft:air", "non-air block rows inside watch")?;
            schema(blocks.insert(p,state.as_str()).is_none(), "unique captured block positions")?;
        }
        let mut slots = BTreeSet::new();
        for (x,y,z,slot,id,count) in &snapshot.inventories {
            let p=[*x,*y,*z];
            schema(c.watch.contains(p) && blocks.contains_key(&p) && *count > 0 && id.starts_with("minecraft:"),
                "occupied inventory rows on watched non-air cells")?;
            schema(slots.insert((p,*slot)), "unique occupied inventory slots")?;
        }
        let mut comparators = BTreeSet::new();
        for (x,y,z,_) in &snapshot.comparator_outputs {
            let p=[*x,*y,*z];
            schema(blocks.get(&p).is_some_and(|s| s.starts_with("minecraft:comparator[")),
                "output row belongs to a current comparator")?;
            schema(comparators.insert(p), "unique comparator outputs")?;
        }
    }
    let mut prior_tick=0;
    for a in &c.actions {
        schema(a.tick >= prior_tick && (1..=c.capture_ticks).contains(&a.tick) && c.watch.contains(a.pos),
            "ordered action ticks/positions within capture")?;
        schema(match a.kind.as_str() {
            "set_block" => a.state.as_ref().is_some_and(|s| !s.is_empty()),
            "notify_neighbors" | "pull_lever" => a.state.is_none(), _ => false,
        }, "only set_block, AIR-source notify_neighbors, or pull_lever actions")?;
        prior_tick=a.tick;
    }
    Ok(())
}

fn simulation(c: &Capture) -> Result<Simulation> {
    let palette = c.fixture.iter().map(|b| palette_entry(&b.state)).collect::<Result<Vec<_>>>()?.join(",");
    let blocks = c.fixture.iter().enumerate().map(|(i,b)| {
        let [x,y,z]=b.pos;
        let nbt=b.nbt.as_ref().map(|v|format!(",nbt:{v}")).unwrap_or_default();
        format!("{{pos:[{x},{y},{z}],state:{i}{nbt}}}")
    }).collect::<Vec<_>>().join(",");
    let [x,y,z]=c.size;
    let text=format!("{{DataVersion:{},size:[{x},{y},{z}],palette:[{palette}],blocks:[{blocks}],entities:[]}}",c.data_version);
    let structure=Structure::parse(&text).map_err(|e|
        fail(Some(0),None,"fixture_nbt",json!("valid native fixture"),json!(format!("{e:?}"))))?;
    schema(structure.data_version == Some(4671), "native fixture save version preserved")?;
    let extras=c.actions.iter().filter_map(|a|a.state.clone()).collect::<Vec<_>>();
    let mut sim=try_build_sim(&structure,pos(c.origin),SettleMode::InWorld,&extras,&[],structure.data_version,&c.case)
        .map_err(|e|fail(Some(0),None,"build",json!("supported fixture"),json!(e)))?;
    sim.set_rng_seed(0);
    schema(sim.recorded().is_empty(), "raw loading must not produce block callbacks")?;
    Ok(sim)
}

fn snapshot(sim: &Simulation, watch: &Watch) -> Result<Snapshot> {
    if !sim.item_entities().is_empty() || !sim.minecarts().is_empty() || !sim.entity_bodies().is_empty()
        || !sim.recorded_entities().is_empty() {
        return Err(fail(Some(sim.tick_count()),None,"unsupported_engine_entities",json!("none"),
            json!({"items":sim.item_entities().len(),"minecarts":sim.minecarts().len(),
                "bodies":sim.entity_bodies().len(),"records":sim.recorded_entities().len()})));
    }
    let mut s=Snapshot {tick:sim.tick_count(),players:0,blocks:Vec::new(),inventories:Vec::new(),comparator_outputs:Vec::new()};
    for y in watch.min[1]..=watch.max[1] { for z in watch.min[2]..=watch.max[2] { for x in watch.min[0]..=watch.max[0] {
        let p=Pos::new(x,y,z);
        let state=sim.registry().descriptor(sim.world().get(p)).ok_or_else(||
            fail(Some(s.tick),Some([x,y,z]),"state",json!("registered state"),Value::Null))?;
        if state == "minecraft:air" {continue;}
        s.blocks.push((x,y,z,state.into()));
        if let Some(inventory)=sim.inventory(p) {
            let mut stacks=inventory.stacks.iter().filter(|stack|stack.count>0).collect::<Vec<_>>();
            stacks.sort_by_key(|stack|stack.slot);
            for stack in stacks {
                if stack.contents.is_some() {
                    return Err(fail(Some(s.tick),Some([x,y,z]),"unsupported_nested_inventory",
                        json!("flat occupied slots only"),json!({"slot":stack.slot,"item":stack.id})));
                }
                s.inventories.push((x,y,z,stack.slot,stack.id.clone(),stack.count));
            }
        }
        if state.starts_with("minecraft:comparator[") {
            s.comparator_outputs.push((x,y,z,sim.comparator_outputs().get(&p).copied().unwrap_or(0)));
        }
    }}}
    Ok(s)
}

fn compare(e: &Snapshot, a: &Snapshot) -> Result<()> {
    if a.tick != e.tick {return Err(fail(Some(e.tick),None,"tick",json!(e.tick),json!(a.tick)));}
    let eb=e.blocks.iter().map(|(x,y,z,s)|([*x,*y,*z],s)).collect::<BTreeMap<_,_>>();
    let ab=a.blocks.iter().map(|(x,y,z,s)|([*x,*y,*z],s)).collect::<BTreeMap<_,_>>();
    for p in eb.keys().chain(ab.keys()).copied().collect::<BTreeSet<_>>() {
        if eb.get(&p)!=ab.get(&p) {
            return Err(fail(Some(e.tick),Some(p),"block",json!(eb.get(&p).map_or("minecraft:air",|s|s.as_str())),
                json!(ab.get(&p).map_or("minecraft:air",|s|s.as_str()))));
        }
    }
    let ei=e.inventories.iter().map(|(x,y,z,n,id,count)|(([*x,*y,*z],*n),(id,*count))).collect::<BTreeMap<_,_>>();
    let ai=a.inventories.iter().map(|(x,y,z,n,id,count)|(([*x,*y,*z],*n),(id,*count))).collect::<BTreeMap<_,_>>();
    for key in ei.keys().chain(ai.keys()).copied().collect::<BTreeSet<_>>() {
        if ei.get(&key)!=ai.get(&key) {
            return Err(fail(Some(e.tick),Some(key.0),&format!("inventory_slot_{}",key.1),json!(ei.get(&key)),json!(ai.get(&key))));
        }
    }
    let ec=e.comparator_outputs.iter().map(|(x,y,z,v)|([*x,*y,*z],*v)).collect::<BTreeMap<_,_>>();
    let ac=a.comparator_outputs.iter().map(|(x,y,z,v)|([*x,*y,*z],*v)).collect::<BTreeMap<_,_>>();
    for p in ec.keys().chain(ac.keys()).copied().collect::<BTreeSet<_>>() {
        if ec.get(&p)!=ac.get(&p) {return Err(fail(Some(e.tick),Some(p),"comparator_output",json!(ec.get(&p)),json!(ac.get(&p))));}
    }
    Ok(())
}

fn hash_snapshot(hash: &mut blake3::Hasher, s: &Snapshot) {
    // Canonical coordinate order means hashes also agree if a producer changes
    // JSON row order without changing the captured values.
    let blocks=s.blocks.iter().map(|(x,y,z,d)|([*x,*y,*z],d)).collect::<BTreeMap<_,_>>();
    let inventories=s.inventories.iter().map(|(x,y,z,n,id,c)|(([*x,*y,*z],*n),(id,*c))).collect::<BTreeMap<_,_>>();
    let outputs=s.comparator_outputs.iter().map(|(x,y,z,v)|([*x,*y,*z],*v)).collect::<BTreeMap<_,_>>();
    hash.update(format!("{}:{blocks:?}:{inventories:?}:{outputs:?}",s.tick).as_bytes());
}

fn run(path: &PathBuf, bytes: &[u8], c: &Capture) -> Value {
    let mut matching=0u64;
    let mut expected_hash=blake3::Hasher::new();
    let mut actual_hash=blake3::Hasher::new();
    let mut changes=0usize;
    let result=(|| -> Result<()> {
        validate(c)?;
        let mut sim=simulation(c)?;
        let mut next=0;
        for expected in std::iter::once(&c.initial).chain(&c.snapshots) {
            if expected.tick > 0 {
                while next<c.actions.len() && c.actions[next].tick==expected.tick {
                    let action=&c.actions[next];
                    match action.kind.as_str() {
                        "set_block" => {
                            let state=sim.registry().get(action.state.as_ref().unwrap()).ok_or_else(||
                                fail(Some(expected.tick),Some(action.pos),"action_state",json!("pre-registered"),json!(action.state)))?;
                            sim.place_block(pos(action.pos),state);
                        }
                        "notify_neighbors" => sim.notify_neighbors(pos(action.pos)),
                        "pull_lever" => {
                            let actual=sim.registry().descriptor(sim.world().get(pos(action.pos))).unwrap_or("<unknown>");
                            if !actual.starts_with("minecraft:lever[") {return Err(fail(Some(expected.tick),Some(action.pos),"action_target",json!("lever"),json!(actual)));}
                            sim.use_block(pos(action.pos));
                        }
                        _ => unreachable!(),
                    }
                    next+=1;
                }
                let stop=sim.step();
                if sim.tick_count()!=expected.tick {return Err(fail(Some(expected.tick),None,"engine_step",json!("completed tick"),json!(format!("{stop:?}"))));}
            }
            if let Some(report)=sim.unknown_report() {return Err(fail(Some(expected.tick),None,"unknown_blocks",json!("none"),json!(report)));}
            let actual=snapshot(&sim,&c.watch)?;
            changes=sim.recorded().len();
            hash_snapshot(&mut expected_hash,expected);
            hash_snapshot(&mut actual_hash,&actual);
            compare(expected,&actual)?;
            matching+=1;
        }
        Ok(())
    })();
    json!({"case":c.case,"negative_control":c.negative_control,"capture":path,
        "capture_blake3":blake3::hash(bytes).to_hex().to_string(),"declared_source_sha256":c.source.sha256,
        "status":if result.is_ok(){"matches_capture"}else{"failed"},"first_difference":result.err(),
        "matching_frames_including_initial":matching,"expected_frames":c.capture_ticks+1,
        "fixture_blocks":c.fixture.len(),"actions":c.actions.len(),"engine_block_changes":changes,
        "compared_capture_state_blake3":expected_hash.finalize().to_hex().to_string(),
        "compared_engine_state_blake3":actual_hash.finalize().to_hex().to_string(),
        "scope":"Initial and every tick-end block state, comparator output and occupied flat inventory slot within watch; no intra-tick, item-component, moving-BE or vanilla runtime-entity equality.",
        "entity_guard":"Authored entities rejected; engine runtime entities rejected. Java capture does not independently census runtime entities."})
}

fn main() {
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    if args.len()!=2 || args[0]!="--capture" {
        eprintln!("usage: check_community_capture --capture PATH");std::process::exit(2);
    }
    let path=PathBuf::from(&args[1]);
    let report=match std::fs::read(&path) {
        Ok(bytes)=>match serde_json::from_slice::<Capture>(&bytes) {
            Ok(capture)=>run(&path,&bytes,&capture),
            Err(error)=>json!({"status":"invalid_capture","capture":path,"error":error.to_string()}),
        },
        Err(error)=>json!({"status":"read_failed","capture":path,"error":error.to_string()}),
    };
    let passed=report["status"]=="matches_capture";
    println!("{report}");
    if !passed {std::process::exit(1);}
}
