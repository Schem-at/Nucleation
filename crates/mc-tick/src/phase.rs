//! The ten phases of a server tick, in the order the game runs them.
//!
//! Reference: <https://techmcdocs.github.io/pages/GameTick/>
//!
//! This module deliberately contains no logic. It exists so that the phase
//! order is a single declared fact that the scheduler walks, rather than an
//! emergent property of however the code happens to be arranged. When a trace
//! diverges from vanilla, the first question is always "which phase?", and that
//! question should have a structural answer.
//!
//! # Why phases we don't simulate are still listed
//!
//! [`Phase::Raids`] and [`Phase::Weather`] are no-ops today and may be for a
//! long time. They are still named, still walked, and still occupy their real
//! position in the order. Filling one in later must never require re-plumbing
//! the ones around it, and a reader comparing this list against the reference
//! should find it complete rather than pruned.

/// One phase of a server tick.
///
/// The discriminants are the authoritative order: `PlayerInputs` runs last,
/// `WorldBorder` first. Do not reorder these to suit an implementation
/// convenience — the order *is* the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Phase {
    /// Resize the world border if a change is in progress.
    WorldBorder = 0,
    /// Rain, thunder, and player sleep handling.
    Weather = 1,
    /// Scheduled block ticks ("tile ticks").
    ///
    /// Ordered by `(scheduled_tick, priority, insertion)`. This is where a
    /// piston *decides* to move; it is not where it moves.
    BlockTicks = 2,
    /// Scheduled fluid ticks. Same cache discipline as [`Phase::BlockTicks`].
    FluidTicks = 3,
    /// Raid processing. Not simulated.
    Raids = 4,
    /// Chunk load/unload, mob spawning, spawners, and random block ticks.
    ///
    /// Only the random-tick portion is likely to ever matter here, and even
    /// that is irrelevant to deterministic redstone timing.
    ChunkManager = 5,
    /// Block events: piston extend/retract, noteblocks, chests, bells.
    ///
    /// Held in **insertion order** and able to **chain within the same tick** —
    /// an event's handler may enqueue another event that runs in this same
    /// phase. This is where a piston actually starts moving.
    BlockEvents = 6,
    /// Entity ticking: despawn checks, movement, removal, spawn population.
    Entities = 7,
    /// Block entities: furnaces, hoppers, sculk sensors, and **moving blocks**.
    ///
    /// A piston's motion *completes* here, a full two phases after the block
    /// event that began it.
    BlockEntities = 8,
    /// Player-driven changes: lever flips, button presses, placement, breaking.
    ///
    /// Last in the tick, which is why an input applied "now" is only observed
    /// by the world on the *following* tick.
    PlayerInputs = 9,
}

/// Every phase, in execution order.
///
/// The scheduler walks exactly this slice. Nothing should iterate phases any
/// other way.
pub const PHASE_ORDER: [Phase; 10] = [
    Phase::WorldBorder,
    Phase::Weather,
    Phase::BlockTicks,
    Phase::FluidTicks,
    Phase::Raids,
    Phase::ChunkManager,
    Phase::BlockEvents,
    Phase::Entities,
    Phase::BlockEntities,
    Phase::PlayerInputs,
];

impl Phase {
    /// A stable name, used in traces and divergence reports.
    ///
    /// These strings are part of the trace format's contract with the Java
    /// capture mod — changing one invalidates recorded goldens.
    pub const fn name(self) -> &'static str {
        match self {
            Phase::WorldBorder => "world_border",
            Phase::Weather => "weather",
            Phase::BlockTicks => "block_ticks",
            Phase::FluidTicks => "fluid_ticks",
            Phase::Raids => "raids",
            Phase::ChunkManager => "chunk_manager",
            Phase::BlockEvents => "block_events",
            Phase::Entities => "entities",
            Phase::BlockEntities => "block_entities",
            Phase::PlayerInputs => "player_inputs",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_order_matches_declared_discriminants() {
        // Guards against someone reordering PHASE_ORDER or the enum without the
        // other. The discriminant order is the specification; this keeps the
        // walked slice honest to it.
        for (index, phase) in PHASE_ORDER.iter().enumerate() {
            assert_eq!(
                *phase as usize, index,
                "PHASE_ORDER[{index}] is {phase:?}, whose discriminant is {}",
                *phase as usize
            );
        }
    }

    #[test]
    fn piston_motion_spans_three_ascending_phases() {
        // The single most load-bearing ordering fact in the crate: a piston
        // decides in BlockTicks, starts moving in BlockEvents, and finishes in
        // BlockEntities. If this ever stops holding, every door timing is wrong.
        assert!(Phase::BlockTicks < Phase::BlockEvents);
        assert!(Phase::BlockEvents < Phase::BlockEntities);
    }

    #[test]
    fn player_input_is_last_so_inputs_are_observed_next_tick() {
        assert_eq!(PHASE_ORDER.last(), Some(&Phase::PlayerInputs));
    }

    #[test]
    fn phase_names_are_unique() {
        let mut names: Vec<&str> = PHASE_ORDER.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate phase name in trace contract");
    }
}
