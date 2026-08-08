//! Redstone EDA over schematics: the `routing` feature's native surface.
//!
//! `nucleation-routing` (and `pnr-core` under it) deliberately never depend
//! on this crate — same one-way rule as mc-tick. This module is the seam:
//! it converts [`UniversalSchematic`] to a routing
//! [`Workspace`](nucleation_routing::Workspace) and applies routed geometry
//! back, and offers thin conveniences (`route_net`, `drc`, `sta`) over a
//! schematic. The full API lives in the re-exported crates.

pub use nucleation_routing::{self as engine, *};

use crate::UniversalSchematic;

/// Build a routing workspace from a schematic's blocks (full block-state
/// strings; air is skipped).
pub fn workspace_from_schematic(schem: &UniversalSchematic) -> Workspace {
    Workspace::from_blocks(schem.iter_blocks().filter_map(|(bp, bs)| {
        if bs.name.as_str() == "minecraft:air" {
            return None;
        }
        Some((Pos::new(bp.x, bp.y, bp.z), bs.to_string()))
    }))
}

/// Write every workspace cell back into the schematic.
pub fn apply_workspace(schem: &mut UniversalSchematic, ws: &Workspace) -> Result<usize, String> {
    let mut n = 0;
    for (p, block) in ws.cells() {
        schem.set_block_from_string(p.x, p.y, p.z, block)?;
        n += 1;
    }
    Ok(n)
}

/// Route one net through a schematic with default rules and write the
/// result back. Returns the routed path.
pub fn route_net(
    schem: &mut UniversalSchematic,
    src: (i32, i32, i32),
    dst: (i32, i32, i32),
    label: &str,
) -> Result<RouteResult, String> {
    let mut ws = workspace_from_schematic(schem);
    let router = RedstoneRouter::new();
    let res = router
        .route(
            &mut ws,
            Pos::new(src.0, src.1, src.2),
            Pos::new(dst.0, dst.1, dst.2),
            label,
            &[],
        )
        .map_err(|e| format!("{e:?}"))?;
    apply_workspace(schem, &ws)?;
    Ok(res)
}

/// Run DRC over a schematic. Labels are unknown to a bare schematic, so
/// this catches the label-free checks (support, repeater cycles, decay);
/// label-aware short checking needs a `Workspace` with labels attached.
pub fn drc_schematic(schem: &UniversalSchematic, opts: &DrcOptions) -> Vec<Violation> {
    let ws = workspace_from_schematic(schem);
    drc(&ws, opts)
}

/// Intersection of two inclusive boxes; `Err` when they do not overlap.
fn intersect_aabb(a: Aabb, b: Aabb) -> Result<Aabb, String> {
    let min = Pos::new(
        a.min.x.max(b.min.x),
        a.min.y.max(b.min.y),
        a.min.z.max(b.min.z),
    );
    let max = Pos::new(
        a.max.x.min(b.max.x),
        a.max.y.min(b.max.y),
        a.max.z.min(b.max.z),
    );
    if min.x > max.x || min.y > max.y || min.z > max.z {
        return Err("bounds and y_band do not overlap".to_string());
    }
    Ok(Aabb { min, max })
}

fn parse_pos(v: &serde_json::Value, what: &str) -> Result<Pos, String> {
    let c = v
        .as_array()
        .filter(|c| c.len() == 3)
        .ok_or_else(|| format!("{what} must be [x, y, z]"))?;
    let g = |i: usize| -> Result<i32, String> {
        c[i].as_i64()
            .map(|v| v as i32)
            .ok_or_else(|| format!("{what} coordinate must be an integer"))
    };
    Ok(Pos::new(g(0)?, g(1)?, g(2)?))
}

