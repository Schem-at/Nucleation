//! Route finding + path emission (port of `router.py`'s `Router`).
//!
//! `route` is single-net A* with the design rules; `route_all` is the
//! negotiated-congestion entry point (the prototype's hand-tuned net
//! ordering took three sessions of whack-a-mole; negotiation replaces it).
//! Emission lays dust, supports, repeaters (every `refresh` straight cells
//! — stairs and climbs refresh or preserve level by construction), and
//! torch-ladder climbs, transactionally.

use crate::blocks::{self, DUST, TORCH};
use crate::budget::SignalBudget;
use crate::fabric::{NetSpec, RMove, RedstoneFabric};
use crate::region::RoutingRegion;
use crate::via::ViaRegistry;
use crate::workspace::{Collision, Workspace};
use pnr_core::astar::{route as astar_route, PathStep, RouteRequest};
use pnr_core::congestion::{self, CongestionOpts, NetReq};
use pnr_core::fabric::RouteCtx;
use pnr_core::{Aabb, Pos};
use std::collections::{BTreeMap, BTreeSet};

/// A completed route.
#[derive(Clone, Debug)]
pub struct RouteResult {
    /// Number of path cells (including source and destination).
    pub cells: usize,
    /// The routed path positions, source first.
    pub path: Vec<Pos>,
}

/// Routing failure.
#[derive(Clone, Debug)]
pub enum RouteError {
    /// No legal path within the iteration budget.
    NoPath {
        /// The net's label.
        label: String,
        /// Route source.
        src: Pos,
    },
    /// Emission hit an occupied cell (a design-rule gap; the transaction
    /// was rolled back).
    Emit(Collision),
    /// `route_all` negotiation failed.
    Congestion {
        /// Labels that could not be routed.
        unrouted: Vec<String>,
        /// Cells still contested when the round budget ran out.
        contested: Vec<Pos>,
    },
    /// A bus-level request was ill-formed (width/form mismatch no pivot
    /// tile covers).
    Bus(String),
}

/// One net for `route_all`.
#[derive(Clone, Debug)]
pub struct NetRoute {
    /// Source cell (must be on the net already, typically a port).
    pub src: Pos,
    /// Target cells; reaching any completes the route. When joining the
    /// net to itself, exclude the source's own side (see
    /// `RouteRequest::to_net_excluding` provenance).
    pub dsts: Vec<Pos>,
    /// Net label.
    pub label: String,
    /// Friendly labels (deliberately the same electrical net).
    pub friendly: Vec<String>,
}

/// The router: rules + moves + budget, applied to a workspace.
#[derive(Clone, Debug)]
pub struct RedstoneRouter {
    /// Signal budget (stair cap, refresh interval).
    pub budget: SignalBudget,
    /// Via templates available to the move generator.
    pub vias: ViaRegistry,
    /// Footprint bounds (cells must not route outside them).
    pub bounds: Option<Aabb>,
    /// Optional region constraints.
    pub region: Option<RoutingRegion>,
    /// Weighted-A* heuristic weight as `num/den` (default 13/10 = 1.3).
    pub eps: (u32, u32),
    /// A* iteration cap.
    pub max_iter: usize,
    /// Negotiation options for `route_all`.
    pub congestion: CongestionOpts,
}

impl Default for RedstoneRouter {
    fn default() -> Self {
        RedstoneRouter {
            budget: SignalBudget::default(),
            vias: ViaRegistry::default(),
            bounds: None,
            region: None,
            eps: (13, 10),
            max_iter: 1_200_000,
            congestion: CongestionOpts::default(),
        }
    }
}

impl RedstoneRouter {
    /// A router with default rules, budget and the torch-ladder via.
    pub fn new() -> Self {
        Self::default()
    }

    fn request(&self, src: Pos, dsts: BTreeSet<Pos>) -> RouteRequest {
        RouteRequest {
            src,
            dsts,
            eps_num: self.eps.0,
            eps_den: self.eps.1,
            max_iter: self.max_iter,
        }
    }

    fn spec(label: &str, friendly: &[String]) -> NetSpec {
        NetSpec {
            label: label.to_string(),
            friendly: friendly.iter().cloned().collect(),
        }
    }

    /// Route one net from `src` to `dst` and emit it. Transactional: on any
    /// failure the workspace is untouched.
    pub fn route(
        &self,
        ws: &mut Workspace,
        src: Pos,
        dst: Pos,
        label: &str,
        friendly: &[String],
    ) -> Result<RouteResult, RouteError> {
        self.route_to_net(ws, src, &[dst], label, friendly)
    }

    /// Route from `src` to ANY of `dsts` (multi-terminal joining) and emit.
    pub fn route_to_net(
        &self,
        ws: &mut Workspace,
        src: Pos,
        dsts: &[Pos],
        label: &str,
        friendly: &[String],
    ) -> Result<RouteResult, RouteError> {
        let specs = [Self::spec(label, friendly)];
        let path = {
            let fabric = self.fabric(ws, &specs);
            let req = self.request(src, dsts.iter().copied().collect());
            astar_route(&fabric, &req, &RouteCtx { net: 0 }, &|_| 0)
        }
        .ok_or_else(|| RouteError::NoPath {
            label: label.to_string(),
            src,
        })?;
        ws.begin();
        match self.emit(ws, &path, label) {
            Ok(()) => {
                ws.commit();
                Ok(RouteResult {
                    cells: path.len(),
                    path: path.iter().map(|s| s.pos).collect(),
                })
            }
            Err(c) => {
                ws.rollback();
                Err(RouteError::Emit(c))
            }
        }
    }

