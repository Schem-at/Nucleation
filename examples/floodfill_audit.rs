//! Full-resolution flood-fill connectivity audit of extracted `world_segment` builds.
//!
//! For every `.schem` in the given directories this loads the build with the
//! crate's own loader, takes its non-air blocks, and runs a per-block
//! connected-components pass (6-connectivity primary, 26-connectivity for
//! sensitivity). It then classifies each build and, crucially, *emulates the
//! fix's own cell-level split criteria* (cell_size=4, min_component_blocks=4096,
//! min_component_share=0.40, min_gap_cells=2) so we can tell which builds the
//! shipped fix would actually split — the ground truth for validating the
//! defaults against real data.
//!
//! It is read-only w.r.t. the extraction; it only reads the `.schem` files.
//!
//! Usage:
//!   cargo run --release --example floodfill_audit -- <out.tsv> <dir> [<dir> ...]
//!
//! Output: a TSV of one row per build to <out.tsv> (raw data for the report),
//! plus a compact aggregate summary to stdout.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use nucleation::formats::schematic::from_schematic;

// ---- The shipped fix's default DisconnectedSplit policy (segment.rs) --------
const CELL_SIZE: i32 = 4;
const MIN_COMPONENT_BLOCKS: u64 = 4_096;
const MIN_COMPONENT_SHARE: f64 = 0.40;
const MIN_GAP_CELLS: i32 = 2;

// ---- Audit-side "substantial component" lens (block level) ------------------
// Mirrors the fix's substantiality lens but at full block resolution.
const SUBSTANTIAL_BLOCKS: u64 = 4_096;
const SUBSTANTIAL_SHARE: f64 = 0.40;
// A MULTI-SUBSTANTIAL flag additionally needs the two substantial components
// to actually be separated (at least one empty block between them).
const MULTI_MIN_GAP_BLOCKS: i64 = 1;
// SINGLE = one component holds essentially everything.
const SINGLE_SHARE: f64 = 0.95;

type P = (i32, i32, i32);

struct Record {
    id: String,
    total: u64,
    // 6-connectivity
    comp6_count: usize,
    comp6_sizes: Vec<u64>, // descending, top few
    // 26-connectivity
    comp26_count: usize,
    comp26_top: Vec<u64>, // descending, top few
    // top-2 (6-conn) geometry
    top2_gap_blocks: i64, // empty-block gap between the two largest comps, -1 if <2 comps
    top2_bbox: Option<((P, P), (P, P))>,
    class: &'static str, // SINGLE | MINOR-FRAGMENTS | MULTI-SUBSTANTIAL
    substantial6: usize, // # block-comps that are substantial (>=4096 & >=40%)
    // fix emulation (cell level)
    fix_would_split: bool,
    fix_seed_count: usize,
    fix_seed_blocks: Vec<u64>, // descending
    fix_seed_gap_cells: i32,   // min pairwise cell gap among seeds, -1 if <2 seeds
    cell_comp_count: usize,
}

fn neighbors6(p: P) -> [P; 6] {
    [
        (p.0 - 1, p.1, p.2),
        (p.0 + 1, p.1, p.2),
        (p.0, p.1 - 1, p.2),
        (p.0, p.1 + 1, p.2),
        (p.0, p.1, p.2 - 1),
        (p.0, p.1, p.2 + 1),
    ]
}

/// Connected components over a point set. `conn26` toggles 6- vs 26-connectivity.
/// Returns each component as a Vec of points (so callers can measure geometry).
fn components(points: &FxHashSet<P>, conn26: bool) -> Vec<Vec<P>> {
    let mut seen: FxHashSet<P> = FxHashSet::default();
    seen.reserve(points.len());
    let mut comps: Vec<Vec<P>> = Vec::new();
    for &start in points {
        if seen.contains(&start) {
            continue;
        }
        seen.insert(start);
        let mut q = VecDeque::new();
        q.push_back(start);
        let mut comp = Vec::new();
        while let Some(c) = q.pop_front() {
            comp.push(c);
            if conn26 {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            if dx == 0 && dy == 0 && dz == 0 {
                                continue;
                            }
                            let nb = (c.0 + dx, c.1 + dy, c.2 + dz);
                            if points.contains(&nb) && seen.insert(nb) {
                                q.push_back(nb);
                            }
                        }
                    }
                }
            } else {
                for nb in neighbors6(c) {
                    if points.contains(&nb) && seen.insert(nb) {
                        q.push_back(nb);
                    }
                }
            }
        }
        comps.push(comp);
    }
    comps
}

