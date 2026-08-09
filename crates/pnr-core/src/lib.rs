//! Fabric-agnostic place-and-route algorithms.
//!
//! This crate is the algorithmic half of the redstone EDA stack: everything
//! here is generic over [`fabric::Fabric`] (the seam) and knows nothing about
//! Minecraft. The concrete redstone fabric lives in `nucleation-routing`.
//!
//! Contents:
//! - [`grid`]: integer 3-D positions and axis-aligned boxes.
//! - [`fabric`]: the `Fabric` trait — `moves()`, `legal()`, `cost()`,
//!   `budget()` — plus the search-state shape (position + fabric memory,
//!   generalizing the `(pos, stair_count, prev_stair_dir)` pattern the Python
//!   prototype converged on).
//! - [`astar`]: weighted grid A* with pluggable moves, route-to-net targets
//!   and external (history) costs.
//! - [`congestion`]: PathFinder-style negotiated congestion `route_all` —
//!   rip-up and reroute with escalating history costs on contested cells.
//! - [`color`]: greedy interval colouring (track assignment).
//! - [`unionfind`] / [`netcheck`]: union-find and the generic
//!   "no two labels share a component" net checker framework.
//! - [`anneal`]: seeded simulated-annealing engine over Move/Cost/Feasibility
//!   traits.
//! - [`sta`]: static timing analysis over a generic delay DAG.
//!
//! Determinism is a hard requirement: all iteration that affects results uses
//! ordered containers or explicit tie-breaking, and the annealer's RNG is a
//! seeded SplitMix64 implemented here. The crate has no dependencies and
//! builds for `wasm32-unknown-unknown`.

pub mod anneal;
pub mod astar;
pub mod color;
pub mod congestion;
pub mod fabric;
pub mod grid;
pub mod netcheck;
pub mod sta;
pub mod unionfind;

pub use astar::{route, PathStep, RouteRequest};
pub use congestion::{route_all, CongestionOpts, NetReq, RouteAllError};
pub use fabric::{Budget, Candidate, Fabric, RouteCtx, State};
pub use grid::{Aabb, Pos};
