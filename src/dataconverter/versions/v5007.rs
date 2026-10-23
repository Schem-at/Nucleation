//! V5007 (26.3 snapshot) — `minecraft:swing_animation` is split in two.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! `net.minecraft.util.datafix.fixes.SwingAnimationComponentSplitFix`
//! (registered at schema 5007 in `DataFixers`): a DATA_COMPONENTS
//! `minecraft:swing_animation` is removed and its value copied to both
//! `minecraft:attack_animation` and `minecraft:interact_animation`.
//! DATA_COMPONENTS is schematic-relevant (item stacks), so this is ported in
//! full.
//!
//! Reverse: `swing_animation` is restored from `attack_animation` (falling
//! back to `interact_animation`) and both new components are removed. This is
//! exact when the two are equal, which is what the forward produces. If they
//! differ, or only one of them is set, the single pre-26.3 component cannot
//! express it; the attack value is kept and the downgrade is reported as an
//! approximation.
//!
//! VERSION = 5007 (Mojang schema id).

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 5007;

const SWING: &str = "minecraft:swing_animation";
const ATTACK: &str = "minecraft:attack_animation";
const INTERACT: &str = "minecraft:interact_animation";

pub fn register(reg: &mut RegistryBuilder) {
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(swing) = data.take(SWING) {
                data.set_generic(ATTACK, swing.clone());
                data.set_generic(INTERACT, swing);
            }
        }),
    );
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let attack = data.take(ATTACK);
            let interact = data.take(INTERACT);
            let restored = match (attack, interact) {
                (None, None) => return,
                (Some(a), Some(i)) => {
                    if a != i {
                        report_loss(
                            VERSION,
                            LossKind::ComponentDropped,
                            Severity::Approximated,
                            "attack_animation and interact_animation differ; pre-26.3 swing_animation keeps the attack value",
                        );
                    }
                    a
                }
                (Some(a), None) => {
                    report_loss(
                        VERSION,
                        LossKind::ComponentDropped,
                        Severity::Approximated,
                        "only attack_animation is set; pre-26.3 swing_animation also applies it to interactions",
                    );
                    a
                }
                (None, Some(i)) => {
                    report_loss(
                        VERSION,
                        LossKind::ComponentDropped,
                        Severity::Approximated,
                        "only interact_animation is set; pre-26.3 swing_animation also applies it to attacks",
                    );
                    i
                }
            };
            data.set_generic(SWING, restored);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{convert_item_stack, convert_item_stack_reverse};

    fn item_with(components: NbtMap) -> NbtMap {
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:iron_sword");
        item.set_i32("count", 1);
        item.set_map("components", components);
        item
    }

    fn swing() -> NbtMap {
        let mut s = NbtMap::new();
        s.set_string("type", "stab");
        s.set_i32("duration", 8);
        s
    }

    #[test]
    fn swing_split_and_rejoined() {
        let mut components = NbtMap::new();
        components.set_map(SWING, swing());
        let mut item = item_with(components);

        convert_item_stack(&mut item, VERSION - 1, VERSION);
        let c = item.get_map("components").unwrap();
        assert!(!c.has_key(SWING));
        assert_eq!(c.get_map(ATTACK), Some(&swing()));
        assert_eq!(c.get_map(INTERACT), Some(&swing()));

        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert!(report.is_empty());
        let c = item.get_map("components").unwrap();
        assert_eq!(c.get_map(SWING), Some(&swing()));
        assert!(!c.has_key(ATTACK) && !c.has_key(INTERACT));
    }

    #[test]
    fn diverging_animations_report_approximation() {
        let mut other = swing();
        other.set_string("type", "whack");
        let mut components = NbtMap::new();
        components.set_map(ATTACK, swing());
        components.set_map(INTERACT, other);
        let mut item = item_with(components);

        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert_eq!(report.len(), 1);
        assert_eq!(report.entries[0].severity, Severity::Approximated);
        assert_eq!(
            item.get_map("components").unwrap().get_map(SWING),
            Some(&swing())
        );
    }
}
