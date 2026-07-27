//! Torches, repeaters and comparators: the components that make redstone *timed*.
//!
//! Dust settles instantly (see [`crate::redstone`]). Everything that gives a
//! contraption its timing lives here, and the timing comes from two things —
//! **delay** and **tick priority**. Both are taken from Minecraft 26.2's own
//! compiled code, which ships unobfuscated; see `redstone_components.md` for the
//! full reading.
//!
//! # Verified delays
//!
//! | component | delay | source |
//! |---|---|---|
//! | repeater | `delay` property (1–4) **× 2** game ticks | `RepeaterBlock.getDelay` |
//! | comparator | 2 game ticks | `ComparatorBlock` |
//! | torch | [`TORCH_DELAY`] game ticks | `RedstoneTorchBlock` |
//!
//! # Verified priorities, and why they matter more than the delays
//!
//! `DiodeBlock.checkTickOnNeighbor` picks one of three:
//!
//! - [`TickPriority::ExtremelyHigh`] when `shouldPrioritize` holds — the block
//!   *behind* the diode is itself a diode facing it. This is "repeater priority",
//!   and it is what makes chains of repeaters resolve in a stable order.
//! - [`TickPriority::VeryHigh`] when the diode is currently powered, i.e. about to
//!   turn **off**.
//! - [`TickPriority::High`] otherwise, i.e. turning **on**.
//!
//! A redstone torch schedules with **no priority argument at all**, so it runs at
//! [`TickPriority::Normal`] — strictly after every diode in the same tick. That
//! difference is not cosmetic: it decides which component observes which, and it is
//! the kind of detail that makes a piston fire one tick early or late.
//!
//! # What is *not* verified here
//!
//! Comparator priming, repeater locking, and torch burnout are described in
//! `redstone_components.md` but deliberately **not** implemented, because they are
//! exactly the behaviours where a plausible reading goes subtly wrong. They need
//! traces first.

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::pos::{Dir, Pos};
use crate::schedule::TickPriority;
use crate::state::StateId;
use crate::world::World;

/// Game ticks between a redstone torch seeing a change and acting on it.
///
/// One "redstone tick". `RedstoneTorchBlock` schedules with the plain
/// `scheduleTick(pos, block, delay)` overload — no priority — so torches always run
/// at [`TickPriority::Normal`].
pub const TORCH_DELAY: u64 = 2;

/// How far back burnout detection looks, in game ticks.
///
/// `RedstoneTorchBlock.RECENT_TOGGLE_TIMER`, read as the literal `60L` in the
/// class's bytecode.
pub const TORCH_BURNOUT_WINDOW: u64 = 60;

/// Turn-offs within [`TORCH_BURNOUT_WINDOW`] before a torch burns out.
///
/// `RedstoneTorchBlock.MAX_RECENT_TOGGLES`. javac inlines it, so it was captured
/// rather than read — and the capture corrected a mistake. Driving a torch with a
/// 4-tick square wave produced:
///
/// ```text
/// turn-OFF at ticks  3, 11, 19, 27, 35, 43, 51, 59   (8)
/// turn-ON  at ticks  7, 15, 23, 31, 39, 47, 55       (7)
/// then nothing, while the driving repeater kept toggling to tick 157
/// ```
///
/// Fifteen state changes but **eight burnouts**: only the transitions to *unlit*
/// count. An implementation counting every toggle stalls a torch at eight state
/// changes instead of fifteen — very nearly half the real budget, which would make
/// any torch-driven clock diverge from the game well before it should.
pub const MAX_RECENT_TOGGLES: usize = 8;

/// Game ticks a comparator takes to act. Fixed, unlike a repeater's.
pub const COMPARATOR_DELAY: u64 = 2;

/// Game ticks per unit of a repeater's `delay` property.
///
/// The property counts *redstone* ticks (1–4); the scheduler counts game ticks.
/// Forgetting this factor is a silent doubling or halving of every repeater in a
/// build.
pub const REPEATER_TICKS_PER_DELAY: u64 = 2;

/// Which way a diode's signal flows.
///
/// Determined empirically from a captured trace rather than assumed: a repeater
/// with `facing=east` and a redstone block to its **east** scheduled a tick, while
/// `facing=west` with the same block did not. So the input side is the side the
/// block's `facing` names.
///
/// This is recorded here because it is easy to get backwards, and backwards means
/// every diode in a build reads the wrong neighbour.
pub const INPUT_IS_FACING_SIDE: bool = true;

/// A powered/unpowered pair of states for one block configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatePair {
    /// The state when unpowered.
    pub off: StateId,
    /// The state when powered.
    pub on: StateId,
}

impl StatePair {
    /// The state for a given powered flag.
    pub fn get(&self, powered: bool) -> StateId {
        if powered {
            self.on
        } else {
            self.off
        }
    }
}

/// Reads whether a position is emitting a redstone signal.
///
/// Supplied by the caller so this module stays independent of any particular
/// power model — the same reasoning that keeps [`crate::state::StateRegistry`] free
/// of Minecraft's block list.
pub trait PowerSource: Send + Sync {
    /// Whether `pos` currently emits a signal toward `toward`.
    ///
    /// `outs` carries the comparator block-entity outputs, because a
    /// comparator's emission is its *stored* strength, not its block state.
    fn is_powered(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        toward: Dir,
    ) -> bool;

    /// The analog (comparator-readable) signal of the block at `pos`, if it has
    /// one — a container's fullness, principally. `None` means the block has no
    /// analog output at all, which is different from an empty container's
    /// `Some(0)`.
    fn analog_signal(
        &self,
        _world: &World,
        _inventories: &crate::inventory::InventoryMap,
        _pos: Pos,
    ) -> Option<u8> {
        None
    }

    /// Whether the block at `pos` conducts strong power — also the block a
    /// comparator can read a container *through*.
    fn is_conductor(&self, _world: &World, _pos: Pos) -> bool {
        false
    }

    /// The slot count of the container block at `pos`, if it is one.
    fn container_slots_at(&self, _world: &World, _pos: Pos) -> Option<u32> {
        None
    }

    /// Whether the block at `pos` is a hopper — the destination-cooldown rule
    /// applies only to hoppers.
    fn hopper_at(&self, _world: &World, _pos: Pos) -> bool {
        false
    }

    /// Whether the block at `pos` is a full collision cube — what blocks a
    /// hopper's suction from above.
    fn is_solid_at(&self, _world: &World, _pos: Pos) -> bool {
        false
    }

    /// Whether the block at `pos` is a diode, for repeater-priority purposes.
    fn is_diode(&self, world: &World, pos: Pos) -> bool;

    /// Which way the diode at `pos` faces, if it is one.
    fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir>;

    /// The signal strength `pos` emits toward `toward`, 0-15.
    ///
    /// Defaults to the boolean answer widened to full strength, so a power model
    /// that only cares about on/off need not implement it. Comparators are the one
    /// component whose output genuinely depends on the *level*.
    fn signal_strength(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        toward: Dir,
    ) -> u8 {
        if self.is_powered(world, outs, pos, toward) {
            15
        } else {
            0
        }
    }
}

/// How a comparator combines its rear and side inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparatorMode {
    /// Output the rear signal, unless a side beats it, in which case nothing.
    Compare,
    /// Output the rear signal reduced by the strongest side.
    Subtract,
}

impl ComparatorMode {
    /// The output strength for a given rear and side signal.
    ///
    /// Every case below was captured from the real game rather than taken from
    /// documentation:
    ///
    /// ```text
    /// subtract  rear 15, side  0  -> 15   (dust beyond read 15 then 14)
    /// subtract  rear 15, side 14  ->  1
    /// compare   rear 15, side 14  -> 15   (side loses, passes through)
    /// compare   rear 13, side 14  ->  0   (side wins, comparator unpowered)
    /// ```
    pub fn output(self, rear: u8, side: u8) -> u8 {
        match self {
            ComparatorMode::Compare => {
                if side > rear {
                    0
                } else {
                    rear
                }
            }
            ComparatorMode::Subtract => rear.saturating_sub(side),
        }
    }
}