/// Route every net in `nets_json` through the schematic with pnr-core's
/// negotiated congestion (`RedstoneRouter::route_all`), write the emitted
/// geometry back, and return a JSON report.
///
/// Request shape:
///
/// ```json
/// {
///   "nets": [{"label": "a", "src": [0,1,0], "dsts": [[6,1,0]],
///             "friendly": ["b"], "class": "bus"}],
///   "classes": {"bus": {"region": "bus_north", "y_band": [1, 2],
///                        "max_len_rt": 12}},
///   "bounds": [[-1,0,-3], [8,2,7]],
///   "budget": {"refresh": 5, "stair_cap": 4},
///   "congestion": {"max_rounds": 40, "history_increment": 4,
///                   "present_penalty": 6}
/// }
/// ```
///
/// `classes` entries are [`crate::io_contract::routing::NetClassRule`]s; a
/// rule's `region` names a routing zone assembled from the schematic's
/// tagged `DefinitionRegion`s
/// ([`crate::io_contract::routing::RoutingRegion::collect`]), so zones are
/// authorable in-world. `y_band` narrows the router bounds; `spacing` /
/// `direction_bias` are recorded as unsupported notes in v1. `max_len_rt`
/// is checked against the emitted route's repeater delay and reported
/// under `violations`.
///
/// Nets sharing a class are negotiated together (one `route_all` per
/// class, in first-appearance order); all nets in a group see every other
/// group's committed geometry. Because a single labelled workspace spans
/// the whole call, two nets squeezed through one window now negotiate or
/// detour instead of silently merging — the single-net `route_net` bridge
/// path rebuilt the workspace per call, losing the labels that make
/// electrical clearance enforceable.
///
/// Response shape: `{"routes": [{"label", "class", "cells", "delay_rt",
/// "path": [[x,y,z], ...]}], "notes": [...], "violations": [...]}`.
pub fn route_all_schematic(
    schem: &mut UniversalSchematic,
    nets_json: &str,
) -> Result<String, String> {
    use crate::io_contract::routing::{NetClassRule, RoutingRegion as ContractRegion};
    use serde_json::json;
    use std::collections::HashMap;

    let parsed: serde_json::Value =
        serde_json::from_str(nets_json).map_err(|e| format!("route_all JSON: {e}"))?;

    // Nets.
    struct NetIn {
        label: String,
        src: Pos,
        dsts: Vec<Pos>,
        friendly: Vec<String>,
        class: Option<String>,
    }
    let nets_in: Vec<NetIn> = parsed["nets"]
        .as_array()
        .ok_or("route_all needs a `nets` array")?
        .iter()
        .map(|n| {
            let label = n["label"]
                .as_str()
                .ok_or("net needs a `label`")?
                .to_string();
            let src = parse_pos(&n["src"], "src")?;
            let dsts = n["dsts"]
                .as_array()
                .ok_or("net needs a `dsts` array")?
                .iter()
                .map(|d| parse_pos(d, "dst"))
                .collect::<Result<Vec<Pos>, String>>()?;
            if dsts.is_empty() {
                return Err(format!("net `{label}` has no destinations"));
            }
            let friendly = n["friendly"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let class = n["class"].as_str().map(str::to_string);
            Ok(NetIn {
                label,
                src,
                dsts,
                friendly,
                class,
            })
        })
        .collect::<Result<_, String>>()?;

    // Per-class rules (io_contract's NetClassRule, deserialized as-is).
    let classes: HashMap<String, NetClassRule> = match &parsed["classes"] {
        serde_json::Value::Null => HashMap::new(),
        v => serde_json::from_value(v.clone()).map_err(|e| format!("classes: {e}"))?,
    };

    // Named routing zones from the schematic's tagged DefinitionRegions.
    let zones = ContractRegion::collect(schem.definition_regions.values());
    let to_engine_region = |zone: &ContractRegion| RoutingRegion {
        include: zone
            .include
            .iter()
            .map(|b| {
                Aabb::new(
                    Pos::new(b.min.0, b.min.1, b.min.2),
                    Pos::new(b.max.0, b.max.1, b.max.2),
                )
            })
            .collect(),
        exclude: zone
            .exclude
            .iter()
            .map(|b| {
                Aabb::new(
                    Pos::new(b.min.0, b.min.1, b.min.2),
                    Pos::new(b.max.0, b.max.1, b.max.2),
                )
            })
            .collect(),
    };

    // Base router configuration.
    let mut base = RedstoneRouter::new();
    if let Some(b) = parsed.get("bounds").filter(|v| !v.is_null()) {
        let arr = b.as_array().filter(|a| a.len() == 2).ok_or("bounds must be [[x,y,z],[x,y,z]]")?;
        base.bounds = Some(Aabb::new(
            parse_pos(&arr[0], "bounds min")?,
            parse_pos(&arr[1], "bounds max")?,
        ));
    }
    if let Some(b) = parsed.get("budget").filter(|v| !v.is_null()) {
        if let Some(r) = b["refresh"].as_u64() {
            base.budget.refresh = r as u32;
        }
        if let Some(s) = b["stair_cap"].as_u64() {
            base.budget.stair_cap = s as u8;
        }
    }
    if let Some(c) = parsed.get("congestion").filter(|v| !v.is_null()) {
        if let Some(r) = c["max_rounds"].as_u64() {
            base.congestion.max_rounds = r as usize;
        }
        if let Some(h) = c["history_increment"].as_u64() {
            base.congestion.history_increment = h as u32;
        }
        if let Some(p) = c["present_penalty"].as_u64() {
            base.congestion.present_penalty = p as u32;
        }
    }

    // Group nets by class, preserving first-appearance order.
    let mut order: Vec<Option<String>> = Vec::new();
    let mut groups: HashMap<Option<String>, Vec<usize>> = HashMap::new();
    for (i, n) in nets_in.iter().enumerate() {
        let key = n.class.clone();
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(i);
    }

    let mut ws = workspace_from_schematic(schem);
    let mut notes: Vec<String> = Vec::new();
    let mut routed: Vec<Option<(RouteResult, Option<String>)>> = vec![None; nets_in.len()];

    for key in &order {
        let members = &groups[key];
        let mut router = base.clone();
        if let Some(class_name) = key {
            let rule = classes.get(class_name).ok_or_else(|| {
                format!("net class `{class_name}` is not defined under `classes`")
            })?;
            if let Some(region_name) = &rule.region {
                let zone = zones.get(region_name).ok_or_else(|| {
                    format!(
                        "routing region `{region_name}` is not tagged on any of the \
                         schematic's DefinitionRegions"
                    )
                })?;
                router.region = Some(to_engine_region(zone));
            }
            if let Some((y0, y1)) = rule.y_band {
                const FAR: i32 = 1 << 24;
                let band = Aabb::new(Pos::new(-FAR, y0, -FAR), Pos::new(FAR, y1, FAR));
                router.bounds = Some(match router.bounds {
                    Some(b) => intersect_aabb(b, band)
                        .map_err(|e| format!("class `{class_name}`: {e}"))?,
                    None => band,
                });
            }
            if rule.spacing != 0 {
                notes.push(format!(
                    "class `{class_name}`: `spacing` is not enforced by route_all v1"
                ));
            }
            if rule.direction_bias.is_some() {
                notes.push(format!(
                    "class `{class_name}`: `direction_bias` is not enforced by route_all v1"
                ));
            }
        }
        let group_nets: Vec<NetRoute> = members
            .iter()
            .map(|&i| NetRoute {
                src: nets_in[i].src,
                dsts: nets_in[i].dsts.clone(),
                label: nets_in[i].label.clone(),
                friendly: nets_in[i].friendly.clone(),
            })
            .collect();
        let results = router
            .route_all(&mut ws, &group_nets)
            .map_err(|e| match e {
                RouteError::Congestion { unrouted, contested } => format!(
                    "congestion did not converge: unrouted {unrouted:?}, contested {} cells",
                    contested.len()
                ),
                other => format!("{other:?}"),
            })?;
        for (&i, res) in members.iter().zip(results) {
            routed[i] = Some((res, key.clone()));
        }
    }

    apply_workspace(schem, &ws)?;

    // Report, in the caller's net order, with per-route repeater delay
    // checked against any class delay budget.
    let mut violations: Vec<serde_json::Value> = Vec::new();
    let routes: Vec<serde_json::Value> = nets_in
        .iter()
        .zip(&routed)
        .map(|(n, slot)| {
            let (res, class) = slot.as_ref().expect("every net routed");
            let delay_rt: u32 = res
                .path
                .iter()
                .filter_map(|p| ws.get(*p))
                .filter(|b| engine::blocks::is_repeater(b))
                .map(engine::blocks::repeater_delay)
                .sum();
            if let Some(max) = class
                .as_ref()
                .and_then(|c| classes.get(c))
                .and_then(|r| r.max_len_rt)
            {
                if delay_rt > max {
                    violations.push(serde_json::json!({
                        "kind": "max_len_rt",
                        "label": n.label,
                        "delay_rt": delay_rt,
                        "max_len_rt": max,
                    }));
                }
            }
            serde_json::json!({
                "label": n.label,
                "class": class,
                "cells": res.cells,
                "delay_rt": delay_rt,
                "path": res.path.iter().map(|p| serde_json::json!([p.x, p.y, p.z])).collect::<Vec<_>>(),
            })
        })
        .collect();

    Ok(json!({
        "routes": routes,
        "notes": notes,
        "violations": violations,
    })
    .to_string())
}

/// Parse an intent netlist from JSON:
/// `{"nets": [{"name": "a", "terminals": [[x, y, z], ...]}, ...]}`.
pub fn parse_intent_nets(json: &str) -> Result<Vec<IntentNet>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("intent netlist JSON: {e}"))?;
    let nets = parsed["nets"]
        .as_array()
        .ok_or("intent netlist needs a `nets` array")?;
    nets.iter()
        .map(|n| {
            let name = n["name"]
                .as_str()
                .ok_or("net needs a `name`")?
                .to_string();
            let terminals = n["terminals"]
                .as_array()
                .ok_or("net needs a `terminals` array")?
                .iter()
                .map(|t| {
                    let c = t.as_array().filter(|c| c.len() == 3).ok_or("terminal must be [x, y, z]")?;
                    let g = |i: usize| -> Result<i32, String> {
                        c[i].as_i64()
                            .map(|v| v as i32)
                            .ok_or_else(|| "terminal coordinate must be an integer".to_string())
                    };
                    Ok(Pos::new(g(0)?, g(1)?, g(2)?))
                })
                .collect::<Result<Vec<Pos>, String>>()?;
            Ok(IntentNet { name, terminals })
        })
        .collect()
}

