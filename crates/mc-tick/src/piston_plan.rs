//! Experimental structural input plans for vanilla piston power and QC.
//! Comparator outputs and dust emission/connection queries remain live.

use crate::behaviour::ComparatorOutputs;
use crate::{Dir, Pos, StateId, World};
use std::sync::{Arc, Mutex, OnceLock};

const CAPACITY: usize = 256;

pub(crate) struct RulesIdentity(Arc<()>);
impl Default for RulesIdentity {
    fn default() -> Self { Self(Arc::new(())) }
}
impl Clone for RulesIdentity {
    fn clone(&self) -> Self { Self::default() }
}
impl RulesIdentity {
    pub fn token(&self) -> &Arc<()> { &self.0 }
}

pub(crate) enum Input {
    Positive,
    Comparator(Pos),
    Wire {
        pos: Pos,
        toward: Dir,
        #[cfg(feature = "experiment-execution-view")]
        address: crate::execution_view::Address,
    },
}

impl Input {
    pub fn wire(world: &World, pos: Pos, toward: Dir) -> Self {
        #[cfg(not(feature = "experiment-execution-view"))]
        let _ = world;
        Self::Wire {
            pos,
            toward,
            #[cfg(feature = "experiment-execution-view")]
            address: world.execution_address(pos),
        }
    }
}

#[derive(Default)]
pub(crate) struct Plan {
    pub inputs: Vec<Input>,
}

