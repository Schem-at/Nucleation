//! Experimental dynamic input plans for the concrete vanilla dust evaluator.
//! Plans contain topology, not wire powers. All instrumentation is opt-in.

use crate::behaviour::ComparatorOutputs;
use crate::pos::Pos;
use crate::state::StateId;
use crate::world::World;
use std::sync::{Arc, Mutex, OnceLock};

const CAPACITY: usize = 1024;
const MAX_COMPARATORS: usize = 25;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Group {
    pub fixed: u8,
    pub comparators: u32,
}

#[derive(Debug, Default)]
pub(crate) struct InputPlan {
    pub groups: [Group; 6],
    pub wire_inputs: Vec<Pos>,
    #[cfg(feature = "experiment-execution-view")]
    wire_addresses: [crate::execution_view::Address; 8],
    pub comparator_inputs: Vec<Pos>,
}

impl InputPlan {
    pub fn add_comparator(&mut self, group: usize, pos: Pos) {
        let index = self.comparator_inputs.iter().position(|p| *p == pos)
            .unwrap_or_else(|| {
                assert!(self.comparator_inputs.len() < MAX_COMPARATORS);
                self.comparator_inputs.push(pos);
                self.comparator_inputs.len() - 1
            });
        self.groups[group].comparators |= 1u32 << index;
    }

    fn live_comparators(&self, outputs: &ComparatorOutputs) -> [u8; MAX_COMPARATORS] {
        let mut values = [0; MAX_COMPARATORS];
        for (value, pos) in values.iter_mut().zip(&self.comparator_inputs) {
            *value = outputs.get(pos).copied().unwrap_or(0);
        }
        values
    }

    fn evaluate(&self, world: &World, comparators: &[u8; MAX_COMPARATORS], mut power: impl FnMut(Pos, StateId) -> u8) -> u8 {
        let mut block = 0u8;
        // Do not collapse these groups into one maximum: the original loop
        // stops at exactly 15 and may intentionally never read a later 255.
        for group in &self.groups {
            let mut value = group.fixed;
            let mut mask = group.comparators;
            while mask != 0 {
                let index = mask.trailing_zeros() as usize;
                value = value.max(comparators[index]);
                mask &= mask - 1;
            }
            block = block.max(value);
            if block == 15 {
                return 15;
            }
        }
        let incoming = self.wire_inputs.iter().copied().enumerate().map(|(index, pos)| {
            #[cfg(feature = "experiment-execution-view")]
            let state = world.get_execution(pos, self.wire_addresses.get(index).copied().unwrap_or_default());
            #[cfg(not(feature = "experiment-execution-view"))]
            let state = { let _ = index; world.get(pos) };
            power(pos, state)
        }).max().unwrap_or(0);
        block.max(incoming.saturating_sub(1))
    }
}

fn prepare_plan(build: impl FnOnce() -> InputPlan, world: &World) -> InputPlan {
    let plan = build();
    #[cfg(feature = "experiment-execution-view")]
    {
        let mut plan = plan;
        // Vanilla has at most two incoming wires per horizontal side. Keep
        // bounded inline address storage; unusual test/custom plans with more
        // inputs safely use ordinary position lookup for their remaining inputs.
        for (address, pos) in plan.wire_addresses.iter_mut().zip(&plan.wire_inputs) {
            *address = world.execution_address(*pos);
        }
        plan
    }
    #[cfg(not(feature = "experiment-execution-view"))]
    { let _ = world; plan }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Transition {
    Irrelevant,
    WireValue,
    Structural,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode { Off, On, Shadow }

pub(crate) fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("MC_TICK_WIRE_PLAN").as_deref() {
        Ok("off") => Mode::Off,
        Ok("shadow") => Mode::Shadow,
        _ => Mode::On,
    })
}

/// A token held by the exact immutable rules instance that built a plan. A
/// cloned rules value may subsequently be changed, so its identity starts fresh.
#[cfg(feature = "experiment-world-plan-cache")]
pub(crate) struct RulesIdentity(Arc<()>);

#[cfg(feature = "experiment-world-plan-cache")]
impl Default for RulesIdentity {
    fn default() -> Self { Self(Arc::new(())) }
}

#[cfg(feature = "experiment-world-plan-cache")]
impl Clone for RulesIdentity {
    fn clone(&self) -> Self { Self::default() }
}

#[cfg(feature = "experiment-world-plan-cache")]
impl RulesIdentity {
    pub fn token(&self) -> &Arc<()> { &self.0 }
}

