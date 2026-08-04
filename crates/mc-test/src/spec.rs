//! The descriptor: what a case is made of, independent of any carrier.
//!
//! Everything here is data. The evaluator that gives it meaning lives in
//! [`crate::eval`]; the files that carry it are the carriers' business.

use std::collections::BTreeMap;

use mc_tick::Pos;
use serde::Deserialize;

/// Air margin around the build, matching mc-tick's `conformance.rs`.
pub const MARGIN: i32 = 4;

/// One scenario.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// What this proves, in one sentence. Used in every failure message.
    pub name: String,
    /// Structure file, relative to the case file. Defaults to `<stem>.snbt`.
    /// Meaningless — and rejected — for a carrier that *is* the structure.
    #[serde(default)]
    pub structure: Option<String>,
    /// Where the capture's (0,0,0) sat in the game's coordinates — wire update
    /// order hashes absolute positions, so an in-world build needs its origin.
    #[serde(default)]
    pub origin: [i32; 3],
    /// How the loaded structure is settled before tick 0.
    #[serde(default)]
    pub settle: SettleMode,
    /// Seed for the vanilla random source. Behaviours that jitter (dispense
    /// trajectories, dispenser slot choice, destroy drops) draw from it in a
    /// fixed order, so a seeded case is exactly reproducible. Omitted: the
    /// engine uses each distribution's mean.
    #[serde(default)]
    pub seed: Option<i64>,
    /// `randomTickSpeed` for the run: random-tick attempts per 4096-block
    /// volume per tick. Zero (the default) disables the pass; lithium's
    /// gametest environment sets 3. Pair with `seed` for reproducibility.
    #[serde(default)]
    pub random_ticks: u8,
    /// Ticks stepped before the case's tick 0, off the record — the gametest
    /// `setup_ticks`. Placement transients (an observer's placement pulse, a
    /// falling block finding its floor) happen here and are excluded from
    /// `initial`, from `changes` counts, and from `events` claims.
    #[serde(default)]
    pub setup: u64,
    /// What a player does, and when.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Block names the author asserts are inert *for this run* — the escape
    /// hatch for foreign structures carrying blocks the engine does not model
    /// (a lithium `test_block`). An explicit per-case assertion rather than a
    /// silent skip: the engine itself never treats an unknown block as inert.
    #[serde(default)]
    pub inert: Vec<String>,
    /// The claims this case makes.
    pub checks: Vec<Check>,
    /// Opt-in event assertions against the recorded change log. Default empty:
    /// a case checks end states only, and an optimisation that preserves them
    /// passes without golden churn. Diagnostics on failure are separate and
    /// always on.
    #[serde(default)]
    pub events: Vec<EventExpect>,
}

/// How the loaded structure is settled before tick 0.
#[derive(Debug, Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SettleMode {
    /// Vanilla placement pass + ordered settle — a build saved at rest.
    #[default]
    Placement,
    /// `onPlace` only, no settle — a knownShape capture.
    Quiet,
    /// Neither — the build was recorded in the world it stood in, mid-state.
    InWorld,
}

/// One thing a player does, during tick `tick`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// The tick this fires on, after that tick's checks.
    pub tick: u64,
    /// Right-click with an empty hand (a lever, a button, a note block).
    #[serde(rename = "use")]
    pub use_pos: Option<[i32; 3]>,
    /// Write a block state (`"minecraft:air"` breaks a block).
    pub place: Option<[i32; 3]>,
    /// The state `place` writes.
    pub state: Option<String>,
}

