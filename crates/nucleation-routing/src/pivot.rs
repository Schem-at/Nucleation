//! Form-pivot adapter tiles: template data for bus form conversion.
//!
//! Exact port of the three tiles verified in `redstone-eda/pivot_tiles.py`
//! (2026-08-09, 96/96 output checks each, zero crosstalk; cell listings in
//! `redstone-eda/pivot_tiles.md`).  Every cell below is a formula in the bit
//! index `n` (0..7); coordinates are tile-local, `y_v(n) = 1 + 2n`, lane
//! `z_n = 2n`.
//!
//! Physics the templates lean on (all probed in the Python suites):
//! - block-sandwich station `[entry][repeater][exit]`: fires from entry
//!   ss >= 1, exit re-emits a fresh 15;
//! - POINTING LAW: dust weak-powers only blocks on its connection axes, so
//!   every station ENTRY is preceded by >= 1 dust cell on the station's own
//!   axis; a station EXIT may sit at a corner for free (strong power);
//! - diode law: solid supports under step-UPPER dusts make a 1y/1x
//!   staircase conduct BOTH ways;
//! - cap law: straight runs tolerate solid caps, so the fan column's
//!   support layers double as separators.
//!
//! NO transparent blocks in this family; any solid works where a support
//! is called for (role colours are cosmetic).

use crate::blocks::{self, DUST};
use crate::bus::{Axis, BusPort};
use crate::cell::Fragment;
use crate::router::{RedstoneRouter, RouteError, RouteResult};
use crate::workspace::Workspace;
use pnr_core::Pos;

/// Bus width the verified templates are drawn for.
pub const PIVOT_BITS: u8 = 8;

const XSTAIR: i32 = 5; // first staircase dust after the lane station
const XPORT: i32 = 19; // horizontal port x (15 dusts after station exit)
const ZPORT: i32 = 19; // flat90 out-port z
const FANSTA: i32 = 6; // bit 7's fan refresh station z = 6..8

/// Which verified pivot tile.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PivotKind {
    /// Vertical (2y stack) -> horizontal (2z flat); flows +X.
    V2H,
    /// Horizontal (2z flat) -> vertical (2y stack); flows -X.
    H2V,
    /// Flat-form 90-degree corner: +X run -> +Z run, concentric lanes.
    Flat90,
}

fn yv(bit: i32) -> i32 {
    1 + 2 * bit
}

fn lane(bit: i32) -> i32 {
    2 * bit
}

fn put(f: &mut Fragment, x: i32, y: i32, z: i32, block: &str) {
    f.cells.insert(Pos::new(x, y, z), block.to_string());
}

fn dust_at(f: &mut Fragment, x: i32, y: i32, z: i32, bit: i32) {
    f.cells.insert(Pos::new(x, y, z), DUST.to_string());
    f.labels.insert(Pos::new(x, y, z), format!("bit{bit}"));
}