/// Shared scheduling logic for repeaters and comparators.
///
/// Mirrors `DiodeBlock.checkTickOnNeighbor`: decide whether the output should
/// change, and if so schedule at the priority the game would choose.
fn schedule_diode(
    ctx: &mut TickCtx<'_>,
    power: &dyn PowerSource,
    pos: Pos,
    facing: Dir,
    powered: bool,
    delay: u64,
) {
    let input_side = if INPUT_IS_FACING_SIDE {
        facing
    } else {
        facing.opposite()
    };
    let input = power.is_powered(ctx.world, ctx.comparator_out, pos.offset(input_side), input_side.opposite());

    if input == powered {
        return;
    }

    // The game refuses to double-schedule a position that already has a tick
    // pending. Skipping this check silently doubles every delay.
    if ctx.ticks.has_pending_at(pos, ctx.tick) {
        return;
    }

    let priority = if should_prioritize(ctx.world, power, pos, facing) {
        TickPriority::ExtremelyHigh
    } else if powered {
        // Turning off outranks turning on.
        TickPriority::VeryHigh
    } else {
        TickPriority::High
    };

    ctx.schedule(pos, delay, priority);
}

/// The two horizontal directions perpendicular to `facing`.
///
/// Locking is checked on these, and only these — a diode above or below cannot
/// lock a repeater.
fn perpendicular(facing: Dir) -> [Dir; 2] {
    match facing {
        Dir::North | Dir::South => [Dir::East, Dir::West],
        Dir::East | Dir::West => [Dir::North, Dir::South],
        // A vertical facing is not a valid diode orientation; return the horizontal
        // pair that cannot match rather than panicking mid-tick.
        Dir::Up | Dir::Down => [Dir::North, Dir::South],
    }
}

/// `DiodeBlock.shouldPrioritize`: is the block behind us a diode facing us?
fn should_prioritize(world: &World, power: &dyn PowerSource, pos: Pos, facing: Dir) -> bool {
    let behind = pos.offset(facing);
    if !power.is_diode(world, behind) {
        return false;
    }
    // A diode behind us only prioritises if it is aimed our way.
    power.diode_facing(world, behind) == Some(facing)
}

/// A redstone repeater.
///
/// One instance is registered per distinct block state, so it knows its own facing,
/// delay and powered flag without needing to parse a descriptor at tick time.
pub struct Repeater<P: PowerSource> {
    /// The `facing` property.
    pub facing: Dir,
    /// The `delay` property, 1–4 in redstone ticks.
    pub delay: u8,
    /// Whether this state is the powered one.
    pub powered: bool,
    /// The powered/unpowered states for this facing and delay.
    pub states: StatePair,
    /// This state with `locked` flipped, when it is interned.
    ///
    /// `RepeaterBlock.updateShape` recomputes `LOCKED` on every horizontal
    /// shape update perpendicular to `FACING`, so the property is *derived*,
    /// not authored — a community build saved mid-cycle carries whatever
    /// value it had, and vanilla corrects it the moment it is placed.
    pub locked_twin: Option<StateId>,
    /// Whether this state is the locked one.
    pub locked: bool,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> Repeater<P> {
    /// Whether the repeater at `pos` is locked by a diode powering it from the side.
    ///
    /// `DiodeBlock.isLocked`: a repeater fed from either side by a *powered* diode
    /// ignores its input entirely — `checkTickOnNeighbor` returns early, so a locked
    /// repeater schedules nothing and holds whatever output it had.
    pub fn locked_at(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
    ) -> bool {
        for side in perpendicular(self.facing) {
            let neighbour = pos.offset(side);
            if self.power.is_diode(world, neighbour)
                && self.power.is_powered(world, outs, neighbour, side.opposite())
            {
                return true;
            }
        }
        false
    }
    /// Delay in game ticks.
    pub fn delay_ticks(&self) -> u64 {
        u64::from(self.delay) * REPEATER_TICKS_PER_DELAY
    }
}

impl<P: PowerSource> BlockBehaviour for Repeater<P> {
    /// `DiodeBlock.onPlace` runs `updateNeighborsInFront`: a diode written
    /// into the world tells the block it points at, and that block's other
    /// neighbours, straight away. Under `knownShape` placement — where no
    /// update passes run — this is the only way a build's diodes reach each
    /// other at all.
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let target = pos.offset(self.facing.opposite());
        ctx.notify(target, self.facing);
        ctx.update_neighbors_except(target, self.facing);
    }

    /// `RepeaterBlock.updateShape`: a horizontal shape update perpendicular to
    /// `FACING` recomputes `LOCKED`.
    fn on_shape_update(&self, ctx: &mut TickCtx<'_>, pos: Pos, from: Dir) {
        if matches!(from, Dir::Up | Dir::Down)
            || from == self.facing
            || from == self.facing.opposite()
        {
            return;
        }
        let Some(twin) = self.locked_twin else { return };
        if self.locked_at(ctx.world, ctx.comparator_out, pos) != self.locked {
            // A shape update writes the state without notifying neighbours.
            ctx.set_quiet(pos, twin);
        }
    }

    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        // A locked repeater schedules nothing: the game returns early before
        // checkTickOnNeighbor ever considers the input.
        if self.locked_at(ctx.world, ctx.comparator_out, pos) {
            return;
        }
        let delay = self.delay_ticks();
        schedule_diode(ctx, &self.power, pos, self.facing, self.powered, delay);
    }

    /// `DiodeBlock.tick`, whose whole body sits under `if (!isLocked(...))`.
    ///
    /// A repeater that is locked when its tick lands does nothing *and does
    /// not reschedule* — it keeps its output until something unlocks it and
    /// notifies it afresh. That is why a build placed with a lit comparator
    /// beside a repeater holds its output for one more cycle: the tick fires
    /// into a locked repeater and is simply dropped.
    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.locked_at(ctx.world, ctx.comparator_out, pos) {
            return;
        }
        let input_side = if INPUT_IS_FACING_SIDE {
            self.facing
        } else {
            self.facing.opposite()
        };
        let should_turn_on =
            self.power
                .is_powered(ctx.world, ctx.comparator_out, pos.offset(input_side), input_side.opposite());
        if self.powered && !should_turn_on {
            ctx.set(pos, self.states.get(false));
        } else if !self.powered {
            // Turning on always happens; if the input has already gone away the
            // repeater books its own turn-off, which is what stretches a pulse
            // shorter than the delay out to the full delay.
            ctx.set(pos, self.states.get(true));
            if !should_turn_on {
                ctx.schedule(pos, self.delay_ticks(), crate::schedule::TickPriority::VeryHigh);
            }
        }
    }

    fn redstone_power(&self, _world: &World, _pos: Pos, dir: Dir) -> u8 {
        // A repeater emits only from its output side, at full strength.
        let output = if INPUT_IS_FACING_SIDE {
            self.facing.opposite()
        } else {
            self.facing
        };
        if self.powered && dir == output {
            15
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "repeater"
    }
}

