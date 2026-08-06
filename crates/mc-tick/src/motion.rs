//! `Entity.load`'s `Motion` handling — and the version boundary that decides
//! whether a NaN velocity survives being read off disk.
//!
//! This is not a detail. The record 3x3 piston door's whole mechanism is
//! minecarts whose velocity is NaN: their physics are dead, so they hold
//! villagers in place and sit inside blocks an ordinary cart would be shoved
//! out of. Whether the door works at all is decided by one branch in
//! `Entity.load`, and that branch changed.
//!
//! Both halves are bytecode-verified across a full bisect; the evidence, with
//! the disassembly and the empirical probe, is in
//! `tools/gametest/NAN-MOTION-VERSIONS.md`.
//!
//! ```text
//! DataVersion <= 4556   (1.21.10 and earlier)
//!     setDeltaMovement(Math.abs(x) > 10.0 ? 0.0 : x, ...same y, z...)
//!     compiles to  Math.abs(d); ldc2_w 10.0; dcmpl; ifle <keep>
//!     `dcmpl` yields -1 for NaN, so `ifle` is taken and NaN is KEPT.
//!     +-Infinity and any |v| > 10 become 0.0.
//!
//! DataVersion >= 4671   (1.21.11 and later, including the 26.2 oracle)
//!     setDeltaMovement(Vec3) gained an isFinite guard that drops the whole
//!     vector silently, leaving the entity's previous velocity in place —
//!     zero, for an entity that has just been constructed.
//! ```
//!
//! Nothing here is a deviation from the oracle. The oracle runs 26.2 and
//! answers for 26.2; a 1.21.3 save is a different game, and modelling it
//! per version is the engine's entire premise.
//!
//! # Where this applies, and where it must not
//!
//! Only on the **NBT load path** — an entity being read out of a saved world
//! or a structure file. Spawning an entity programmatically (the gametest
//! oracle's `--spawn`, or a capture replay) does not go through `Entity.load`,
//! and in 26.2 the capture harness has to write `Entity.deltaMovement`
//! directly to get a NaN in there at all. So [`crate::sim::Simulation`]'s
//! three-argument spawns take the velocity verbatim, and only the
//! structure-authored spawns run it through here.

/// Which version's `Entity.load` Motion handling to apply.
///
/// Deliberately an enum rather than a raw `DataVersion`, so that reading it
/// back off a [`crate::sim::Simulation`] answers the question a caller
/// actually has — "which semantics ran?" — rather than a number they would
/// have to re-derive the boundary from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotionSemantics {
    /// `DataVersion <= 4556`. Per-component `|v| > 10 -> 0.0`, via `dcmpl`,
    /// which keeps NaN and kills infinities. **Nan carts work from a cold
    /// load.**
    ClampAbsTen,
    /// `DataVersion >= 4671`. `isFinite` guard on the whole vector: any
    /// non-finite component discards all three and the entity keeps the
    /// velocity it already had. **Nan carts do not survive loading.**
    DropNonFinite,
}

/// The last DataVersion whose `Entity.load` keeps NaN — 1.21.10.
pub const LAST_NAN_KEEPING_DATA_VERSION: i32 = 4556;
/// The first DataVersion with the `isFinite` guard — 1.21.11.
pub const FIRST_NAN_DROPPING_DATA_VERSION: i32 = 4671;

impl MotionSemantics {
    /// Which semantics a save of this DataVersion loads under.
    ///
    /// The bisect left a gap between 4556 and 4671 with no released
    /// DataVersion in it; anything landing there is treated as the new
    /// behaviour, because the guard is present by 4671 and the snapshots
    /// between are where it appeared.
    pub fn for_data_version(data_version: i32) -> MotionSemantics {
        if data_version <= LAST_NAN_KEEPING_DATA_VERSION {
            MotionSemantics::ClampAbsTen
        } else {
            MotionSemantics::DropNonFinite
        }
    }

    /// Apply `Entity.load`'s Motion handling.
    ///
    /// `previous` is the velocity the entity already carries — what the
    /// `isFinite` guard leaves behind when it drops a vector. For a
    /// freshly-constructed entity that is zero, which is every case the
    /// structure loader has.
    pub fn load_motion(self, motion: [f64; 3], previous: [f64; 3]) -> [f64; 3] {
        match self {
            // `Math.abs(v) > 10.0 ? 0.0 : v`, compiled with `dcmpl; ifle`.
            // Written as `!(abs > 10.0)` rather than `abs <= 10.0` so the NaN
            // case takes the keep branch exactly as `dcmpl` makes it: both
            // comparisons are false for NaN, and only the negated form then
            // keeps the value.
            MotionSemantics::ClampAbsTen => {
                let clamp = |v: f64| if v.abs() > 10.0 { 0.0 } else { v };
                [clamp(motion[0]), clamp(motion[1]), clamp(motion[2])]
            }
            // `setDeltaMovement(Vec3)`: one `isFinite` over all three, and the
            // write is skipped whole. Not per-component — a vector with a NaN
            // in z loses its finite x too.
            MotionSemantics::DropNonFinite => {
                if motion.iter().all(|v| v.is_finite()) {
                    motion
                } else {
                    previous
                }
            }
        }
    }