fn bbox(comp: &[P]) -> (P, P) {
    let mut mn = comp[0];
    let mut mx = comp[0];
    for &p in comp {
        mn.0 = mn.0.min(p.0);
        mn.1 = mn.1.min(p.1);
        mn.2 = mn.2.min(p.2);
        mx.0 = mx.0.max(p.0);
        mx.1 = mx.1.max(p.1);
        mx.2 = mx.2.max(p.2);
    }
    (mn, mx)
}

/// Minimum empty-block gap (Chebyshev distance minus 1) between two components.
///
/// Exact min Chebyshev distance via a lower-bound-pruned sweep. `bbox_cheb` is a
/// hard lower bound on the true min distance, so we test candidate distances `d`
/// starting there; the first `d` that yields any within-`d` pair *is* the min.
/// For each `d` we only examine scan points whose Chebyshev distance to the
/// other component's bbox is `<= d` (near-boundary points) and probe a ball of
/// radius `d` in a hash of the other set, breaking on the first hit. Deep
/// interior points are skipped by the bbox lower bound, so even two 180k-block
/// side-by-side walls stay cheap.
fn min_gap_blocks(a: &[P], b: &[P]) -> i64 {
    let (amn, amx) = bbox(a);
    let (bmn, bmx) = bbox(b);
    let sep = |lo1: i32, hi1: i32, lo2: i32, hi2: i32| -> i32 {
        if hi1 < lo2 {
            lo2 - hi1
        } else if hi2 < lo1 {
            lo1 - hi2
        } else {
            0
        }
    };
    let bbox_cheb = sep(amn.0, amx.0, bmn.0, bmx.0)
        .max(sep(amn.1, amx.1, bmn.1, bmx.1))
        .max(sep(amn.2, amx.2, bmn.2, bmx.2));

    let (scan, other) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let (omn, omx) = if a.len() <= b.len() { (bmn, bmx) } else { (amn, amx) };
    let oset: FxHashSet<P> = other.iter().copied().collect();

    // Chebyshev distance from a point to the other component's bbox (0 if inside).
    let lb_to_bbox = |p: P| -> i32 {
        let dx = (omn.0 - p.0).max(p.0 - omx.0).max(0);
        let dy = (omn.1 - p.1).max(p.1 - omx.1).max(0);
        let dz = (omn.2 - p.2).max(p.2 - omx.2).max(0);
        dx.max(dy).max(dz)
    };

    // Subsample the scan set so pathological interleaved mega-components stay
    // cheap. Deterministic stride; only affects the reported gap value slightly
    // (never the classification, which needs only gap >= 1). Points nearest the
    // true minimum are dense, so a stride sample still lands on the boundary.
    const SCAN_CAP: usize = 3000;
    let stride = (scan.len() / SCAN_CAP).max(1);
    let sampled: Vec<P> = scan.iter().step_by(stride).copied().collect();

    // Ascending `d` from the exact lower bound: because any point whose nearest
    // neighbour sat at distance < d would already have returned at that earlier
    // `d`, at iteration `d` only the Chebyshev *shell* of radius `d` can hold a
    // first hit. That turns an O(d^3) ball probe into an O(d^2) shell probe.
    let d_cap = bbox_cheb + 16;
    for d in bbox_cheb..=d_cap {
        for &c in &sampled {
            if lb_to_bbox(c) > d {
                continue;
            }
            for dx in -d..=d {
                for dy in -d..=d {
                    let on_x = dx == d || dx == -d;
                    let on_y = dy == d || dy == -d;
                    // if neither x nor y is on the shell face, z must be extreme
                    let z_range: &[i32] = if on_x || on_y {
                        // full z span lies on the shell
                        &[]
                    } else {
                        &[-d, d]
                    };
                    if z_range.is_empty() {
                        for dz in -d..=d {
                            if oset.contains(&(c.0 + dx, c.1 + dy, c.2 + dz)) {
                                return (d as i64 - 1).max(0);
                            }
                        }
                    } else {
                        for &dz in z_range {
                            if oset.contains(&(c.0 + dx, c.1 + dy, c.2 + dz)) {
                                return (d as i64 - 1).max(0);
                            }
                        }
                    }
                }
            }
        }
    }
    // No pair within the cap: report the bbox lower bound as the gap.
    (bbox_cheb as i64 - 1).max(0)
}

