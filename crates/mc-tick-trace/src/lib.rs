//! The trace format: a contract between the Rust tick engine and the Java
//! vanilla-capture mod.
//!
//! # Why this crate exists
//!
//! Correctness in this project is established by *differential testing against
//! the real game*, not by reading anyone's source. That only works if both sides
//! can produce the same shape of observation. This crate is that shape, and
//! nothing else — no engine, no capture logic, just the schema and the ordering
//! rules that make two traces comparable.
//!
//! # What a trace records
//!
//! A flat, ordered list of [`TraceEvent`]s per tick. Every event carries the
//! [`phase`](TraceEvent::phase) it happened in, which is the whole point: when a
//! trace diverges, the first question is "which phase?", and a trace that only
//! recorded *what* changed without *when* could not answer it.
//!
//! # Why events are ordered and comparison is exact
//!
//! Two traces are equal only if their events match in order. That strictness is
//! deliberate — a piston door's behaviour *is* its update order, so a comparison
//! tolerant of reordering would accept exactly the bugs we are hunting.
//!
//! Entity motion is the one exception. Positions are floats, and bit-exact
//! agreement across two independent implementations is not a reasonable target,
//! so [`EventKind::EntityMoved`] compares within a tolerance. See
//! [`Trace::diff_with_tolerance`].

use serde::{Deserialize, Serialize};

/// The schema version.
///
/// Bumped whenever the meaning of an existing field changes, which invalidates
/// recorded goldens. Additive changes that old readers can ignore do not bump it.
pub const FORMAT_VERSION: u32 = 1;

/// A block position.
///
/// Serialised as a three-element array, so a trace stays readable and compact:
/// `[1, 2, 3]` rather than an object per coordinate, of which a trace holds
/// thousands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TracePos(pub i32, pub i32, pub i32);

impl TracePos {
    /// A position.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(x, y, z)
    }
}

impl std::fmt::Display for TracePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

/// What happened.
///
/// `#[serde(tag = "kind", rename_all = "snake_case")]` gives an externally
/// legible discriminant, because these files get read by people diagnosing a
/// divergence, not only by machines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    /// A block's state changed.
    ///
    /// The primary observable. `from` and `to` are state descriptors —
    /// `minecraft:repeater[delay=2,facing=north,powered=false]` — because
    /// numeric state ids are not stable across versions or implementations, and
    /// a trace has to survive both.
    BlockChanged {
        /// Where.
        pos: TracePos,
        /// State before.
        from: String,
        /// State after.
        to: String,
    },

    /// A block tick was scheduled.
    ScheduledTickAdded {
        /// Where.
        pos: TracePos,
        /// Ticks from now until it fires.
        delay: u64,
        /// Raw priority, using the game's numbering (-3 highest, 3 lowest).
        priority: i8,
    },

    /// A scheduled block tick fired.
    ScheduledTickFired {
        /// Where.
        pos: TracePos,
        /// Raw priority.
        priority: i8,
    },

    /// A block event fired — the mechanism pistons move by.
    BlockEvent {
        /// Where.
        pos: TracePos,
        /// Block-defined event type.
        id: u8,
        /// Block-defined parameter.
        param: u8,
    },

    /// A neighbour was notified of a change.
    ///
    /// Extremely numerous, so capture is opt-in via [`Detail::Verbose`]. Worth
    /// having because when a redstone divergence is *not* explained by the
    /// scheduled-tick order, the neighbour notification order is where the answer
    /// almost always is.
    NeighborUpdate {
        /// The block being notified.
        pos: TracePos,
        /// The block that changed, causing the notification.
        from: TracePos,
    },

    /// A container slot's contents changed.
    ///
    /// The observable of item logistics: a hopper transfer changes only
    /// block-entity NBT, which a block-state diff cannot see. `from` and `to`
    /// render as `"<count>x <id>"` (`"3x minecraft:redstone"`), or `""` for an
    /// empty slot — strings for the same reason block states are.
    InventoryChanged {
        /// The container's position.
        pos: TracePos,
        /// Which slot.
        slot: u32,
        /// Contents before.
        from: String,
        /// Contents after.
        to: String,
    },

    /// An entity moved.
    ///
    /// The only event compared with tolerance rather than exactly. See the module
    /// docs.
    EntityMoved {
        /// Stable id within the trace.
        id: u32,
        /// Entity type, e.g. `minecraft:item`.
        entity_type: String,
        /// Position after the move.
        pos: [f64; 3],
        /// Velocity after the move.
        velocity: [f64; 3],
    },
}

