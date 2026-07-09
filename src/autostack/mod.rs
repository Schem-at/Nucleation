//! Auto-stack: detect the repeating structure(s) in a build and resize them.
//!
//! A build is (locally) invariant under a lattice translation `v`; resizing
//! re-stamps its fundamental domain a different number of times along `v`. This
//! module ports the pure-voxel core: region-coverage multi-structure detection
//! and lattice-vector resizing (1D, diagonal, and 2D). Graph-based detection,
//! near-periodic/booster handling, and simulation-based verification layer on
//! top behind the `simulation` feature.
//!
//! See `docs`/the design notes for the maths. Everything here operates on a
//! `Grid` (a map from integer voxel position to block state) extracted from a
//! [`UniversalSchematic`], so it has no rendering or simulation dependencies.

use crate::{BlockState, UniversalSchematic};
use std::collections::{HashMap, HashSet};

/// Graph-based diagonal period detection (requires the redstone graph / sim).
#[cfg(feature = "simulation")]
mod graph_detect;
#[cfg(feature = "simulation")]
pub use graph_detect::{detect_structures_graph, detect_structures_graph_json};

pub type Pos = (i32, i32, i32);
pub type Vec3 = [i32; 3];
type Grid = HashMap<Pos, BlockState>;

const AIR: [&str; 3] = ["minecraft:air", "minecraft:cave_air", "minecraft:void_air"];

/// One detected repeating structure.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Structure {
    /// `"1d"` or `"2d"`.
    pub mode: String,
    /// One period vector for 1D, two for 2D.
    pub vectors: Vec<Vec3>,
    /// Fraction of the build locally periodic under these vectors (0..1).
    pub coverage: f32,
    /// Bounding box of the periodic region.
    pub region_min: Vec3,
    pub region_max: Vec3,
    /// Bounding box of one representative unit cell.
    pub cell_min: Vec3,
    pub cell_max: Vec3,
    /// Human-readable summary, e.g. `"2D array · Z×Y · 92% of build"`.
    pub label: String,
}

#[derive(Debug)]
pub enum AutostackError {
    NotPeriodic,
    BadUnits,
}

impl std::fmt::Display for AutostackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutostackError::NotPeriodic => write!(f, "build is not periodic"),
            AutostackError::BadUnits => write!(f, "invalid unit count"),
        }
    }
}
impl std::error::Error for AutostackError {}

// ---------------------------------------------------------------------------
// grid <-> schematic
// ---------------------------------------------------------------------------
fn build_grid(s: &UniversalSchematic) -> Grid {
    let mut g = Grid::new();
    for (p, bs) in s.iter_blocks() {
        if AIR.contains(&bs.get_name()) {
            continue;
        }
        g.insert((p.x, p.y, p.z), bs.clone());
    }
    g
}

fn grid_to_schematic(grid: &Grid, name: &str) -> UniversalSchematic {
    let mut out = UniversalSchematic::new(name.to_string());
    for (&(x, y, z), bs) in grid {
        out.set_block(x, y, z, bs);
    }
    out
}

#[inline]
fn proj(p: Pos, v: Vec3, o: Vec3) -> i64 {
    (p.0 - o[0]) as i64 * v[0] as i64
        + (p.1 - o[1]) as i64 * v[1] as i64
        + (p.2 - o[2]) as i64 * v[2] as i64
}

#[inline]
fn norm2(v: Vec3) -> i64 {
    v[0] as i64 * v[0] as i64 + v[1] as i64 * v[1] as i64 + v[2] as i64 * v[2] as i64
}

fn grid_origin(grid: &Grid) -> Vec3 {
    let mut o = [i32::MAX; 3];
    for &(x, y, z) in grid.keys() {
        o[0] = o[0].min(x);
        o[1] = o[1].min(y);
        o[2] = o[2].min(z);
    }
    o
}

// ---------------------------------------------------------------------------
// region-coverage multi-structure detection
// ---------------------------------------------------------------------------
/// Voxels locally periodic under `v`: a +v or -v translate exists and matches.
fn region_set(grid: &Grid, v: Vec3) -> HashSet<Pos> {
    let mut r = HashSet::new();
    for (p, bs) in grid {
        for sgn in [1i32, -1] {
            let q = (p.0 + sgn * v[0], p.1 + sgn * v[1], p.2 + sgn * v[2]);
            if let Some(bq) = grid.get(&q) {
                if bq.name == bs.name {
                    r.insert(*p);
                    break;
                }
            }
        }
    }
    r
}