/// The vertical<->horizontal pivot: one geometry, two tiles (repeaters make
/// it one-way; `to_horizontal` picks v2h, otherwise h2v).
fn build_pivot(to_horizontal: bool) -> Fragment {
    let mut f = Fragment::new();
    let route = blocks::role_block("route");
    let lid = blocks::role_block("lid");
    let lane_c = blocks::role_block("lane");
    let gate = blocks::role_block("gate");
    for bit in 0..i32::from(PIVOT_BITS) {
        let (y, zn) = (yv(bit), lane(bit));
        let fansta = zn + 2 > 15; // port->lane run over budget: bit 7 only
                                  // fan column x=0: support layer below doubles as the cap over bit
                                  // n-1's fan dust (cap law: straight run)
        for z in 0..=zn {
            put(&mut f, 0, y - 1, z, lid);
            if fansta && (FANSTA..=FANSTA + 2).contains(&z) {
                continue; // station cells, filled below
            }
            dust_at(&mut f, 0, y, z, bit);
        }
        if fansta {
            // inline fan refresh, straight on the +Z axis
            if to_horizontal {
                put(&mut f, 0, y, FANSTA, route); // entry
                put(&mut f, 0, y, FANSTA + 1, &blocks::repeater("north", 1));
                put(&mut f, 0, y, FANSTA + 2, route); // exit
            } else {
                put(&mut f, 0, y, FANSTA + 2, route); // entry
                put(&mut f, 0, y, FANSTA + 1, &blocks::repeater("south", 1));
                put(&mut f, 0, y, FANSTA, route); // exit
            }
        }
        f.ports.insert(format!("v{bit}"), Pos::new(0, y, 0));
        // lane approach dust at x=1: its corner neighbour (or the strong
        // exit block behind it) gives it the X axis
        put(&mut f, 1, y - 1, zn, lane_c);
        dust_at(&mut f, 1, y, zn, bit);
        // block-sandwich lane station at x=2..4; floor under the repeater
        put(&mut f, 3, y - 1, zn, lid);
        if to_horizontal {
            put(&mut f, 2, y, zn, route); // entry (weak drive)
            put(&mut f, 3, y, zn, &blocks::repeater("west", 1)); // flows +X
            put(&mut f, 4, y, zn, route); // exit (fresh 15)
        } else {
            put(&mut f, 4, y, zn, route); // entry
            put(&mut f, 3, y, zn, &blocks::repeater("east", 1)); // flows -X
            put(&mut f, 2, y, zn, route); // exit
        }
        // staircase (1y per 1x) + flat run to the horizontal port
        for x in XSTAIR..=XPORT {
            let yy = (y - (x - XSTAIR)).max(1);
            // diode law: step-UPPER dust needs a CONDUCTING support; all
            // supports in this family are solid anyway
            put(&mut f, x, yy - 1, zn, gate);
            dust_at(&mut f, x, yy, zn, bit);
        }
        f.ports.insert(format!("h{bit}"), Pos::new(XPORT, 1, zn));
    }
    f
}

/// The flat-form corner: bit n enters at `z = 2n` (+X run) and leaves at
/// `x = 14 - 2n` (+Z run); concentric lanes preserve bus order in the
/// travel frame.
fn build_flat90() -> Fragment {
    let mut f = Fragment::new();
    let route = blocks::role_block("route");
    let lid = blocks::role_block("lid");
    for bit in 0..i32::from(PIVOT_BITS) {
        let zn = lane(bit);
        let xc = 14 - 2 * bit; // concentric corner column
        let s1 = xc >= 4; // room for the in-leg station (bits 0..5)
        for x in 0..=xc {
            if s1 && (1..=3).contains(&x) {
                continue; // S1 cells, filled below (blocks need no floor)
            }
            put(&mut f, x, 0, zn, lid);
            dust_at(&mut f, x, 1, zn, bit);
        }
        if s1 {
            put(&mut f, 1, 1, zn, route); // entry
            put(&mut f, 2, 0, zn, lid); // repeater floor
            put(&mut f, 2, 1, zn, &blocks::repeater("west", 1)); // flows +X
            put(&mut f, 3, 1, zn, route); // exit (fresh 15)
        }
        f.ports.insert(format!("in{bit}"), Pos::new(0, 1, zn));
        // one dust past the corner acquires the Z axis and points into S2
        put(&mut f, xc, 0, zn + 1, lid);
        dust_at(&mut f, xc, 1, zn + 1, bit);
        put(&mut f, xc, 1, zn + 2, route); // S2 entry
        put(&mut f, xc, 0, zn + 3, lid); // repeater floor
        put(&mut f, xc, 1, zn + 3, &blocks::repeater("north", 1)); // flows +Z
        put(&mut f, xc, 1, zn + 4, route); // exit (fresh 15)
        for z in zn + 5..=ZPORT {
            put(&mut f, xc, 0, z, lid);
            dust_at(&mut f, xc, 1, z, bit);
        }
        f.ports.insert(format!("out{bit}"), Pos::new(xc, 1, ZPORT));
    }
    f
}