/// One end-state assertion, evaluated after exactly `tick` ticks.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    /// The tick this looks at: the world after exactly this many steps,
    /// before any action scheduled on it fires.
    pub tick: u64,
    /// What is asserted.
    pub expect: Expect,
    /// With `expect: "same-as"`: the earlier check's tick to compare against.
    #[serde(default)]
    pub as_tick: Option<u64>,
    /// Restrict the comparison to an inclusive box; whole world when absent.
    #[serde(default)]
    pub region: Option<[[i32; 3]; 2]>,
    /// With `expect: "blocks"`: `"x,y,z"` → expected state. A descriptor
    /// without properties matches on block name alone; listed properties must
    /// each hold, unlisted ones are free (`redstone_wire[power=15]` matches
    /// any fully-connected dust at power 15).
    #[serde(default)]
    pub blocks: Option<BTreeMap<String, String>>,
    /// With `expect: "entities"`: each entry must be satisfied.
    #[serde(default)]
    pub entities: Option<Vec<EntityExpect>>,
    /// With `expect: "fill"`: the cell set whose non-air members are counted.
    /// A doorway is nine cells; how many of them are filled is the whole
    /// question, and *which* nine is the authored part.
    #[serde(default)]
    pub cells: Option<Vec<String>>,
    /// Exact count, for `entity-count`, `fill` and `changes`.
    #[serde(default)]
    pub count: Option<usize>,
    /// Lower bound, for the same three. A count is pinned with `count`;
    /// a budget is expressed with these.
    #[serde(default)]
    pub at_least: Option<usize>,
    /// Upper bound, the other half of a budget.
    #[serde(default)]
    pub at_most: Option<usize>,
    /// With `expect: "min-entity-y"`: no entity may sit below this y. The
    /// cheap, backend-agnostic form of "the build did not fall apart" — a cart
    /// that lost its NaN velocity leaves through the floor.
    #[serde(default)]
    pub y: Option<f64>,
    /// With `expect: "riders"`: the passenger kind.
    #[serde(default)]
    pub kind: Option<String>,
    /// With `expect: "riders"`: the exact seat heights the save records,
    /// ascending. Entity *seats* are structure, not physics: a rider 0.1875
    /// above its cart is where the file put it.
    #[serde(default)]
    pub seats: Option<Vec<f64>>,
}

/// One entity expectation inside an `entities` check.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityExpect {
    /// Item id for item entities (`minecraft:iron_ingot`).
    pub item: Option<String>,
    /// Entity kind for minecarts (`minecraft:minecart`).
    pub kind: Option<String>,
    /// Inclusive block box the entity's position must fall inside.
    pub region: [[i32; 3]; 2],
    /// Exact count; when absent, at least one.
    pub count: Option<usize>,
    /// Container contents the item must carry (a dropped shulker box's
    /// slots): every listed `{id, count}` must appear.
    #[serde(default)]
    pub with_contents: Option<Vec<ContentExpect>>,
    /// The item must carry no container contents at all — a shulker box that
    /// was drained before it dropped.
    #[serde(default)]
    pub empty_contents: Option<bool>,
    /// Total item count summed over matching entities. Two ejected diamonds
    /// may merge into one entity of two, or land as two of one — this asserts
    /// the diamonds, not the entity bookkeeping.
    #[serde(default)]
    pub items_total: Option<u32>,
}

/// One expectation against the recorded event log.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventExpect {
    /// The tick it must land on; any tick when absent.
    #[serde(default)]
    pub tick: Option<u64>,
    /// Count only events at or after this tick. The block-based auto-ports
    /// use it to keep an emulated start pulse from satisfying its own accept
    /// claim through adjacent wiring.
    #[serde(default)]
    pub after: Option<u64>,
    /// What kind of event. This format version understands `"block-changed"`.
    pub kind: String,
    /// The cell it must touch; anywhere when absent.
    #[serde(default)]
    pub pos: Option<[i32; 3]>,
    /// State the change must leave, subset-matched like a `blocks` check:
    /// listed properties must hold, unlisted ones are free.
    #[serde(default)]
    pub from: Option<String>,
    /// State the change must arrive at, subset-matched the same way.
    #[serde(default)]
    pub to: Option<String>,
    /// Exact match count; with `at_least`/`at_most` all absent, "at least one".
    #[serde(default)]
    pub count: Option<usize>,
    /// Lower bound on matches.
    #[serde(default)]
    pub at_least: Option<usize>,
    /// Upper bound on matches.
    #[serde(default)]
    pub at_most: Option<usize>,
}

