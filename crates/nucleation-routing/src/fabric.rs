//! The redstone `Fabric`: move generation and design-rule legality.
//!
//! Every rule below was paid for by a real in-sim bug (see the table in
//! `ROUTING_CRATE_DESIGN.md`); each has a regression test naming its
//! provenance in `tests/rules.rs`.

use crate::budget::SignalBudget;
use crate::nets;
use crate::region::RoutingRegion;
use crate::via::ViaRegistry;
use crate::workspace::Workspace;
use pnr_core::fabric::{Budget, Candidate, Fabric, RouteCtx, State};
use pnr_core::{Aabb, Pos};
use std::collections::BTreeSet;

/// The 4 horizontal step directions, in the prototype's order.
pub const H_MOVES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Per-net routing identity: the net's own label plus friendly labels the
/// route may touch or reuse (multi-terminal joins need the target net to be
/// friendly, and alias-aware checking mirrors this).
#[derive(Clone, Debug)]
pub struct NetSpec {
    /// The label routed cells are tagged with.
    pub label: String,
    /// Additional labels that are electrically the same net.
    pub friendly: BTreeSet<String>,
}

impl NetSpec {
    /// A net with no extra friendly labels.
    pub fn new(label: &str) -> Self {
        NetSpec {
            label: label.to_string(),
            friendly: BTreeSet::new(),
        }
    }

    /// The full accept-set: `friendly ∪ {label}`.
    pub fn ok_labels(&self) -> BTreeSet<String> {
        let mut ok = self.friendly.clone();
        ok.insert(self.label.clone());
        ok
    }
}

/// Path memory: consecutive-stair count + previous stair direction. This is
/// the state the switchback ban and the stair cap live in.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct StairMem {
    /// Consecutive stairs taken to reach this cell.
    pub chain: u8,
    /// Horizontal direction of the previous stair, if the previous move
    /// was a stair.
    pub prev: Option<(i32, i32)>,
}

/// A routed move, recorded per path step and consumed by the emitter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RMove {
    /// Horizontal step (dust + support).
    H,
    /// Stair up (dy = +1 with the horizontal step).
    Up,
    /// Stair down.
    Down,
    /// Via climb: template `via` in horizontal direction `(dx, dz)`.
    Climb {
        /// X direction of the climb.
        dx: i32,
        /// Z direction of the climb.
        dz: i32,
        /// Via-template id in the registry.
        via: usize,
    },
}

/// The redstone fabric: a read-only view of the workspace plus routing
/// configuration. Implements `pnr_core::Fabric`.
pub struct RedstoneFabric<'a> {
    /// The build being routed through.
    pub ws: &'a Workspace,
    /// Net table; `RouteCtx::net` indexes it.
    pub nets: &'a [NetSpec],
    /// Footprint bounds: cells must not route outside them (cells routing
    /// outside their own footprint stole the composer's port space).
    pub bounds: Option<Aabb>,
    /// Optional include/exclude region constraints.
    pub region: Option<&'a RoutingRegion>,
    /// The signal budget (stair cap, refresh interval).
    pub budget: SignalBudget,
    /// Vertical via templates.
    pub vias: &'a ViaRegistry,
}

impl<'a> RedstoneFabric<'a> {
    fn in_bounds(&self, p: Pos) -> bool {
        self.bounds.map_or(true, |b| b.contains(p)) && self.region.map_or(true, |r| r.contains(p))
    }

    /// Design rules for placing (or reusing) a dust cell at `p` for a net
    /// accepting `ok` labels. Exact port of the prototype's `dust_ok`.
    pub fn dust_ok(&self, p: Pos, ok: &BTreeSet<String>) -> bool {
        let cells = self.ws.cells();
        if cells.contains_key(&p) {
            // Reuse own net only: the occupied cell must carry a friendly
            // label (foreign dust, stone, anything else: illegal).
            return self.ws.label(p).is_some_and(|l| ok.contains(l));
        }
        let sup = p.offset(0, -1, 0);
        match cells.get(&sup) {
            Some(_) if !self.ws.solid_at(sup) => return false, // support cell blocked
            Some(_) => {}
            None => {
                // Support will be placed. RULE: a support may not cap
                // diagonal-using dust — a block above dust cuts that dust's
                // up-diagonals, changing someone else's circuit. A flat-run
                // dust caps harmlessly (which is what makes y+1 bridges
                // over existing lanes legal).
                let below = p.offset(0, -2, 0);
                if cells.get(&below).is_some_and(|b| crate::blocks::is_dust(b)) {
                    for q in nets::neighbours(cells, below) {
                        if q.y != below.y {
                            return false;
                        }
                    }
                }
            }
        }
        // Electrical clearance: ask who a dust at `p` would touch.
        // Exact-label match only: "sig#13" (a pre-gate collector) is a
        // DIFFERENT electrical net from "sig" — touching it is a short.
        for q in nets::neighbours(cells, p) {
            if let Some(lab) = self.ws.label(q) {
                if !ok.contains(lab) {
                    return false;
                }
            }
        }
        true
    }