/// One observation, tagged with the phase it occurred in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    /// The tick phase this happened in, e.g. `block_events`.
    ///
    /// A string rather than an enum so this crate need not depend on the engine,
    /// and so an unrecognised phase from a future capture deserialises instead of
    /// failing. The names come from `mc_tick::Phase::name`.
    pub phase: String,
    /// What happened.
    #[serde(flatten)]
    pub kind: EventKind,
}

/// Everything observed during one tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickRecord {
    /// The tick number this covers.
    pub tick: u64,
    /// Events in the order they occurred.
    pub events: Vec<TraceEvent>,
}

/// How much to capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Detail {
    /// Block changes, scheduled ticks, and block events.
    #[default]
    Normal,
    /// Also neighbour updates. Large, and the tool of choice for an ordering bug.
    Verbose,
}

/// A complete recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    /// Schema version; see [`FORMAT_VERSION`].
    pub format_version: u32,
    /// Which Minecraft version produced this, e.g. `26.2`.
    ///
    /// Recorded because vanilla behaviour changes between versions, and a golden
    /// captured from a different version is worse than no golden — it looks
    /// authoritative while encoding different rules.
    pub mc_version: String,
    /// Which structure was run.
    pub structure: String,
    /// Capture detail.
    pub detail: Detail,
    /// Per-tick records, ascending.
    pub ticks: Vec<TickRecord>,
}

/// Where two traces first differ.
#[derive(Debug, Clone, PartialEq)]
pub struct Divergence {
    /// The tick it happened on.
    pub tick: u64,
    /// The phase, if both sides agree there is one.
    pub phase: Option<String>,
    /// Index into that tick's event list.
    pub event_index: usize,
    /// Human-readable account of the difference.
    pub detail: String,
}

impl std::fmt::Display for Divergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tick {}", self.tick)?;
        if let Some(phase) = &self.phase {
            write!(f, ", phase {phase}")?;
        }
        write!(f, ", event {}: {}", self.event_index, self.detail)
    }
}

