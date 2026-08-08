//! Redstone EDA: the minimal `routing`-feature bridge surface.
//!
//! New surface — no old `ffi/` counterpart. Thin wrappers over
//! `crate::routing` (which itself re-exports `nucleation-routing`): route a
//! net through a schematic, route many with negotiated congestion and
//! per-net-class rules (`route_all`), run DRC, run LVS, run the STA upper
//! bound. Structured results cross as JSON strings (PORTING.md rule 9).
//! The full Workspace / cell-template API stays native-only until the
//! compositor needs it bridged.
//!
//! Bindings are regenerated (`tools/gen-bindings.sh`); the module compiles
//! under `--features bridge-full,routing`.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Namespacing opaque for routing entry points (static methods taking
    /// `&Schematic` explicitly, like `Autostack`).
    #[diplomat::opaque]
    pub struct Routing;

    impl Routing {
        /// Route one net from `(sx, sy, sz)` to `(dx, dy, dz)` with default
        /// rules (torch-ladder vias, stair cap 4, refresh 5) and write the
        /// emitted geometry into the schematic. Writes the routed path as a
        /// JSON array of `[x, y, z]` cells.
        #[allow(clippy::too_many_arguments)]
        pub fn route_net(
            schematic: &mut Schematic,
            sx: i32,
            sy: i32,
            sz: i32,
            dx: i32,
            dy: i32,
            dz: i32,
            label: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let label =
                core::str::from_utf8(label).map_err(|_| NucleationError::InvalidArgument)?;
            let res = crate::routing::route_net(
                &mut schematic.0,
                (sx, sy, sz),
                (dx, dy, dz),
                label,
            )
            .map_err(|_| NucleationError::InvalidArgument)?;
            let cells: Vec<String> = res
                .path
                .iter()
                .map(|p| format!("[{},{},{}]", p.x, p.y, p.z))
                .collect();
            let _ = write!(out, "[{}]", cells.join(","));
            Ok(())
        }

        /// Route every net in `nets_json` with negotiated congestion
        /// (pnr-core PathFinder) in one labelled workspace, write the
        /// geometry into the schematic, and write the JSON report
        /// (`routes` with per-net `path`/`delay_rt`, `notes`,
        /// `violations`). Supports per-net-class rule overrides
        /// (`classes`: io_contract `NetClassRule`s, with `region`
        /// resolving named route zones tagged on the schematic's
        /// DefinitionRegions), plus `bounds`, `budget` and `congestion`
        /// options — see `crate::routing::route_all_schematic` for the
        /// exact request shape.
        pub fn route_all(
            schematic: &mut Schematic,
            nets_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let json =
                core::str::from_utf8(nets_json).map_err(|_| NucleationError::InvalidArgument)?;
            let report =
                crate::routing::route_all_schematic(&mut schematic.0, json).map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })?;
            let _ = write!(out, "{report}");
            Ok(())
        }

        /// LVS v1: compare an intended netlist (`{"nets": [{"name",
        /// "terminals": [[x,y,z], ...]}]}`) against the conduction
        /// netlist extracted statically from the schematic (dust
        /// adjacency incl. cut diagonals plus repeater/comparator/torch
        /// through-component edges). Writes `{"clean", "matched",
        /// "opens", "shorts", "cycles"}`.
        pub fn lvs(
            schematic: &Schematic,
            intent_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let json =
                core::str::from_utf8(intent_json).map_err(|_| NucleationError::InvalidArgument)?;
            let report = crate::routing::lvs_schematic(&schematic.0, json).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            let _ = write!(out, "{}", crate::routing::lvs_report_json(&report));
            Ok(())
        }

        /// Run design-rule checks (support audit, repeater-cycle detection,
        /// optional decay) over the schematic. Writes a JSON array; each
        /// element has `kind` plus violation-specific fields. Label-aware
        /// short checking needs a labelled workspace and stays native.
        pub fn drc(
            schematic: &Schematic,
            check_decay: bool,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            use crate::routing::Violation;
            let opts = crate::routing::DrcOptions {
                aliases: vec![],
                skip_decay: !check_decay,
            };
            let vs = crate::routing::drc_schematic(&schematic.0, &opts);
            let items: Vec<String> = vs
                .iter()
                .map(|v| match v {
                    Violation::Short {
                        label_a,
                        label_b,
                        at_a,
                        at_b,
                    } => format!(
                        "{{\"kind\":\"short\",\"label_a\":{label_a:?},\"label_b\":{label_b:?},\"at_a\":[{},{},{}],\"at_b\":[{},{},{}]}}",
                        at_a.x, at_a.y, at_a.z, at_b.x, at_b.y, at_b.z
                    ),
                    Violation::Floating { at, block } => format!(
                        "{{\"kind\":\"floating\",\"at\":[{},{},{}],\"block\":{block:?}}}",
                        at.x, at.y, at.z
                    ),
                    Violation::UnattachedWallTorch { at, anchor } => format!(
                        "{{\"kind\":\"unattached_wall_torch\",\"at\":[{},{},{}],\"anchor\":[{},{},{}]}}",
                        at.x, at.y, at.z, anchor.x, anchor.y, anchor.z
                    ),
                    Violation::RepeaterCycle { diodes } => {
                        let ds: Vec<String> = diodes
                            .iter()
                            .map(|p| format!("[{},{},{}]", p.x, p.y, p.z))
                            .collect();
                        format!("{{\"kind\":\"repeater_cycle\",\"diodes\":[{}]}}", ds.join(","))
                    }
                    Violation::PowerStarved { at, distance } => format!(
                        "{{\"kind\":\"power_starved\",\"at\":[{},{},{}],\"distance\":{distance}}}",
                        at.x, at.y, at.z
                    ),
                })
                .collect();
            let _ = write!(out, "[{}]", items.join(","));
            Ok(())
        }

        /// Static timing over the schematic plus a gate netlist given as
        /// JSON: `{"inputs": ["a", ...], "gates": [{"out": "y",
        /// "ins": ["a", "b"], "delay_rt": 2}, ...]}`. Writes
        /// `{"arrival_rt": {sig: rt}, "critical": [sig, ...]}`.
        pub fn sta(
            schematic: &Schematic,
            netlist_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let json = core::str::from_utf8(netlist_json)
                .map_err(|_| NucleationError::InvalidArgument)?;
            let parsed: serde_json::Value =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            let inputs: Vec<String> = parsed["inputs"]
                .as_array()
                .ok_or(NucleationError::InvalidArgument)?
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            let gates: Vec<crate::routing::sta::Gate> = parsed["gates"]
                .as_array()
                .ok_or(NucleationError::InvalidArgument)?
                .iter()
                .map(|g| {
                    Some(crate::routing::sta::Gate {
                        out: g["out"].as_str()?.to_string(),
                        ins: g["ins"]
                            .as_array()?
                            .iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect(),
                        delay_rt: g["delay_rt"].as_u64().unwrap_or(1) as u32,
                    })
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(NucleationError::InvalidArgument)?;
            let report = crate::routing::sta_schematic(&schematic.0, &inputs, &gates)
                .map_err(|_| NucleationError::InvalidArgument)?;
            let arrivals: Vec<String> = report
                .arrival_rt
                .iter()
                .map(|(k, v)| format!("{k:?}:{v}"))
                .collect();
            let critical: Vec<String> =
                report.critical.iter().map(|s| format!("{s:?}")).collect();
            let _ = write!(
                out,
                "{{\"arrival_rt\":{{{}}},\"critical\":[{}]}}",
                arrivals.join(","),
                critical.join(",")
            );
            Ok(())
        }
    }
}
