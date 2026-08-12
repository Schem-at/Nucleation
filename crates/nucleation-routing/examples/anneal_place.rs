//! Deterministic placement annealing over pnr-core's engine, as a text
//! filter: the Python compositor (redstone-eda/anneal_genlib.py) feeds it
//! instance boxes + nets on stdin and reads annealed placements on stdout.
//!
//! Input lines (whitespace separated, `#` comments ignored):
//!   area  X0 Z0 X1 Z1          placement window for box ORIGINS + extents
//!   seed  N                    RNG seed (SplitMix64; reproducible)
//!   cell  ID W D X Z FIXED     claimed box (w x d), initial origin, 0|1
//!   pin   CELL DX DZ           the next `net` references pins by index
//!   net   PIN_IDX PIN_IDX...   one net over previously declared pins
//! Output:
//!   cost  BEFORE AFTER HPWL_BEFORE HPWL_AFTER ACCEPTED PROPOSED
//!   place ID X Z               one line per cell, input order
//!
//! Cost = HPWL + 1000 x inflated-box overlap area (margin 2: routes need a
//! corridor between boxes) + congestion (net-bbox load per 16x16 bin over
//! a soft capacity).  Feasibility gate: boxes inside the window.

use pnr_core::anneal::{anneal, AnnealProblem, Schedule, SplitMix64};
use std::io::Read;

#[derive(Clone)]
struct Cell {
    id: String,
    w: i64,
    d: i64,
    fixed: bool,
}

struct Pin {
    cell: usize,
    dx: i64,
    dz: i64,
}

struct Problem {
    cells: Vec<Cell>,
    pins: Vec<Pin>,
    nets: Vec<Vec<usize>>,
    area: (i64, i64, i64, i64),
    movable: Vec<usize>,
}

const MARGIN: i64 = 2;
const BIN: i64 = 16;
const BIN_CAP: i64 = 220; // soft per-bin routing capacity (bbox half-perim)

impl Problem {
    fn hpwl(&self, s: &[(i64, i64)]) -> i64 {
        let mut total = 0;
        for net in &self.nets {
            let mut x0 = i64::MAX;
            let mut x1 = i64::MIN;
            let mut z0 = i64::MAX;
            let mut z1 = i64::MIN;
            for &pi in net {
                let p = &self.pins[pi];
                let (cx, cz) = s[p.cell];
                x0 = x0.min(cx + p.dx);
                x1 = x1.max(cx + p.dx);
                z0 = z0.min(cz + p.dz);
                z1 = z1.max(cz + p.dz);
            }
            total += (x1 - x0) + (z1 - z0);
        }
        total
    }

    fn overlap(&self, s: &[(i64, i64)]) -> i64 {
        let mut total = 0;
        for i in 0..self.cells.len() {
            for j in i + 1..self.cells.len() {
                if self.cells[i].fixed && self.cells[j].fixed {
                    continue;
                }
                let (xi, zi) = s[i];
                let (xj, zj) = s[j];
                let ox =
                    (xi + self.cells[i].w + MARGIN).min(xj + self.cells[j].w + MARGIN) - xi.max(xj);
                let oz =
                    (zi + self.cells[i].d + MARGIN).min(zj + self.cells[j].d + MARGIN) - zi.max(zj);
                if ox > 0 && oz > 0 {
                    total += ox * oz;
                }
            }
        }
        total
    }

