//! Insign DSL extensions for the IO contract layer.
//!
//! Insign's grammar is `@geometry` + `#[target:]key=<json>` metadata (see the
//! `insign` crate). These parsers add the cell-authoring vocabulary on top of
//! that grammar — no new syntax, only new well-known keys:
//!
//! **Cell header** (global metadata):
//! ```text
//! #$global:cell.name="full_adder"
//! #$global:cell.version="1.2.0"
//! ```
//!
//! **Bus annotation** — one sign annotates a whole bus on an `io.*` region:
//! ```text
//! @io.a=rc([0,0,0],[0,0,6])
//! #type="input"
//! #data_type="unsigned"
//! #bus.width=4
//! #bus.bit_order="lsb_first"
//! #bus.face="east"
//! #bus.encoding="binary"      // optional, default binary
//! ```
//!
//! **Route zones** — sign a zone (`#route_zone="<name> include|exclude"`,
//! or the expanded `#route_zone.name=` / `#route_zone.mode=` pair):
//! ```text
//! @rc([0,64,0],[31,66,3])
//! #route_zone="bus_north include"
//! ```

use super::bus::BusEncoding;
use super::physical::Face;
use super::routing::{RouteZoneMode, RoutingRegion};
use crate::bounding_box::BoundingBox;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Errors from IO-contract Insign parsing.
#[derive(Debug, thiserror::Error)]
pub enum InsignContractError {
    #[error("Insign compilation error: {0}")]
    Compile(String),

    #[error("Invalid value for '{key}' on region '{region}': {reason}")]
    InvalidValue {
        region: String,
        key: String,
        reason: String,
    },

    #[error("Missing required key '{key}' on region '{region}'")]
    MissingKey { region: String, key: String },
}

/// `#cell` header: name and optional version of the cell being authored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellHeader {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Bit order of a bus annotation relative to position order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BitOrder {
    #[default]
    LsbFirst,
    MsbFirst,
}

impl BitOrder {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "lsb_first" | "lsb" | "little" => Some(BitOrder::LsbFirst),
            "msb_first" | "msb" | "big" => Some(BitOrder::MsbFirst),
            _ => None,
        }
    }
}

/// A whole-bus annotation attached to an `io.<name>` region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusAnnotation {
    /// Port name (the `io.` prefix stripped).
    pub name: String,
    pub width: u8,
    #[serde(default)]
    pub bit_order: BitOrder,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub face: Option<Face>,
    pub encoding: BusEncoding,
}

fn compile(input: &[([i32; 3], String)]) -> Result<::insign::DslMap, InsignContractError> {
    ::insign::compile(input).map_err(|e| InsignContractError::Compile(e.to_string()))
}

fn meta_str<'a>(
    metadata: &'a std::collections::BTreeMap<String, JsonValue>,
    key: &str,
) -> Option<&'a str> {
    metadata.get(key).and_then(|v| v.as_str())
}

/// Parse the `#cell` header from global metadata (`cell.name`,
/// `cell.version`). Returns `Ok(None)` when no header is present.
pub fn parse_cell_header(
    input: &[([i32; 3], String)],
) -> Result<Option<CellHeader>, InsignContractError> {
    let map = compile(input)?;
    parse_cell_header_from_map(&map)
}

/// Same as [`parse_cell_header`] but over an already-compiled map.
pub fn parse_cell_header_from_map(
    map: &::insign::DslMap,
) -> Result<Option<CellHeader>, InsignContractError> {
    let Some(global) = map.get("$global") else {
        return Ok(None);
    };
    let Some(name_val) = global.metadata.get("cell.name") else {
        // A version without a name is an authoring mistake worth flagging.
        if global.metadata.contains_key("cell.version") {
            return Err(InsignContractError::MissingKey {
                region: "$global".to_string(),
                key: "cell.name".to_string(),
            });
        }
        return Ok(None);
    };
    let name = name_val
        .as_str()
        .ok_or_else(|| InsignContractError::InvalidValue {
            region: "$global".to_string(),
            key: "cell.name".to_string(),
            reason: format!("expected string, got {}", name_val),
        })?
        .to_string();
    let version = match global.metadata.get("cell.version") {
        None => None,
        Some(v) => Some(
            v.as_str()
                .ok_or_else(|| InsignContractError::InvalidValue {
                    region: "$global".to_string(),
                    key: "cell.version".to_string(),
                    reason: format!("expected string, got {}", v),
                })?
                .to_string(),
        ),
    };
    Ok(Some(CellHeader { name, version }))
}

