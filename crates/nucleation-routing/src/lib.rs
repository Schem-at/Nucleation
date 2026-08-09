//! The redstone fabric: Minecraft-specific place-and-route over `pnr-core`.
//!
//! Ported from the Python prototypes in `redstone-eda/` (`router.py`,
//! `nets.py`, `audit.py`, `timing.py`, `cells.py`), which are the spec AND
//! the test vectors — every design rule in [`fabric`] was paid for by a real
//! in-sim bug, and each carries a regression test naming its provenance.
//!
//! - [`blocks`]: block-state strings and property parsing (the fully
//!   spelled-out dust state, repeaters whose `facing` names the INPUT side).
//! - [`workspace`]: transactional voxel occupancy + net labels (claim,
//!   commit, roll back — rip-up needs cheap undo).
//! - [`nets`]: dust electrical adjacency including cut diagonals, and the
//!   short checker (exact port of `nets.py`).
//! - [`audit`]: structural support audit (port of `audit.py`).
//! - [`budget`]: the signal budget (decay, refresh interval, stair caps).
//! - [`via`]: vertical via templates, seeded with the verified torch ladder.
//! - [`fabric`]: the `pnr_core::Fabric` implementation — every rule from the
//!   design table.
//! - [`router`]: route / route_all + path emission (dust, supports,
//!   repeater insertion, ladder climbs).
//! - [`cell`]: `Fragment` / `CellTemplate` stamping and edge contracts.
//! - [`bus`]: `BusSpec` / `BusPort` and abutment compatibility.
//! - [`pivot`]: the verified form-pivot tiles (v2h / h2v / flat90) and the
//!   bus planner that stamps them implicitly when endpoint forms differ.
//! - [`region`]: include/exclude routing regions.
//! - [`drc`]: shorts, support, decay, and repeater-cycle detection.
//! - [`lvs`]: intended-netlist vs extracted-conduction comparison (opens,
//!   shorts, repeater rings).
//! - [`sta`]: static timing (torch 1 rt, repeater delay, dust free) over
//!   `pnr-core`'s delay DAG.

pub mod audit;
pub mod blocks;
pub mod budget;
pub mod bus;
pub mod cell;
pub mod drc;
pub mod fabric;
pub mod lvs;
pub mod nets;
pub mod pivot;
pub mod region;
pub mod router;
pub mod sta;
pub mod via;
pub mod workspace;

pub use budget::SignalBudget;
pub use bus::{Axis, BusPort, BusSpec, Encoding, Face, InOut};
pub use cell::{CellTemplate, EdgeContract, Fragment};
pub use drc::{drc, DrcOptions, Violation};
pub use fabric::NetSpec;
pub use lvs::{lvs, IntentNet, LvsOpen, LvsReport, LvsShort};
pub use pivot::{pivot_for, pivot_fragment, route_bus, BusForm, BusRouteReport, PivotKind};
pub use pnr_core::{Aabb, Pos};
pub use region::RoutingRegion;
pub use router::{NetRoute, RedstoneRouter, RouteError, RouteResult};
pub use via::{ViaRegistry, ViaTemplate};
pub use workspace::Workspace;
