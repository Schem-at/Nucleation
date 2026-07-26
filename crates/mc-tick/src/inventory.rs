//! Container inventories — the state a comparator reads and a hopper moves.
//!
//! Vanilla keeps these in block entities. The engine keeps them beside the
//! world (like comparator output memory), keyed by position, because block
//! *states* cannot express "27 slots holding 40 redstone".
//!
//! # The analog signal, from bytecode
//!
//! `AbstractContainerMenu.getRedstoneSignalFromContainer`:
//!
//! ```text
//! fullness = Σ over occupied slots (count / maxStackSize) / containerSize
//! signal   = Mth.lerpDiscrete(fullness, 0, 15)
//!          = floor(fullness * 14) + (fullness > 0 ? 1 : 0)
//! ```
//!
//! So any item at all produces at least 1, and only a completely full
//! container reaches 15.
//!
//! # Known simplification
//!
//! `maxStackSize` is per-item data the engine does not carry; everything is
//! treated as stacking to 64. Conformance structures must therefore use
//! 64-stackable items (redstone, stone, ...). When a 16- or 1-stack item
//! matters, the fix is a real max-stack table, not a guess.

/// One occupied slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    /// Which slot it sits in.
    pub slot: u8,
    /// The item's identifier, e.g. `minecraft:redstone`.
    pub id: String,
    /// How many.
    pub count: u8,
}

/// A container's contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Total slot count — a barrel has 27, a hopper 5.
    pub slots: u32,
    /// The occupied slots.
    pub stacks: Vec<ItemStack>,
}

/// The stack size everything is assumed to reach; see the module docs.
const ASSUMED_MAX_STACK: f32 = 64.0;

impl Inventory {
    /// An empty container with `slots` slots.
    pub fn empty(slots: u32) -> Self {
        Self { slots, stacks: Vec::new() }
    }

    /// Whether nothing is stored.
    pub fn is_empty(&self) -> bool {
        self.stacks.iter().all(|stack| stack.count == 0)
    }

    /// The comparator signal this container produces, 0-15.
    pub fn analog_signal(&self) -> u8 {
        if self.slots == 0 {
            return 0;
        }
        let fullness: f32 = self
            .stacks
            .iter()
            .map(|stack| f32::from(stack.count) / ASSUMED_MAX_STACK)
            .sum::<f32>()
            / self.slots as f32;
        let stepped = (fullness * 14.0).floor() as u8;
        stepped + u8::from(fullness > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn barrel(stacks: Vec<(u8, u8)>) -> Inventory {
        Inventory {
            slots: 27,
            stacks: stacks
                .into_iter()
                .map(|(slot, count)| ItemStack {
                    slot,
                    id: "minecraft:redstone".to_string(),
                    count,
                })
                .collect(),
        }
    }

    #[test]
    fn an_empty_container_reads_zero() {
        assert_eq!(barrel(vec![]).analog_signal(), 0);
    }

    #[test]
    fn a_single_item_already_reads_one() {
        // The `fullness > 0` term: any content at all lifts the floor to 1.
        assert_eq!(barrel(vec![(0, 1)]).analog_signal(), 1);
    }

    #[test]
    fn a_full_container_reads_fifteen() {
        let full: Vec<(u8, u8)> = (0..27).map(|slot| (slot, 64)).collect();
        assert_eq!(barrel(full).analog_signal(), 15);
    }

    #[test]
    fn the_formula_steps_where_vanillas_does() {
        // Three full stacks in a barrel: fullness 3/27, * 14 = 1.55 -> 1, +1 = 2.
        assert_eq!(barrel(vec![(0, 64), (1, 64), (2, 64)]).analog_signal(), 2);
        // Fourteen full stacks: 14/27 * 14 = 7.26 -> 7, +1 = 8.
        let fourteen: Vec<(u8, u8)> = (0..14).map(|slot| (slot, 64)).collect();
        assert_eq!(barrel(fourteen).analog_signal(), 8);
    }
}

/// Container contents by position — the map the simulation owns.
pub type InventoryMap = std::collections::HashMap<crate::pos::Pos, Inventory>;