/// Six-connected components over a cell set (mirrors segment.rs cell logic).
fn cell_components(cells: &FxHashSet<P>) -> Vec<Vec<P>> {
    components(cells, false)
}

fn min_cell_gap(a: &[P], b: &[P]) -> i32 {
    // exact min chebyshev between cell sets (sets are small)
    let (scan, other) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let oset: FxHashSet<P> = other.iter().copied().collect();
    let mut best = i32::MAX;
    for &c in scan {
        for &o in oset.iter() {
            let d = (c.0 - o.0).abs().max((c.1 - o.1).abs()).max((c.2 - o.2).abs());
            best = best.min(d);
        }
    }
    best
}

fn audit_one(path: &Path) -> Option<Record> {
    let id = path.file_stem()?.to_string_lossy().to_string();
    let bytes = fs::read(path).ok()?;
    let schem = from_schematic(&bytes).ok()?;

    // Non-air block positions.
    let mut set: FxHashSet<P> = FxHashSet::default();
    for (pos, bs) in schem.iter_blocks() {
        let n = bs.get_name();
        if n == "minecraft:air" || n == "air" || n == "minecraft:cave_air" || n == "minecraft:void_air" {
            continue;
        }
        set.insert((pos.x, pos.y, pos.z));
    }
    let total = set.len() as u64;
    if total == 0 {
        return None;
    }

    // 6-connectivity components.
    let mut comps6 = components(&set, false);
    comps6.sort_by(|a, b| b.len().cmp(&a.len()));
    let comp6_sizes: Vec<u64> = comps6.iter().take(5).map(|c| c.len() as u64).collect();
    let comp6_count = comps6.len();

    // 26-connectivity components.
    let mut comps26 = components(&set, true);
    comps26.sort_by(|a, b| b.len().cmp(&a.len()));
    let comp26_top: Vec<u64> = comps26.iter().take(5).map(|c| c.len() as u64).collect();
    let comp26_count = comps26.len();

    // Substantial block-components (fix lens at block resolution).
    let share_floor = (SUBSTANTIAL_SHARE * total as f64).ceil() as u64;
    let substantial6 = comps6
        .iter()
        .filter(|c| c.len() as u64 >= SUBSTANTIAL_BLOCKS && c.len() as u64 >= share_floor)
        .count();

    // Top-2 geometry.
    let (top2_gap_blocks, top2_bbox) = if comps6.len() >= 2 {
        let g = min_gap_blocks(&comps6[0], &comps6[1]);
        (g, Some((bbox(&comps6[0]), bbox(&comps6[1]))))
    } else {
        (-1, None)
    };

    // Classification.
    let largest_share = comps6[0].len() as f64 / total as f64;
    let class = if largest_share >= SINGLE_SHARE {
        "SINGLE"
    } else if substantial6 >= 2 && top2_gap_blocks >= MULTI_MIN_GAP_BLOCKS {
        "MULTI-SUBSTANTIAL"
    } else {
        "MINOR-FRAGMENTS"
    };

    // ---- Fix emulation at cell level (cell_size=4, no dilation) -------------
    // Bin blocks into cells; count blocks per cell; six-connected cell comps.
    let mut cell_blocks: FxHashMap<P, u64> = FxHashMap::default();
    for &(x, y, z) in &set {
        let cell = (x.div_euclid(CELL_SIZE), y.div_euclid(CELL_SIZE), z.div_euclid(CELL_SIZE));
        *cell_blocks.entry(cell).or_insert(0) += 1;
    }
    let cell_set: FxHashSet<P> = cell_blocks.keys().copied().collect();
    let mut cell_comps = cell_components(&cell_set);
    cell_comps.sort_by(|a, b| {
        let ba: u64 = b.iter().map(|c| cell_blocks[c]).sum();
        let aa: u64 = a.iter().map(|c| cell_blocks[c]).sum();
        ba.cmp(&aa)
    });
    let cell_comp_count = cell_comps.len();
    let cell_comp_blocks: Vec<u64> = cell_comps
        .iter()
        .map(|comp| comp.iter().map(|c| cell_blocks[c]).sum())
        .collect();
    let cell_total: u64 = cell_comp_blocks.iter().sum();
    let cell_share_floor = (MIN_COMPONENT_SHARE * cell_total as f64).ceil() as u64;
    let seed_idx: Vec<usize> = (0..cell_comps.len())
        .filter(|&i| cell_comp_blocks[i] >= MIN_COMPONENT_BLOCKS && cell_comp_blocks[i] >= cell_share_floor)
        .collect();
    let fix_seed_blocks: Vec<u64> = seed_idx.iter().map(|&i| cell_comp_blocks[i]).collect();
    let fix_seed_count = seed_idx.len();
    let (fix_seed_gap_cells, all_pairs_ok) = if seed_idx.len() >= 2 {
        let mut min_gap = i32::MAX;
        let mut ok = true;
        for a in 0..seed_idx.len() {
            for b in (a + 1)..seed_idx.len() {
                let g = min_cell_gap(&cell_comps[seed_idx[a]], &cell_comps[seed_idx[b]]);
                min_gap = min_gap.min(g);
                if g < MIN_GAP_CELLS {
                    ok = false;
                }
            }
        }
        (min_gap, ok)
    } else {
        (-1, false)
    };
    let fix_would_split = seed_idx.len() >= 2 && all_pairs_ok;

    Some(Record {
        id,
        total,
        comp6_count,
        comp6_sizes,
        comp26_count,
        comp26_top,
        top2_gap_blocks,
        top2_bbox,
        class,
        substantial6,
        fix_would_split,
        fix_seed_count,
        fix_seed_blocks,
        fix_seed_gap_cells,
        cell_comp_count,
    })
}

