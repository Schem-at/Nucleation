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

struct Entry { pos: Pos, facing: Dir, validated_revision: u64, plan: Plan }

#[derive(Default)]
struct State {
    rules: Option<Arc<()>>,
    world: Option<Arc<()>>,
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
    validations: u64,
    journal_changes: u64,
    distant_changes: u64,
    history_losses: u64,
    // These count nearby classifications during per-entry validation, not
    // distinct world writes. Replay stops at the first invalidating change.
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

    fn synchronize_identity(&mut self, rules: &Arc<()>, world: &World) {
        if self.rules.as_ref().is_none_or(|token| !Arc::ptr_eq(token, rules))
            || self.world.as_ref().is_none_or(|token| !Arc::ptr_eq(token, world.plan_identity())) {
            self.clear();
            self.rules = Some(rules.clone());
            self.world = Some(world.plan_identity().clone());
        }
    }

    fn validate(&mut self, index: usize, pos: Pos, facing: Dir, revision: u64,
        world: &World, same_structure: impl Fn(StateId, StateId) -> bool) -> bool {
        let Some(entry) = self.entries[index].as_mut()
            .filter(|entry| entry.pos == pos && entry.facing == facing) else { return false; };
        if entry.validated_revision == revision { return true; }
        #[cfg(feature = "experiment-piston-plan-counters")]
        { self.counts.validations += 1; }
        let Some(changes) = world.plan_changes_since(entry.validated_revision) else {
            #[cfg(feature = "experiment-piston-plan-counters")]
            {
                self.counts.history_losses += 1;
                self.counts.invalidations += 1;
            }
            return false;
        };
        for (changed, from, to) in changes {
            #[cfg(feature = "experiment-piston-plan-counters")]
            { self.counts.journal_changes += 1; }
            // Own/strong inputs lie in radius two around the piston or around
            // the position above it. Dust's farther connection reads stay live.
            // Check distance before classifying states, and validate only this
            // queried entry; inactive entries keep their own older revision.
            if u64::from(entry.pos.x.abs_diff(changed.x))
                + u64::from(entry.pos.y.abs_diff(changed.y))
                + u64::from(entry.pos.z.abs_diff(changed.z)) > 3 {
                #[cfg(feature = "experiment-piston-plan-counters")]
                { self.counts.distant_changes += 1; }
                continue;
            }
            if same_structure(from, to) {
                #[cfg(feature = "experiment-piston-plan-counters")]
                { self.counts.ignored_changes += 1; }
                continue;
            }
            #[cfg(feature = "experiment-piston-plan-counters")]
            {
                self.counts.structural_changes += 1;
                self.counts.invalidations += 1;
            }
            return false;
        }
        entry.validated_revision = revision;
        true
    }
}