impl Plan {
    fn evaluate(&self, world: &World, outputs: &ComparatorOutputs,
        mut emitted: impl FnMut(Pos, StateId, Dir) -> bool) -> bool {
        self.inputs.iter().any(|input| match input {
            Input::Positive => true,
            Input::Comparator(pos) => outputs.get(pos).copied().unwrap_or(0) > 0,
            Input::Wire { pos, toward, #[cfg(feature = "experiment-execution-view")] address } => {
                #[cfg(feature = "experiment-execution-view")]
                let state = world.get_execution(*pos, *address);
                #[cfg(not(feature = "experiment-execution-view"))]
                let state = world.get(*pos);
                emitted(*pos, state, *toward)
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode { Off, On, Shadow }

pub(crate) fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("MC_TICK_PISTON_PLAN").as_deref() {
        Ok("off") => Mode::Off,
        Ok("shadow") => Mode::Shadow,
        _ => Mode::On,
    })
}

struct Entry { pos: Pos, facing: Dir, plan: Plan }

#[derive(Default)]
struct State {
    rules: Option<Arc<()>>,
    world: Option<Arc<()>>,
    revision: u64,
    entries: Vec<Option<Entry>>,
    #[cfg(feature = "experiment-piston-plan-counters")]
    counts: Counts,
    #[cfg(test)]
    test_builds: usize,
}

#[cfg(feature = "experiment-piston-plan-counters")]
#[derive(Default, Debug)]
struct Counts {
    queries: u64,
    builds: u64,
    invalidations: u64,
    structural_changes: u64,
    ignored_changes: u64,
    resets: u64,
}

#[derive(Default)]
pub(crate) struct Cache(Mutex<State>);
impl Clone for Cache { fn clone(&self) -> Self { Self::default() } }
impl PartialEq for Cache { fn eq(&self, _: &Self) -> bool { true } }
impl Eq for Cache {}

#[cfg(feature = "experiment-piston-plan-counters")]
impl Drop for Cache {
    fn drop(&mut self) {
        if let Ok(state) = self.0.get_mut() {
            if state.counts.queries != 0 { eprintln!("piston-plan-counters {:?}", state.counts); }
        }
    }
}

fn slot(pos: Pos, facing: Dir) -> usize {
    let h = (pos.x as u32).wrapping_mul(0x9e3779b1)
        ^ (pos.y as u32).wrapping_mul(0x85ebca77)
        ^ (pos.z as u32).wrapping_mul(0xc2b2ae3d)
        ^ (facing as u32).wrapping_mul(0x27d4eb2d);
    ((h ^ (h >> 16)) as usize) & (CAPACITY - 1)
}

impl State {
    fn clear(&mut self) {
        self.entries.clear();
        self.entries.resize_with(CAPACITY, || None);
        #[cfg(feature = "experiment-piston-plan-counters")]
        { self.counts.resets += 1; }
    }

    fn synchronize(&mut self, rules: &Arc<()>, world: &World,
        same_structure: impl Fn(StateId, StateId) -> bool) {
        if self.rules.as_ref().is_none_or(|token| !Arc::ptr_eq(token, rules))
            || self.world.as_ref().is_none_or(|token| !Arc::ptr_eq(token, world.plan_identity())) {
            self.clear();
            self.rules = Some(rules.clone());
            self.world = Some(world.plan_identity().clone());
            self.revision = world.plan_revision();
            return;
        }
        if self.revision == world.plan_revision() { return; }
        let Some(changes) = world.plan_changes_since(self.revision) else {
            self.clear();
            self.revision = world.plan_revision();
            return;
        };
        for (pos, from, to) in changes {
            if same_structure(from, to) {
                #[cfg(feature = "experiment-piston-plan-counters")]
                { self.counts.ignored_changes += 1; }
                continue;
            }
            #[cfg(feature = "experiment-piston-plan-counters")]
            { self.counts.structural_changes += 1; }
            // Own/strong inputs lie in radius two around the piston or around
            // the position above it. Dust's farther connection reads stay live.
            for entry in &mut self.entries {
                let affected = entry.as_ref().is_some_and(|entry| {
                    u64::from(entry.pos.x.abs_diff(pos.x))
                        + u64::from(entry.pos.y.abs_diff(pos.y))
                        + u64::from(entry.pos.z.abs_diff(pos.z)) <= 3
                });
                if affected {
                    *entry = None;
                    #[cfg(feature = "experiment-piston-plan-counters")]
                    { self.counts.invalidations += 1; }
                }
            }
        }
        self.revision = world.plan_revision();
    }
}

impl Cache {
    pub fn query(&self, rules: &Arc<()>, world: &World, outputs: &ComparatorOutputs,
        pos: Pos, facing: Dir, same_structure: impl Fn(StateId, StateId) -> bool,
        build: impl FnOnce() -> Plan, emitted: impl FnMut(Pos, StateId, Dir) -> bool) -> bool {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.synchronize(rules, world, same_structure);
        #[cfg(feature = "experiment-piston-plan-counters")]
        { state.counts.queries += 1; }
        let index = slot(pos, facing);
        if state.entries[index].as_ref().is_none_or(|entry| entry.pos != pos || entry.facing != facing) {
            state.entries[index] = Some(Entry { pos, facing, plan: build() });
            #[cfg(feature = "experiment-piston-plan-counters")]
            { state.counts.builds += 1; }
            #[cfg(test)]
            { state.test_builds += 1; }
        }
        state.entries[index].as_ref().unwrap().plan.evaluate(world, outputs, emitted)
    }

    #[cfg(test)]
    pub(crate) fn test_builds(&self) -> usize {
        self.0.lock().unwrap_or_else(|error| error.into_inner()).test_builds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bounds;

    fn fixed(cache: &Cache, rules: &RulesIdentity, world: &World,
        pos: Pos, facing: Dir, source: Pos) -> bool {
        cache.query(rules.token(), world, &ComparatorOutputs::new(), pos, facing,
            |from, to| from == to,
            || Plan { inputs: if world.get(source) == StateId::AIR { vec![] } else { vec![Input::Positive] } },
            |_, _, _| false)
    }

    #[test]
    fn structural_footprint_invalidates_immediately_including_qc_height() {
        let rules = RulesIdentity::default();
        let center = Pos::new(-2, -2, -2);
        for dx in -3i32..=3 { for dy in -3i32..=3 { for dz in -3i32..=3 {
            if dx.abs() + dy.abs() + dz.abs() > 3 { continue; }
            let cache = Cache::default();
            let mut world = World::new(Bounds::new(Pos::new(-6, -6, -6), Pos::new(3, 3, 3)));
            let source = Pos::new(center.x + dx, center.y + dy, center.z + dz);
            assert!(!fixed(&cache, &rules, &world, center, Dir::East, source));
            world.set(source, StateId(1));
            assert!(fixed(&cache, &rules, &world, center, Dir::East, source));
        } } }
    }

    #[test]
    fn full_keys_rules_tokens_and_history_loss_prevent_stale_inputs() {
        let cache = Cache::default();
        let rules = RulesIdentity::default();
        let pos = Pos::new(0, 0, 0);
        let other = (1..8192).map(|x| Pos::new(x, 0, 0))
            .find(|other| slot(*other, Dir::East) == slot(pos, Dir::East)).unwrap();
        let mut world = World::new(Bounds::new(Pos::new(-2, -2, -2), Pos::new(8192, 2, 2)));
        world.set(pos, StateId(1));
        assert!(fixed(&cache, &rules, &world, pos, Dir::East, pos));
        assert!(!fixed(&cache, &rules, &world, other, Dir::East, other));
        assert!(fixed(&cache, &rules, &world, pos, Dir::East, pos));
        world.set(pos, StateId::AIR);
        for i in 0..600 { world.set(Pos::new(100, 1, 1), StateId(1 + i % 2)); }
        assert!(!fixed(&cache, &rules, &world, pos, Dir::East, pos));
        let changed_rules = rules.clone();
        assert!(!Arc::ptr_eq(rules.token(), changed_rules.token()));
        // A new rules identity must rebuild even with unchanged world revision.
        let before = cache.test_builds();
        fixed(&cache, &changed_rules, &world, pos, Dir::East, pos);
        assert_eq!(cache.test_builds(), before + 1);
    }

    #[test]
    fn positive_input_short_circuits_later_live_emission_queries() {
        let world = World::new(Bounds::new(Pos::new(-2, -2, -2), Pos::new(2, 2, 2)));
        let plan = Plan { inputs: vec![Input::Positive, Input::wire(&world, Pos::new(1, 0, 0), Dir::West)] };
        assert!(plan.evaluate(&world, &ComparatorOutputs::new(), |_, _, _| panic!("later input evaluated")));
    }
}