    /// Whether this path can carry a non-finite velocity out of a save at all.
    ///
    /// A door whose mechanism is NaN velocities is a different build under the
    /// two, and a caller that cannot tell them apart cannot report why it came
    /// apart.
    pub fn preserves_non_finite(self) -> bool {
        matches!(self, MotionSemantics::ClampAbsTen)
    }
}

impl Default for MotionSemantics {
    /// The oracle's own version. Every captured trace in this repo is 26.2, so
    /// a simulation nobody has told about a DataVersion behaves like the game
    /// the goldens came from.
    fn default() -> MotionSemantics {
        MotionSemantics::DropNonFinite
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The boundary, from `tools/gametest/NAN-MOTION-VERSIONS.md`. 4082 is the
    /// record door's own save.
    #[test]
    fn the_version_boundary_is_where_the_bisect_put_it() {
        assert_eq!(
            MotionSemantics::for_data_version(4082),
            MotionSemantics::ClampAbsTen
        );
        assert_eq!(
            MotionSemantics::for_data_version(4556),
            MotionSemantics::ClampAbsTen
        );
        assert_eq!(
            MotionSemantics::for_data_version(4671),
            MotionSemantics::DropNonFinite
        );
        assert_eq!(
            MotionSemantics::for_data_version(4903),
            MotionSemantics::DropNonFinite
        );
    }

    /// The `nanprobe` table, byte-identical bytecode to the game's:
    /// NaN survives, +-Inf and 99.0 are zeroed, 0.5 passes.
    #[test]
    fn pre_4671_keeps_nan_and_kills_infinity() {
        let s = MotionSemantics::ClampAbsTen;
        let out = s.load_motion([f64::NAN, f64::INFINITY, 99.0], [0.0; 3]);
        assert!(out[0].is_nan(), "dcmpl takes the keep branch for NaN");
        assert_eq!(out[1], 0.0);
        assert_eq!(out[2], 0.0);
        assert_eq!(
            s.load_motion([0.5, -0.5, 10.0], [0.0; 3]),
            [0.5, -0.5, 10.0]
        );
        assert_eq!(
            s.load_motion([f64::NEG_INFINITY, 0.0, 0.0], [0.0; 3])[0],
            0.0
        );
    }

    /// The door's own six carts: `Motion.z` NaN with a finite `Motion.x`
    /// inside the clamp. Under 4082 semantics they load intact.
    #[test]
    fn the_doors_carts_load_intact_under_their_own_version() {
        let s = MotionSemantics::for_data_version(4082);
        let out = s.load_motion([-0.542609, -0.05605, f64::NAN], [0.0; 3]);
        assert_eq!(out[0], -0.542609);
        assert_eq!(out[1], -0.05605);
        assert!(out[2].is_nan());
    }

    /// The same cart under 26.2: the whole vector goes, finite components
    /// included, and the cart keeps what it had.
    #[test]
    fn post_4671_drops_the_whole_vector_not_just_the_bad_component() {
        let s = MotionSemantics::DropNonFinite;
        assert_eq!(
            s.load_motion([-0.542609, -0.05605, f64::NAN], [0.0; 3]),
            [0.0; 3]
        );
        assert_eq!(s.load_motion([0.5, 0.0, 0.0], [0.0; 3]), [0.5, 0.0, 0.0]);
        // The guard leaves the *previous* velocity, which is not always zero.
        assert_eq!(
            s.load_motion([f64::NAN, 0.0, 0.0], [0.25, 0.0, 0.0]),
            [0.25, 0.0, 0.0]
        );
    }

    /// 26.2 does not clamp large finite velocities — that was the old path's
    /// job, and conflating the two would silently zero a fast cart.
    #[test]
    fn the_ten_clamp_is_not_carried_forward() {
        assert_eq!(
            MotionSemantics::DropNonFinite.load_motion([99.0, 0.0, 0.0], [0.0; 3])[0],
            99.0
        );
    }

    #[test]
    fn only_the_old_path_can_carry_a_nan_out_of_a_save() {
        assert!(MotionSemantics::ClampAbsTen.preserves_non_finite());
        assert!(!MotionSemantics::DropNonFinite.preserves_non_finite());
    }
}
