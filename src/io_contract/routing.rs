//! Wiring-area constraints (region model).
//!
//! DEF-style region constraints layered on [`DefinitionRegion`]: a named
//! [`RoutingRegion`] is a union of include boxes minus exclude boxes, and a
//! [`NetClassRule`] confines a class of nets to a region with layer / bias /
//! spacing / length discipline. Both attach to `DefinitionRegion`s by name
//! through metadata, so zones are authorable in-world (Insign) or via API.

use crate::bounding_box::BoundingBox;
use crate::definition_region::DefinitionRegion;
use crate::transforms::Axis;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata key marking a DefinitionRegion as part of a routing zone.
pub const ROUTE_ZONE_NAME_KEY: &str = "route_zone.name";
/// Metadata key holding the zone mode ("include" | "exclude").
pub const ROUTE_ZONE_MODE_KEY: &str = "route_zone.mode";
/// Metadata key holding a JSON-serialized [`NetClassRule`].
pub const NET_CLASS_KEY: &str = "net_class";

/// Whether a zone's boxes are routable area or forbidden area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteZoneMode {
    #[default]
    Include,
    Exclude,
}

impl RouteZoneMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "include" => Some(RouteZoneMode::Include),
            "exclude" => Some(RouteZoneMode::Exclude),
            _ => None,
        }
    }
}

/// A named routing area: union of include boxes minus union of exclude boxes.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoutingRegion {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<BoundingBox>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<BoundingBox>,
}

impl RoutingRegion {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }

    /// The router's legality check: inside some include box (or no include
    /// boxes were declared) and inside no exclude box.
    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let inside = |b: &BoundingBox| {
            x >= b.min.0
                && x <= b.max.0
                && y >= b.min.1
                && y <= b.max.1
                && z >= b.min.2
                && z <= b.max.2
        };
        if self.exclude.iter().any(inside) {
            return false;
        }
        self.include.is_empty() || self.include.iter().any(inside)
    }

    /// Fold a tagged [`DefinitionRegion`]'s boxes into this zone.
    pub fn absorb(&mut self, mode: RouteZoneMode, region: &DefinitionRegion) {
        let target = match mode {
            RouteZoneMode::Include => &mut self.include,
            RouteZoneMode::Exclude => &mut self.exclude,
        };
        target.extend(region.boxes_ref().iter().cloned());
    }

    /// Tag a DefinitionRegion as belonging to the named zone with the given
    /// mode (stored in its metadata; the region's boxes carry the geometry).
    pub fn tag(region: &mut DefinitionRegion, name: &str, mode: RouteZoneMode) {
        region.set_metadata(ROUTE_ZONE_NAME_KEY, name);
        region.set_metadata(
            ROUTE_ZONE_MODE_KEY,
            match mode {
                RouteZoneMode::Include => "include",
                RouteZoneMode::Exclude => "exclude",
            },
        );
    }

    /// Read a DefinitionRegion's zone tag, if any.
    pub fn tag_of(region: &DefinitionRegion) -> Option<(String, RouteZoneMode)> {
        let name = region.get_metadata(ROUTE_ZONE_NAME_KEY)?.clone();
        let mode = region
            .get_metadata(ROUTE_ZONE_MODE_KEY)
            .and_then(|m| RouteZoneMode::parse(m))
            .unwrap_or_default();
        Some((name, mode))
    }

    /// Collect all tagged DefinitionRegions into named routing regions.
    pub fn collect<'a>(
        regions: impl IntoIterator<Item = &'a DefinitionRegion>,
    ) -> HashMap<String, RoutingRegion> {
        let mut zones: HashMap<String, RoutingRegion> = HashMap::new();
        for region in regions {
            if let Some((name, mode)) = Self::tag_of(region) {
                zones
                    .entry(name.clone())
                    .or_insert_with(|| RoutingRegion::new(name))
                    .absorb(mode, region);
            }
        }
        zones
    }
}

