//! Predicates that decide whether a block should be reached by a flood.
//!
//! A `Mask` answers "is this block part of the thing I'm trying to select?"
//! at integer block coordinates. Adapter masks (`Not`, `And`, `Or`) compose
//! primitives without allocating.

use crate::universal_schematic::UniversalSchematic;
use crate::BlockState;

/// A predicate over block coordinates. Implementations should be cheap — they
/// will be called once per neighbour of every visited block during a flood.
pub trait Mask: Sync {
    fn test(&self, x: i32, y: i32, z: i32) -> bool;
}

// ── Blanket impls so closures + references work as masks ────────────────────

impl<F: Fn(i32, i32, i32) -> bool + Sync> Mask for F {
    #[inline]
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        (self)(x, y, z)
    }
}

// ── Mask: non-air blocks in a UniversalSchematic ───────────────────────────

/// Matches any block that exists in the schematic and is not `minecraft:air`.
///
/// This is the analogue of WorldEdit's `#existing` mask used by `/that` by
/// default — it floods through every player-placed (or naturally-generated)
/// block while stopping at air.
pub struct NotAirMask<'a> {
    schematic: &'a UniversalSchematic,
}

impl<'a> NotAirMask<'a> {
    pub fn new(schematic: &'a UniversalSchematic) -> Self {
        Self { schematic }
    }
}

impl<'a> Mask for NotAirMask<'a> {
    #[inline]
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        match self.schematic.get_block(x, y, z) {
            Some(b) => !is_air(b),
            None => false,
        }
    }
}

#[inline]
fn is_air(block: &BlockState) -> bool {
    matches!(
        block.get_name(),
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

// ── Mask: explicit allow / deny list of block names ────────────────────────

/// Matches blocks whose `BlockState` name is in the allow-list.
///
/// `BlocklistMask::allow(...)` keeps only the listed names; pair with `NotMask`
/// for a deny-list. Names are matched exactly (e.g. `"minecraft:stone"`).
pub struct BlocklistMask<'a> {
    schematic: &'a UniversalSchematic,
    names: Vec<String>,
}

impl<'a> BlocklistMask<'a> {
    pub fn allow<I, S>(schematic: &'a UniversalSchematic, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            schematic,
            names: names.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Mask for BlocklistMask<'a> {
    #[inline]
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        match self.schematic.get_block(x, y, z) {
            Some(b) => self.names.iter().any(|n| n == b.get_name()),
            None => false,
        }
    }
}

// ── Combinators ────────────────────────────────────────────────────────────

pub struct NotMask<M: Mask>(pub M);
impl<M: Mask> Mask for NotMask<M> {
    #[inline]
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        !self.0.test(x, y, z)
    }
}

pub struct AndMask<A: Mask, B: Mask>(pub A, pub B);
impl<A: Mask, B: Mask> Mask for AndMask<A, B> {
    #[inline]
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        self.0.test(x, y, z) && self.1.test(x, y, z)
    }
}

pub struct OrMask<A: Mask, B: Mask>(pub A, pub B);
impl<A: Mask, B: Mask> Mask for OrMask<A, B> {
    #[inline]
    fn test(&self, x: i32, y: i32, z: i32) -> bool {
        self.0.test(x, y, z) || self.1.test(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_is_a_mask() {
        let m = |x: i32, _y: i32, _z: i32| x == 0;
        assert!(m.test(0, 5, 7));
        assert!(!m.test(1, 5, 7));
    }

    #[test]
    fn not_and_or_combine() {
        let xpos = |x: i32, _, _| x > 0;
        let ypos = |_, y: i32, _| y > 0;

        let both = AndMask(xpos, ypos);
        assert!(both.test(1, 1, 0));
        assert!(!both.test(1, -1, 0));

        let either = OrMask(xpos, ypos);
        assert!(either.test(1, -1, 0));
        assert!(!either.test(-1, -1, 0));

        let neg = NotMask(xpos);
        assert!(neg.test(-1, 0, 0));
        assert!(!neg.test(1, 0, 0));
    }
}