    /// Whether the vertical column `x, z, y0..=y1` is entirely free.
    pub fn col_free(&self, x: i32, z: i32, y0: i32, y1: i32) -> bool {
        (y0..=y1).all(|y| !self.ws.cells().contains_key(&Pos::new(x, y, z)))
    }

    fn ok_of(&self, ctx: &RouteCtx) -> BTreeSet<String> {
        self.nets[ctx.net].ok_labels()
    }
}

impl<'a> Fabric for RedstoneFabric<'a> {
    type Memory = StairMem;
    type Tag = RMove;

    fn start_memory(&self) -> StairMem {
        StairMem::default()
    }

    fn moves(&self, from: &State<StairMem>, _ctx: &RouteCtx) -> Vec<Candidate<StairMem, RMove>> {
        let p = from.pos;
        let mut out = Vec::new();
        for (dx, dz) in H_MOVES {
            let to = p.offset(dx, 0, dz);
            out.push(Candidate {
                to: State {
                    pos: to,
                    mem: StairMem::default(),
                },
                base_cost: 1,
                tag: RMove::H,
                footprint: vec![to, to.offset(0, -1, 0)],
            });
            if from.mem.chain < self.budget.stair_cap {
                for (dy, tag) in [(1, RMove::Up), (-1, RMove::Down)] {
                    let to = p.offset(dx, dy, dz);
                    out.push(Candidate {
                        to: State {
                            pos: to,
                            mem: StairMem {
                                chain: from.mem.chain + 1,
                                prev: Some((dx, dz)),
                            },
                        },
                        base_cost: 3,
                        tag,
                        footprint: vec![to, to.offset(0, -1, 0)],
                    });
                }
            }
        }
        // Via climbs, one per template per direction. The climb lays its
        // OWN entry cell one step ahead (see ViaTemplate docs).
        for (vid, via) in self.vias.templates().iter().enumerate() {
            for (dx, dz) in H_MOVES {
                let exit = p.offset(via.reach * dx, via.rise(), via.reach * dz);
                let entry = p.offset(dx, 0, dz);
                let mut footprint = vec![entry, entry.offset(0, -1, 0)];
                for k in 0..=via.rise() {
                    footprint.push(Pos::new(exit.x, p.y + k, exit.z));
                }
                out.push(Candidate {
                    to: State {
                        pos: exit,
                        mem: StairMem::default(),
                    },
                    base_cost: via.cost,
                    tag: RMove::Climb { dx, dz, via: vid },
                    footprint,
                });
            }
        }
        out
    }

    fn legal(
        &self,
        from: &State<StairMem>,
        cand: &Candidate<StairMem, RMove>,
        ctx: &RouteCtx,
    ) -> bool {
        let q = cand.to.pos;
        let p = from.pos;
        if !self.in_bounds(q) {
            return false;
        }
        let ok = self.ok_of(ctx);
        if !self.dust_ok(q, &ok) {
            return false;
        }
        match &cand.tag {
            RMove::H => true,
            RMove::Up => {
                // RULE: stair corner cells must be clear — a solid corner
                // above the lower dust cuts the diagonal (silent opens).
                if self.ws.solid_at(p.offset(0, 1, 0)) {
                    return false;
                }
                // RULE: no switchback stairs — a stair exactly reversing
                // the previous stair places its support on the cell that
                // cuts the previous diagonal.
                !is_switchback(from, q, p)
            }
            RMove::Down => {
                // Same corner rule, descending.
                if self.ws.solid_at(Pos::new(q.x, p.y, q.z)) {
                    return false;
                }
                !is_switchback(from, q, p)
            }
            RMove::Climb { dx, dz, via } => {
                let t = self.vias.get(*via);
                let entry = p.offset(*dx, 0, *dz);
                if !self.in_bounds(entry) {
                    return false;
                }
                // Entry cell must itself be a legal dust cell (the dead-end
                // dust pointing into the base), and the ladder column free.
                self.dust_ok(entry, &ok) && self.col_free(q.x, q.z, p.y, p.y + t.column_top())
            }
        }
    }

    fn budget(&self) -> Budget {
        self.budget.core()
    }
}

fn is_switchback(from: &State<StairMem>, q: Pos, p: Pos) -> bool {
    if let Some(prev) = from.mem.prev {
        (q.x - p.x, q.z - p.z) == (-prev.0, -prev.1)
    } else {
        false
    }
}