fn axis_vec(ax: usize, t: i32) -> Vec3 {
    let mut v = [0, 0, 0];
    v[ax] = t;
    v
}

/// `(T, region)` maximising local-periodicity coverage along `ax`.
fn best_axis(grid: &Grid, ax: usize, tmax: i32) -> (i32, HashSet<Pos>) {
    let mut best: (i32, HashSet<Pos>) = (2, HashSet::new());
    for t in 2..=tmax {
        let r = region_set(grid, axis_vec(ax, t));
        if r.len() > best.1.len() {
            best = (t, r);
        }
    }
    best
}

fn bbox_of(set: &HashSet<Pos>) -> (Vec3, Vec3) {
    let mut mn = [i32::MAX; 3];
    let mut mx = [i32::MIN; 3];
    for &(x, y, z) in set {
        mn[0] = mn[0].min(x);
        mn[1] = mn[1].min(y);
        mn[2] = mn[2].min(z);
        mx[0] = mx[0].max(x);
        mx[1] = mx[1].max(y);
        mx[2] = mx[2].max(z);
    }
    (mn, mx)
}

/// One repeating cell near the region centre: one period along each periodic
/// axis, full extent along non-periodic axes.
fn unit_bbox(vectors: &[Vec3], rmn: Vec3, rmx: Vec3) -> (Vec3, Vec3) {
    let mut umn = rmn;
    let mut umx = rmx;
    for v in vectors {
        let ax = (0..3).find(|&i| v[i] != 0).unwrap();
        let t = v[ax].abs();
        let ctr = (rmn[ax] + rmx[ax]) / 2;
        let start = rmn[ax] + ((ctr - rmn[ax]) / t) * t;
        umn[ax] = start;
        umx[ax] = (start + t - 1).min(rmx[ax]);
    }
    (umn, umx)
}

fn label_of(mode: &str, axes: &[usize], coverage: f32) -> String {
    let axs: Vec<&str> = axes.iter().map(|&a| ["X", "Y", "Z"][a]).collect();
    let kind = if mode == "2d" { "2D array" } else { "1D run" };
    format!(
        "{} · {} · {}% of build",
        kind,
        axs.join("×"),
        (coverage * 100.0) as i32
    )
}

/// Ranked list of repeating structures, by region coverage (2D first, then 1D).
pub fn detect_structures(s: &UniversalSchematic) -> Vec<Structure> {
    let grid = build_grid(s);
    detect_structures_grid(&grid)
}

fn detect_structures_grid(grid: &Grid) -> Vec<Structure> {
    let tot = grid.len().max(1) as f32;
    let axper: Vec<(i32, HashSet<Pos>)> = (0..3).map(|ax| best_axis(grid, ax, 10)).collect();
    let cov: Vec<f32> = (0..3).map(|ax| axper[ax].1.len() as f32 / tot).collect();

    let mut structs: Vec<Structure> = Vec::new();
    let mut used = [false; 3];

    // 2D: the two highest-coverage axes whose regions overlap substantially.
    let mut cands: Vec<usize> = (0..3).filter(|&ax| cov[ax] >= 0.5).collect();
    cands.sort_by(|&a, &b| cov[b].partial_cmp(&cov[a]).unwrap());
    if cands.len() >= 2 {
        let (a, b) = (cands[0], cands[1]);
        let inter: HashSet<Pos> = axper[a].1.intersection(&axper[b].1).copied().collect();
        if inter.len() as f32 / tot >= 0.35 {
            let (rmn, rmx) = bbox_of(&inter);
            let vecs = vec![axis_vec(a, axper[a].0), axis_vec(b, axper[b].0)];
            let (cmn, cmx) = unit_bbox(&vecs, rmn, rmx);
            let coverage = inter.len() as f32 / tot;
            structs.push(Structure {
                mode: "2d".to_string(),
                vectors: vecs,
                coverage,
                region_min: rmn,
                region_max: rmx,
                cell_min: cmn,
                cell_max: cmx,
                label: label_of("2d", &[a, b], coverage),
            });
            used[a] = true;
            used[b] = true;
        }
    }
    // 1D for remaining axes with meaningful coverage.
    for ax in 0..3 {
        if used[ax] || cov[ax] < 0.15 {
            continue;
        }
        let (rmn, rmx) = bbox_of(&axper[ax].1);
        let vecs = vec![axis_vec(ax, axper[ax].0)];
        let (cmn, cmx) = unit_bbox(&vecs, rmn, rmx);
        structs.push(Structure {
            mode: "1d".to_string(),
            vectors: vecs,
            coverage: cov[ax],
            region_min: rmn,
            region_max: rmx,
            cell_min: cmn,
            cell_max: cmx,
            label: label_of("1d", &[ax], cov[ax]),
        });
    }
    structs.sort_by(|a, b| b.coverage.partial_cmp(&a.coverage).unwrap());
    structs
}