/// A redstone torch.
///
/// Inverts the block it is attached to, after [`TORCH_DELAY`] game ticks, and always
/// at [`TickPriority::Normal`] — torches schedule without a priority argument, so
/// they run strictly after every diode in the same tick.
pub struct Torch<P: PowerSource> {
    /// The direction of the block this torch is attached to.
    pub attached: Dir,
    /// Whether this state is lit.
    pub lit: bool,
    /// Lit/unlit states.
    pub states: StatePair,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> BlockBehaviour for Torch<P> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let support = pos.offset(self.attached);
        let powered = self
            .power
            .is_powered(ctx.world, ctx.comparator_out, support, self.attached.opposite());
        // A torch is lit exactly when its support is *not* powered.
        if self.lit == powered && !ctx.ticks.has_pending_at(pos, ctx.tick) {
            ctx.schedule(pos, TORCH_DELAY, TickPriority::Normal);
        }
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let support = pos.offset(self.attached);
        let powered = self
            .power
            .is_powered(ctx.world, ctx.comparator_out, support, self.attached.opposite());
        if self.lit != powered {
            return;
        }
        // Burnout: a torch driven too hard stops responding. Without this a
        // torch-based clock would run forever in simulation and stall in the game,
        // which is the sort of divergence that invalidates a whole timing result.
        if ctx.recent_toggles(pos, TORCH_BURNOUT_WINDOW) >= MAX_RECENT_TOGGLES {
            return;
        }
        // Only turning *off* counts toward burnout — confirmed by capture. The
        // torch is lit exactly when its support is unpowered, so `powered` here
        // means it is about to go dark.
        if powered {
            ctx.record_toggle(pos);
        }
        ctx.set(pos, self.states.get(!powered));
    }

    fn redstone_power(&self, _world: &World, _pos: Pos, dir: Dir) -> u8 {
        // A lit torch powers every side except the one it is attached to.
        if self.lit && dir != self.attached {
            15
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "redstone_torch"
    }
}

/// A redstone comparator.
///
/// Shares `DiodeBlock`'s scheduling with a fixed [`COMPARATOR_DELAY`]. Comparison
/// and subtraction modes, container reading, and priming are **not** implemented —
/// see the module docs.
pub struct Comparator<P: PowerSource> {
    /// The `facing` property.
    pub facing: Dir,
    /// Whether this state is powered.
    pub powered: bool,
    /// Compare or subtract.
    pub mode: ComparatorMode,
    /// Powered/unpowered states.
    pub states: StatePair,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> Comparator<P> {
    /// The side this comparator reads its main input from.
    fn input_side(&self) -> Dir {
        if INPUT_IS_FACING_SIDE {
            self.facing
        } else {
            self.facing.opposite()
        }
    }

    /// The strength this comparator should be emitting.
    ///
    /// Rear input comes from [`Comparator::input_side`]; side inputs from the two
    /// horizontal directions perpendicular to it, matching the game's use of the
    /// same perpendicular pair that governs repeater locking.
    ///
    /// The container path is `ComparatorBlock.getInputSignal`, from bytecode: a
    /// block with an analog signal directly behind **overrides** the rear
    /// redstone reading; failing that, when the rear reading is below 15 and
    /// the block behind is a conductor, the comparator reads a container one
    /// block further back *through* it.
    pub fn output_strength(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        inventories: &crate::inventory::InventoryMap,
        pos: Pos,
    ) -> u8 {
        let back = self.input_side();
        let rear_pos = pos.offset(back);
        let mut rear = self.power.signal_strength(world, outs, rear_pos, back.opposite());
        if let Some(analog) = self.power.analog_signal(world, inventories, rear_pos) {
            rear = analog;
        } else if rear < 15 && self.power.is_conductor(world, rear_pos) {
            if let Some(analog) =
                self.power
                    .analog_signal(world, inventories, rear_pos.offset(back))
            {
                rear = analog;
            }
        }
        let side = perpendicular(back)
            .into_iter()
            .map(|dir| {
                self.power
                    .signal_strength(world, outs, pos.offset(dir), dir.opposite())
            })
            .max()
            .unwrap_or(0);
        self.mode.output(rear, side)
    }
}

impl<P: PowerSource> BlockBehaviour for Comparator<P> {
    /// `DiodeBlock.onPlace`: see [`Repeater::on_placed`].
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let target = pos.offset(self.input_side().opposite());
        ctx.notify(target, self.input_side());
        ctx.update_neighbors_except(target, self.input_side());
    }

    /// `ComparatorBlock.checkTickOnNeighbor` — and the source of comparator priming.
    ///
    /// A comparator differs from a repeater in two ways, both read from the class:
    ///
    /// 1. It schedules when its **output strength** changes, not only when its
    ///    powered flag would. The comparison is against a value held in a
    ///    `ComparatorBlockEntity`, because the block state cannot express "on at
    ///    strength 9". A comparator can therefore sit with a pending tick caused
    ///    purely by a strength change — *primed* — and resolve later alongside
    ///    components that never saw a change at all.
    /// 2. Its priority is `HIGH` when fed by a diode and `NORMAL` otherwise. It
    ///    never uses the `VERY_HIGH`/`EXTREMELY_HIGH` that a repeater reaches for,
    ///    so a primed comparator resolves *after* every repeater in the same tick.
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        if ctx.ticks.has_pending_at(pos, ctx.tick) {
            return;
        }
        let output = self.output_strength(ctx.world, ctx.comparator_out, ctx.inventories, pos);
        let stored = ctx.stored_comparator_output(pos);
        let should_be_on = output > 0;

        if output == stored && should_be_on == self.powered {
            return;
        }

        let priority = if should_prioritize(ctx.world, &self.power, pos, self.facing) {
            TickPriority::High
        } else {
            TickPriority::Normal
        };
        ctx.schedule(pos, COMPARATOR_DELAY, priority);
    }

    /// `ComparatorBlock.refreshOutputState`.
    ///
    /// The subtle part is what counts as "something happened": vanilla acts
    /// when the **stored strength** changes (or the mode is `compare`), not
    /// only when the `powered` flag flips — and it acts by writing the state
    /// silently (flag 2) and then calling `updateNeighborsInFront` itself.
    ///
    /// A comparator placed `powered=true` with a fresh block entity holding 0
    /// hits exactly that case: it recomputes to its real strength, the flag
    /// never moves, and only the explicit front update tells the dust. Without
    /// it a whole branch of the 4x4 vault door stayed dark, its pistons never
    /// fired, and the opposed pistons on the door's other side won a race they
    /// lose in the real game.
    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let output = self.output_strength(ctx.world, ctx.comparator_out, ctx.inventories, pos);
        let stored = ctx.stored_comparator_output(pos);
        ctx.store_comparator_output(pos, output);
        if stored == output && self.mode != ComparatorMode::Compare {
            return;
        }
        let should_be_on = output > 0;
        if should_be_on != self.powered {
            // setBlock flag 2: the shape pass still runs, neighbours do not.
            ctx.set_shape_only(pos, self.states.get(should_be_on));
        }
        // updateNeighborsInFront: the block this comparator outputs into, then
        // that block's neighbours except back toward us.
        let target = pos.offset(self.input_side().opposite());
        ctx.notify(target, self.input_side());
        ctx.update_neighbors_except(target, self.input_side());
    }

    fn redstone_power(&self, world: &World, pos: Pos, dir: Dir) -> u8 {
        let output = self.input_side().opposite();
        if dir == output {
            // No inventory view reaches this trait method, so a container-fed
            // comparator's *strength* is not visible here — only its powered
            // block state is. Nothing consumes analog strength through this
            // path yet (dust is not integrated); revisit when it is.
            self.output_strength(world, &Default::default(), &Default::default(), pos)
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "comparator"
    }
}

/// The cooldown a hopper takes after moving an item, in block-entity ticks.
///
/// `HopperBlockEntity.MOVE_ITEM_SPEED` — one transfer per 8 game ticks.
pub const HOPPER_COOLDOWN: i32 = 8;

/// A hopper's slot count.
pub const HOPPER_SLOTS: u8 = 5;

/// Ticks between a dispenser's trigger and its dispense.
///
/// `DispenserBlock.neighborChanged` schedules with delay 4.
pub const DISPENSER_DELAY: u64 = 4;

/// The assumed max stack size; see `crate::inventory`'s module docs.
const MERGE_LIMIT: u8 = 64;