#[cfg(feature = "experiment-wire-certificate")]
struct Certificate {
    target: u8,
    comparators: [u8; MAX_COMPARATORS],
    dirty: bool,
}

struct Entry {
    pos: Pos,
    plan: InputPlan,
    #[cfg(feature = "experiment-wire-certificate")]
    certificate: Option<Certificate>,
}

#[derive(Default)]
struct State {
    #[cfg(feature = "experiment-world-plan-cache")]
    rules: Option<Arc<()>>,
    world: Option<Arc<()>>,
    revision: u64,
    entries: Vec<Option<Entry>>,
    #[cfg(feature = "experiment-wire-plan-counters")]
    counts: Counts,
    #[cfg(all(test, feature = "experiment-world-plan-cache"))]
    test_builds: usize,
}

#[cfg(feature = "experiment-wire-plan-counters")]
#[derive(Default, Debug)]
struct Counts {
    queries: u64,
    builds: u64,
    structural_invalidations: u64,
    value_dirties: u64,
    certificate_hits: u64,
    resets: u64,
    classify_structural: u64,
    classify_wire_value: u64,
    classify_irrelevant: u64,
}

/// Scratch derived data: cloning rules starts empty and Debug never exposes
/// cache occupancy, so semantic snapshots do not depend on query history.
#[derive(Default)]
pub(crate) struct Cache(Mutex<State>);

impl Clone for Cache {
    fn clone(&self) -> Self { Self::default() }
}

// The cache is derived scratch data, not part of World semantic equality.
#[cfg(feature = "experiment-world-plan-cache")]
impl PartialEq for Cache {
    fn eq(&self, _: &Self) -> bool { true }
}
#[cfg(feature = "experiment-world-plan-cache")]
impl Eq for Cache {}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WirePlanCache")
    }
}

#[cfg(feature = "experiment-wire-plan-counters")]
impl Drop for Cache {
    fn drop(&mut self) {
        if let Ok(state) = self.0.get_mut() {
            if state.counts.queries != 0 {
                eprintln!("wire-plan-counters {:?}", state.counts);
            }
        }
    }
}

fn slot(pos: Pos) -> usize {
    let hash = (pos.x as u32).wrapping_mul(0x9e3779b1)
        ^ (pos.y as u32).wrapping_mul(0x85ebca77)
        ^ (pos.z as u32).wrapping_mul(0xc2b2ae3d);
    ((hash ^ (hash >> 16)) as usize) & (CAPACITY - 1)
}

impl State {
    fn clear(&mut self) {
        self.entries.clear();
        self.entries.resize_with(CAPACITY, || None);
        #[cfg(feature = "experiment-wire-plan-counters")]
        { self.counts.resets += 1; }
    }