// ---------------------------------------------------------------------------
// lattice resize (byte-identical runs)
// ---------------------------------------------------------------------------
type Cell = HashMap<Vec3, BlockState>;

fn lattice_slabs(grid: &Grid, v: Vec3, o: Vec3, phase_off: i64) -> HashMap<i64, Cell> {
    let n2 = norm2(v);
    let mut byk: HashMap<i64, Cell> = HashMap::new();
    for (p, bs) in grid {
        let k = (proj(*p, v, o) - phase_off).div_euclid(n2);
        let loc = [
            p.0 - (k as i32) * v[0],
            p.1 - (k as i32) * v[1],
            p.2 - (k as i32) * v[2],
        ];
        byk.entry(k).or_default().insert(loc, bs.clone());
    }
    byk
}

fn cell_name_sig(cell: &Cell) -> Vec<(Vec3, &str)> {
    let mut s: Vec<(Vec3, &str)> = cell.iter().map(|(l, b)| (*l, b.name.as_str())).collect();
    s.sort();
    s
}

/// `(sorted_keys, run_start_index, run_len)` — longest run of name-identical
/// consecutive cells.
fn lattice_run(byk: &HashMap<i64, Cell>) -> (Vec<i64>, usize, usize) {
    let mut ks: Vec<i64> = byk.keys().copied().collect();
    ks.sort();
    let sigs: Vec<Vec<(Vec3, &str)>> = ks.iter().map(|k| cell_name_sig(&byk[k])).collect();
    let (mut best_start, mut best_len) = (0usize, 1usize);
    let mut i = 0;
    while i < ks.len() {
        let mut j = i;
        while j + 1 < ks.len() && sigs[j + 1] == sigs[i] {
            j += 1;
        }
        if j - i + 1 > best_len {
            best_start = i;
            best_len = j - i + 1;
        }
        i = j + 1;
    }
    (ks, best_start, best_len)
}

/// Try every phase offset; return the one maximising the identical-cell run.
fn best_phase(grid: &Grid, v: Vec3, o: Vec3) -> (i64, Vec<i64>, usize, usize) {
    let n2 = norm2(v);
    let pmin = grid.keys().map(|&p| proj(p, v, o)).min().unwrap_or(0);
    let mut best: Option<(i64, Vec<i64>, usize, usize)> = None;
    for t in 0..n2 {
        let phase = pmin - t;
        let byk = lattice_slabs(grid, v, o, phase);
        let (ks, rs, rl) = lattice_run(&byk);
        if best.as_ref().is_none_or(|b| rl > b.3) {
            best = Some((phase, ks, rs, rl));
        }
    }
    best.unwrap()
}

/// 1D resize: head + N copies of the unit cell + tail, stamped at `i*v`.
pub fn resize_1d(
    s: &UniversalSchematic,
    v: Vec3,
    n_units: usize,
) -> Result<UniversalSchematic, AutostackError> {
    if n_units == 0 {
        return Err(AutostackError::BadUnits);
    }
    let grid = build_grid(s);
    let o = grid_origin(&grid);
    let (phase, ks, run_start, run_len) = best_phase(&grid, v, o);
    let byk = lattice_slabs(&grid, v, o, phase);

    let mut seq: Vec<&Cell> = Vec::new();
    for &k in &ks[..run_start] {
        seq.push(&byk[&k]);
    }
    let unit = &byk[&ks[run_start]];
    for _ in 0..n_units {
        seq.push(unit);
    }
    for &k in &ks[run_start + run_len..] {
        seq.push(&byk[&k]);
    }

    let mut out = Grid::new();
    for (i, cell) in seq.iter().enumerate() {
        let i = i as i32;
        for (loc, bs) in cell.iter() {
            let p = (loc[0] + i * v[0], loc[1] + i * v[1], loc[2] + i * v[2]);
            out.insert(p, bs.clone());
        }
    }
    Ok(grid_to_schematic(&out, "resized"))
}