/// The tile template, in tile-local coordinates.  Dust cells carry
/// per-bit labels `bit0`..`bit7`; stamping renames them onto real nets.
pub fn pivot_fragment(kind: PivotKind) -> Fragment {
    match kind {
        PivotKind::V2H => build_pivot(true),
        PivotKind::H2V => build_pivot(false),
        PivotKind::Flat90 => build_flat90(),
    }
}

/// The input-side port names of a tile, bit order.
pub fn input_ports(kind: PivotKind) -> Vec<String> {
    let stem = match kind {
        PivotKind::V2H => "v",
        PivotKind::H2V => "h",
        PivotKind::Flat90 => "in",
    };
    (0..PIVOT_BITS).map(|b| format!("{stem}{b}")).collect()
}

/// The output-side port names of a tile, bit order.
pub fn output_ports(kind: PivotKind) -> Vec<String> {
    let stem = match kind {
        PivotKind::V2H => "h",
        PivotKind::H2V => "v",
        PivotKind::Flat90 => "out",
    };
    (0..PIVOT_BITS).map(|b| format!("{stem}{b}")).collect()
}

/// The gross geometric form a bus spec presents.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BusForm {
    /// Bits stack in y (the dense 2y form).
    Vertical,
    /// Bits advance in a ground-plane axis (the flat 2-pitch form).
    Flat(Axis),
}

/// Classify a port's form from its pitch axis.
pub fn bus_form(port: &BusPort) -> BusForm {
    match port.spec.pitch.axis {
        Axis::Y => BusForm::Vertical,
        a => BusForm::Flat(a),
    }
}

/// Which pivot (if any) a `from -> to` bus route needs: vertical<->flat
/// endpoints need v2h/h2v; two flat forms on different plan axes need the
/// corner.  Same form: none.
pub fn pivot_for(from: &BusPort, to: &BusPort) -> Option<PivotKind> {
    match (bus_form(from), bus_form(to)) {
        (BusForm::Vertical, BusForm::Flat(_)) => Some(PivotKind::V2H),
        (BusForm::Flat(_), BusForm::Vertical) => Some(PivotKind::H2V),
        (BusForm::Flat(a), BusForm::Flat(b)) if a != b => Some(PivotKind::Flat90),
        _ => None,
    }
}

/// A completed bus route.
#[derive(Clone, Debug)]
pub struct BusRouteReport {
    /// The pivot stamped, if the endpoint forms differed.
    pub pivot: Option<PivotKind>,
    /// Stamped pivot input ports (bit order), when a pivot was used.
    pub pivot_in: Vec<Pos>,
    /// Stamped pivot output ports (bit order), when a pivot was used.
    pub pivot_out: Vec<Pos>,
    /// Per-bit routes: `width` routes without a pivot, `2 * width` with
    /// one (from->pivot then pivot->to, bit-major).
    pub routes: Vec<RouteResult>,
}

