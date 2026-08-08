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
}