fn collect_schems(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_schems(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("schem") {
                out.push(p);
            }
        }
    }
}

fn fmt_bbox(b: &(P, P)) -> String {
    format!("({},{},{})..({},{},{})", (b.0).0, (b.0).1, (b.0).2, (b.1).0, (b.1).1, (b.1).2)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: floodfill_audit <out.tsv> <dir> [<dir> ...]");
        std::process::exit(2);
    }
    let out_path = PathBuf::from(&args[1]);
    let mut files: Vec<PathBuf> = Vec::new();
    for d in &args[2..] {
        collect_schems(Path::new(d), &mut files);
    }
    files.sort();
    files.dedup();
    eprintln!("floodfill_audit: {} .schem files", files.len());

    let done = AtomicUsize::new(0);
    let mut records: Vec<Record> = files
        .par_iter()
        .filter_map(|p| {
            let r = audit_one(p);
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n % 500 == 0 {
                eprintln!("  .. {n} processed");
            }
            r
        })
        .collect();
    records.sort_by(|a, b| a.id.cmp(&b.id));

    // ---- Write TSV ----------------------------------------------------------
    let mut tsv = String::new();
    tsv.push_str("id\ttotal\tcomp6\tcomp26\tsubstantial6\tclass\ttop2_gap_blocks\ttop1\ttop2\ttop3\tfix_would_split\tfix_seed_count\tfix_seed_gap_cells\tfix_seed_blocks\tcell_comps\ttop1_bbox\ttop2_bbox\n");
    for r in &records {
        let g = |v: &Vec<u64>, i: usize| v.get(i).copied().unwrap_or(0);
        let seeds = r
            .fix_seed_blocks
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let (b1, b2) = match &r.top2_bbox {
            Some((a, b)) => (fmt_bbox(a), fmt_bbox(b)),
            None => ("-".into(), "-".into()),
        };
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.id,
            r.total,
            r.comp6_count,
            r.comp26_count,
            r.substantial6,
            r.class,
            r.top2_gap_blocks,
            g(&r.comp6_sizes, 0),
            g(&r.comp6_sizes, 1),
            g(&r.comp6_sizes, 2),
            r.fix_would_split,
            r.fix_seed_count,
            r.fix_seed_gap_cells,
            seeds,
            r.cell_comp_count,
            b1,
            b2,
        ));
    }
    fs::write(&out_path, tsv).expect("write tsv");

    // ---- Aggregate summary to stdout ---------------------------------------
    let n = records.len();
    let single = records.iter().filter(|r| r.class == "SINGLE").count();
    let minor = records.iter().filter(|r| r.class == "MINOR-FRAGMENTS").count();
    let multi = records.iter().filter(|r| r.class == "MULTI-SUBSTANTIAL").count();
    let multi26 = records
        .iter()
        .filter(|r| {
            // 26-conn "MULTI-SUBSTANTIAL": >=2 comps26 each >=4096 & >=40%
            let total = r.total;
            let sf = (SUBSTANTIAL_SHARE * total as f64).ceil() as u64;
            r.comp26_top.iter().filter(|&&s| s >= SUBSTANTIAL_BLOCKS && s >= sf).count() >= 2
        })
        .count();
    let would_split = records.iter().filter(|r| r.fix_would_split).count();
    let multi_not_split = records
        .iter()
        .filter(|r| r.class == "MULTI-SUBSTANTIAL" && !r.fix_would_split)
        .count();
    let split_not_multi = records
        .iter()
        .filter(|r| r.fix_would_split && r.class != "MULTI-SUBSTANTIAL")
        .count();

    println!("=== floodfill audit summary ===");
    println!("builds audited: {n}");
    println!("SINGLE: {single}");
    println!("MINOR-FRAGMENTS: {minor}");
    println!("MULTI-SUBSTANTIAL (6-conn): {multi}");
    println!("MULTI-SUBSTANTIAL (26-conn): {multi26}");
    println!("fix would split: {would_split}");
    println!("MULTI-SUBSTANTIAL but fix would NOT split (potential MISS): {multi_not_split}");
    println!("fix would split but NOT flagged MULTI-SUBSTANTIAL (potential OVER-split): {split_not_multi}");

    // component-count histogram (6-conn)
    let mut hist: FxHashMap<usize, usize> = FxHashMap::default();
    for r in &records {
        let bucket = match r.comp6_count {
            1 => 1,
            2 => 2,
            3 => 3,
            4..=5 => 5,
            6..=10 => 10,
            11..=50 => 50,
            _ => 999,
        };
        *hist.entry(bucket).or_insert(0) += 1;
    }
    println!("--- comp6 count histogram (bucket=upper bound; 999=51+) ---");
    let mut keys: Vec<usize> = hist.keys().copied().collect();
    keys.sort();
    for k in keys {
        println!("  <= {k}: {}", hist[&k]);
    }

    // worst offenders: MULTI-SUBSTANTIAL sorted by 2nd comp size desc
    println!("--- MULTI-SUBSTANTIAL builds (id | total | comp6 | top1 | top2 | gap_blocks | fix_split | seed_gap_cells) ---");
    let mut ms: Vec<&Record> = records.iter().filter(|r| r.class == "MULTI-SUBSTANTIAL").collect();
    ms.sort_by(|a, b| {
        let sb = b.comp6_sizes.get(1).copied().unwrap_or(0);
        let sa = a.comp6_sizes.get(1).copied().unwrap_or(0);
        sb.cmp(&sa)
    });
    for r in &ms {
        println!(
            "  {} | {} | {} | {} | {} | {} | {} | {}",
            r.id,
            r.total,
            r.comp6_count,
            r.comp6_sizes.first().copied().unwrap_or(0),
            r.comp6_sizes.get(1).copied().unwrap_or(0),
            r.top2_gap_blocks,
            r.fix_would_split,
            r.fix_seed_gap_cells,
        );
    }

    // d44bbed2 spotlight
    if let Some(r) = records.iter().find(|r| r.id.starts_with("d44bbed2")) {
        println!("--- d44bbed2 spotlight ---");
        println!(
            "  total={} comp6={} comp26={} class={} top1={} top2={} gap_blocks={} fix_split={} seeds={:?} seed_gap_cells={}",
            r.total,
            r.comp6_count,
            r.comp26_count,
            r.class,
            r.comp6_sizes.first().copied().unwrap_or(0),
            r.comp6_sizes.get(1).copied().unwrap_or(0),
            r.top2_gap_blocks,
            r.fix_would_split,
            r.fix_seed_blocks,
            r.fix_seed_gap_cells,
        );
    }

    println!("wrote TSV: {}", out_path.display());
}
