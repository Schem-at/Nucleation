//! Scheduled block ticks and block events — the ordering contract.
//!
//! Two queues with genuinely different semantics, which is why they are separate
//! types rather than one generic queue:
//!
//! - **Scheduled ticks** ([`TickQueue`]) fire in a *later* phase-3 of a *future*
//!   tick, ordered by `(target_tick, priority, insertion)`.
//! - **Block events** ([`EventQueue`]) fire in phase 7 of the *current* tick, in
//!   insertion order, and may enqueue further events that run in the same phase.
//!
//! Conflating the two is the single easiest way to produce a simulation that
//! looks plausible and gets every piston door wrong.

use crate::pos::Pos;
use std::collections::BTreeMap;

/// Ordering class for a scheduled tick.
///
/// Lower runs earlier. The values match the game's, including the negatives, so
/// that a trace carrying a raw priority is directly comparable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(i8)]
pub enum TickPriority {
    /// -3
    ExtremelyHigh = -3,
    /// -2
    VeryHigh = -2,
    /// -1
    High = -1,
    /// 0, the default.
    #[default]
    Normal = 0,
    /// 1
    Low = 1,
    /// 2
    VeryLow = 2,
    /// 3
    ExtremelyLow = 3,
}

/// A block tick scheduled to fire on a future tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduledTick {
    /// Where it fires.
    pub pos: Pos,
    /// The tick number it fires on.
    pub target: u64,
    /// Ordering class within that tick.
    pub priority: TickPriority,
    /// Monotonic insertion counter, breaking ties within a `(target, priority)`.
    ///
    /// Without this, two ticks scheduled for the same moment at the same
    /// priority would run in whatever order the container produced, and the
    /// simulation would stop being reproducible.
    pub sequence: u64,
}

/// Scheduled block ticks, keyed by the tick they fire on.
///
/// # Implementation note
///
/// A `BTreeMap` of buckets, not the ring buffer the design sketch called for.
/// The ring is faster but needs a bounded lookahead and an overflow path, and
/// nothing here is measured yet. Correctness first; the benchmark suite is the
/// gate that decides whether this ever needs replacing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickQueue {
    buckets: BTreeMap<u64, Vec<ScheduledTick>>,
    next_sequence: u64,
    /// Collected for this tick and not yet run — vanilla's `toRunThisTickSet`.
    running: Vec<Pos>,
}

impl TickQueue {
    /// An empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Schedule a tick at `pos` to fire `delay` ticks after `now`.
    ///
    /// A `delay` of 0 fires on the current tick's own block-tick phase if that
    /// phase has not run yet; the game permits this and some contraptions rely
    /// on it, so it is not rejected here.
    pub fn schedule(&mut self, pos: Pos, now: u64, delay: u64, priority: TickPriority) {
        let entry = ScheduledTick {
            pos,
            target: now.saturating_add(delay),
            priority,
            sequence: self.next_sequence,
        };
        self.next_sequence += 1;
        self.buckets.entry(entry.target).or_default().push(entry);
    }

    /// Whether a tick is already scheduled at `pos` on or after `now`.
    ///
    /// The game refuses to double-schedule the same position, and blocks query
    /// this before scheduling. Getting it wrong produces doubled delays that
    /// look like an off-by-one in the block's logic.
    pub fn has_pending_at(&self, pos: Pos, now: u64) -> bool {
        self.running.iter().any(|entry| *entry == pos)
            || self
                .buckets
                .range(now..)
                .any(|(_, entries)| entries.iter().any(|entry| entry.pos == pos))
    }

    /// Take everything due, keeping it visible to [`Self::has_pending_at`]
    /// until it has actually run.
    ///
    /// `LevelTicks` collects the due ticks into `toRunThisTick` and polls them
    /// one at a time, so a tick still waiting its turn answers
    /// `willTickThisTick` with *true*. Draining into a list and forgetting
    /// about it answers false instead, and a torch notified while an earlier
    /// tick runs books a second tick vanilla never books.
    pub fn collect_due(&mut self, tick: u64) -> Vec<ScheduledTick> {
        let due = self.drain_due(tick);
        self.running = due.iter().map(|entry| entry.pos).collect();
        due
    }

    /// One collected tick has now run, so it stops counting as pending.
    pub fn finished(&mut self, pos: Pos) {
        if let Some(index) = self.running.iter().position(|entry| *entry == pos) {
            self.running.swap_remove(index);
        }
    }