/// LVS v1 over a schematic: compare an intended netlist (JSON, see
/// [`parse_intent_nets`]) against the conduction netlist extracted
/// statically from the block states (dust adjacency including cut
/// diagonals, plus repeater/comparator/torch through-component edges).
///
/// Static extraction is used rather than the MCHPRS `export_graph` so LVS
/// works under the `routing` feature alone and sees exactly the geometry
/// DRC sees; the compile-graph is a valid alternative extractor when the
/// `simulation` feature is present.
pub fn lvs_schematic(schem: &UniversalSchematic, intent_json: &str) -> Result<LvsReport, String> {
    let intent = parse_intent_nets(intent_json)?;
    let ws = workspace_from_schematic(schem);
    Ok(lvs(ws.cells(), &intent))
}

/// Serialize an [`LvsReport`] to JSON (`matched`, `opens`, `shorts`,
/// `cycles`, `clean`).
pub fn lvs_report_json(report: &LvsReport) -> String {
    use serde_json::json;
    let pos = |p: &Pos| json!([p.x, p.y, p.z]);
    json!({
        "clean": report.clean(),
        "matched": report.matched,
        "opens": report.opens.iter().map(|o| json!({
            "net": o.net,
            "fragments": o.fragments.iter()
                .map(|f| f.iter().map(pos).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "shorts": report.shorts.iter().map(|s| json!({
            "net_a": s.net_a,
            "net_b": s.net_b,
            "at_a": pos(&s.at_a),
            "at_b": pos(&s.at_b),
        })).collect::<Vec<_>>(),
        "cycles": report.cycles.iter()
            .map(|c| c.iter().map(pos).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
    })
    .to_string()
}

/// Run the STA upper bound over a schematic plus a gate netlist.
pub fn sta_schematic(
    schem: &UniversalSchematic,
    inputs: &[String],
    gates: &[sta::Gate],
) -> Result<sta::TimingReport, sta::StaError> {
    let ws = workspace_from_schematic(schem);
    sta::sta(&ws, inputs, gates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_route_over_schematic() {
        let mut schem = UniversalSchematic::new("routing_test".to_string());
        // A small obstacle the route must respect.
        for z in -1..=1 {
            schem
                .set_block_from_string(3, 1, z, "minecraft:stone")
                .unwrap();
        }
        let res = route_net(&mut schem, (0, 1, 0), (6, 1, 0), "n").unwrap();
        assert_eq!(res.path.first().copied(), Some(Pos::new(0, 1, 0)));
        assert_eq!(res.path.last().copied(), Some(Pos::new(6, 1, 0)));
        // The dust and supports landed in the schematic.
        let end = schem.get_block(6, 1, 0).expect("dst block");
        assert!(end.name.contains("redstone_wire"));
        assert!(end.to_string().contains("power=0"), "full state expected");
        // DRC on the written-back schematic: nothing floats, no cycles.
        let vs = drc_schematic(
            &schem,
            &DrcOptions {
                aliases: vec![],
                skip_decay: true,
            },
        );
        assert!(vs.is_empty(), "{vs:?}");
    }

    #[test]
    fn lvs_matches_a_routed_net_and_catches_a_break() {
        let mut schem = UniversalSchematic::new("lvs_test".to_string());
        let res = route_net(&mut schem, (0, 1, 0), (6, 1, 0), "n").unwrap();
        let intent = r#"{"nets": [{"name": "n", "terminals": [[0, 1, 0], [6, 1, 0]]}]}"#;
        let r = lvs_schematic(&schem, intent).unwrap();
        assert_eq!(r.matched, vec!["n".to_string()], "{r:?}");
        assert!(r.clean(), "{r:?}");
        // Break the route mid-way: LVS reports the open the quiescent sim
        // would never show.
        let mid = res.path[res.path.len() / 2];
        schem
            .set_block_from_string(mid.x, mid.y, mid.z, "minecraft:air")
            .unwrap();
        let r = lvs_schematic(&schem, intent).unwrap();
        assert_eq!(r.opens.len(), 1, "{r:?}");
        assert_eq!(r.opens[0].net, "n");
        let json = lvs_report_json(&r);
        assert!(json.contains("\"clean\":false"), "{json}");
    }

    /// Regression for the known single-net-bridge limitation: two nets
    /// squeezed toward one wall window merged, because each `route_net`
    /// call rebuilt the workspace without the other net's labels.
    /// `route_all` negotiates both in one labelled workspace.
    #[test]
    fn route_all_negotiates_two_nets_through_separate_windows() {
        let mut schem = UniversalSchematic::new("windows".to_string());
        // A wall at x = 3 with two one-cell windows, at z = 0 and z = 5.
        for z in -2..=6 {
            for y in 1..=2 {
                if z == 0 || z == 5 {
                    continue;
                }
                schem
                    .set_block_from_string(3, y, z, "minecraft:stone")
                    .unwrap();
            }
        }
        // Net `a` runs straight through the z = 0 window; net `b` (two
        // lanes south) also prefers z = 0 — negotiation must push one of
        // them to z = 5 instead of shorting them through the same gap.
        let req = r#"{
            "nets": [
                {"label": "a", "src": [0, 1, 0], "dsts": [[6, 1, 0]]},
                {"label": "b", "src": [0, 1, 2], "dsts": [[6, 1, 2]]}
            ],
            "bounds": [[-1, 1, -2], [8, 2, 6]]
        }"#;
        let report = route_all_schematic(&mut schem, req).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        let routes = parsed["routes"].as_array().unwrap();
        assert_eq!(routes.len(), 2);
        // Each route crosses the wall plane exactly through a window, and
        // not the same one.
        let crossing = |route: &serde_json::Value| -> Vec<i64> {
            route["path"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p[0].as_i64() == Some(3))
                .map(|p| p[2].as_i64().unwrap())
                .collect()
        };
        let za = crossing(&routes[0]);
        let zb = crossing(&routes[1]);
        assert!(!za.is_empty() && !zb.is_empty(), "{report}");
        for z in za.iter().chain(&zb) {
            assert!(*z == 0 || *z == 5, "crossed outside a window: {report}");
        }
        assert!(
            za.iter().all(|z| zb.iter().all(|w| w != z)),
            "both nets took the same window: {report}"
        );
        // The routed schematic passes LVS: both nets connected, no merge.
        let intent = r#"{"nets": [
            {"name": "a", "terminals": [[0, 1, 0], [6, 1, 0]]},
            {"name": "b", "terminals": [[0, 1, 2], [6, 1, 2]]}
        ]}"#;
        let lvs = lvs_schematic(&schem, intent).unwrap();
        assert!(lvs.clean(), "{lvs:?}");
        assert_eq!(lvs.matched.len(), 2, "{lvs:?}");
    }

    #[test]
    fn route_all_honours_class_region_and_y_band() {
        use crate::io_contract::routing::{RouteZoneMode, RoutingRegion as ContractRegion};

        let mut schem = UniversalSchematic::new("classes".to_string());
        // A wall at x = 3 blocking z in -1..=1: the route must detour.
        for z in -1..=1 {
            for y in 1..=2 {
                schem
                    .set_block_from_string(3, y, z, "minecraft:stone")
                    .unwrap();
            }
        }
        // Tag a keepout zone over the whole south side (z >= 2), so the
        // detour is forced north, and confine the class to y = 1.
        let mut dr = crate::definition_region::DefinitionRegion::from_bounds(
            (-2, 0, 2),
            (9, 3, 9),
        );
        ContractRegion::tag(&mut dr, "north_only", RouteZoneMode::Exclude);
        schem.definition_regions.insert("north_only".to_string(), dr);

        let req = r#"{
            "nets": [{"label": "n", "src": [0, 1, 0], "dsts": [[6, 1, 0]],
                       "class": "c"}],
            "classes": {"c": {"region": "north_only", "y_band": [1, 1],
                               "max_len_rt": 8}},
            "bounds": [[-1, 0, -6], [8, 2, 6]]
        }"#;
        let report = route_all_schematic(&mut schem, req).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        let path = parsed["routes"][0]["path"].as_array().unwrap();
        for p in path {
            let (y, z) = (p[1].as_i64().unwrap(), p[2].as_i64().unwrap());
            assert_eq!(y, 1, "y_band violated: {report}");
            assert!(z < 2, "excluded region entered: {report}");
        }
        assert!(parsed["violations"].as_array().unwrap().is_empty(), "{report}");
        // An unknown region name is an error, not a silent no-op.
        let bad = r#"{
            "nets": [{"label": "n2", "src": [0, 1, -4], "dsts": [[6, 1, -4]],
                       "class": "c"}],
            "classes": {"c": {"region": "nope"}}
        }"#;
        let err = route_all_schematic(&mut schem, bad).unwrap_err();
        assert!(err.contains("nope"), "{err}");
    }
}