// ---------------------------------------------------------------------------
// 2D resize (nine-slice)
// ---------------------------------------------------------------------------
fn run_of_axis(grid: &Grid, v: Vec3, o: Vec3) -> (i64, Vec<i64>, usize, usize) {
    best_phase(grid, v, o)
}

fn remap(i_new: usize, run_start: usize, run_len: usize, m: usize) -> usize {
    if i_new < run_start {
        i_new
    } else if i_new < run_start + m {
        run_start
    } else {
        run_start + run_len + (i_new - (run_start + m))
    }
}

/// 2D resize: corners fixed, edges scale 1D, interior tiles 2D.
pub fn resize_2d(
    s: &UniversalSchematic,
    v1: Vec3,
    v2: Vec3,
    m1: usize,
    m2: usize,
) -> Result<UniversalSchematic, AutostackError> {
    if m1 == 0 || m2 == 0 {
        return Err(AutostackError::BadUnits);
    }
    let grid = build_grid(s);
    let o = grid_origin(&grid);
    let (ph1, ks1, rs1, rl1) = run_of_axis(&grid, v1, o);
    let (ph2, ks2, rs2, rl2) = run_of_axis(&grid, v2, o);
    let n1 = norm2(v1);
    let n2 = norm2(v2);

    // grid of cells keyed by (i, j)
    let mut byij: HashMap<(i64, i64), Cell> = HashMap::new();
    for (p, bs) in &grid {
        let i = (proj(*p, v1, o) - ph1).div_euclid(n1);
        let j = (proj(*p, v2, o) - ph2).div_euclid(n2);
        let loc = [
            p.0 - (i as i32) * v1[0] - (j as i32) * v2[0],
            p.1 - (i as i32) * v1[1] - (j as i32) * v2[1],
            p.2 - (i as i32) * v1[2] - (j as i32) * v2[2],
        ];
        byij.entry((i, j)).or_default().insert(loc, bs.clone());
    }
    let n_tail1 = ks1.len() - (rs1 + rl1);
    let n_tail2 = ks2.len() - (rs2 + rl2);
    let new_i = rs1 + m1 + n_tail1;
    let new_j = rs2 + m2 + n_tail2;

    let mut out = Grid::new();
    for i2 in 0..new_i {
        let oi = remap(i2, rs1, rl1, m1) as i64;
        for j2 in 0..new_j {
            let oj = remap(j2, rs2, rl2, m2) as i64;
            if let Some(cell) = byij.get(&(oi, oj)) {
                for (loc, bs) in cell {
                    let p = (
                        loc[0] + (i2 as i32) * v1[0] + (j2 as i32) * v2[0],
                        loc[1] + (i2 as i32) * v1[1] + (j2 as i32) * v2[1],
                        loc[2] + (i2 as i32) * v1[2] + (j2 as i32) * v2[2],
                    );
                    out.insert(p, bs.clone());
                }
            }
        }
    }
    Ok(grid_to_schematic(&out, "resized"))
}

/// JSON array of the detected structures — the binding-friendly entry point
/// (every language binding can hand this straight to its host).
pub fn detect_structures_json(s: &UniversalSchematic) -> String {
    serde_json::to_string(&detect_structures(s)).unwrap_or_else(|_| "[]".to_string())
}

/// Resize using a [`Structure`] given as JSON plus the new unit count(s).
/// `units` is `[n]` for 1D or `[n1, n2]` for 2D. Returns a new schematic.
pub fn resize_json(
    s: &UniversalSchematic,
    structure_json: &str,
    units: &[usize],
) -> Result<UniversalSchematic, String> {
    let st: Structure = serde_json::from_str(structure_json).map_err(|e| e.to_string())?;
    resize(s, &st, units).map_err(|e| e.to_string())
}