/// Routing discipline for a class of nets.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NetClassRule {
    /// Confine nets of this class to the named [`RoutingRegion`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Layer assignment: inclusive Y band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y_band: Option<(i32, i32)>,
    /// Corridor discipline: preferred running axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction_bias: Option<Axis>,
    /// Extra clearance in blocks beyond the base design rules (e.g. clock).
    #[serde(default)]
    pub spacing: u8,
    /// Delay budget: maximum route length in redstone ticks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_len_rt: Option<u32>,
}

impl NetClassRule {
    /// Attach this rule to a DefinitionRegion (JSON in metadata under
    /// [`NET_CLASS_KEY`]).
    pub fn attach_to(&self, region: &mut DefinitionRegion) -> Result<(), String> {
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        region.set_metadata(NET_CLASS_KEY, json);
        Ok(())
    }

    /// Read an attached rule back from a DefinitionRegion, if present.
    pub fn from_region(region: &DefinitionRegion) -> Result<Option<Self>, String> {
        match region.get_metadata(NET_CLASS_KEY) {
            None => Ok(None),
            Some(json) => serde_json::from_str(json)
                .map(Some)
                .map_err(|e| e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(min: (i32, i32, i32), max: (i32, i32, i32)) -> DefinitionRegion {
        DefinitionRegion::from_bounds(min, max)
    }

    #[test]
    fn contains_include_exclude() {
        let zone = RoutingRegion {
            name: "bus_north".into(),
            include: vec![BoundingBox {
                min: (0, 0, 0),
                max: (10, 3, 10),
            }],
            exclude: vec![BoundingBox {
                min: (4, 0, 4),
                max: (6, 3, 6),
            }],
        };
        assert!(zone.contains(1, 1, 1));
        assert!(!zone.contains(5, 1, 5)); // excluded hole
        assert!(!zone.contains(11, 1, 1)); // outside include
                                           // no include boxes = everywhere except excludes
        let open = RoutingRegion {
            name: "open".into(),
            include: vec![],
            exclude: vec![BoundingBox {
                min: (0, 0, 0),
                max: (1, 1, 1),
            }],
        };
        assert!(open.contains(100, 0, 100));
        assert!(!open.contains(0, 0, 0));
    }

    #[test]
    fn tag_and_collect_by_name() {
        let mut a = region((0, 0, 0), (10, 0, 2));
        let mut b = region((0, 0, 8), (10, 0, 10));
        let mut hole = region((3, 0, 0), (4, 0, 2));
        let mut untagged = region((50, 0, 50), (60, 0, 60));
        RoutingRegion::tag(&mut a, "bus_north", RouteZoneMode::Include);
        RoutingRegion::tag(&mut b, "bus_north", RouteZoneMode::Include);
        RoutingRegion::tag(&mut hole, "bus_north", RouteZoneMode::Exclude);
        let _ = &mut untagged;

        let zones = RoutingRegion::collect([&a, &b, &hole, &untagged]);
        assert_eq!(zones.len(), 1);
        let zone = &zones["bus_north"];
        assert_eq!(zone.include.len(), 2);
        assert_eq!(zone.exclude.len(), 1);
        assert!(zone.contains(0, 0, 9));
        assert!(!zone.contains(3, 0, 1));
    }

    #[test]
    fn net_class_rule_attach_round_trip() {
        let rule = NetClassRule {
            region: Some("bus_north".into()),
            y_band: Some((64, 68)),
            direction_bias: Some(Axis::X),
            spacing: 2,
            max_len_rt: Some(12),
        };
        let mut dr = region((0, 0, 0), (5, 5, 5));
        rule.attach_to(&mut dr).unwrap();
        let back = NetClassRule::from_region(&dr).unwrap().unwrap();
        assert_eq!(back, rule);
        // untouched region carries no rule
        assert_eq!(
            NetClassRule::from_region(&region((0, 0, 0), (1, 1, 1))).unwrap(),
            None
        );
    }

    #[test]
    fn net_class_rule_json_defaults() {
        let json = "{}";
        let rule: NetClassRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule, NetClassRule::default());
    }
}