impl Cache {
    pub fn query(&self, rules: &Arc<()>, world: &World, outputs: &ComparatorOutputs,
        pos: Pos, facing: Dir, same_structure: impl Fn(StateId, StateId) -> bool,
        build: impl FnOnce() -> Plan, emitted: impl FnMut(Pos, StateId, Dir) -> bool) -> bool {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.synchronize_identity(rules, world);
        let revision = world.plan_revision();
        #[cfg(feature = "experiment-piston-plan-counters")]
        { state.counts.queries += 1; }
        let index = slot(pos, facing);
        if !state.validate(index, pos, facing, revision, world, same_structure) {
            state.entries[index] = Some(Entry { pos, facing, validated_revision: revision, plan: build() });
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
    fn querying_one_entry_does_not_validate_stale_unrelated_entries() {
        let cache = Cache::default();
        let rules = RulesIdentity::default();
        let pos = Pos::new(0, 0, 0);
        let other = (8..64).map(|x| Pos::new(x, 0, 0))
            .find(|other| slot(*other, Dir::East) != slot(pos, Dir::East)).unwrap();
        let mut world = World::new(Bounds::new(Pos::new(-4, -4, -4), Pos::new(132, 4, 4)));
        assert!(!fixed(&cache, &rules, &world, pos, Dir::East, pos));
        assert!(!fixed(&cache, &rules, &world, other, Dir::East, other));
        let initial_revision = world.plan_revision();
        world.set(other, StateId(1));
        assert!(!fixed(&cache, &rules, &world, pos, Dir::East, pos));
        assert_eq!(cache.test_builds(), 2);
        {
            let state = cache.0.lock().unwrap();
            assert_eq!(state.entries[slot(pos, Dir::East)].as_ref().unwrap().validated_revision,
                world.plan_revision());
            assert_eq!(state.entries[slot(other, Dir::East)].as_ref().unwrap().validated_revision,
                initial_revision);
        }
        assert!(fixed(&cache, &rules, &world, other, Dir::East, other));
        assert_eq!(cache.test_builds(), 3);

        // Keep one entry fresh while the other loses the structural write from
        // bounded history. A global last-seen revision would miss this change.
        let old_revision = world.plan_revision();
        world.set(other, StateId::AIR);
        for i in 0..600 {
            world.set(Pos::new(128, 0, 0), StateId(1 + i % 2));
            assert!(!fixed(&cache, &rules, &world, pos, Dir::East, pos));
        }
        assert_eq!(cache.test_builds(), 3);
        assert!(world.plan_changes_since(old_revision).is_none());
        assert!(!fixed(&cache, &rules, &world, other, Dir::East, other));
        assert_eq!(cache.test_builds(), 4);
    }

    #[test]
    fn distant_changes_are_not_classified_and_near_value_changes_keep_the_plan() {
        use std::cell::Cell;
        let cache = Cache::default();
        let rules = RulesIdentity::default();
        let pos = Pos::new(-2, -2, -2);
        let mut world = World::new(Bounds::new(Pos::new(-5, -5, -5), Pos::new(12, 5, 5)));
        world.set(pos, StateId(1));
        assert!(fixed(&cache, &rules, &world, pos, Dir::East, pos));
        world.set(Pos::new(10, 0, 0), StateId(1));
        assert!(cache.query(rules.token(), &world, &ComparatorOutputs::new(), pos, Dir::East,
            |_, _| panic!("distant change was classified"),
            || panic!("distant change rebuilt the plan"), |_, _, _| false));
        world.set(pos, StateId(2));
        let classifications = Cell::new(0);
        assert!(cache.query(rules.token(), &world, &ComparatorOutputs::new(), pos, Dir::East,
            |from, to| {
                classifications.set(classifications.get() + 1);
                assert_eq!((from, to), (StateId(1), StateId(2)));
                true
            }, || panic!("equivalent structure rebuilt the plan"), |_, _, _| false));
        assert_eq!(classifications.get(), 1);
        assert_eq!(cache.test_builds(), 1);
    }

    #[test]
    fn revision_regression_rebuilds_and_identity_reset_clears_all_entries() {
        let cache = Cache::default();
        let rules = RulesIdentity::default();
        let pos = Pos::new(0, 0, 0);
        let other = (8..64).map(|x| Pos::new(x, 0, 0))
            .find(|other| slot(*other, Dir::East) != slot(pos, Dir::East)).unwrap();
        let mut world = World::new(Bounds::new(Pos::new(-4, -4, -4), Pos::new(68, 4, 4)));
        assert!(!fixed(&cache, &rules, &world, pos, Dir::East, pos));
        assert!(!fixed(&cache, &rules, &world, other, Dir::East, other));
        world.set(pos, StateId(1));
        // Model a pre-rollover entry: a revision ahead of the journal must not
        // validate. The journal's own rollover test covers rotating its token.
        cache.0.lock().unwrap().entries[slot(pos, Dir::East)].as_mut().unwrap()
            .validated_revision = u64::MAX;
        assert!(fixed(&cache, &rules, &world, pos, Dir::East, pos));
        assert_eq!(cache.test_builds(), 3);
        let old_identity = world.plan_identity().clone();
        assert!(world.grow_to_include(Pos::new(-10, 0, 0)));
        assert!(!Arc::ptr_eq(&old_identity, world.plan_identity()));
        assert!(fixed(&cache, &rules, &world, pos, Dir::East, pos));
        assert_eq!(cache.test_builds(), 4);
        assert!(cache.0.lock().unwrap().entries[slot(other, Dir::East)].is_none());
    }

    #[test]
    fn positive_input_short_circuits_later_live_emission_queries() {
        let world = World::new(Bounds::new(Pos::new(-2, -2, -2), Pos::new(2, 2, 2)));
        let plan = Plan { inputs: vec![Input::Positive, Input::wire(&world, Pos::new(1, 0, 0), Dir::West)] };
        assert!(plan.evaluate(&world, &ComparatorOutputs::new(), |_, _, _| panic!("later input evaluated")));
    }
}