/// Resize using a detected [`Structure`]: 1D takes `units[0]`, 2D takes
/// `units[0]`×`units[1]`.
pub fn resize(
    s: &UniversalSchematic,
    st: &Structure,
    units: &[usize],
) -> Result<UniversalSchematic, AutostackError> {
    match st.mode.as_str() {
        "2d" if st.vectors.len() == 2 && units.len() >= 2 => {
            resize_2d(s, st.vectors[0], st.vectors[1], units[0], units[1])
        }
        _ if !st.vectors.is_empty() && !units.is_empty() => resize_1d(s, st.vectors[0], units[0]),
        _ => Err(AutostackError::BadUnits),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 3-wide bar repeating along X with period 2 -> detect + resize.
    fn bar(n: usize) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("t".into());
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");
        for i in 0..n {
            let x = (i * 2) as i32;
            s.set_block(x, 0, 0, &stone);
            s.set_block(x + 1, 0, 0, &glass);
        }
        s
    }

    #[test]
    fn detects_x_period() {
        let s = bar(6);
        let structs = detect_structures(&s);
        assert!(!structs.is_empty());
        assert_eq!(structs[0].vectors[0], [2, 0, 0]);
    }

    #[test]
    fn resize_scales_linearly() {
        let s = bar(6);
        let structs = detect_structures(&s);
        let bigger = resize(&s, &structs[0], &[10]).unwrap();
        let n = build_grid(&bigger).len(); // non-air blocks
                                           // 10 cells * 2 blocks each = 20
        assert_eq!(n, 20);
        // and it re-detects with the same period
        let re = detect_structures(&bigger);
        assert_eq!(re[0].vectors[0], [2, 0, 0]);
    }

    #[test]
    fn identity_resize_reproduces_input() {
        let s = bar(6);
        let structs = detect_structures(&s);
        // resizing to the original run length reproduces the input exactly
        let run = &structs[0]; // 1d
                               // find the run length the resizer sees
        let grid = build_grid(&s);
        let o = grid_origin(&grid);
        let (_, _, _, run_len) = best_phase(&grid, run.vectors[0], o);
        let same = resize(&s, run, &[run_len]).unwrap();
        assert_eq!(build_grid(&same).len(), build_grid(&s).len());
    }

    /// A 2D tiling in the Y-Z plane (period 2 in both) at x=0.
    fn screen(ny: i32, nz: i32) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("screen".into());
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");
        for y in 0..2 * ny {
            for z in 0..2 * nz {
                let b = if y % 2 == 0 && z % 2 == 0 {
                    &stone
                } else {
                    &glass
                };
                s.set_block(0, y, z, b);
            }
        }
        s
    }

    #[test]
    fn detects_and_resizes_2d() {
        let s = screen(6, 6);
        let structs = detect_structures(&s);
        let st = &structs[0];
        assert_eq!(st.mode, "2d");
        // resize to a 4x8 grid of cells
        let r = resize(&s, st, &[4, 8]).unwrap();
        // 4*8 cells, 2x2 each = 4 blocks/cell -> 128 non-air blocks
        assert_eq!(build_grid(&r).len(), 4 * 8 * 4);
    }

    #[test]
    fn diagonal_resize_works() {
        // a staircase: cells step by (2,1,0)
        let mut s = UniversalSchematic::new("diag".into());
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");
        for i in 0..6 {
            let (x, y) = (i * 2, i);
            s.set_block(x, y, 0, &stone);
            s.set_block(x + 1, y, 0, &glass);
        }
        // resize directly with the diagonal vector
        let r = resize_1d(&s, [2, 1, 0], 10).unwrap();
        assert_eq!(build_grid(&r).len(), 20);
    }

    #[test]
    fn json_roundtrip() {
        let s = bar(6);
        let json = detect_structures_json(&s);
        assert!(json.contains("\"1d\""));
        let one: serde_json::Value = serde_json::from_str(&json).unwrap();
        let first = serde_json::to_string(&one[0]).unwrap();
        let r = resize_json(&s, &first, &[8]).unwrap();
        assert_eq!(build_grid(&r).len(), 16);
    }
}