/// A hopper.
///
/// Mechanics from `HopperBlockEntity` bytecode, pinned by capture:
///
/// - `pushItemsTick` runs every block-entity tick: decrement the cooldown,
///   stamp `tickedGameTime`, and when off cooldown try to move — **eject
///   first, then suck**; either success sets the cooldown to 8.
/// - `ejectItems` moves **one** item from the first occupied slot into the
///   container the hopper faces (first empty-or-mergeable slot, in slot order).
/// - `suckInItems` pulls one item from the first occupied slot of the container
///   above.
/// - Inserting into a **completely empty hopper** puts that hopper on cooldown
///   `8 - 1` when it has already ticked this game tick (it is earlier in the
///   block-entity order), else `8` — `tryMoveInItem`'s `tickedGameTime`
///   comparison, and the reason hopper order is observable.
/// - The `enabled` property gates everything; `HopperBlock` keeps it at
///   `!hasNeighborSignal` (no quasi-connectivity), written silently (flag 2).
pub struct Hopper<P: PowerSource> {
    /// The output direction — down, or one of the four horizontals.
    pub facing: Dir,
    /// Whether this state is the enabled (unpowered) one.
    pub enabled: bool,
    /// Disabled/enabled states (`off` = `enabled=false`).
    pub states: StatePair,
    /// How power and containers are read.
    pub power: P,
}

impl<P: PowerSource> Hopper<P> {
    fn is_empty(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool {
        ctx.inventories
            .get(&pos)
            .is_none_or(crate::inventory::Inventory::is_empty)
    }

    fn is_full(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool {
        inventory_is_full(ctx, pos, u32::from(HOPPER_SLOTS))
    }

    /// `ejectItems`: one item from our first occupied slot into the container
    /// we face.
    fn eject(&self, ctx: &mut TickCtx<'_>, pos: Pos) -> bool {
        let target = pos.offset(self.facing);
        let Some(target_slots) = self.power.container_slots_at(ctx.world, target) else {
            return false;
        };
        if inventory_is_full(ctx, target, target_slots) {
            return false;
        }
        for slot in 0..HOPPER_SLOTS {
            if let Some((id, count)) = ctx.inventory_slot(pos, slot) {
                if insert_one(ctx, &self.power, Some(pos), target, target_slots, &id) {
                    let remaining = count - 1;
                    ctx.set_inventory_slot(
                        pos,
                        slot,
                        (remaining > 0).then(|| (id, remaining)),
                    );
                    return true;
                }
            }
        }
        false
    }

    /// `suckInItems`: one item from the first occupied slot of the container
    /// above us — or, with no container there and no full block capping us,
    /// vacuum an item entity out of the suck column (the full block above from
    /// y+11/16 to y+2).
    fn suck(&self, ctx: &mut TickCtx<'_>, pos: Pos) -> bool {
        let source = pos.offset(Dir::Up);
        let Some(source_slots) = self.power.container_slots_at(ctx.world, source) else {
            if self.power.is_solid_at(ctx.world, source) {
                return false; // a full block above blocks suction
            }
            return self.suck_entities(ctx, pos);
        };
        for slot in 0..source_slots.min(255) as u8 {
            if let Some((id, count)) = ctx.inventory_slot(source, slot) {
                if insert_one(ctx, &self.power, None, pos, u32::from(HOPPER_SLOTS), &id) {
                    let remaining = count - 1;
                    ctx.set_inventory_slot(
                        source,
                        slot,
                        (remaining > 0).then(|| (id, remaining)),
                    );
                    return true;
                }
            }
        }
        false
    }

    /// `getItemsAtAndAbove` + `addItem(container, itemEntity)`: absorb the
    /// first intersecting item entity, whole stack when it fits. Vanilla only
    /// reports success — and takes the cooldown — when the entity was fully
    /// consumed; a partial absorb modifies both sides and returns false.
    fn suck_entities(&self, ctx: &mut TickCtx<'_>, pos: Pos) -> bool {
        let suck_min = [f64::from(pos.x), f64::from(pos.y) + 0.6875, f64::from(pos.z)];
        let suck_max = [f64::from(pos.x) + 1.0, f64::from(pos.y) + 2.0, f64::from(pos.z) + 1.0];
        for index in 0..ctx.item_entities.items.len() {
            let (item_id, count, intersects) = {
                let entity = &ctx.item_entities.items[index];
                if entity.removed {
                    continue;
                }
                let (emin, emax) = crate::entity::item_aabb(entity.pos);
                let intersects = emin[0] < suck_max[0]
                    && emax[0] > suck_min[0]
                    && emin[1] < suck_max[1]
                    && emax[1] > suck_min[1]
                    && emin[2] < suck_max[2]
                    && emax[2] > suck_min[2];
                (entity.item.0.clone(), entity.item.1, intersects)
            };
            if !intersects {
                continue;
            }
            let mut absorbed = 0u8;
            while absorbed < count
                && insert_one(ctx, &self.power, None, pos, u32::from(HOPPER_SLOTS), &item_id)
            {
                absorbed += 1;
            }
            if absorbed == count {
                ctx.item_entities.items[index].removed = true;
                return true;
            } else if absorbed > 0 {
                ctx.item_entities.items[index].item.1 = count - absorbed;
                return false;
            }
        }
        false
    }
}

impl<P: PowerSource> BlockBehaviour for Hopper<P> {
    /// `HopperBlock.checkPoweredState`: enabled tracks `!hasNeighborSignal`,
    /// written silently.
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let powered = crate::pos::ALL_DIRS.iter().any(|dir| {
            self.power
                .is_powered(ctx.world, ctx.comparator_out, pos.offset(*dir), dir.opposite())
        });
        let enabled = !powered;
        if enabled != self.enabled {
            ctx.set_quiet(pos, self.states.get(enabled));
        }
    }

    /// `HopperBlockEntity.pushItemsTick`.
    fn on_block_entity_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        {
            let state = ctx.hopper_state.entry(pos).or_default();
            state.cooldown -= 1;
            state.ticked_at = ctx.tick as i64;
            if state.cooldown > 0 {
                return;
            }
            state.cooldown = 0;
        }
        if !self.enabled {
            return;
        }
        let mut moved = false;
        if !self.is_empty(ctx, pos) {
            moved = self.eject(ctx, pos);
        }
        if !self.is_full(ctx, pos) {
            moved |= self.suck(ctx, pos);
        }
        if moved {
            ctx.hopper_state.entry(pos).or_default().cooldown = HOPPER_COOLDOWN;
        }
    }

    fn ticks_as_block_entity(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "hopper"
    }
}

/// Whether every slot of the container at `pos` holds a full stack.
fn inventory_is_full(ctx: &TickCtx<'_>, pos: Pos, slots: u32) -> bool {
    let Some(inventory) = ctx.inventories.get(&pos) else {
        return slots == 0;
    };
    let occupied = inventory
        .stacks
        .iter()
        .filter(|stack| stack.count >= MERGE_LIMIT)
        .count() as u32;
    occupied >= slots
}

