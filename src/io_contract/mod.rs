//! IO contract layer — the shared core between the typed executor, the
//! Insign authoring front door, and the (future) routing crate.
//!
//! Three existing representations of "a circuit with named IO" are layers
//! over ONE contract:
//!
//! ```text
//! IoContract           (IoLayout: name -> IoType + ordered positions)
//!   ^ compiled from      Insign signs (in-world authoring)
//!   ^ consumed by        TypedCircuitExecutor (typed set/read/execute)
//!   ^ embedded in        CellTemplate = Schematic + IoLayout + PhysicalContract
//! ```
//!
//! Everything in this module is plain data + serde (JSON): no simulation,
//! no filesystem, wasm-safe. A saved cell = a schematic file + a
//! [`CellContract`] file; a cell library is a directory of them.

pub mod bus;
pub mod io_layout_builder;
pub mod io_mapping;
pub mod io_type;
pub mod layout_function;
pub mod physical;
pub mod routing;
pub mod sort_strategy;
pub mod value;

pub use bus::{BusEncoding, BusPort, BusSpec, Pitch};
pub use io_layout_builder::{IoLayout, IoLayoutBuilder};
pub use io_mapping::IoMapping;
pub use io_type::IoType;
pub use layout_function::LayoutFunction;
pub use physical::{
    CellContract, EdgeContract, EdgeWindow, Face, InitialState, PhysicalContract, PortDirection,
    PortPairDelay,
};
pub use routing::{NetClassRule, RouteZoneMode, RoutingRegion};
pub use sort_strategy::SortStrategy;
pub use value::Value;
