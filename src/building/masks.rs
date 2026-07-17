//! Fill masks: which existing cells a fill operation may overwrite.
//!
//! [`FillMode`] is consulted per point before the brush is asked for a block,
//! so masked fills never disturb cells the mode protects.

use crate::BlockState;

/// Controls which existing cells a fill may overwrite.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FillMode {
    /// Overwrite every cell the shape covers (plain fill behavior).
    #[default]
    Replace,
    /// Only write where the schematic has nothing yet — unset cells or air.
    KeepExisting,
    /// Only overwrite cells whose current block id is in the list (e.g.
    /// replace the stone family with bricks inside a shape). Ids are compared
    /// against the block name, ignoring state properties.
    ReplaceOnly(Vec<String>),
}

impl FillMode {
    /// Whether a cell currently holding `existing` (`None` = outside any
    /// region) may be written under this mode.
    pub fn allows(&self, existing: Option<&BlockState>) -> bool {
        match self {
            FillMode::Replace => true,
            FillMode::KeepExisting => existing.is_none_or(|b| is_air(b.name.as_str())),
            FillMode::ReplaceOnly(targets) => {
                existing.is_some_and(|b| targets.iter().any(|t| t == b.name.as_str()))
            }
        }
    }
}

fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}