/// Parse bus annotations from `io.*` regions carrying `bus.*` metadata.
pub fn parse_bus_annotations(
    input: &[([i32; 3], String)],
) -> Result<Vec<BusAnnotation>, InsignContractError> {
    let map = compile(input)?;
    parse_bus_annotations_from_map(&map)
}

/// Same as [`parse_bus_annotations`] but over an already-compiled map.
pub fn parse_bus_annotations_from_map(
    map: &::insign::DslMap,
) -> Result<Vec<BusAnnotation>, InsignContractError> {
    let mut buses = Vec::new();
    for (region_name, entry) in map.iter() {
        let Some(port_name) = region_name.strip_prefix("io.") else {
            continue;
        };
        let Some(width_val) = entry.metadata.get("bus.width") else {
            continue; // not a bus-annotated port
        };
        let width = width_val
            .as_u64()
            .filter(|w| (1..=255).contains(w))
            .ok_or_else(|| InsignContractError::InvalidValue {
                region: region_name.clone(),
                key: "bus.width".to_string(),
                reason: format!("expected integer 1-255, got {}", width_val),
            })? as u8;

        let bit_order = match meta_str(&entry.metadata, "bus.bit_order") {
            None => BitOrder::default(),
            Some(s) => BitOrder::parse(s).ok_or_else(|| InsignContractError::InvalidValue {
                region: region_name.clone(),
                key: "bus.bit_order".to_string(),
                reason: format!("expected lsb_first|msb_first, got '{}'", s),
            })?,
        };

        let face = match meta_str(&entry.metadata, "bus.face") {
            None => None,
            Some(s) => Some(
                Face::parse(s).ok_or_else(|| InsignContractError::InvalidValue {
                    region: region_name.clone(),
                    key: "bus.face".to_string(),
                    reason: format!("expected a direction name, got '{}'", s),
                })?,
            ),
        };

        let encoding = match meta_str(&entry.metadata, "bus.encoding") {
            None => BusEncoding::Binary1PerWire,
            Some(s) => BusEncoding::parse(s).ok_or_else(|| InsignContractError::InvalidValue {
                region: region_name.clone(),
                key: "bus.encoding".to_string(),
                reason: format!("expected binary|hex_analog, got '{}'", s),
            })?,
        };

        buses.push(BusAnnotation {
            name: port_name.to_string(),
            width,
            bit_order,
            face,
            encoding,
        });
    }
    Ok(buses)
}

/// Parse `#route_zone` annotations into named [`RoutingRegion`]s.
///
/// Accepted forms on any region with geometry:
/// - compact: `#route_zone="<name> include"` / `#route_zone="<name> exclude"`
///   (mode defaults to `include` when only a name is given)
/// - expanded: `#route_zone.name="<name>"` + `#route_zone.mode="include"`
pub fn parse_route_zones(
    input: &[([i32; 3], String)],
) -> Result<HashMap<String, RoutingRegion>, InsignContractError> {
    let map = compile(input)?;
    parse_route_zones_from_map(&map)
}

