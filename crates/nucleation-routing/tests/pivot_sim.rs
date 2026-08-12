//! In-sim verification of the planner-stamped pivot (feature `mc-tick`):
//! the same scene as `pivot_integration.rs`, wired into a
//! `mc_tick::Simulation` the way nucleation's bridge does, then driven
//! through real levers — all-off + walking-ones(8) + all-on + 0xAA + 0x55
//! = 12 patterns x 8 output bits = 96 checks, zero crosstalk tolerated
//! (the same gate `redstone-eda/pivot_tiles.py` holds the Python template
//! to).

mod common;

use std::collections::HashMap;

use common::{build_scene, PIVOT_AT};
use mc_tick::{Pos as TPos, Simulation};
use nucleation_routing::pivot::{route_bus, PivotKind};
use nucleation_routing::{Aabb, Pos, RedstoneRouter, Workspace};

/// Every state the sim may need to intern beyond the build's own palette
/// (mc-tick binds behaviour at construction; late states sit inert).
fn extra_states() -> Vec<String> {
    const DIRS: [&str; 4] = ["north", "south", "east", "west"];
    let mut out: Vec<String> = Vec::new();
    for d in DIRS {
        for p in ["true", "false"] {
            out.push(format!(
                "minecraft:lever[face=floor,facing={d},powered={p}]"
            ));
        }
    }
    out.push("minecraft:redstone_torch[lit=true]".into());
    out.push("minecraft:redstone_torch[lit=false]".into());
    for d in DIRS {
        for p in ["true", "false"] {
            out.push(format!("minecraft:redstone_wall_torch[facing={d},lit={p}]"));
        }
    }
    out.push("minecraft:redstone_lamp[lit=true]".into());
    out.push("minecraft:redstone_lamp[lit=false]".into());
    for d in DIRS {
        for dl in [1, 2] {
            for lk in ["true", "false"] {
                for pw in ["true", "false"] {
                    out.push(format!(
                        "minecraft:repeater[facing={d},delay={dl},locked={lk},powered={pw}]"
                    ));
                }
            }
        }
    }
    out
}

/// A simulation addressable in workspace coordinates (the world is placed
/// with the workspace's min corner at the origin).
struct Sim {
    sim: Simulation,
    off: (i32, i32, i32),
}

impl Sim {
    fn p(&self, p: Pos) -> TPos {
        TPos::new(p.x - self.off.0, p.y - self.off.1, p.z - self.off.2)
    }

    fn block(&self, p: Pos) -> &str {
        let id = self.sim.world().get(self.p(p));
        self.sim
            .registry()
            .descriptor(id)
            .unwrap_or("minecraft:air")
    }

    fn power(&self, p: Pos) -> i32 {
        let b = self.block(p);
        let Some(i) = b.find("power=") else {
            return -1;
        };
        b[i + 6..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .unwrap_or(-1)
    }

    fn on(&self, p: Pos) -> bool {
        self.power(p) > 0
    }

    fn powered(&self, p: Pos) -> bool {
        self.block(p).contains("powered=true")
    }

    fn use_block(&mut self, p: Pos) {
        let q = self.p(p);
        self.sim.use_block(q);
    }

    fn settle(&mut self, budget: u64) -> bool {
        self.sim.run_until_quiescent(budget);
        self.sim.is_quiescent()
    }
}

/// Wire the workspace into a settled simulation, exactly the bridge way:
/// place, intern extras + companions, register vanilla behaviours, vanilla
/// placement order, `onPlace` pass, ordered settle, then run to quiescence.
fn simulate(ws: &Workspace, settle_budget: u64) -> Sim {
    let bb = ws.bounds().expect("non-empty workspace");
    let lo = (bb.min.x, bb.min.y, bb.min.z);
    let size = (
        bb.max.x - bb.min.x + 1,
        bb.max.y - bb.min.y + 1,
        bb.max.z - bb.min.z + 1,
    );
    let mut palette: Vec<String> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut blocks: Vec<(TPos, usize)> = Vec::new();
    for (p, state) in ws.cells() {
        let entry = *index.entry(state.as_str()).or_insert_with(|| {
            palette.push(state.clone());
            palette.len() - 1
        });
        blocks.push((TPos::new(p.x - lo.0, p.y - lo.1, p.z - lo.2), entry));
    }
    let structure = mc_tick::Structure {
        size,
        data_version: None,
        palette,
        blocks,
        inventories: Vec::new(),
        inventory_blocked_slots: Vec::new(),
        comparator_outputs: Vec::new(),
        block_entities: Vec::new(),
        commands: Vec::new(),
        entities: Vec::new(),
        item_entities: Vec::new(),
    };
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, TPos::new(0, 0, 0));
    }
    for descriptor in extra_states() {
        sim.registry_mut().intern(&descriptor).expect("intern");
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, TPos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    if let Some(report) = sim.unknown_report() {
        panic!("blocks without behaviour: {report}");
    }
    {
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    sim.record();
    let mut wrapped = Sim { sim, off: lo };
    assert!(wrapped.settle(settle_budget), "initial settle ran away");
    wrapped
}

/// Toggle-to-target lever driver: levers only respond to `use_block`, and
/// a player flips them one at a time with a settle after each.
fn drive(sim: &mut Sim, levers: &[Pos], state: &mut Vec<bool>, bits: u8) {
    for (i, lever) in levers.iter().enumerate() {
        let want = (bits >> i) & 1 == 1;
        if state[i] != want {
            sim.use_block(*lever);
            state[i] = want;
            assert!(sim.settle(800), "settle after lever {i}");
        }
    }
    assert!(sim.settle(800), "final settle");
}

#[test]
fn planner_stamped_pivot_conducts_all_96_checks() {
    let mut scene = build_scene();
    let mut router = RedstoneRouter::new();
    router.bounds = Some(Aabb::new(Pos::new(0, 0, -3), Pos::new(46, 17, 17)));
    let report = route_bus(
        &mut scene.ws,
        &router,
        &scene.from,
        &scene.to,
        PIVOT_AT,
        "bus",
        None,
    )
    .expect("bus routes");
    assert_eq!(report.pivot, Some(PivotKind::V2H));

    let mut sim = simulate(&scene.ws, 4000);
    let mut state: Vec<bool> = scene.levers.iter().map(|&p| sim.powered(p)).collect();

    let patterns: Vec<u8> = std::iter::once(0u8)
        .chain((0..8).map(|b| 1u8 << b))
        .chain([0xFF, 0xAA, 0x55])
        .collect();
    assert_eq!(patterns.len(), 12);
    let mut checks = 0;
    for &pat in &patterns {
        drive(&mut sim, &scene.levers, &mut state, pat);
        for n in 0..8u8 {
            let got = sim.on(scene.to.bit(n));
            let want = (pat >> n) & 1 == 1;
            assert_eq!(
                got, want,
                "pattern {pat:#04x} bit {n}: got {got}, want {want}"
            );
            checks += 1;
        }
    }
    assert_eq!(checks, 96);
}