/// `HopperBlockEntity.addItem`/`tryMoveInItem`: place one `id` item into the
/// first slot of `target` that is empty or mergeable, in slot order.
///
/// `source_hopper` is the ticking hopper doing the insert, if any — the
/// destination-cooldown rule needs its `tickedGameTime`.
fn insert_one<P: PowerSource>(
    ctx: &mut TickCtx<'_>,
    power: &P,
    source_hopper: Option<Pos>,
    target: Pos,
    target_slots: u32,
    id: &str,
) -> bool {
    let target_was_empty = ctx
        .inventories
        .get(&target)
        .is_none_or(crate::inventory::Inventory::is_empty);
    for slot in 0..target_slots.min(255) as u8 {
        match ctx.inventory_slot(target, slot) {
            None => {
                ctx.set_inventory_slot(target, slot, Some((id.to_string(), 1)));
                // A previously-empty destination hopper is put on cooldown by
                // the *inserter*: 7 when it already ticked this game tick,
                // 8 otherwise.
                if target_was_empty && power.hopper_at(ctx.world, target) {
                    let source_ticked = source_hopper
                        .and_then(|p| ctx.hopper_state.get(&p))
                        .map(|s| s.ticked_at);
                    let already_ticked = match source_ticked {
                        Some(source) => {
                            ctx.hopper_state.entry(target).or_default().ticked_at >= source
                        }
                        None => false,
                    };
                    ctx.hopper_state.entry(target).or_default().cooldown =
                        HOPPER_COOLDOWN - i32::from(already_ticked);
                }
                return true;
            }
            Some((existing, count)) if existing == id && count < MERGE_LIMIT => {
                ctx.set_inventory_slot(target, slot, Some((existing, count + 1)));
                return true;
            }
            Some(_) => continue,
        }
    }
    false
}

/// A dropper or dispenser.
///
/// Trigger mechanics from `DispenserBlock.neighborChanged`: powered is
/// `hasNeighborSignal(pos) || hasNeighborSignal(pos.above())` — full
/// quasi-connectivity, no direction skips — and a rising edge schedules a
/// 4-tick delay while flipping `triggered` silently (flag 2).
///
/// Dispensing: a dropper facing a container moves one item into it
/// (`HopperBlockEntity.addItem` semantics). With no container in front — and
/// always, for a dispenser — the item leaves the world as an item entity,
/// which is Milestone B; the engine decrements the slot and the departure is
/// exactly what container NBT shows.
///
/// # Known simplification
///
/// Vanilla picks a **random occupied slot** (`getRandomSlot`); the engine
/// deterministically takes the first occupied slot. Identical whenever at most
/// one slot is occupied, which conformance structures keep to.
pub struct Dropper<P: PowerSource> {
    /// The output direction.
    pub facing: Dir,
    /// Whether this state is the triggered one.
    pub triggered: bool,
    /// Untriggered/triggered states.
    pub states: StatePair,
    /// True for a dispenser (never inserts into containers).
    pub dispenser: bool,
    /// How power and containers are read.
    pub power: P,
}

impl<P: PowerSource> Dropper<P> {
    fn has_signal(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool {
        crate::pos::ALL_DIRS.iter().any(|dir| {
            self.power
                .is_powered(ctx.world, ctx.comparator_out, pos.offset(*dir), dir.opposite())
        })
    }
}

impl<P: PowerSource> BlockBehaviour for Dropper<P> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let powered = self.has_signal(ctx, pos) || self.has_signal(ctx, pos.offset(Dir::Up));
        if powered && !self.triggered {
            ctx.schedule(pos, DISPENSER_DELAY, TickPriority::Normal);
            ctx.set_quiet(pos, self.states.get(true));
        } else if !powered && self.triggered {
            ctx.set_quiet(pos, self.states.get(false));
        }
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let slots = 9u8;
        let Some((slot, id, count)) = (0..slots)
            .find_map(|s| ctx.inventory_slot(pos, s).map(|(id, c)| (s, id, c)))
        else {
            return; // empty: vanilla just clicks
        };
        if !self.dispenser {
            let front = pos.offset(self.facing);
            if let Some(front_slots) = self.power.container_slots_at(ctx.world, front) {
                if insert_one(ctx, &self.power, None, front, front_slots, &id) {
                    let remaining = count - 1;
                    ctx.set_inventory_slot(pos, slot, (remaining > 0).then(|| (id, remaining)));
                }
                // Insert refused (target full): the item stays put.
                return;
            }
        }
        // No container in front: the item is ejected into the world as an item
        // entity. Spawn position and mean velocity are
        // `DefaultDispenseItemBehavior.spawnItem` with the jitter removed:
        // 0.7 blocks out of the face (0.125 down for vertical facings,
        // 0.15625 for horizontal), speed the mean of `0.2 + 0.1 * U(0,1)`,
        // and a constant upward 0.2 regardless of facing. Vanilla adds
        // `triangle(_, 0.103)` noise to every component; the engine is
        // deterministic and conformance for ejected trajectories uses
        // tolerance rather than exactness.
        let (dx, dy, dz) = self.facing.delta();
        let vertical = dy != 0;
        let x = f64::from(pos.x) + 0.5 + 0.7 * f64::from(dx);
        let y = f64::from(pos.y) + 0.5 + 0.7 * f64::from(dy)
            - if vertical { 0.125 } else { 0.15625 };
        let z = f64::from(pos.z) + 0.5 + 0.7 * f64::from(dz);
        let speed = 0.25;
        ctx.item_entities.spawn(
            (id.clone(), 1),
            [x, y, z],
            [f64::from(dx) * speed, 0.2, f64::from(dz) * speed],
            10,
        );
        let remaining = count - 1;
        ctx.set_inventory_slot(pos, slot, (remaining > 0).then(|| (id, remaining)));
    }

    fn name(&self) -> &'static str {
        if self.dispenser {
            "dispenser"
        } else {
            "dropper"
        }
    }
}

/// A button.
///
/// `ButtonBlock.press`: power on (loudly), schedule `ticksToStayPressed` —
/// 20 game ticks for stone, 30 for wood (`BlockSetType`) — and unpower when
/// the tick fires. Powers everything weakly and its attached block strongly.
pub struct Button<P: PowerSource> {
    /// Whether this state is pressed.
    pub powered: bool,
    /// Unpressed/pressed states.
    pub states: StatePair,
    /// 20 for stone, 30 for wood.
    pub duration: u64,
    /// How power is read (unused today; kept for parity with siblings).
    pub power: P,
}

impl<P: PowerSource> BlockBehaviour for Button<P> {
    fn on_used(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.powered {
            return; // pressing a pressed button does nothing
        }
        ctx.set(pos, self.states.get(true));
        ctx.schedule(pos, self.duration, TickPriority::Normal);
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.powered {
            ctx.set(pos, self.states.get(false));
        }
    }

    fn name(&self) -> &'static str {
        "button"
    }
}

/// Ticks a redstone lamp waits before going dark; turning on is immediate.
pub const LAMP_OFF_DELAY: u64 = 4;

/// A redstone lamp.
pub struct Lamp<P: PowerSource> {
    /// Whether this state is lit.
    pub lit: bool,
    /// Unlit/lit states.
    pub states: StatePair,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> Lamp<P> {
    fn has_signal(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool {
        crate::pos::ALL_DIRS.iter().any(|dir| {
            self.power
                .is_powered(ctx.world, ctx.comparator_out, pos.offset(*dir), dir.opposite())
        })
    }
}

impl<P: PowerSource> BlockBehaviour for Lamp<P> {
    /// `RedstoneLampBlock.neighborChanged`: light immediately, dim after a
    /// 4-tick recheck.
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let signal = self.has_signal(ctx, pos);
        if signal && !self.lit {
            ctx.set_quiet(pos, self.states.get(true));
        } else if !signal && self.lit {
            ctx.schedule(pos, LAMP_OFF_DELAY, TickPriority::Normal);
        }
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.lit && !self.has_signal(ctx, pos) {
            ctx.set_quiet(pos, self.states.get(false));
        }
    }

    fn name(&self) -> &'static str {
        "redstone_lamp"
    }
}

/// Ticks between a pressure plate's presence rechecks.
pub const PLATE_RECHECK: u64 = 20;

/// A pressure plate. Wooden plates sense every entity — items included, which
/// is what makes an item-on-plate capture deterministic; stone plates sense
/// only living entities, so items never trigger them here.
pub struct PressurePlate<P: PowerSource> {
    /// Whether this state is pressed.
    pub powered: bool,
    /// Unpressed/pressed states.
    pub states: StatePair,
    /// Whether items press it (wooden yes, stone no).
    pub senses_items: bool,
    /// How power is read (parity).
    pub power: P,
}

