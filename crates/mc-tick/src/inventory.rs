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
    /// Container contents carried *by the item* — a shulker box in a
    /// dispenser keeps its slots (vanilla's `minecraft:container` component).
    /// `None` for ordinary items.
    pub contents: Option<Vec<ItemStack>>,
}

/// A container's contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Total slot count — a barrel has 27, a hopper 5.
    pub slots: u32,
    /// The occupied slots.
    pub stacks: Vec<ItemStack>,
    /// Slots that reject insertion.
    ///
    /// Vanilla's crafter stores this as its `disabled_slots` int array. A
    /// disabled empty slot is deliberately distinct from an occupied one: it
    /// contributes one to the crafter's comparator signal, but there is no item
    /// for a hopper underneath to extract.
    pub blocked_slots: u16,
}

/// The stack size everything is assumed to reach; see the module docs.
const ASSUMED_MAX_STACK: f32 = 64.0;

impl Inventory {
    /// An empty container with `slots` slots.
    pub fn empty(slots: u32) -> Self {
        Self {
            slots,
            stacks: Vec::new(),
            blocked_slots: 0,
        }
    }

    /// Whether nothing is stored.
    pub fn is_empty(&self) -> bool {
        self.stacks.iter().all(|stack| stack.count == 0)
    }

    /// The comparator signal this container produces, 0-15, using its own
    /// stored slot count. Prefer [`analog_signal_in`](Self::analog_signal_in)
    /// with the block's authoritative count where one is known: an inventory
    /// materialised by a runtime insertion (a hopper pushing into a container
    /// the save left empty) carries `slots: 0`, and trusting that read every
    /// such container as permanently empty.
    pub fn analog_signal(&self) -> u8 {
        self.analog_signal_in(self.slots)
    }

    /// The comparator signal at an authoritative container size.
    pub fn analog_signal_in(&self, slots: u32) -> u8 {
        analog_from(self.fullness_sum(), slots)
    }

    /// The sum of per-stack fullness fractions — the numerator of
    /// `AbstractContainerMenu.getRedstoneSignalFromContainer`, before dividing
    /// by the container size. Summable across a double chest's halves.
    pub fn fullness_sum(&self) -> f32 {
        self.stacks
            .iter()
            .map(|stack| f32::from(stack.count) / ASSUMED_MAX_STACK)
            .sum()
    }

    /// Whether external insertion is forbidden for `slot`.
    pub fn slot_blocked(&self, slot: u8) -> bool {
        slot < 16 && self.blocked_slots & (1 << slot) != 0
    }

    /// `CrafterBlockEntity.getRedstoneSignal`: one strength for each occupied
    /// or disabled slot, over the crafter's fixed nine slots.
    pub fn crafter_signal(&self) -> u8 {
        (0u8..9)
            .filter(|slot| {
                self.slot_blocked(*slot)
                    || self
                        .stacks
                        .iter()
                        .any(|stack| stack.slot == *slot && stack.count > 0)
            })
            .count() as u8
    }
}

/// The comparator signal for a total fullness over `slots` slots — the
/// stepping half of `getRedstoneSignalFromContainer`, shared by single
/// containers and combined double-chest reads.
pub fn analog_from(fullness_sum: f32, slots: u32) -> u8 {
    if slots == 0 {
        return 0;
    }
    let fullness = fullness_sum / slots as f32;
    let stepped = (fullness * 14.0).floor() as u8;
    stepped + u8::from(fullness > 0.0)
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
                    contents: None,
                    count,
                })
                .collect(),
            blocked_slots: 0,
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

    #[test]
    fn a_crafter_counts_disabled_and_occupied_slots_once_each() {
        let mut crafter = Inventory::empty(9);
        crafter.blocked_slots = 1;
        assert_eq!(crafter.crafter_signal(), 1);

        crafter.stacks.push(ItemStack {
            slot: 1,
            id: "minecraft:stone".to_string(),
            count: 1,
            contents: None,
        });
        assert_eq!(crafter.crafter_signal(), 2);

        // Invalid save data cannot make the same slot contribute twice.
        crafter.stacks.push(ItemStack {
            slot: 0,
            id: "minecraft:stone".to_string(),
            count: 1,
            contents: None,
        });
        assert_eq!(crafter.crafter_signal(), 2);
    }
}

/// Container contents by position — the map the simulation owns.
pub type InventoryMap = std::collections::HashMap<crate::pos::Pos, Inventory>;