    /// Remove and return everything due on or before `tick`, in firing order.
    ///
    /// Ordering is `(target, priority, sequence)`. Anything scheduled for an
    /// earlier tick that was somehow missed is included rather than stranded —
    /// a stuck entry would silently freeze part of a contraption.
    pub fn drain_due(&mut self, tick: u64) -> Vec<ScheduledTick> {
        let due: Vec<u64> = self.buckets.range(..=tick).map(|(k, _)| *k).collect();
        let mut out = Vec::new();
        for key in due {
            if let Some(entries) = self.buckets.remove(&key) {
                out.extend(entries);
            }
        }
        out.sort_by_key(|entry| (entry.target, entry.priority, entry.sequence));
        out
    }

    /// Whether anything is scheduled.
    pub fn is_empty(&self) -> bool {
        self.buckets.values().all(|entries| entries.is_empty())
    }

    /// Total scheduled entries.
    pub fn len(&self) -> usize {
        self.buckets.values().map(Vec::len).sum()
    }

    /// The earliest tick anything is scheduled for.
    ///
    /// Used to decide whether a world has gone quiescent.
    /// Every pending tick as `(tick it fires on, position)`, ascending.
    ///
    /// For comparing against a capture's scheduled list: agreeing on the world
    /// while disagreeing on what is pending in it is the normal shape of a
    /// divergence, and it is invisible to a snapshot diff.
    pub fn pending(&self) -> Vec<(u64, Pos)> {
        self.buckets
            .iter()
            .flat_map(|(at, list)| list.iter().map(move |t| (*at, t.pos)))
            .collect()
    }

    pub fn next_due(&self) -> Option<u64> {
        self.buckets
            .iter()
            .find(|(_, entries)| !entries.is_empty())
            .map(|(tick, _)| *tick)
    }
}

/// A block event — the mechanism pistons move by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEvent {
    /// Where it fires.
    pub pos: Pos,
    /// Event type, block-defined (for a piston: extend or retract).
    pub id: u8,
    /// Event parameter, block-defined (for a piston: the facing).
    pub param: u8,
    /// A state of the block this was queued for.
    ///
    /// `doBlockEvent` refuses the event when the position no longer holds
    /// that **block** — properties may change freely. Part of the identity for
    /// the queue's set semantics, exactly as `BlockEventData`'s record
    /// equality includes it.
    pub block: crate::state::StateId,
}

/// Block events for the current tick, in insertion order.
///
/// Drained during [`Phase::BlockEvents`]. A handler may push further events,
/// which run in the same phase — the game chains them, and piston contraptions
/// depend on that chaining.
///
/// [`Phase::BlockEvents`]: crate::phase::Phase::BlockEvents
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventQueue {
    events: Vec<BlockEvent>,
}

impl EventQueue {
    /// An empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event, unless an identical one is already queued.
    ///
    /// `ServerLevel` holds pending block events in an `ObjectLinkedOpenHashSet` —
    /// insertion-ordered, but a *set*: queueing the same `(pos, block, id, param)`
    /// twice is a no-op. That dedup is load-bearing. Placing a structure gives a
    /// piston several neighbour updates in a row, each of which queues the same
    /// extend event; without the set semantics the piston would extend and then
    /// receive the stale duplicates in its extended state, which vanilla reads as
    /// a retract request.
    pub fn push(&mut self, event: BlockEvent) {
        if self.events.contains(&event) {
            return;
        }
        self.events.push(event);
    }

    /// Take everything currently queued, leaving the queue empty.
    ///
    /// Callers loop on this: handling a batch may enqueue the next batch, and
    /// the phase is over only when a drain comes back empty.
    /// The queued events without draining them.
    pub fn peek(&self) -> &[BlockEvent] {
        &self.events
    }

    pub fn take(&mut self) -> Vec<BlockEvent> {
        std::mem::take(&mut self.events)
    }