impl<P: PowerSource> PressurePlate<P> {
    fn pressed_by_item(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool {
        if !self.senses_items {
            return false;
        }
        // BasePressurePlateBlock.TOUCH_AABB: the plate cell inset by a pixel.
        let min = [f64::from(pos.x) + 0.0625, f64::from(pos.y), f64::from(pos.z) + 0.0625];
        let max = [
            f64::from(pos.x) + 0.9375,
            f64::from(pos.y) + 0.25,
            f64::from(pos.z) + 0.9375,
        ];
        ctx.item_entities.items.iter().any(|item| {
            if item.removed {
                return false;
            }
            let (emin, emax) = crate::entity::item_aabb(item.pos);
            emin[0] < max[0]
                && emax[0] > min[0]
                && emin[1] < max[1]
                && emax[1] > min[1]
                && emin[2] < max[2]
                && emax[2] > min[2]
        })
    }
}

impl<P: PowerSource> BlockBehaviour for PressurePlate<P> {
    /// `entityInside`: press and start the recheck cadence.
    fn on_entity_inside(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if !self.powered && self.pressed_by_item(ctx, pos) {
            ctx.set(pos, self.states.get(true));
            ctx.schedule(pos, PLATE_RECHECK, TickPriority::Normal);
        }
    }

    /// `tick`: still pressed → check again in 20; empty → release.
    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if !self.powered {
            return;
        }
        if self.pressed_by_item(ctx, pos) {
            ctx.schedule(pos, PLATE_RECHECK, TickPriority::Normal);
        } else {
            ctx.set(pos, self.states.get(false));
        }
    }

    fn name(&self) -> &'static str {
        "pressure_plate"
    }
}

/// How many pitches a note block cycles through before wrapping.
///
/// `NoteBlock.NOTE` is `IntegerProperty.create("note", 0, 24)`; `cycle` wraps 24
/// back to 0.
pub const NOTE_VALUES: u8 = 25;

/// A note block.
///
/// Everything here is read from `NoteBlock`'s bytecode and confirmed by capture
/// (`note_powered.json`, `note_click.json`):
///
/// - `neighborChanged` compares `hasNeighborSignal` with the `powered` property
///   and updates it **synchronously** — no scheduled tick, so the flip lands on
///   the same tick as the change that caused it.
/// - The note *plays* via a block event (`level.blockEvent(pos, this, 0, 0)`),
///   queued only on the rising edge, and only if the instrument can sound —
///   for the ordinary instruments that means **air above**.
/// - A right-click (`useWithoutItem`) cycles the `note` property and then plays.
///   The pitch change is what an adjacent observer sees, which is how a note
///   block acts as the trigger of a manual contraption.
pub struct NoteBlock<P: PowerSource> {
    /// Whether this state is the powered one.
    pub powered: bool,
    /// Unpowered/powered states at this pitch.
    pub states: StatePair,
    /// The state a click turns this one into: same powered flag, next pitch.
    pub cycled: StateId,
    /// This state's `instrument`.
    pub instrument: &'static str,
    /// The same state under each instrument this engine knows, for the
    /// shape-update recomputation.
    pub instrument_states: Vec<(&'static str, StateId)>,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> NoteBlock<P> {
    /// Vanilla's `Level.hasNeighborSignal`: any of the six neighbours powering us.
    fn has_neighbor_signal(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
    ) -> bool {
        crate::pos::ALL_DIRS
            .iter()
            .any(|dir| self.power.is_powered(world, outs, pos.offset(*dir), dir.opposite()))
    }

    /// Queue the "play a note" block event, if the instrument can sound.
    ///
    /// `playNote` refuses when a block sits on top (for instruments that do not
    /// work above a note block, which is all the ordinary ones). The event has no
    /// observable effect on the world — it is sound — but it is queued for
    /// structural fidelity, and [`NoteBlock::on_block_event`] consumes it.
    fn play(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if ctx.get(pos.offset(Dir::Up)) == StateId::AIR {
            ctx.queue_event(pos, 0, 0);
        }
    }
}

impl<P: PowerSource> BlockBehaviour for NoteBlock<P> {
    /// `NoteBlock.updateShape`: a **vertical** shape update recomputes the
    /// instrument, which comes from the block above when that instrument
    /// works above a note block, and otherwise from the block below. Like a
    /// repeater's `locked`, the property is derived, so a community build
    /// carries whatever it was saved with until placement corrects it.
    fn on_shape_update(&self, ctx: &mut TickCtx<'_>, pos: Pos, from: Dir) {
        if !matches!(from, Dir::Up | Dir::Down) {
            return;
        }
        let below = ctx
            .states
            .descriptor(ctx.world.get(pos.offset(Dir::Down)))
            .unwrap_or("minecraft:air");
        let wanted = crate::vanilla::instrument_below(below);
        if wanted == self.instrument {
            return;
        }
        if let Some((_, state)) = self
            .instrument_states
            .iter()
            .find(|(name, _)| *name == wanted)
        {
            ctx.set_quiet(pos, *state);
        }
    }

    /// `NoteBlock.neighborChanged`: follow the neighbour signal synchronously,
    /// playing on the rising edge only.
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let signal = self.has_neighbor_signal(ctx.world, ctx.comparator_out, pos);
        if signal == self.powered {
            return;
        }
        if signal {
            self.play(ctx, pos);
        }
        ctx.set(pos, self.states.get(signal));
    }

    /// `NoteBlock.useWithoutItem`: cycle the pitch, then play.
    fn on_used(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        ctx.set(pos, self.cycled);
        self.play(ctx, pos);
    }

    /// `NoteBlock.triggerEvent`: the note sounds; nothing in the world changes.
    fn on_block_event(&self, _ctx: &mut TickCtx<'_>, _pos: Pos, _id: u8, _param: u8) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "note_block"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos::Bounds;
    use crate::schedule::{EventQueue, TickQueue};
    use crate::state::StateRegistry;

    /// A power model where designated states emit in all directions.
    #[derive(Clone)]
    struct Sources {
        powered: Vec<StateId>,
        diodes: Vec<(StateId, Dir)>,
    }

    impl PowerSource for Sources {
        fn is_powered(
            &self,
            world: &World,
            _outs: &crate::behaviour::ComparatorOutputs,
            pos: Pos,
            _toward: Dir,
        ) -> bool {
            self.powered.contains(&world.get(pos))
        }
        fn is_diode(&self, world: &World, pos: Pos) -> bool {
            let state = world.get(pos);
            self.diodes.iter().any(|(s, _)| *s == state)
        }
        fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir> {
            let state = world.get(pos);
            self.diodes
                .iter()
                .find(|(s, _)| *s == state)
                .map(|(_, d)| *d)
        }
    }

    fn ctx_parts() -> (World, TickQueue, EventQueue, StateRegistry) {
        (
            World::new(Bounds::new(Pos::new(-4, 0, -4), Pos::new(8, 4, 4))),
            TickQueue::new(),
            EventQueue::new(),
            StateRegistry::new(),
        )
    }

    #[test]
    fn repeater_delay_is_the_property_times_two() {
        // Verified from RepeaterBlock.getDelay: the property is in redstone ticks,
        // the scheduler in game ticks.
        for (property, expected) in [(1u8, 2u64), (2, 4), (3, 6), (4, 8)] {
            let repeater = Repeater {
                facing: Dir::East,
                delay: property,
                powered: false,
                states: StatePair { off: StateId(1), on: StateId(2) },
                locked: false,
                locked_twin: None,
                power: Sources { powered: vec![], diodes: vec![] },
            };
            assert_eq!(repeater.delay_ticks(), expected, "delay={property}");
        }
    }

    #[test]
    fn a_repeater_schedules_at_high_when_turning_on() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        world.set(Pos::new(1, 1, 0), source);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            locked: false,
            locked_twin: None,
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        repeater.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(2);
        assert_eq!(due.len(), 1, "must schedule");
        assert_eq!(due[0].priority, TickPriority::High, "turning on is HIGH");
        assert_eq!(due[0].target, 2, "delay 1 == 2 game ticks");
    }

