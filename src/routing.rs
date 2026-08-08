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
}