    fn synchronize(&mut self, world: &World, classify: impl Fn(StateId, StateId) -> Transition) {
        if self.world.as_ref().is_none_or(|token| !Arc::ptr_eq(token, world.plan_identity())) {
            self.clear();
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
        for (changed, from, to) in changes {
            let kind = classify(from, to);
            #[cfg(feature = "experiment-wire-plan-counters")]
            match kind {
                Transition::Structural => self.counts.classify_structural += 1,
                Transition::WireValue => self.counts.classify_wire_value += 1,
                Transition::Irrelevant => self.counts.classify_irrelevant += 1,
            }
            if kind == Transition::Irrelevant { continue; }
            #[cfg(not(feature = "experiment-wire-certificate"))]
            if kind == Transition::WireValue { continue; }
            // The wire target's topology is confined to Manhattan radius two.
            // Compare full keys after hashing; collisions never cause stale hits.
            for dx in -2i32..=2 {
                for dy in -2i32..=2 {
                    for dz in -2i32..=2 {
                        if dx.abs() + dy.abs() + dz.abs() > 2 { continue; }
                        let pos = Pos::new(changed.x.wrapping_add(dx),
                            changed.y.wrapping_add(dy), changed.z.wrapping_add(dz));
                        let entry_slot = &mut self.entries[slot(pos)];
                        let Some(_entry) = entry_slot.as_mut().filter(|entry| entry.pos == pos) else { continue; };
                        if kind == Transition::Structural {
                            *entry_slot = None;
                            #[cfg(feature = "experiment-wire-plan-counters")]
                            { self.counts.structural_invalidations += 1; }
                        } else {
                            #[cfg(feature = "experiment-wire-certificate")]
                            if _entry.plan.wire_inputs.contains(&changed) {
                                if let Some(certificate) = &mut _entry.certificate {
                                    certificate.dirty = true;
                                    #[cfg(feature = "experiment-wire-plan-counters")]
                                    { self.counts.value_dirties += 1; }
                                }
                            }
                        }
                    }
                }
            }
        }
        self.revision = world.plan_revision();
    }
}

impl Cache {
    #[cfg(not(feature = "experiment-world-plan-cache"))]
    pub fn query(
        &self,
        world: &World,
        outputs: &ComparatorOutputs,
        pos: Pos,
        classify: impl Fn(StateId, StateId) -> Transition,
        build: impl FnOnce() -> InputPlan,
        power: impl FnMut(Pos, StateId) -> u8,
    ) -> u8 {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.synchronize(world, classify);
        #[cfg(feature = "experiment-wire-plan-counters")]
        { state.counts.queries += 1; }
        let index = slot(pos);
        if state.entries[index].as_ref().is_none_or(|entry| entry.pos != pos) {
            state.entries[index] = Some(Entry {
                pos,
                plan: prepare_plan(build, world),
                #[cfg(feature = "experiment-wire-certificate")]
                certificate: None,
            });
            #[cfg(feature = "experiment-wire-plan-counters")]
            { state.counts.builds += 1; }
        }
        let entry = state.entries[index].as_mut().unwrap();
        let comparators = entry.plan.live_comparators(outputs);
        #[cfg(feature = "experiment-wire-certificate")]
        if let Some(certificate) = &entry.certificate {
            if !certificate.dirty && certificate.comparators == comparators {
                let target = certificate.target;
                #[cfg(feature = "experiment-wire-plan-counters")]
                { state.counts.certificate_hits += 1; }
                return target;
            }
        }
        let target = entry.plan.evaluate(world, &comparators, power);
        #[cfg(feature = "experiment-wire-certificate")]
        { entry.certificate = Some(Certificate { target, comparators, dirty: false }); }
        target
    }

    #[cfg(all(feature = "experiment-world-plan-cache", test))]
    pub fn query(
        &self,
        world: &World,
        outputs: &ComparatorOutputs,
        pos: Pos,
        classify: impl Fn(StateId, StateId) -> Transition,
        build: impl FnOnce() -> InputPlan,
        power: impl FnMut(Pos, StateId) -> u8,
    ) -> u8 {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        Self::query_locked(&mut state, world, outputs, pos, classify, build, power)
    }

    #[cfg(feature = "experiment-world-plan-cache")]
    pub fn query_for_rules(
        &self,
        rules: &Arc<()>,
        world: &World,
        outputs: &ComparatorOutputs,
        pos: Pos,
        classify: impl Fn(StateId, StateId) -> Transition,
        build: impl FnOnce() -> InputPlan,
        power: impl FnMut(Pos, StateId) -> u8,
    ) -> u8 {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if state.rules.as_ref().is_none_or(|token| !Arc::ptr_eq(token, rules)) {
            state.clear();
            state.rules = Some(rules.clone());
            state.world = Some(world.plan_identity().clone());
            state.revision = world.plan_revision();
        }
        Self::query_locked(&mut state, world, outputs, pos, classify, build, power)
    }

    // Keep the original experiment untouched when the storage feature is off.
    #[cfg(feature = "experiment-world-plan-cache")]
    fn query_locked(
        state: &mut State,
        world: &World,
        outputs: &ComparatorOutputs,
        pos: Pos,
        classify: impl Fn(StateId, StateId) -> Transition,
        build: impl FnOnce() -> InputPlan,
        power: impl FnMut(Pos, StateId) -> u8,
    ) -> u8 {
        state.synchronize(world, classify);
        #[cfg(feature = "experiment-wire-plan-counters")]
        { state.counts.queries += 1; }
        let index = slot(pos);
        if state.entries[index].as_ref().is_none_or(|entry| entry.pos != pos) {
            #[cfg(test)]
            { state.test_builds += 1; }
            state.entries[index] = Some(Entry {
                pos,
                plan: prepare_plan(build, world),
                #[cfg(feature = "experiment-wire-certificate")]
                certificate: None,
            });
            #[cfg(feature = "experiment-wire-plan-counters")]
            { state.counts.builds += 1; }
        }
        let entry = state.entries[index].as_mut().unwrap();
        let comparators = entry.plan.live_comparators(outputs);
        #[cfg(feature = "experiment-wire-certificate")]
        if let Some(certificate) = &entry.certificate {
            if !certificate.dirty && certificate.comparators == comparators {
                let target = certificate.target;
                #[cfg(feature = "experiment-wire-plan-counters")]
                { state.counts.certificate_hits += 1; }
                return target;
            }
        }
        let target = entry.plan.evaluate(world, &comparators, power);
        #[cfg(feature = "experiment-wire-certificate")]
        { entry.certificate = Some(Certificate { target, comparators, dirty: false }); }
        target
    }