impl Trace {
    /// An empty trace for `structure`.
    pub fn new(mc_version: impl Into<String>, structure: impl Into<String>, detail: Detail) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            mc_version: mc_version.into(),
            structure: structure.into(),
            detail,
            ticks: Vec::new(),
        }
    }

    /// Pretty JSON, newline-terminated.
    ///
    /// Pretty rather than compact, and stable in field order, because these are
    /// committed as goldens and reviewed in diffs. A compact trace would turn a
    /// one-event change into an unreadable single-line diff.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        Ok(json)
    }

    /// Parse a trace.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Sort each tick's events into a canonical order.
    ///
    /// # When this is the right thing, and when it is a lie
    ///
    /// A trace captured by **diffing snapshots between ticks** cannot observe the
    /// order events happened in — what it records is the scan order of whatever
    /// walked the region. Comparing that against an engine's emission order
    /// compares two arbitrary iteration orders and calls the difference a bug.
    /// Canonicalising both sides first is the honest comparison: it asserts *what*
    /// changed on each tick, which is exactly what such a capture knows.
    ///
    /// A trace captured by **instrumenting the tick loop** does know the real
    /// order, and canonicalising it would throw away the most valuable thing it
    /// has. Do not call this on one.
    ///
    /// Ordering is by position in the world's canonical order (y, then z, then x),
    /// then by the event's own debug rendering to break remaining ties.
    pub fn canonicalize(&mut self) {
        for record in &mut self.ticks {
            record.events.sort_by_key(|event| {
                let pos = match &event.kind {
                    EventKind::BlockChanged { pos, .. }
                    | EventKind::ScheduledTickAdded { pos, .. }
                    | EventKind::ScheduledTickFired { pos, .. }
                    | EventKind::BlockEvent { pos, .. }
                    | EventKind::NeighborUpdate { pos, .. }
                    | EventKind::InventoryChanged { pos, .. } => *pos,
                    EventKind::EntityMoved { .. } => TracePos::new(0, 0, 0),
                };
                (pos.1, pos.2, pos.0, format!("{:?}", event.kind))
            });
        }
    }

    /// A copy with each tick's events canonically ordered. See [`Trace::canonicalize`].
    pub fn canonicalized(&self) -> Trace {
        let mut copy = self.clone();
        copy.canonicalize();
        copy
    }

    /// Compare against `other` exactly, returning the first divergence.
    ///
    /// `self` is the expected (golden) side and `other` the actual.
    pub fn diff(&self, other: &Trace) -> Option<Divergence> {
        self.diff_with_tolerance(other, 0.0)
    }

    /// Compare, allowing entity positions and velocities to differ by up to
    /// `tolerance` per component.
    ///
    /// Everything else still compares exactly: a block either changed to the
    /// right state or it did not, and there is no meaningful "close" for a
    /// redstone signal.
    pub fn diff_with_tolerance(&self, other: &Trace, tolerance: f64) -> Option<Divergence> {
        if self.ticks.len() != other.ticks.len() {
            return Some(Divergence {
                tick: self.ticks.len().min(other.ticks.len()) as u64,
                phase: None,
                event_index: 0,
                detail: format!(
                    "tick count differs: expected {}, got {}",
                    self.ticks.len(),
                    other.ticks.len()
                ),
            });
        }

        for (expected, actual) in self.ticks.iter().zip(&other.ticks) {
            if expected.tick != actual.tick {
                return Some(Divergence {
                    tick: expected.tick,
                    phase: None,
                    event_index: 0,
                    detail: format!(
                        "tick number differs: expected {}, got {}",
                        expected.tick, actual.tick
                    ),
                });
            }

            let common = expected.events.len().min(actual.events.len());
            for index in 0..common {
                let a = &expected.events[index];
                let b = &actual.events[index];
                if a.phase != b.phase {
                    return Some(Divergence {
                        tick: expected.tick,
                        phase: Some(a.phase.clone()),
                        event_index: index,
                        detail: format!(
                            "phase differs: expected {}, got {}",
                            a.phase, b.phase
                        ),
                    });
                }
                if !kinds_match(&a.kind, &b.kind, tolerance) {
                    return Some(Divergence {
                        tick: expected.tick,
                        phase: Some(a.phase.clone()),
                        event_index: index,
                        detail: format!("expected {:?}, got {:?}", a.kind, b.kind),
                    });
                }
            }

            if expected.events.len() != actual.events.len() {
                // Report the shorter side's end as the divergence point: that is
                // where the traces stopped agreeing, and it is the first place
                // worth looking.
                let (longer, which) = if expected.events.len() > actual.events.len() {
                    (&expected.events, "expected")
                } else {
                    (&actual.events, "actual")
                };
                return Some(Divergence {
                    tick: expected.tick,
                    phase: longer.get(common).map(|e| e.phase.clone()),
                    event_index: common,
                    detail: format!(
                        "event count differs: expected {}, got {} ({which} has the extra: {:?})",
                        expected.events.len(),
                        actual.events.len(),
                        longer.get(common).map(|e| &e.kind)
                    ),
                });
            }
        }

        None
    }
}

/// Whether two events match, applying `tolerance` only to entity motion.
fn kinds_match(a: &EventKind, b: &EventKind, tolerance: f64) -> bool {
    match (a, b) {
        (
            EventKind::EntityMoved {
                id: id_a,
                entity_type: type_a,
                pos: pos_a,
                velocity: vel_a,
            },
            EventKind::EntityMoved {
                id: id_b,
                entity_type: type_b,
                pos: pos_b,
                velocity: vel_b,
            },
        ) => {
            id_a == id_b
                && type_a == type_b
                && within(pos_a, pos_b, tolerance)
                && within(vel_a, vel_b, tolerance)
        }
        _ => a == b,
    }
}

