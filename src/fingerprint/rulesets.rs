//! Shipped default rulesets. Tunable starting points — the redstone ones encode
//! domain knowledge (functional equivalence) that will be refined over time.
//! (`id` / `exact` are not fixed-class rules; they live in `BlockPolicy`.)

use crate::fingerprint::classifier::{BpCategory, Classifier, Match, Rule};

fn rule(matcher: Match, class: &str, keep: &[&str]) -> Rule {
    Rule {
        matcher,
        class: class.into(),
        keep_props: keep.iter().map(|s| s.to_string()).collect(),
    }
}

fn names(list: &[&str]) -> Match {
    Match::Names(list.iter().map(|s| s.to_string()).collect())
}

/// All solid blocks → "solid"; transparent ignored. Material-agnostic silhouette.
pub fn structural() -> Classifier {
    Classifier {
        name: "structural".into(),
        rules: vec![rule(Match::Category(BpCategory::Solid), "solid", &[])],
    }
}

const REDSTONE_COMPONENTS: &[&str] = &[
    "minecraft:redstone_wire",
    "minecraft:repeater",
    "minecraft:comparator",
    "minecraft:redstone_torch",
    "minecraft:redstone_wall_torch",
    "minecraft:piston",
    "minecraft:sticky_piston",
    "minecraft:observer",
    "minecraft:dropper",
    "minecraft:hopper",
    "minecraft:lever",
    "minecraft:redstone_lamp",
    "minecraft:target",
];

fn component_rules(keep: &[&str]) -> Vec<Rule> {
    REDSTONE_COMPONENTS
        .iter()
        .map(|n| rule(names(&[n]), &n.replace("minecraft:", ""), keep))
        .collect()
}

fn glass_match() -> Match {
    Match::Any(vec![
        Match::Glob("*glass".into()),
        names(&["minecraft:glass"]),
    ])
}

fn top_slab_match() -> Match {
    Match::Glob("*_slab".into()).and(Match::Prop("type".into(), "top".into()))
}

/// Computational redstone: top slab ≡ glass ≡ "transparent"; materials collapse.
pub fn redstone_computational() -> Classifier {
    let mut rules = component_rules(&["facing", "delay", "mode"]);
    rules.push(rule(top_slab_match(), "transparent", &[]));
    rules.push(rule(glass_match(), "transparent", &[]));
    rules.push(rule(Match::Category(BpCategory::Solid), "solid", &[]));
    Classifier {
        name: "redstone-computational".into(),
        rules,
    }
}

/// Survival redstone: glass and slabs stay distinct (entity hitbox differs).
pub fn redstone_survival() -> Classifier {
    let mut rules = component_rules(&["facing", "delay", "mode"]);
    rules.push(rule(top_slab_match(), "slab_top", &[]));
    rules.push(rule(glass_match(), "glass", &[]));
    rules.push(rule(Match::Category(BpCategory::Solid), "solid", &[]));
    Classifier {
        name: "redstone-survival".into(),
        rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::BlockState;

    fn bs(name: &str, props: &[(&str, &str)]) -> BlockState {
        BlockState::new(name).with_properties(
            props
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
        )
    }

    #[test]
    fn structural_collapses_solids() {
        let c = structural();
        let wool = c.tokenize(&BlockState::new("minecraft:orange_wool"));
        let concrete = c.tokenize(&BlockState::new("minecraft:purple_concrete"));
        assert_eq!(wool, concrete);
        assert_eq!(wool.as_deref(), Some("solid"));
    }

    #[test]
    fn computational_vs_survival_glass_slab() {
        let top_slab = bs("minecraft:oak_slab", &[("type", "top")]);
        let glass = BlockState::new("minecraft:glass");

        let comp = redstone_computational();
        assert_eq!(comp.tokenize(&top_slab), comp.tokenize(&glass));

        let surv = redstone_survival();
        assert_ne!(surv.tokenize(&top_slab), surv.tokenize(&glass));
    }
}
