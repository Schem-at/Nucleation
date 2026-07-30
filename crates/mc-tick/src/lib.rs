//! A deterministic, steppable, vanilla-accurate Minecraft tick simulation.
//!
//! # What this crate is for
//!
//! Simulating a schematic tick-by-tick the way the real game would, with full
//! control over the run: single-step it, run it to quiescence, snapshot it,
//! rewind it. Timing analysis (piston doors and the like) is built *on top* of
//! this, not inside it.
//!
//! # The two rules that shape everything here
//!
//! **1. The tick phase order is the backbone.** A tick is not "process the
//! pending updates" — it is ten named phases in a fixed order, and a piston's
//! motion alone spans three of them ([`Phase::BlockTicks`] →
//! [`Phase::BlockEvents`] → [`Phase::BlockEntities`]). Any structure that
//! blurs those phases cannot produce correct doors. See [`phase`].
//!
//! **2. Correctness is established by differential testing against the real
//! game**, not by reading anyone's source. No code in this crate is copied from
//! any reference implementation; behaviour is derived from captured vanilla
//! traces. That is what makes this a clean-room implementation.
//!
//! # Determinism
//!
//! The core is single-threaded on purpose. Identical inputs must produce
//! byte-identical traces, because the trace differ is the entire quality
//! mechanism. Throughput comes from running many *independent* simulations in
//! parallel — which is the real workload — never from parallelising one tick.

#![forbid(unsafe_code)]

pub mod behaviour;
pub mod entity;
pub mod entity_kind;
pub mod fluid;
pub mod minecart;
pub mod motion;
pub mod inventory;
pub mod components;
pub mod machine_graph;
pub mod observer;
pub mod phase;
pub mod piston;
pub mod pos;
pub mod redstone;
pub mod rng;
pub mod schedule;
pub mod sim;
pub mod state;
pub mod structure;
pub mod wire;
pub mod vanilla;
pub mod world;

pub use behaviour::{BehaviourTable, BlockBehaviour, Inert, TickCtx, UpdateKind, UpdateRecord};
pub use components::{Comparator, PowerSource, Repeater, StatePair, Torch,
    COMPARATOR_DELAY, REPEATER_TICKS_PER_DELAY, TORCH_DELAY};
pub use observer::{Observer, OBSERVER_PULSE_TICKS};
pub use phase::{Phase, PHASE_ORDER};
pub use machine_graph::{analyse as analyse_machine, MachineGraph};
pub use piston::{resolve_push, Movability, Piston, PushPlan, MAX_PUSH_DEPTH,
    TRIGGER_CONTRACT, TRIGGER_DROP, TRIGGER_EXTEND};
pub use pos::{Bounds, Dir, Pos, ALL_DIRS};
pub use redstone::{RedstoneNetwork, MAX_POWER};
pub use schedule::{BlockEvent, EventQueue, ScheduledTick, TickPriority, TickQueue};
pub use sim::{Checkpoint, Simulation, StopReason};
pub use state::{StateError, StateId, StateRegistry};
pub use structure::{Structure, StructureError};
pub use entity::{ItemEntities, ItemEntityState};
pub use motion::MotionSemantics;
pub use inventory::{Inventory, ItemStack};
pub use vanilla::{intern_companions, register_all, register_all_at, Descriptor, VanillaRules};
pub use world::World;