/// Same as [`parse_route_zones`] but over an already-compiled map.
pub fn parse_route_zones_from_map(
    map: &::insign::DslMap,
) -> Result<HashMap<String, RoutingRegion>, InsignContractError> {
    let mut zones: HashMap<String, RoutingRegion> = HashMap::new();
    for (region_name, entry) in map.iter() {
        let tag = if let Some(compact) = entry.metadata.get("route_zone") {
            let s = compact
                .as_str()
                .ok_or_else(|| InsignContractError::InvalidValue {
                    region: region_name.clone(),
                    key: "route_zone".to_string(),
                    reason: format!("expected \"<name> include|exclude\", got {}", compact),
                })?;
            let mut parts = s.split_whitespace();
            let name = parts.next().filter(|n| !n.is_empty()).ok_or_else(|| {
                InsignContractError::InvalidValue {
                    region: region_name.clone(),
                    key: "route_zone".to_string(),
                    reason: "missing zone name".to_string(),
                }
            })?;
            let mode = match parts.next() {
                None => RouteZoneMode::Include,
                Some(m) => {
                    RouteZoneMode::parse(m).ok_or_else(|| InsignContractError::InvalidValue {
                        region: region_name.clone(),
                        key: "route_zone".to_string(),
                        reason: format!("expected include|exclude, got '{}'", m),
                    })?
                }
            };
            Some((name.to_string(), mode))
        } else if let Some(name) = meta_str(&entry.metadata, "route_zone.name") {
            let mode = match meta_str(&entry.metadata, "route_zone.mode") {
                None => RouteZoneMode::Include,
                Some(m) => {
                    RouteZoneMode::parse(m).ok_or_else(|| InsignContractError::InvalidValue {
                        region: region_name.clone(),
                        key: "route_zone.mode".to_string(),
                        reason: format!("expected include|exclude, got '{}'", m),
                    })?
                }
            };
            Some((name.to_string(), mode))
        } else {
            None
        };

        let Some((name, mode)) = tag else { continue };
        let Some(boxes) = entry.bounding_boxes.as_ref() else {
            return Err(InsignContractError::MissingKey {
                region: region_name.clone(),
                key: "geometry (@rc/@ac)".to_string(),
            });
        };
        let zone = zones
            .entry(name.clone())
            .or_insert_with(|| RoutingRegion::new(name));
        let target = match mode {
            RouteZoneMode::Include => &mut zone.include,
            RouteZoneMode::Exclude => &mut zone.exclude,
        };
        for (min, max) in boxes {
            target.push(BoundingBox {
                min: (min[0], min[1], min[2]),
                max: (max[0], max[1], max[2]),
            });
        }
    }
    Ok(zones)
}

