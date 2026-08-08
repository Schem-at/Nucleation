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