fn within(a: &[f64; 3], b: &[f64; 3], tolerance: f64) -> bool {
    a.iter()
        .zip(b)
        .all(|(x, y)| (x - y).abs() <= tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(phase: &str, pos: i32) -> TraceEvent {
        TraceEvent {
            phase: phase.to_string(),
            kind: EventKind::BlockChanged {
                pos: TracePos::new(pos, 0, 0),
                from: "minecraft:air".into(),
                to: "minecraft:stone".into(),
            },
        }
    }

    fn trace(ticks: Vec<TickRecord>) -> Trace {
        Trace {
            format_version: FORMAT_VERSION,
            mc_version: "26.2".into(),
            structure: "test".into(),
            detail: Detail::Normal,
            ticks,
        }
    }

    #[test]
    fn identical_traces_do_not_diverge() {
        let a = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_ticks", 1), event("block_events", 2)],
        }]);
        assert_eq!(a.diff(&a.clone()), None);
    }

    #[test]
    fn reordered_events_are_a_divergence() {
        // The whole point: a door's behaviour *is* its update order, so a
        // comparison tolerant of reordering would accept the bugs we hunt.
        let expected = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_ticks", 1), event("block_ticks", 2)],
        }]);
        let actual = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_ticks", 2), event("block_ticks", 1)],
        }]);
        let divergence = expected.diff(&actual).expect("reordering must diverge");
        assert_eq!(divergence.event_index, 0);
        assert_eq!(divergence.tick, 0);
    }

    #[test]
    fn a_wrong_phase_is_reported_as_a_phase_divergence() {
        let expected = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_events", 1)],
        }]);
        let actual = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_ticks", 1)],
        }]);
        let d = expected.diff(&actual).unwrap();
        assert!(d.detail.contains("phase differs"), "{}", d.detail);
        assert_eq!(d.phase.as_deref(), Some("block_events"));
    }

    #[test]
    fn a_missing_event_reports_which_side_had_the_extra() {
        let expected = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_ticks", 1), event("block_ticks", 2)],
        }]);
        let actual = trace(vec![TickRecord {
            tick: 0,
            events: vec![event("block_ticks", 1)],
        }]);
        let d = expected.diff(&actual).unwrap();
        assert_eq!(d.event_index, 1);
        assert!(d.detail.contains("expected has the extra"), "{}", d.detail);
    }

    #[test]
    fn entity_motion_compares_within_tolerance_but_blocks_never_do() {
        let moved = |x: f64| TickRecord {
            tick: 0,
            events: vec![TraceEvent {
                phase: "entities".into(),
                kind: EventKind::EntityMoved {
                    id: 1,
                    entity_type: "minecraft:item".into(),
                    pos: [x, 64.0, 0.0],
                    velocity: [0.0, 0.0, 0.0],
                },
            }],
        };
        let expected = trace(vec![moved(1.0)]);
        let close = trace(vec![moved(1.0 + 1e-9)]);
        let far = trace(vec![moved(1.5)]);

        assert_eq!(expected.diff_with_tolerance(&close, 1e-6), None);
        assert!(expected.diff_with_tolerance(&far, 1e-6).is_some());
        // Exact comparison rejects even the tiny difference.
        assert!(expected.diff(&close).is_some());
    }

    #[test]
    fn json_round_trips() {
        let original = trace(vec![TickRecord {
            tick: 3,
            events: vec![
                event("block_ticks", 1),
                TraceEvent {
                    phase: "block_events".into(),
                    kind: EventKind::BlockEvent {
                        pos: TracePos::new(1, 2, 3),
                        id: 0,
                        param: 5,
                    },
                },
            ],
        }]);
        let json = original.to_json().unwrap();
        assert_eq!(Trace::from_json(&json).unwrap(), original);
        assert!(json.ends_with('\n'), "goldens must be newline-terminated");
    }

    #[test]
    fn positions_serialise_as_compact_arrays() {
        // A trace holds thousands of positions; an object per coordinate would
        // make goldens unreadable and needlessly large.
        let json = serde_json::to_string(&TracePos::new(1, -2, 3)).unwrap();
        assert_eq!(json, "[1,-2,3]");
    }

    #[test]
    fn differing_tick_counts_are_reported_before_event_comparison() {
        let expected = trace(vec![
            TickRecord { tick: 0, events: vec![] },
            TickRecord { tick: 1, events: vec![] },
        ]);
        let actual = trace(vec![TickRecord { tick: 0, events: vec![] }]);
        let d = expected.diff(&actual).unwrap();
        assert!(d.detail.contains("tick count differs"), "{}", d.detail);
    }

    #[test]
    fn event_kind_tags_are_readable_in_json() {
        let json = serde_json::to_string(&EventKind::ScheduledTickFired {
            pos: TracePos::new(0, 0, 0),
            priority: -3,
        })
        .unwrap();
        assert!(json.contains("\"kind\":\"scheduled_tick_fired\""), "{json}");
    }
}