    #[cfg(all(test, feature = "experiment-world-plan-cache"))]
    pub(crate) fn test_builds(&self) -> usize {
        self.0.lock().unwrap_or_else(|error| error.into_inner()).test_builds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos::Bounds;
    use std::cell::Cell;

    fn fixed_query(cache: &Cache, world: &World, pos: Pos, source: Pos) -> u8 {
        cache.query(world, &ComparatorOutputs::new(), pos,
            |_, _| Transition::Structural,
            || {
                let mut plan = InputPlan::default();
                plan.groups[0].fixed = world.get(source).raw() as u8;
                plan
            }, |_, _| 0)
    }

    #[cfg(feature = "experiment-execution-view")]
    #[test]
    fn execution_addresses_remain_live_and_excess_inputs_use_the_reference_lookup() {
        let mut world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(10, 2, 2)));
        let positions: Vec<_> = (0..9).map(|x| Pos::new(x, 1, 1)).collect();
        for (n, &pos) in positions.iter().enumerate() { world.set(pos, StateId(n as u16)); }
        let plan = prepare_plan(|| InputPlan { wire_inputs: positions.clone(), ..InputPlan::default() }, &world);
        let check = |world: &World| {
            let expected = positions.iter().map(|&pos| world.get(pos).raw() as u8).max().unwrap().saturating_sub(1);
            assert_eq!(plan.evaluate(world, &[0; MAX_COMPARATORS], |_, state| state.raw() as u8), expected);
        };
        check(&world);
        world.set(positions[0], StateId(20));
        check(&world);
        world.set(positions[8], StateId(255));
        check(&world);
        assert!(world.grow_to_include(Pos::new(-5, 0, 0)));
        check(&world);
        let mut clone = world.clone();
        clone.set(positions[8], StateId(0));
        check(&clone);
    }

    #[test]
    fn every_structural_dependency_offset_invalidates_before_the_next_query() {
        let pos = Pos::new(-3, -2, -4);
        for dx in -2i32..=2 { for dy in -2i32..=2 { for dz in -2i32..=2 {
            if dx.abs() + dy.abs() + dz.abs() > 2 { continue; }
            let cache = Cache::default();
            let mut world = World::new(Bounds::new(Pos::new(-8, -8, -8), Pos::new(2, 2, 2)));
            let source = Pos::new(pos.x + dx, pos.y + dy, pos.z + dz);
            assert_eq!(fixed_query(&cache, &world, pos, source), 0);
            world.set(source, StateId(7));
            assert_eq!(fixed_query(&cache, &world, pos, source), 7,
                "dependency offset {dx},{dy},{dz}");
        } } }
    }

    #[test]
    fn full_position_keys_and_lost_history_prevent_stale_reuse() {
        let cache = Cache::default();
        let pos = Pos::new(0, 0, 0);
        let collision = (1..8192).map(|x| Pos::new(x, 0, 0)).find(|p| slot(*p) == slot(pos)).unwrap();
        let mut world = World::new(Bounds::new(Pos::new(-2, -2, -2), Pos::new(8192, 2, 2)));
        world.set(pos, StateId(1));
        world.set(collision, StateId(2));
        assert_eq!(fixed_query(&cache, &world, pos, pos), 1);
        assert_eq!(fixed_query(&cache, &world, collision, collision), 2);
        assert_eq!(fixed_query(&cache, &world, pos, pos), 1);
        world.set(pos, StateId(3));
        for i in 0..600 {
            world.set(Pos::new(100, 1, 1), StateId(4 + (i % 2)));
        }
        assert_eq!(fixed_query(&cache, &world, pos, pos), 3);
    }