/// One `{id, count}` a container must hold.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentExpect {
    /// The item id.
    pub id: String,
    /// The stack size.
    pub count: u8,
}

/// What a check asserts.
#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Expect {
    /// Equals the settled pre-action world (a reset check).
    Initial,
    /// Differs from initial (the machine actually moved).
    Changed,
    /// Equals the world at an earlier check's tick (`as_tick`).
    SameAs,
    /// Every block in `region` is air.
    Air,
    /// Exact states at named positions.
    Blocks,
    /// Item entities and minecarts.
    Entities,
    /// Nothing is pending: no scheduled tick, no queued update. The
    /// backend-agnostic spelling of "the run finished", and the one thing that
    /// says a door came to rest rather than still being mid-cycle.
    Quiescent,
    /// How many entities the world holds. A door glued together by nan carts
    /// is a door whose entity count is load-bearing.
    EntityCount,
    /// How many of `cells` are non-air. The doorway metric.
    Fill,
    /// The most of `cells` that were *ever* non-air, up to this tick.
    ///
    /// How far the door got, rather than where it stopped. A door leaf sweeps
    /// through the doorway and settles somewhere; the width of the sweep is the
    /// claim worth pinning, and unlike a reading at one named tick it does not
    /// care which tick the sweep peaked on.
    PeakFill,
    /// How many blocks changed over the run so far.
    Changes,
    /// The lowest y any entity occupies.
    MinEntityY,
    /// The passenger seats, ascending.
    Riders,
}

/// Does `actual` satisfy `expected`? Same block name, and every property the
/// expectation lists holds in the actual state; unlisted properties are free.
pub fn state_matches(expected: &str, actual: &str) -> bool {
    let (want_name, want_props) = match expected.split_once('[') {
        Some((name, props)) => (name, props.trim_end_matches(']')),
        None => (expected, ""),
    };
    let (got_name, got_props) = match actual.split_once('[') {
        Some((name, props)) => (name, props.trim_end_matches(']')),
        None => (actual, ""),
    };
    if want_name != got_name {
        return false;
    }
    want_props
        .split(',')
        .filter(|p| !p.is_empty())
        .all(|want| got_props.split(',').any(|got| got == want))
}

/// A world snapshot: every non-air block.
pub type Snapshot = BTreeMap<Pos, String>;

/// The versioned suite object — the third spelling of an embedded test.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SuiteDoc {
    format: u32,
    cases: Vec<Case>,
}

/// Parse an embedded test: a bare case, an array of them, or a versioned
/// `{"format": 1, "cases": [...]}` object. `what` names the carrier file so an
/// error says which build refused to parse.
pub fn parse_suite(text: &str, what: &str) -> Result<Vec<Case>, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("{what}: parsing the embedded test: {e}"))?;
    let cases: Vec<Case> = match &value {
        serde_json::Value::Array(_) => serde_json::from_value(value.clone())
            .map_err(|e| format!("{what}: parsing the embedded tests: {e}"))?,
        serde_json::Value::Object(map) if map.contains_key("cases") => {
            let doc: SuiteDoc = serde_json::from_value(value.clone())
                .map_err(|e| format!("{what}: parsing the embedded suite: {e}"))?;
            if doc.format != 1 {
                return Err(format!(
                    "{what}: suite format {} is newer than this runner understands (max 1)",
                    doc.format
                ));
            }
            doc.cases
        }
        _ => serde_json::from_value(value.clone())
            .map(|one| vec![one])
            .map_err(|e| format!("{what}: parsing the embedded test: {e}"))?,
    };
    if cases.is_empty() {
        return Err(format!("{what}: the suite has no cases — it would pass by saying nothing"));
    }
    Ok(cases)
}