/// All IO-contract annotations of a sign set, as one JSON value:
/// `{ "cell": ..., "buses": [...], "route_zones": {...} }`.
///
/// This is the bridge-facing entry point.
pub fn contracts_json(input: &[([i32; 3], String)]) -> Result<JsonValue, InsignContractError> {
    let map = compile(input)?;
    let cell = parse_cell_header_from_map(&map)?;
    let buses = parse_bus_annotations_from_map(&map)?;
    let zones = parse_route_zones_from_map(&map)?;
    Ok(serde_json::json!({
        "cell": cell,
        "buses": buses,
        "route_zones": zones,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signs(texts: &[&str]) -> Vec<([i32; 3], String)> {
        texts
            .iter()
            .enumerate()
            .map(|(i, t)| ([i as i32 * 16, 64, 0], t.to_string()))
            .collect()
    }

    #[test]
    fn cell_header_parses_name_and_version() {
        let input = signs(&["#$global:cell.name=\"full_adder\"\n#$global:cell.version=\"1.2.0\""]);
        let header = parse_cell_header(&input).unwrap().unwrap();
        assert_eq!(header.name, "full_adder");
        assert_eq!(header.version.as_deref(), Some("1.2.0"));
    }

    #[test]
    fn cell_header_absent_is_none() {
        let input = signs(&["@rc([0,0,0],[1,1,1])\n#doc.label=\"x\""]);
        assert_eq!(parse_cell_header(&input).unwrap(), None);
    }

    #[test]
    fn cell_header_version_without_name_is_error() {
        let input = signs(&["#$global:cell.version=\"1.0\""]);
        assert!(matches!(
            parse_cell_header(&input),
            Err(InsignContractError::MissingKey { .. })
        ));
    }

    #[test]
    fn cell_header_non_string_name_is_error() {
        let input = signs(&["#$global:cell.name=42"]);
        assert!(matches!(
            parse_cell_header(&input),
            Err(InsignContractError::InvalidValue { .. })
        ));
    }

    #[test]
    fn bus_annotation_full_form() {
        let input = signs(&[concat!(
            "@io.a=rc([0,0,0],[0,0,6])\n",
            "#type=\"input\"\n#data_type=\"unsigned\"\n",
            "#bus.width=4\n#bus.bit_order=\"msb_first\"\n",
            "#bus.face=\"east\"\n#bus.encoding=\"hex_analog\""
        )]);
        let buses = parse_bus_annotations(&input).unwrap();
        assert_eq!(buses.len(), 1);
        let b = &buses[0];
        assert_eq!(b.name, "a");
        assert_eq!(b.width, 4);
        assert_eq!(b.bit_order, BitOrder::MsbFirst);
        assert_eq!(b.face, Some(Face::East));
        assert_eq!(b.encoding, BusEncoding::HexAnalog);
    }

    #[test]
    fn bus_annotation_defaults() {
        let input = signs(&["@io.d=rc([0,0,0],[7,0,0])\n#bus.width=8"]);
        let buses = parse_bus_annotations(&input).unwrap();
        let b = &buses[0];
        assert_eq!(b.bit_order, BitOrder::LsbFirst);
        assert_eq!(b.face, None);
        assert_eq!(b.encoding, BusEncoding::Binary1PerWire);
    }

    #[test]
    fn non_bus_io_region_is_skipped() {
        let input = signs(&["@io.clk=rc([0,0,0],[0,0,0])\n#type=\"input\"\n#data_type=\"bool\""]);
        assert!(parse_bus_annotations(&input).unwrap().is_empty());
    }

    #[test]
    fn bus_width_out_of_range_is_error() {
        let input = signs(&["@io.a=rc([0,0,0],[0,0,6])\n#bus.width=0"]);
        assert!(matches!(
            parse_bus_annotations(&input),
            Err(InsignContractError::InvalidValue { .. })
        ));
        let input = signs(&["@io.a=rc([0,0,0],[0,0,6])\n#bus.width=\"four\""]);
        assert!(parse_bus_annotations(&input).is_err());
    }

    #[test]
    fn route_zone_compact_form() {
        let input = signs(&[
            "@rc([0,0,0],[31,2,3])\n#route_zone=\"bus_north include\"",
            "@rc([10,0,1],[12,2,2])\n#route_zone=\"bus_north exclude\"",
        ]);
        let zones = parse_route_zones(&input).unwrap();
        assert_eq!(zones.len(), 1);
        let z = &zones["bus_north"];
        assert_eq!(z.include.len(), 1);
        assert_eq!(z.exclude.len(), 1);
        // sign 0 at [0,64,0]: include box spans [0,64,0]..[31,66,3]
        assert!(z.contains(0, 64, 0));
        // sign 1 at [16,64,0]: exclude box spans [26,64,1]..[28,66,2]
        assert!(!z.contains(27, 65, 1));
        assert!(!z.contains(40, 64, 0)); // outside include
    }

    #[test]
    fn route_zone_expanded_form_and_named_regions() {
        let input = signs(&[concat!(
            "@zones.n=rc([0,0,0],[7,0,7])\n",
            "#zones.n:route_zone.name=\"north\"\n",
            "#zones.n:route_zone.mode=\"exclude\""
        )]);
        let zones = parse_route_zones(&input).unwrap();
        let z = &zones["north"];
        assert!(z.include.is_empty());
        assert_eq!(z.exclude.len(), 1);
    }

    #[test]
    fn route_zone_mode_defaults_to_include() {
        let input = signs(&["@rc([0,0,0],[1,1,1])\n#route_zone=\"lane\""]);
        let zones = parse_route_zones(&input).unwrap();
        assert_eq!(zones["lane"].include.len(), 1);
    }

    #[test]
    fn route_zone_bad_mode_is_error() {
        let input = signs(&["@rc([0,0,0],[1,1,1])\n#route_zone=\"lane sideways\""]);
        assert!(matches!(
            parse_route_zones(&input),
            Err(InsignContractError::InvalidValue { .. })
        ));
    }

    #[test]
    fn route_zone_without_geometry_is_error() {
        // metadata attached to a named region that never gets geometry
        let input = signs(&["#lane.a:route_zone=\"lane include\""]);
        let res = parse_route_zones(&input);
        // insign itself may reject metadata on undefined regions; either way
        // we must not silently produce a zone with no boxes
        match res {
            Ok(zones) => assert!(zones.is_empty() || !zones.contains_key("lane")),
            Err(_) => {}
        }
    }

    #[test]
    fn contracts_json_shape() {
        let input = signs(&[
            "#$global:cell.name=\"cell_x\"",
            "@io.a=rc([0,0,0],[3,0,0])\n#bus.width=4",
            "@rc([0,0,0],[9,2,9])\n#route_zone=\"main include\"",
        ]);
        let json = contracts_json(&input).unwrap();
        assert_eq!(json["cell"]["name"], "cell_x");
        assert_eq!(json["buses"][0]["name"], "a");
        assert!(json["route_zones"]["main"]["include"].is_array());
    }
}