    #[test]
    fn wire_value_and_shape_equivalent_changes_do_not_recompile_topology() {
        let cache = Cache::default();
        let pos = Pos::new(0, 0, 0);
        let source = Pos::new(1, 0, 0);
        let mut world = World::new(Bounds::new(Pos::new(-3, -3, -3), Pos::new(3, 3, 3)));
        let builds = Cell::new(0);
        let query = |world: &World| cache.query(world, &ComparatorOutputs::new(), pos,
            |_, _| Transition::WireValue,
            || {
                builds.set(builds.get() + 1);
                InputPlan { wire_inputs: vec![source], ..InputPlan::default() }
            }, |_, state| state.raw() as u8);
        assert_eq!(query(&world), 0);
        for value in [1, 7, 15, 255, 0] {
            world.set(source, StateId(value));
            assert_eq!(query(&world), (value as u8).saturating_sub(1));
        }
        assert_eq!(builds.get(), 1);
    }

    #[cfg(feature = "experiment-wire-certificate")]
    #[test]
    fn certificates_skip_only_unchanged_dependencies_and_read_comparators_live() {
        let cache = Cache::default();
        let pos = Pos::new(0, 0, 0);
        let source = Pos::new(1, 0, 0);
        let comparator = Pos::new(0, 1, 0);
        let mut world = World::new(Bounds::new(Pos::new(-3, -3, -3), Pos::new(3, 3, 3)));
        let reads = Cell::new(0);
        let mut outputs = ComparatorOutputs::new();
        let query = |world: &World, outputs: &ComparatorOutputs| cache.query(world, outputs, pos,
            |_, _| Transition::WireValue,
            || {
                let mut plan = InputPlan { wire_inputs: vec![source], ..InputPlan::default() };
                plan.add_comparator(0, comparator);
                plan
            }, |_, state| { reads.set(reads.get() + 1); state.raw() as u8 });
        assert_eq!(query(&world, &outputs), 0);
        assert_eq!(reads.get(), 1);
        assert_eq!(query(&world, &outputs), 0);
        assert_eq!(reads.get(), 1);
        // This write is in the radius-two footprint but is not an actual input.
        world.set(Pos::new(0, 0, 1), StateId(7));
        assert_eq!(query(&world, &outputs), 0);
        assert_eq!(reads.get(), 1);
        world.set(source, StateId(8));
        assert_eq!(query(&world, &outputs), 7);
        assert_eq!(reads.get(), 2);
        outputs.insert(comparator, 12);
        assert_eq!(query(&world, &outputs), 12);
        assert_eq!(reads.get(), 3);
        outputs.clear();
        assert_eq!(query(&world, &outputs), 7);
        assert_eq!(reads.get(), 4);
    }

    #[test]
    fn shared_cache_is_correct_across_interleaved_and_concurrent_worlds() {
        use std::sync::{Barrier, atomic::{AtomicUsize, Ordering}};
        let cache = Cache::default();
        let builds = AtomicUsize::new(0);
        let pos = Pos::new(0, 0, 0);
        let source = Pos::new(1, 0, 0);
        let comparator = Pos::new(0, 1, 0);
        let bounds = Bounds::new(Pos::new(-3, -3, -3), Pos::new(3, 3, 3));
        let mut first = World::new(bounds);
        first.set(source, StateId(9));
        let mut second = first.clone();
        second.set(source, StateId(3));
        let first_outputs = [(comparator, 5)].into_iter().collect();
        let second_outputs = [(comparator, 12)].into_iter().collect();
        // The same immutable rule interpretation/cache serves both worlds;
        // overlapping positions and StateIds must not share validity.
        let query = |world: &World, outputs: &ComparatorOutputs| {
            cache.query(world, outputs, pos, |_, _| Transition::Structural,
                || {
                    builds.fetch_add(1, Ordering::Relaxed);
                    let mut plan = InputPlan { wire_inputs: vec![source], ..InputPlan::default() };
                    plan.add_comparator(0, comparator);
                    plan
                }, |_, state| state.raw() as u8)
        };
        for _ in 0..8 {
            assert_eq!(query(&first, &first_outputs), 8);
            assert_eq!(query(&second, &second_outputs), 12);
        }
        // This also documents the bounded single-world cache's tradeoff:
        // alternating worlds rebuild on every query, even without mutations.
        assert_eq!(builds.load(Ordering::Relaxed), 16);
        for _ in 0..16 { assert_eq!(query(&first, &first_outputs), 8); }
        assert_eq!(builds.load(Ordering::Relaxed), 17);

        let barrier = Barrier::new(2);
        std::thread::scope(|scope| {
            scope.spawn(|| {
                barrier.wait();
                for _ in 0..64 {
                    assert_eq!(query(&first, &first_outputs), 8);
                }
            });
            scope.spawn(|| {
                barrier.wait();
                for _ in 0..64 {
                    assert_eq!(query(&second, &second_outputs), 12);
                }
            });
        });
    }
}