/// Route a bus from an Out port to an In port, stamping a pivot tile
/// implicitly when the endpoint forms differ (or when the caller — e.g. a
/// Gate declaring a form change — forces one via `force_pivot`).
///
/// Endpoint bit cells must already exist as dust; their labels are set to
/// `{label_prefix}{bit}` here so the router may join them.  The pivot is
/// stamped at `pivot_origin` (tile-local origin) and its dust is renamed
/// onto the same per-bit labels.
pub fn route_bus(
    ws: &mut Workspace,
    router: &RedstoneRouter,
    from: &BusPort,
    to: &BusPort,
    pivot_origin: Pos,
    label_prefix: &str,
    force_pivot: Option<PivotKind>,
) -> Result<BusRouteReport, RouteError> {
    if from.spec.width != to.spec.width {
        return Err(RouteError::Bus(format!(
            "width mismatch: {} vs {}",
            from.spec.width, to.spec.width
        )));
    }
    if from.spec.width > PIVOT_BITS {
        return Err(RouteError::Bus(format!(
            "verified pivot templates are {PIVOT_BITS} bits; got {}",
            from.spec.width
        )));
    }
    let kind = force_pivot.or_else(|| pivot_for(from, to));
    let width = from.spec.width;
    let label = |bit: u8| format!("{label_prefix}{bit}");
    for bit in 0..width {
        ws.set_label(from.bit(bit), &label(bit));
        ws.set_label(to.bit(bit), &label(bit));
    }
    let mut report = BusRouteReport {
        pivot: kind,
        pivot_in: Vec::new(),
        pivot_out: Vec::new(),
        routes: Vec::new(),
    };
    match kind {
        None => {
            for bit in 0..width {
                let r = router.route(ws, from.bit(bit), to.bit(bit), &label(bit), &[])?;
                report.routes.push(r);
            }
        }
        Some(k) => {
            let frag = pivot_fragment(k);
            let rename = |s: &str| -> String {
                if let Some(b) = s.strip_prefix("bit") {
                    format!("{label_prefix}{b}")
                } else {
                    s.to_string()
                }
            };
            let ports = frag
                .stamp(
                    ws,
                    (pivot_origin.x, pivot_origin.y, pivot_origin.z),
                    &rename,
                )
                .map_err(RouteError::Emit)?;
            for name in input_ports(k) {
                report.pivot_in.push(ports[&name]);
            }
            for name in output_ports(k) {
                report.pivot_out.push(ports[&name]);
            }
            for bit in 0..width {
                let i = usize::from(bit);
                let r = router.route(ws, from.bit(bit), report.pivot_in[i], &label(bit), &[])?;
                report.routes.push(r);
            }
            for bit in 0..width {
                let i = usize::from(bit);
                let r = router.route(ws, report.pivot_out[i], to.bit(bit), &label(bit), &[])?;
                report.routes.push(r);
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit;
    use pnr_core::Aabb;

    fn repeater_cells(f: &Fragment) -> Vec<(Pos, String)> {
        f.cells
            .iter()
            .filter(|(_, b)| blocks::is_repeater(b))
            .map(|(p, b)| (*p, b.clone()))
            .collect()
    }

    #[test]
    fn v2h_matches_the_verified_listing() {
        let f = pivot_fragment(PivotKind::V2H);
        // Tile footprint 20 x 16 x 15: bit 7's port dust at y=15 is the
        // top (pivot_tiles.md's "y=0..16" measured the verification build,
        // whose lever bank adds one support layer above the tile).
        assert_eq!(
            f.bounds().unwrap(),
            Aabb::new(Pos::new(0, 0, 0), Pos::new(19, 15, 14))
        );
        // 9 repeaters: 8 lane stations + bit 7's fan station.
        let reps = repeater_cells(&f);
        assert_eq!(reps.len(), 9);
        // Ports: v_n at (0, 1+2n, 0); h_n at (19, 1, 2n).
        for n in 0..8 {
            assert_eq!(f.ports[&format!("v{n}")], Pos::new(0, 1 + 2 * n, 0));
            assert_eq!(f.ports[&format!("h{n}")], Pos::new(19, 1, 2 * n));
        }
        // Lane stations flow +X: repeater[facing=west] at (3, 1+2n, 2n).
        for n in 0..8 {
            let p = Pos::new(3, 1 + 2 * n, 2 * n);
            assert_eq!(blocks::facing_of(&f.cells[&p]), Some("west"), "{p:?}");
        }
        // Bit-7 fan station at (0, 15, 6..8): entry block, +Z repeater, exit.
        assert!(blocks::is_solid_block(&f.cells[&Pos::new(0, 15, 6)]));
        assert_eq!(
            blocks::facing_of(&f.cells[&Pos::new(0, 15, 7)]),
            Some("north")
        );
        assert!(blocks::is_solid_block(&f.cells[&Pos::new(0, 15, 8)]));
        // Structural audit: every dust/repeater is supported.
        assert!(audit::audit(&f.cells).is_clean());
    }

    #[test]
    fn h2v_is_the_same_geometry_with_flipped_repeaters() {
        let a = pivot_fragment(PivotKind::V2H);
        let b = pivot_fragment(PivotKind::H2V);
        let pa: Vec<Pos> = a.cells.keys().copied().collect();
        let pb: Vec<Pos> = b.cells.keys().copied().collect();
        assert_eq!(pa, pb, "cell positions must be identical");
        // Every difference is a repeater with the opposite facing.
        for (p, blk) in &a.cells {
            let other = &b.cells[p];
            if blk == other {
                continue;
            }
            assert!(blocks::is_repeater(blk), "non-repeater differs at {p:?}");
            let fa = blocks::facing_of(blk).unwrap();
            let fb = blocks::facing_of(other).unwrap();
            let opposite = matches!(
                (fa, fb),
                ("west", "east") | ("east", "west") | ("north", "south") | ("south", "north")
            );
            assert!(opposite, "{p:?}: {fa} vs {fb}");
        }
    }

    #[test]
    fn flat90_matches_the_verified_listing() {
        let f = pivot_fragment(PivotKind::Flat90);
        // Footprint 15 x 2 x 20.
        assert_eq!(
            f.bounds().unwrap(),
            Aabb::new(Pos::new(0, 0, 0), Pos::new(14, 1, 19))
        );
        // 14 repeaters: S1 for bits 0..5 only (corner x >= 4) + S2 for all.
        assert_eq!(repeater_cells(&f).len(), 14);
        for n in 0..8i32 {
            let (zn, xc) = (2 * n, 14 - 2 * n);
            assert_eq!(f.ports[&format!("in{n}")], Pos::new(0, 1, zn));
            assert_eq!(f.ports[&format!("out{n}")], Pos::new(xc, 1, 19));
            // S2 repeater flows +Z at (xc, 1, zn+3).
            let s2 = Pos::new(xc, 1, zn + 3);
            assert_eq!(blocks::facing_of(&f.cells[&s2]), Some("north"), "{s2:?}");
            // S1 present iff xc >= 4 (bits 0..5), repeater flows +X.  For
            // bit 6 the in-leg is plain dust; bit 7's corner IS its port
            // (xc = 0), so x=2 on its lane is another bit's out-leg or air.
            let s1 = Pos::new(2, 1, zn);
            if xc >= 4 {
                assert_eq!(blocks::facing_of(&f.cells[&s1]), Some("west"));
            } else {
                assert!(
                    f.cells.get(&s1).is_none_or(|b| !blocks::is_repeater(b)),
                    "bits 6,7 skip S1"
                );
            }
        }
        assert!(audit::audit(&f.cells).is_clean());
    }

    #[test]
    fn pivot_selection_follows_endpoint_forms() {
        use crate::bus::{BusSpec, Encoding, Face, InOut, Pitch};
        let spec = |axis| BusSpec {
            width: 8,
            pitch: Pitch { axis, spacing: 2 },
            face: Face::East,
            encoding: Encoding::Binary1PerWire,
        };
        let port = |axis, dir| BusPort {
            spec: spec(axis),
            bit0: Pos::new(0, 1, 0),
            dir,
        };
        let v = port(Axis::Y, InOut::Out);
        let hz = port(Axis::Z, InOut::In);
        let hx = port(Axis::X, InOut::In);
        assert_eq!(pivot_for(&v, &hz), Some(PivotKind::V2H));
        assert_eq!(
            pivot_for(&hz, &port(Axis::Y, InOut::In)),
            Some(PivotKind::H2V)
        );
        assert_eq!(pivot_for(&hz, &hx), Some(PivotKind::Flat90));
        assert_eq!(pivot_for(&hz, &port(Axis::Z, InOut::In)), None);
    }
}