    fn congestion(&self, s: &[(i64, i64)]) -> i64 {
        // load = sum of net half-perimeters smeared over the bins their
        // bbox crosses; cost = quadratic excess over the soft capacity
        let mut bins: std::collections::HashMap<(i64, i64), i64> = std::collections::HashMap::new();
        for net in &self.nets {
            let mut x0 = i64::MAX;
            let mut x1 = i64::MIN;
            let mut z0 = i64::MAX;
            let mut z1 = i64::MIN;
            for &pi in net {
                let p = &self.pins[pi];
                let (cx, cz) = s[p.cell];
                x0 = x0.min(cx + p.dx);
                x1 = x1.max(cx + p.dx);
                z0 = z0.min(cz + p.dz);
                z1 = z1.max(cz + p.dz);
            }
            let load = (x1 - x0) + (z1 - z0);
            let (bx0, bx1) = (x0.div_euclid(BIN), x1.div_euclid(BIN));
            let (bz0, bz1) = (z0.div_euclid(BIN), z1.div_euclid(BIN));
            let nbins = ((bx1 - bx0 + 1) * (bz1 - bz0 + 1)).max(1);
            for bx in bx0..=bx1 {
                for bz in bz0..=bz1 {
                    *bins.entry((bx, bz)).or_insert(0) += load / nbins;
                }
            }
        }
        bins.values()
            .map(|&l| {
                let e = (l - BIN_CAP).max(0);
                e * e / 8
            })
            .sum()
    }
}

impl AnnealProblem for Problem {
    type State = Vec<(i64, i64)>;
    type Move = (usize, i64, i64);

    fn propose(&self, s: &Self::State, rng: &mut SplitMix64) -> Option<Self::Move> {
        if self.movable.is_empty() {
            return None;
        }
        let i = self.movable[rng.gen_range(self.movable.len())];
        let dx = rng.gen_range(17) as i64 - 8;
        let dz = rng.gen_range(17) as i64 - 8;
        let (x0, z0, x1, z1) = self.area;
        let (cx, cz) = s[i];
        let nx = (cx + dx).clamp(x0, (x1 - self.cells[i].w).max(x0));
        let nz = (cz + dz).clamp(z0, (z1 - self.cells[i].d).max(z0));
        Some((i, nx, nz))
    }

    fn apply(&self, s: &Self::State, mv: &Self::Move) -> Self::State {
        let mut n = s.clone();
        n[mv.0] = (mv.1, mv.2);
        n
    }

    fn cost(&self, s: &Self::State) -> f64 {
        (self.hpwl(s) + 1000 * self.overlap(s) + self.congestion(s)) as f64
    }
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("stdin");
    let mut cells = Vec::new();
    let mut pins = Vec::new();
    let mut nets = Vec::new();
    let mut init: Vec<(i64, i64)> = Vec::new();
    let mut area = (0i64, 0i64, 400i64, 400i64);
    let mut seed = 42u64;
    let mut ids: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in input.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "area" => {
                area = (
                    t[1].parse().unwrap(),
                    t[2].parse().unwrap(),
                    t[3].parse().unwrap(),
                    t[4].parse().unwrap(),
                )
            }
            "seed" => seed = t[1].parse().unwrap(),
            "cell" => {
                ids.insert(t[1].to_string(), cells.len());
                cells.push(Cell {
                    id: t[1].to_string(),
                    w: t[2].parse().unwrap(),
                    d: t[3].parse().unwrap(),
                    fixed: t[6] == "1",
                });
                init.push((t[4].parse().unwrap(), t[5].parse().unwrap()));
            }
            "pin" => pins.push(Pin {
                cell: ids[t[1]],
                dx: t[2].parse().unwrap(),
                dz: t[3].parse().unwrap(),
            }),
            "net" => nets.push(t[1..].iter().map(|s| s.parse().unwrap()).collect()),
            other => panic!("unknown directive {other}"),
        }
    }
    let movable = (0..cells.len()).filter(|&i| !cells[i].fixed).collect();
    let problem = Problem {
        cells,
        pins,
        nets,
        area,
        movable,
    };
    let before = problem.cost(&init);
    let hpwl_before = problem.hpwl(&init);
    let schedule = Schedule {
        t0: 60.0,
        cooling: 0.95,
        steps_per_temp: 600,
        t_min: 0.05,
    };
    let result = anneal(&problem, init, &schedule, seed);
    println!(
        "cost {before} {} {hpwl_before} {} {} {}",
        result.best_cost,
        problem.hpwl(&result.best),
        result.accepted,
        result.proposed
    );
    for (i, cell) in problem.cells.iter().enumerate() {
        println!(
            "place {} {} {}",
            cell.id, result.best[i].0, result.best[i].1
        );
    }
}