    #[test]
    fn a_powered_repeater_turning_off_schedules_at_very_high() {
        // Verified ordering: turning off outranks turning on.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: true,
            states: StatePair { off: StateId(1), on: StateId(2) },
            locked: false,
            locked_twin: None,
            power: Sources { powered: vec![], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        repeater.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(2);
        assert_eq!(due[0].priority, TickPriority::VeryHigh);
    }

    #[test]
    fn a_repeater_fed_by_another_diode_jumps_the_queue() {
        // DiodeBlock.shouldPrioritize -> EXTREMELY_HIGH. This is what makes repeater
        // chains resolve in a stable order.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let upstream = StateId(5);
        world.set(Pos::new(1, 1, 0), upstream);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            locked: false,
            locked_twin: None,
            power: Sources {
                powered: vec![source, upstream],
                // The upstream diode faces east: the same way we look at it.
                diodes: vec![(upstream, Dir::East)],
            },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        repeater.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(2);
        assert_eq!(due[0].priority, TickPriority::ExtremelyHigh);
    }

    #[test]
    fn a_diode_never_double_schedules() {
        // The game checks for a pending tick first. Without it every delay doubles.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        world.set(Pos::new(1, 1, 0), source);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 2,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            locked: false,
            locked_twin: None,
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let pos = Pos::new(0, 1, 0);
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);

        assert_eq!(ticks.len(), 1, "three notifications, one scheduled tick");
    }

    #[test]
    fn a_torch_schedules_at_normal_so_it_runs_after_every_diode() {
        // RedstoneTorchBlock uses the scheduleTick overload without a priority, so
        // torches sit at NORMAL while diodes sit at HIGH or above. That ordering
        // decides which component observes which.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let support = Pos::new(0, 0, 0);
        world.set(support, source);

        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        torch.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::Down);

