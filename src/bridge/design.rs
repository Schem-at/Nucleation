//! The Design compositor: the `routing` feature's composition surface.
//!
//! Thin wrapper over [`crate::design::Design`] (see
//! `redstone-eda/DESIGN_SPEC.md`): declare typed ports over endpoint
//! hardware, route buses with implicit dip-under crossings, flatten to a
//! self-describing schematic, check (DRC + LVS) and bake (mc-tick).
//! Structured results cross as JSON strings (PORTING.md rule 9);
//! unroutability is a bus STATE (`"failed: ..."`), never an error return.
//!
//! Bindings are regenerated (`tools/gen-bindings.sh`); the module compiles
//! under `--features bridge-full,routing` (bake additionally wants
//! `mc-tick`).

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// A composition document: loose blocks + cell instances + bus layers
    /// over a shared coordinate space.
    #[diplomat::opaque_mut]
    pub struct Design(pub(crate) crate::design::Design);

    impl Design {
        /// An empty design.
        pub fn create(name: &DiplomatStr) -> Result<Box<Design>, NucleationError> {
            let name = utf8(name)?;
            Ok(Box::new(Design(crate::design::Design::new(name))))
        }

        /// A design whose loose block layer is a copy of `base` (endpoint
        /// hardware placed with raw `set_block`).
        pub fn for_schematic(
            name: &DiplomatStr,
            base: &Schematic,
        ) -> Result<Box<Design>, NucleationError> {
            let name = utf8(name)?;
            Ok(Box::new(Design(crate::design::Design::for_schematic(
                name,
                base.0.clone(),
            ))))
        }

        /// Register a library cell; its contract is resolved from the
        /// schematic (embedded metadata first, Insign signs as fallback)
        /// and registration fails loudly when no source defines one.
        /// Writes resolution warnings as a JSON array of strings.
        pub fn add_cell(
            &mut self,
            name: &DiplomatStr,
            cell: &Schematic,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let name = utf8(name)?;
            let warnings = self.0.add_cell(name, cell.0.clone()).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            let items: Vec<String> = warnings.iter().map(|w| format!("{w:?}")).collect();
            let _ = write!(out, "[{}]", items.join(","));
            Ok(())
        }

        /// Place an instance layer referencing a library cell. `rot_y` is
        /// in degrees, a multiple of 90.
        pub fn place(
            &mut self,
            name: &DiplomatStr,
            cell: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            rot_y: i32,
        ) -> Result<(), NucleationError> {
            let (name, cell) = (utf8(name)?, utf8(cell)?);
            self.0.place(name, cell, (x, y, z), rot_y).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })
        }

        /// Declare a drivable input port: anchor = bit-0 connection cell,
        /// step to the next bit, `width` bits of `ty` (`"uint"` or
        /// `"bool"`). The hardware is scanned (adjacent lever per bit) and
        /// validated loudly.
        #[allow(clippy::too_many_arguments)]
        pub fn declare_input(
            &mut self,
            name: &DiplomatStr,
            ax: i32,
            ay: i32,
            az: i32,
            sx: i32,
            sy: i32,
            sz: i32,
            width: u8,
            ty: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let (name, ty) = (utf8(name)?, utf8(ty)?);
            let ty = parse_ty(ty, width)?;
            self.0
                .declare_input(name, (ax, ay, az), (sx, sy, sz), width, ty)
                .map(|_| ())
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })
        }

        /// Declare a readable output port (adjacent lamp per bit); same
        /// shape as `declare_input`.
        #[allow(clippy::too_many_arguments)]
        pub fn declare_output(
            &mut self,
            name: &DiplomatStr,
            ax: i32,
            ay: i32,
            az: i32,
            sx: i32,
            sy: i32,
            sz: i32,
            width: u8,
            ty: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let (name, ty) = (utf8(name)?, utf8(ty)?);
            let ty = parse_ty(ty, width)?;
            self.0
                .declare_output(name, (ax, ay, az), (sx, sy, sz), width, ty)
                .map(|_| ())
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })
        }

        /// Declare AND realize a bus. `sinks_json` is a JSON array of port
        /// names; `gates_json` an array of `{"name", "anchor": [x,y,z],
        /// "step": [x,y,z]}` (pass `[]` for none); `style_json` an object
        /// with optional `bus_block` / `transparent_block`. Declaration
        /// errors are error returns; geometric unroutability is the
        /// written STATE: `"routed"` or `"failed: reason"` — realization
        /// is atomic, never half-routed.
        pub fn route_bus(
            &mut self,
            name: &DiplomatStr,
            driver: &DiplomatStr,
            sinks_json: &DiplomatStr,
            gates_json: &DiplomatStr,
            style_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let (name, driver) = (utf8(name)?, utf8(driver)?);
            let sinks: Vec<String> =
                serde_json::from_str(utf8(sinks_json)?).map_err(|_| NucleationError::Parse)?;
            let sink_refs: Vec<&str> = sinks.iter().map(String::as_str).collect();
            let gates = parse_gates(utf8(gates_json)?)?;
            let style = parse_style(utf8(style_json)?)?;
            let state = self
                .0
                .route_bus(name, driver, &sink_refs, gates, style)
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })?;
            let _ = write!(out, "{}", state_str(&state));
            Ok(())
        }

        /// Declare AND realize a wired-OR bus: `drivers_json` is a JSON
        /// array of port names — multiple drivers are legal ONLY through
        /// this explicit merge (`merge="or"`). Extra drivers join the
        /// trunk as diode-isolated dust-merge branches; the LVS intent
        /// stays ONE net per bit. Same shapes as `route_bus` otherwise.
        pub fn route_bus_or(
            &mut self,
            name: &DiplomatStr,
            drivers_json: &DiplomatStr,
            sinks_json: &DiplomatStr,
            gates_json: &DiplomatStr,
            style_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let name = utf8(name)?;
            let drivers: Vec<String> =
                serde_json::from_str(utf8(drivers_json)?).map_err(|_| NucleationError::Parse)?;
            let driver_refs: Vec<&str> = drivers.iter().map(String::as_str).collect();
            let sinks: Vec<String> =
                serde_json::from_str(utf8(sinks_json)?).map_err(|_| NucleationError::Parse)?;
            let sink_refs: Vec<&str> = sinks.iter().map(String::as_str).collect();
            let gates = parse_gates(utf8(gates_json)?)?;
            let style = parse_style(utf8(style_json)?)?;
            let state = self
                .0
                .route_bus_or(name, &driver_refs, &sink_refs, gates, style)
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })?;
            let _ = write!(out, "{}", state_str(&state));
            Ok(())
        }

        /// Edit the loose block layer: plain `set_block` on the base
        /// schematic (participates in occupancy and flatten).
        pub fn set_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            self.0.set_block((x, y, z), utf8(block)?).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })
        }

        /// Drag an instance layer to a new position/rotation. The move
        /// itself ALWAYS succeeds (the document's truth); the affected bus
        /// set — fragments intersecting the old or new footprint +
        /// influence halo, plus every already-failed bus — is ripped and
        /// co-rerouted deterministically with bounded retry rounds.
        /// Writes `{"rerouted": [...], "failed": {name: reason}}`.
        pub fn move_instance(
            &mut self,
            name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            rot_y: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let report = self
                .0
                .move_instance(utf8(name)?, (x, y, z), rot_y)
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })?;
            let _ = write!(out, "{}", report.to_json());
            Ok(())
        }

        /// Add a gate to an existing bus (splitting the segment it lands
        /// in) and re-realize it. Writes the resulting bus state.
        #[allow(clippy::too_many_arguments)]
        pub fn add_gate(
            &mut self,
            bus: &DiplomatStr,
            gate: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            sx: i32,
            sy: i32,
            sz: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let state = self
                .0
                .add_gate(utf8(bus)?, utf8(gate)?, (x, y, z), (sx, sy, sz))
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })?;
            let _ = write!(out, "{}", state_str(&state));
            Ok(())
        }

        /// Drag a gate: the anchor moves unconditionally, then EXACTLY the
        /// two adjacent segments are ripped and rerouted atomically. An
        /// unroutable move leaves the bus `failed: reason` — visible,
        /// never half-routed. Writes `{"state": "...",
        /// "rerouted_segments": n}`.
        pub fn move_gate(
            &mut self,
            bus: &DiplomatStr,
            gate: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let report = self
                .0
                .move_gate(utf8(bus)?, utf8(gate)?, (x, y, z))
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::InvalidArgument
                })?;
            let _ = write!(
                out,
                "{{\"state\":{:?},\"rerouted_segments\":{}}}",
                state_str(&report.state),
                report.rerouted_segments
            );
            Ok(())
        }

        /// Attach a net-class discipline to a bus (JSON `NetClassRule`:
        /// optional `max_len_rt` delay budget, `y_band` layer band, …);
        /// `check()` enforces it.
        pub fn set_bus_rule(
            &mut self,
            bus: &DiplomatStr,
            rule_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let rule: crate::io_contract::NetClassRule =
                serde_json::from_str(utf8(rule_json)?).map_err(|_| NucleationError::Parse)?;
            self.0.set_bus_rule(utf8(bus)?, rule).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::NotFound
            })
        }

        /// Per-bus skew from the routed fragment: writes
        /// `{"per_bit_rt": [...], "skew_rt": n, "max_rt": n}`.
        pub fn bus_skew(
            &self,
            name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let json = self
                .0
                .bus_skew_json(utf8(name)?)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// The lifecycle state of a bus: `"intended"`, `"routed"` or
        /// `"failed: reason"`.
        pub fn bus_state(
            &self,
            name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let name = utf8(name)?;
            let state = self.0.bus_state(name).ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", state_str(state));
            Ok(())
        }

        /// Rip a bus: clear its fragment, back to `intended`.
        pub fn rip(&mut self, name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0.rip(utf8(name)?).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::NotFound
            })
        }

        /// Collapse the layer stack into ONE self-describing schematic:
        /// named regions per layer (`inst:x`, `bus:y`) and the merged
        /// contract embedded in the metadata — itself placeable as a cell.
        pub fn flatten(&self) -> Result<Box<Schematic>, NucleationError> {
            let flat = self.0.flatten().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            Ok(Box::new(Schematic(flat)))
        }

        /// DRC + LVS over the flattened artifact. Writes `{"clean",
        /// "drc": [...], "lvs": {...}, "buses": {...}}`.
        pub fn check(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let report = self.0.check().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            let _ = write!(out, "{}", report.json);
            Ok(())
        }

        /// Settle the flattened artifact in the vanilla-accurate tick
        /// engine and return it with every settled state written back and
        /// `InitialState::Baked` stamped into the embedded contract (needs
        /// the `mc-tick` feature, else errors).
        pub fn bake(&self, budget: u32) -> Result<Box<Schematic>, NucleationError> {
            #[cfg(all(feature = "simulation", feature = "mc-tick"))]
            {
                let baked = self.0.bake(budget).map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::Simulation
                })?;
                Ok(Box::new(Schematic(baked)))
            }
            #[cfg(not(all(feature = "simulation", feature = "mc-tick")))]
            {
                let _ = budget;
                crate::bridge::set_last_error_detail(
                    "bake needs a simulator: rebuild with the `simulation` and `mc-tick` features",
                );
                Err(NucleationError::Simulation)
            }
        }

        /// Serialize the FULL design document to `.nucm` project-tier
        /// bytes (magic `NUCM`): cells deduped by content hash, instance
        /// transforms, ports with scanned hardware, every bus layer with
        /// its fragment, runs and `intended`/`routed`/`failed: reason`
        /// state, and the loose base layer. Base64 across the bridge.
        pub fn to_nucm_b64(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let bytes = self.0.to_nucm_bytes().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Serialize
            })?;
            let _ = write!(out, "{}", crate::bridge::schematic::b64(&bytes));
            Ok(())
        }

        /// Reopen a `.nucm` design document from raw bytes. The reloaded
        /// design is the same model mid-edit: rerouting works.
        pub fn from_nucm(data: &[u8]) -> Result<Box<Design>, NucleationError> {
            let d = crate::design::Design::from_nucm_bytes(data).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Parse
            })?;
            Ok(Box::new(Design(d)))
        }

        /// Save the `.nucm` project document to a file. Not available in
        /// JS: the WASM build has no filesystem — use `to_nucm_b64`.
        #[diplomat::attr(js, disable)]
        pub fn save_nucm(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            let bytes = self.0.to_nucm_bytes().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Serialize
            })?;
            std::fs::write(utf8(path)?, bytes).map_err(|_| NucleationError::Io)
        }

        /// Load a `.nucm` project document from a file. Not available in
        /// JS — read the bytes yourself and use `from_nucm`.
        #[diplomat::attr(js, disable)]
        pub fn load_nucm(path: &DiplomatStr) -> Result<Box<Design>, NucleationError> {
            let bytes = std::fs::read(utf8(path)?).map_err(|_| NucleationError::Io)?;
            Self::from_nucm(&bytes)
        }

        /// Export the design as a LAYERED `.litematic` (interchange tier):
        /// one named region per layer (`inst:{name}`, `bus:{name}`, loose
        /// base) plus the design manifest as a root-level
        /// `NucleationDesign` tag. Opens in Litematica as a plain
        /// multi-region litematic; reimports as a design whose cell
        /// references have degraded to embedded copies. Base64 across the
        /// bridge.
        pub fn to_litematic_b64(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let bytes = self.0.to_litematic_layered_bytes().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Serialize
            })?;
            let _ = write!(out, "{}", crate::bridge::schematic::b64(&bytes));
            Ok(())
        }

        /// Import a layered `.litematic` (with a `NucleationDesign`
        /// manifest) from raw bytes; a plain litematic errors loudly —
        /// open those with `Schematic.from_litematic`.
        pub fn from_litematic(data: &[u8]) -> Result<Box<Design>, NucleationError> {
            let d = crate::design::Design::from_litematic_layered_bytes(data).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Parse
            })?;
            Ok(Box::new(Design(d)))
        }

        /// Export the layered `.litematic` to a file. Not available in JS
        /// — use `to_litematic_b64`.
        #[diplomat::attr(js, disable)]
        pub fn export_litematic(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            let bytes = self.0.to_litematic_layered_bytes().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Serialize
            })?;
            std::fs::write(utf8(path)?, bytes).map_err(|_| NucleationError::Io)
        }

        /// Import a layered `.litematic` from a file. Not available in JS
        /// — read the bytes yourself and use `from_litematic`.
        #[diplomat::attr(js, disable)]
        pub fn import_litematic(path: &DiplomatStr) -> Result<Box<Design>, NucleationError> {
            let bytes = std::fs::read(utf8(path)?).map_err(|_| NucleationError::Io)?;
            Self::from_litematic(&bytes)
        }
    }

    fn utf8(s: &DiplomatStr) -> Result<&str, NucleationError> {
        core::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
    }

    fn parse_ty(ty: &str, width: u8) -> Result<crate::io_contract::IoType, NucleationError> {
        use crate::io_contract::IoType;
        match ty.to_ascii_lowercase().as_str() {
            "bool" | "boolean" if width == 1 => Ok(IoType::Boolean),
            "uint" | "unsigned" | "u" => Ok(IoType::UnsignedInt {
                bits: width as usize,
            }),
            "int" | "signed" | "i" => Ok(IoType::SignedInt {
                bits: width as usize,
            }),
            other => {
                crate::bridge::set_last_error_detail(format!(
                    "unsupported port type `{other}` (width {width}); use uint, int or bool"
                ));
                Err(NucleationError::InvalidArgument)
            }
        }
    }

    fn parse_gates(json: &str) -> Result<Vec<crate::design::Gate>, NucleationError> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
        let arr = v.as_array().ok_or(NucleationError::Parse)?;
        let mut gates = Vec::new();
        for g in arr {
            let name = g["name"].as_str().ok_or(NucleationError::Parse)?.to_string();
            let p3 = |key: &str| -> Result<(i32, i32, i32), NucleationError> {
                let a = g[key].as_array().filter(|a| a.len() == 3).ok_or(NucleationError::Parse)?;
                let c = |i: usize| a[i].as_i64().map(|v| v as i32).ok_or(NucleationError::Parse);
                Ok((c(0)?, c(1)?, c(2)?))
            };
            gates.push(crate::design::Gate {
                name,
                anchor: p3("anchor")?,
                step: p3("step")?,
            });
        }
        Ok(gates)
    }

    fn parse_style(json: &str) -> Result<crate::design::BusStyle, NucleationError> {
        let mut style = crate::design::BusStyle::default();
        if json.trim().is_empty() {
            return Ok(style);
        }
        let v: serde_json::Value = serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
        if let Some(b) = v.get("bus_block").and_then(|b| b.as_str()) {
            style.bus_block = b.to_string();
        }
        if let Some(b) = v.get("transparent_block").and_then(|b| b.as_str()) {
            style.transparent_block = b.to_string();
        }
        Ok(style)
    }

    fn state_str(state: &crate::design::BusState) -> String {
        match state {
            crate::design::BusState::Intended => "intended".to_string(),
            crate::design::BusState::Routed => "routed".to_string(),
            crate::design::BusState::Failed(reason) => format!("failed: {reason}"),
        }
    }
}