    /// Whether anything is queued.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// How many events are queued.
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: i32) -> Pos {
        Pos::new(x, 0, 0)
    }

    #[test]
    fn ticks_fire_in_target_then_priority_then_insertion_order() {
        let mut queue = TickQueue::new();
        // Same target tick, deliberately scheduled worst-first.
        queue.schedule(pos(1), 0, 1, TickPriority::Low);
        queue.schedule(pos(2), 0, 1, TickPriority::ExtremelyHigh);
        queue.schedule(pos(3), 0, 1, TickPriority::Normal);
        queue.schedule(pos(4), 0, 1, TickPriority::ExtremelyHigh);
        // A later target must not jump the queue.
        queue.schedule(pos(5), 0, 2, TickPriority::ExtremelyHigh);

        let fired: Vec<i32> = queue.drain_due(1).iter().map(|t| t.pos.x).collect();
        // 2 and 4 share the highest priority, so insertion order separates them.
        assert_eq!(fired, vec![2, 4, 3, 1]);
        assert_eq!(queue.next_due(), Some(2));
    }

    #[test]
    fn priority_ordering_matches_the_games_numeric_values() {
        assert!(TickPriority::ExtremelyHigh < TickPriority::Normal);
        assert!(TickPriority::Normal < TickPriority::ExtremelyLow);
        assert_eq!(TickPriority::Normal as i8, 0);
        assert_eq!(TickPriority::ExtremelyHigh as i8, -3);
    }

    #[test]
    fn overdue_ticks_are_not_stranded() {
        // An entry scheduled for a tick we have already passed must still fire,
        // or part of a contraption silently freezes.
        let mut queue = TickQueue::new();
        queue.schedule(pos(1), 0, 1, TickPriority::Normal);
        let fired = queue.drain_due(50);
        assert_eq!(fired.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn pending_lookup_sees_future_ticks_only() {
        let mut queue = TickQueue::new();
        queue.schedule(pos(1), 10, 5, TickPriority::Normal);
        assert!(queue.has_pending_at(pos(1), 10));
        assert!(!queue.has_pending_at(pos(2), 10));
        queue.drain_due(15);
        assert!(!queue.has_pending_at(pos(1), 15));
    }

    #[test]
    fn zero_delay_scheduling_targets_the_current_tick() {
        let mut queue = TickQueue::new();
        queue.schedule(pos(1), 7, 0, TickPriority::Normal);
        assert_eq!(queue.next_due(), Some(7));
    }

    #[test]
    fn events_drain_in_insertion_order_and_support_chaining() {
        let mut queue = EventQueue::new();
        queue.push(BlockEvent { pos: pos(1), id: 0, param: 0, block: crate::state::StateId::AIR });
        queue.push(BlockEvent { pos: pos(2), id: 1, param: 0, block: crate::state::StateId::AIR });

        let batch = queue.take();
        assert_eq!(batch.iter().map(|e| e.pos.x).collect::<Vec<_>>(), vec![1, 2]);
        assert!(queue.is_empty(), "take must leave the queue empty");

        // Handling a batch may enqueue the next one; that is the chaining the
        // block-events phase loops on.
        queue.push(BlockEvent { pos: pos(3), id: 0, param: 0, block: crate::state::StateId::AIR });
        assert_eq!(queue.take().len(), 1);
        assert!(queue.take().is_empty(), "chain terminates on an empty drain");
    }

    #[test]
    fn duplicate_events_collapse_like_vanillas_set() {
        // ServerLevel's block-event container is an ObjectLinkedOpenHashSet:
        // insertion-ordered but deduplicating. A piston notified from several
        // sides queues its extend event once, not once per side.
        let mut queue = EventQueue::new();
        queue.push(BlockEvent { pos: pos(1), id: 0, param: 2, block: crate::state::StateId::AIR });
        queue.push(BlockEvent { pos: pos(1), id: 0, param: 2, block: crate::state::StateId::AIR });
        queue.push(BlockEvent { pos: pos(1), id: 1, param: 2, block: crate::state::StateId::AIR });
        assert_eq!(queue.len(), 2, "identical events collapse, distinct ones do not");

        // Dedup is against the *currently queued* batch only — once drained, the
        // same event may be queued again, which chained piston cycles rely on.
        queue.take();
        queue.push(BlockEvent { pos: pos(1), id: 0, param: 2, block: crate::state::StateId::AIR });
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn sequence_numbers_keep_identical_schedules_reproducible() {
        let mut a = TickQueue::new();
        let mut b = TickQueue::new();
        for queue in [&mut a, &mut b] {
            for x in 0..8 {
                queue.schedule(pos(x), 0, 1, TickPriority::Normal);
            }
        }
        assert_eq!(a.drain_due(1), b.drain_due(1));
    }
}