        let due = ticks.drain_due(TORCH_DELAY);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].priority, TickPriority::Normal);
        assert_eq!(due[0].target, TORCH_DELAY);
    }

    #[test]
    fn a_torch_inverts_its_support() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let lit = StateId(2);
        let unlit = StateId(1);
        let torch_pos = Pos::new(0, 1, 0);
        world.set(Pos::new(0, 0, 0), source);
        world.set(torch_pos, lit);

        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: unlit, on: lit },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        torch.on_scheduled_tick(&mut ctx, torch_pos);

        assert_eq!(world.get(torch_pos), unlit, "powered support unlights the torch");
    }

    #[test]
    fn a_lit_torch_powers_every_side_but_its_support() {
        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![], diodes: vec![] },
        };
        let world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(1, 1, 1)));
        assert_eq!(torch.redstone_power(&world, Pos::new(0, 1, 0), Dir::Up), 15);
        assert_eq!(torch.redstone_power(&world, Pos::new(0, 1, 0), Dir::North), 15);
        assert_eq!(
            torch.redstone_power(&world, Pos::new(0, 1, 0), Dir::Down),
            0,
            "never back into its own support"
        );
    }

    #[test]
    fn comparator_delay_is_fixed_at_two() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        world.set(Pos::new(1, 1, 0), source);

        let comparator = Comparator {
            facing: Dir::East,
            powered: false,
            mode: ComparatorMode::Subtract,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        comparator.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(COMPARATOR_DELAY);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].target, COMPARATOR_DELAY, "always 2, unlike a repeater");
    }

    #[test]
    fn a_repeater_locked_from_the_side_schedules_nothing() {
        // DiodeBlock.isLocked returns early, before the input is even considered.
        // A locked repeater that still scheduled would flicker its output whenever
        // its input moved, which is precisely what locking exists to prevent.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let side_diode = StateId(5);
        let pos = Pos::new(0, 1, 0);

        world.set(pos.offset(Dir::East), source); // live input
        world.set(pos.offset(Dir::North), side_diode); // locking diode on the side

        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            locked: false,
            locked_twin: None,
            power: Sources {
                powered: vec![source, side_diode],
                diodes: vec![(side_diode, Dir::South)],
            },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);

        assert!(ticks.is_empty(), "a locked repeater must schedule nothing");
    }

    #[test]
    fn only_a_powered_side_diode_locks() {
        // An unpowered diode beside a repeater does not lock it.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let side_diode = StateId(5);
        let pos = Pos::new(0, 1, 0);
        world.set(pos.offset(Dir::East), source);
        world.set(pos.offset(Dir::North), side_diode);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            locked: false,
            locked_twin: None,
            power: Sources {
                powered: vec![source], // side diode present but NOT powered
                diodes: vec![(side_diode, Dir::South)],
            },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);

        assert_eq!(ticks.len(), 1, "an unpowered side diode must not lock");
    }

    #[test]
    fn a_diode_above_or_below_cannot_lock() {
        // Locking is checked only on the two horizontal perpendicular sides.
        assert_eq!(perpendicular(Dir::East), [Dir::North, Dir::South]);
        assert_eq!(perpendicular(Dir::North), [Dir::East, Dir::West]);
    }

    #[test]
    fn subtract_mode_passes_the_rear_signal_when_no_side_input() {
        // Trace-confirmed: a subtract comparator fed 15 from behind with nothing at
        // its sides output 15, lighting the dust beyond it at 15 then 14.
        assert_eq!(ComparatorMode::Subtract.output(15, 0), 15);
        assert_eq!(ComparatorMode::Subtract.output(9, 0), 9);
    }

    #[test]
    fn subtract_mode_reduces_by_the_side_signal() {
        // Captured: rear 15 against a side dust at 14 lit the output dust at 1.
        assert_eq!(ComparatorMode::Subtract.output(15, 14), 1);
        assert_eq!(ComparatorMode::Subtract.output(15, 4), 11);
        assert_eq!(ComparatorMode::Subtract.output(3, 9), 0, "never below zero");
    }

    #[test]
    fn compare_mode_is_all_or_nothing() {
        // Both captured: rear 15 / side 14 passed 15 through, while rear 13 against
        // a side of 14 left the comparator unpowered and its output dust at 0.
        assert_eq!(ComparatorMode::Compare.output(15, 14), 15, "side loses, pass through");
        assert_eq!(ComparatorMode::Compare.output(13, 14), 0, "side wins, output nothing");
        assert_eq!(ComparatorMode::Compare.output(15, 15), 15, "a tie still passes");
    }

    #[test]
    fn a_comparator_emits_only_from_its_output_side() {
        let (mut world, _t, _e, _s) = ctx_parts();
        let source = StateId(9);
        let pos = Pos::new(0, 1, 0);
        world.set(pos.offset(Dir::East), source);

        let comparator = Comparator {
            facing: Dir::East,
            powered: true,
            mode: ComparatorMode::Subtract,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        // Input arrives on the facing side, so output leaves the opposite side.
        assert_eq!(comparator.redstone_power(&world, pos, Dir::West), 15);
        assert_eq!(comparator.redstone_power(&world, pos, Dir::East), 0);
        assert_eq!(comparator.redstone_power(&world, pos, Dir::North), 0);
    }

    #[test]
    fn a_torch_burns_out_when_driven_too_hard() {
        // Without this a torch clock runs forever in simulation and stalls in the
        // game — a divergence that invalidates any timing result built on it.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let lit = StateId(2);
        let unlit = StateId(1);
        let pos = Pos::new(0, 1, 0);
        let support = Pos::new(0, 0, 0);
        world.set(support, source);
        world.set(pos, lit);

        let mut toggles = Vec::new();
        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: unlit, on: lit },
            power: Sources { powered: vec![source], diodes: vec![] },
        };

        // Pre-load the history with the maximum allowed toggles inside the window.
        for t in 0..MAX_RECENT_TOGGLES {
            toggles.push((pos, t as u64));
        }

        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 10,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut toggles,
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        torch.on_scheduled_tick(&mut ctx, pos);

        assert_eq!(world.get(pos), lit, "burnt out: the torch must not toggle");
    }

    #[test]
    fn a_torch_toggles_normally_below_the_burnout_threshold() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let lit = StateId(2);
        let unlit = StateId(1);
        let pos = Pos::new(0, 1, 0);
        world.set(Pos::new(0, 0, 0), source);
        world.set(pos, lit);

        let mut toggles: Vec<(Pos, u64)> =
            (0..MAX_RECENT_TOGGLES - 1).map(|t| (pos, t as u64)).collect();
        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: unlit, on: lit },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 10,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut toggles,
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        torch.on_scheduled_tick(&mut ctx, pos);

        assert_eq!(world.get(pos), unlit, "one below the limit still toggles");
    }

    #[test]
    fn toggles_outside_the_window_do_not_count_toward_burnout() {
        // RECENT_TOGGLE_TIMER is 60 ticks, read as the literal 60L in the class.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let lit = StateId(2);
        let unlit = StateId(1);
        let pos = Pos::new(0, 1, 0);
        world.set(Pos::new(0, 0, 0), source);
        world.set(pos, lit);

        // Plenty of toggles, but all long expired.
        let mut toggles: Vec<(Pos, u64)> =
            (0..MAX_RECENT_TOGGLES * 2).map(|t| (pos, t as u64)).collect();
        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: unlit, on: lit },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 500,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut toggles,
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        torch.on_scheduled_tick(&mut ctx, pos);

        assert_eq!(world.get(pos), unlit, "expired toggles must not burn it out");
    }

    /// A power model with per-position strengths, needed to exercise priming.
    #[derive(Clone)]
    struct Levels {
        levels: Vec<(Pos, u8)>,
        diodes: Vec<(StateId, Dir)>,
    }

    impl PowerSource for Levels {
        fn is_powered(
            &self,
            world: &World,
            outs: &crate::behaviour::ComparatorOutputs,
            pos: Pos,
            toward: Dir,
        ) -> bool {
            self.signal_strength(world, outs, pos, toward) > 0
        }
        fn is_diode(&self, world: &World, pos: Pos) -> bool {
            let s = world.get(pos);
            self.diodes.iter().any(|(d, _)| *d == s)
        }
        fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir> {
            let s = world.get(pos);
            self.diodes.iter().find(|(d, _)| *d == s).map(|(_, f)| *f)
        }
        fn signal_strength(
            &self,
            _world: &World,
            _outs: &crate::behaviour::ComparatorOutputs,
            pos: Pos,
            _toward: Dir,
        ) -> u8 {
            self.levels
                .iter()
                .find(|(p, _)| *p == pos)
                .map(|(_, l)| *l)
                .unwrap_or(0)
        }
    }

    fn primed_comparator(levels: Levels, powered: bool) -> Comparator<Levels> {
        Comparator {
            facing: Dir::East,
            powered,
            mode: ComparatorMode::Subtract,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: levels,
        }
    }

    #[test]
    fn a_comparator_schedules_on_a_strength_change_alone() {
        // Priming. The comparator stays on either way — only the *strength* moves,
        // from a stored 15 to a computed 9. A repeater in the same situation would
        // do nothing, because its powered flag is unchanged.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let pos = Pos::new(0, 1, 0);
        let comparator = primed_comparator(
            Levels { levels: vec![(pos.offset(Dir::East), 9)], diodes: vec![] },
            true,
        );

        let mut stored = std::collections::HashMap::new();
        stored.insert(pos, 15u8);

        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut stored,
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        comparator.on_neighbor_changed(&mut ctx, pos, Dir::East);

        assert_eq!(ticks.len(), 1, "a strength-only change must still schedule");
    }

    #[test]
    fn a_comparator_whose_output_is_unchanged_schedules_nothing() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let pos = Pos::new(0, 1, 0);
        let comparator = primed_comparator(
            Levels { levels: vec![(pos.offset(Dir::East), 15)], diodes: vec![] },
            true,
        );
        let mut stored = std::collections::HashMap::new();
        stored.insert(pos, 15u8);

        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut stored,
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        comparator.on_neighbor_changed(&mut ctx, pos, Dir::East);

        assert!(ticks.is_empty(), "nothing changed, nothing scheduled");
    }

    #[test]
    fn a_primed_comparator_resolves_after_every_repeater() {
        // A comparator only ever schedules at HIGH or NORMAL, never the VERY_HIGH or
        // EXTREMELY_HIGH a repeater reaches for. So when both fire in one tick the
        // repeater always goes first — which is what makes priming observable.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let pos = Pos::new(0, 1, 0);
        let comparator = primed_comparator(
            Levels { levels: vec![(pos.offset(Dir::East), 9)], diodes: vec![] },
            false,
        );
        let mut stored = std::collections::HashMap::new();

        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut stored,
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        comparator.on_neighbor_changed(&mut ctx, pos, Dir::East);

        let due = ticks.drain_due(COMPARATOR_DELAY);
        assert_eq!(due[0].priority, TickPriority::Normal, "not fed by a diode");
        assert!(
            TickPriority::VeryHigh < TickPriority::Normal,
            "a repeater turning off outranks it"
        );
    }

    #[test]
    fn a_comparator_fed_by_a_diode_schedules_at_high() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let pos = Pos::new(0, 1, 0);
        let upstream = StateId(5);
        world.set(pos.offset(Dir::East), upstream);

        let comparator = primed_comparator(
            Levels {
                levels: vec![(pos.offset(Dir::East), 9)],
                diodes: vec![(upstream, Dir::East)],
            },
            false,
        );
        let mut stored = std::collections::HashMap::new();

        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut stored,
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        comparator.on_neighbor_changed(&mut ctx, pos, Dir::East);

        let due = ticks.drain_due(COMPARATOR_DELAY);
        assert_eq!(due[0].priority, TickPriority::High, "diode-fed comparators prioritise");
    }

    #[test]
    fn diode_priorities_order_correctly_against_each_other() {
        // The ordering that actually matters when several fire in one tick.
        assert!(TickPriority::ExtremelyHigh < TickPriority::VeryHigh);
        assert!(TickPriority::VeryHigh < TickPriority::High);
        assert!(TickPriority::High < TickPriority::Normal);
    }
}

/// A lever — `LeverBlock`.
///
/// `useWithoutItem` cycles `powered` (loud write), then additionally updates
/// the neighbours of the block it is attached to: a powered lever powers
/// every face weakly and its support block **strongly**, so components on the
/// far side of that block hear the flip through the extra update wave.
pub struct Lever {
    /// Current `powered`.
    pub powered: bool,
    /// The unpowered/powered pair.
    pub states: StatePair,
    /// The direction from the lever to its support block (floor → down,
    /// ceiling → up, wall → behind the facing).
    pub attached: crate::pos::Dir,
}

impl crate::behaviour::BlockBehaviour for Lever {
    fn on_used(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        // pull: setBlock (entry 1, via set), then updateNeighbours —
        // updateNeighborsAt(pos) again and at the support block.
        ctx.set(pos, if self.powered { self.states.off } else { self.states.on });
        ctx.update_neighbors_at(pos);
        ctx.update_neighbors_at(pos.offset(self.attached));
    }

    fn name(&self) -> &'static str {
        "lever"
    }
}