    /// Route all nets with negotiated congestion, then emit them in order.
    /// Electrical adjacency between tentative paths counts as a conflict
    /// (two dust runs one cell apart short without sharing a cell).
    pub fn route_all(
        &self,
        ws: &mut Workspace,
        nets: &[NetRoute],
    ) -> Result<Vec<RouteResult>, RouteError> {
        let specs: Vec<NetSpec> = nets
            .iter()
            .map(|n| Self::spec(&n.label, &n.friendly))
            .collect();
        let paths = {
            let fabric = self.fabric(ws, &specs);
            let reqs: Vec<NetReq> = nets
                .iter()
                .enumerate()
                .map(|(i, n)| NetReq {
                    net: i,
                    req: self.request(n.src, n.dsts.iter().copied().collect()),
                })
                .collect();
            congestion::route_all(&fabric, &reqs, &self.congestion, &electrical_conflicts)
        }
        .map_err(|e| RouteError::Congestion {
            unrouted: e.unrouted.iter().map(|i| nets[*i].label.clone()).collect(),
            contested: e.contested,
        })?;
        ws.begin();
        let mut out = Vec::new();
        for (i, n) in nets.iter().enumerate() {
            let path = &paths[&i];
            if let Err(c) = self.emit(ws, path, &n.label) {
                ws.rollback();
                return Err(RouteError::Emit(c));
            }
            out.push(RouteResult {
                cells: path.len(),
                path: path.iter().map(|s| s.pos).collect(),
            });
        }
        ws.commit();
        Ok(out)
    }

    fn fabric<'a>(&'a self, ws: &'a Workspace, specs: &'a [NetSpec]) -> RedstoneFabric<'a> {
        RedstoneFabric {
            ws,
            nets: specs,
            bounds: self.bounds,
            region: self.region.as_ref(),
            budget: self.budget,
            vias: &self.vias,
        }
    }

    /// Lay a found path into the workspace (port of `Router.emit`).
    fn emit(
        &self,
        ws: &mut Workspace,
        path: &[PathStep<RMove>],
        label: &str,
    ) -> Result<(), Collision> {
        let refresh = self.budget.refresh;
        let mut since: u32 = 0;
        for (i, step) in path.iter().enumerate() {
            let p = step.pos;
            if let Some(RMove::Climb { dx, dz, via }) = &step.tag {
                let t = self.vias.get(*via).clone();
                let prev = path[i - 1].pos;
                // Fresh dead-end entry dust: single neighbour behind ->
                // straight line into the base, so the base is reliably
                // weak-powered (the dead-ladder-climbs rule).
                let entry = prev.offset(*dx, 0, *dz);
                ws.stone(entry.offset(0, -1, 0), "route")?;
                ws.put(entry, DUST)?;
                ws.set_label(entry, label);
                // Verified template: base, torch, block, torch, cap.
                for k in 0..=t.torches as i32 {
                    ws.stone(Pos::new(p.x, prev.y + 2 * k, p.z), "route")?;
                    if k < t.torches as i32 {
                        ws.put(Pos::new(p.x, prev.y + 2 * k + 1, p.z), TORCH)?;
                    }
                }
                // Exit dust on the cap (strongly powered: fresh 15).
                ws.put(p, DUST)?;
                ws.set_label(p, label);
                since = 0;
                continue;
            }
            if ws.get(p).is_some_and(blocks::is_dust) {
                // Reused own-net trunk: treat as due for refresh.
                since = refresh;
                continue;
            }
            ws.stone(p.offset(0, -1, 0), "route")?;
            let prev = if i > 0 { Some(path[i - 1].pos) } else { None };
            let nxt = path.get(i + 1);
            let straight = match (prev, nxt) {
                (Some(pr), Some(nx)) => {
                    let n = nx.pos;
                    pr.y == p.y
                        && p.y == n.y
                        && ((pr.x == p.x && p.x == n.x) || (pr.z == p.z && p.z == n.z))
                        && !matches!(nx.tag, Some(RMove::Climb { .. }))
                }
                _ => false,
            };
            since += 1;
            if straight && since >= refresh {
                let pr = prev.expect("straight requires prev");
                let facing = blocks::facing_name(pr.x - p.x, pr.z - p.z)
                    .expect("collinear step is axis-aligned");
                ws.put(p, &blocks::repeater(facing, 1))?;
                since = 0;
            } else {
                ws.put(p, DUST)?;
                ws.set_label(p, label);
            }
        }
        Ok(())
    }
}

/// Conflict hook for negotiation: cells of different nets that would be
/// electrically adjacent (side, or diagonal one step up/down) if both paths
/// were emitted. Conservative: does not model lid cuts, because lids are
/// not laid until emission.
fn electrical_conflicts(footprints: &BTreeMap<usize, Vec<Pos>>) -> Vec<Pos> {
    let nets: Vec<(&usize, &Vec<Pos>)> = footprints.iter().collect();
    let mut out = Vec::new();
    for i in 0..nets.len() {
        let a_set: BTreeSet<Pos> = nets[i].1.iter().copied().collect();
        for j in i + 1..nets.len() {
            for b in nets[j].1 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    for dy in [-1i32, 0, 1] {
                        let q = b.offset(dx, dy, dz);
                        if a_set.contains(&q) {
                            out.push(*b);
                            out.push(q);
                        }
                    }
                }
            }
        }
    }
    out
}
