//! Fingerprint a directory of .schem builds, report duplicate clusters, and
//! write a browsable markdown report. Parallel (rayon); progress on stderr.
//!
//!   cargo run --release --example fingerprint_dedup -- <dir> [max_files] [max_kb] [preset] [out.md]
//!   max_kb : skip .schem larger than this (cheap pre-load filter; default 32)
//!   preset : redstone | structural | exact   (default: redstone)
//!   out.md : report path (default: fingerprint_report.md)

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use rayon::prelude::*;

use nucleation::fingerprint::{fingerprint, FingerprintSpec};
use nucleation::UniversalSchematic;

struct Build {
    fp: u128,
    path: PathBuf,
    dims: (i32, i32, i32),
    blocks: usize,
}

fn collect_schems(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    cap: usize,
    max_bytes: u64,
    skipped: &mut usize,
) {
    if out.len() >= cap {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        if out.len() >= cap {
            return;
        }
        let p = entry.path();
        if p.is_dir() {
            collect_schems(&p, out, cap, max_bytes, skipped);
        } else if p.extension().and_then(|e| e.to_str()) == Some("schem") {
            match entry.metadata() {
                Ok(m) if m.len() <= max_bytes => out.push(p),
                Ok(_) => *skipped += 1,
                Err(_) => {}
            }
        }
    }
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: <dir> [max_files] [max_kb] [preset] [out.md]");
    let cap: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);
    let max_kb: u64 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(32);
    let preset = std::env::args()
        .nth(4)
        .unwrap_or_else(|| "redstone".to_string());
    let out_path = std::env::args()
        .nth(5)
        .unwrap_or_else(|| "fingerprint_report.md".to_string());
    let spec = match preset.as_str() {
        "structural" => FingerprintSpec::structural(),
        "exact" => FingerprintSpec::exact(),
        _ => FingerprintSpec::redstone_computational(),
    };

    let base = Path::new(&dir);
    let mut files = Vec::new();
    let mut skipped_large = 0usize;
    collect_schems(base, &mut files, cap, max_kb * 1024, &mut skipped_large);
    let n = files.len();
    eprintln!(
        "{n} files ≤{max_kb}KB selected, {skipped_large} skipped (too large); preset={preset}; threads={}",
        rayon::current_num_threads()
    );

    let done = AtomicUsize::new(0);
    let t_all = Instant::now();
    let builds: Vec<Build> = files
        .par_iter()
        .filter_map(|p| {
            let r = UniversalSchematic::open(p.to_str().unwrap()).ok().map(|s| {
                let (dims, blocks) = match s.get_tight_bounds() {
                    Some(b) => (
                        (
                            b.max.0 - b.min.0 + 1,
                            b.max.1 - b.min.1 + 1,
                            b.max.2 - b.min.2 + 1,
                        ),
                        s.iter_blocks().count(),
                    ),
                    None => ((0, 0, 0), 0),
                };
                Build {
                    fp: fingerprint(&s, &spec).0,
                    path: p.clone(),
                    dims,
                    blocks,
                }
            });
            let c = done.fetch_add(1, Ordering::Relaxed) + 1;
            if c.is_multiple_of(500) {
                let el = t_all.elapsed().as_secs_f64();
                eprintln!("  …{c}/{n}  ({:.0}/s, {:.0}s)", c as f64 / el, el);
            }
            r
        })
        .collect();
    let total = t_all.elapsed().as_secs_f64();

    // Group by fingerprint.
    let mut clusters: HashMap<u128, Vec<Build>> = HashMap::new();
    for b in builds {
        clusters.entry(b.fp).or_default().push(b);
    }
    let ok: usize = clusters.values().map(|v| v.len()).sum();
    let unique = clusters.len();
    let dup_copies = ok.saturating_sub(unique);

    let mut dups: Vec<(u128, Vec<Build>)> =
        clusters.into_iter().filter(|(_, v)| v.len() > 1).collect();
    dups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    // ── markdown report ──
    let mut md = String::new();
    let _ = writeln!(md, "# Fingerprint dedup report\n");
    let _ = writeln!(md, "| metric | value |");
    let _ = writeln!(md, "|---|---|");
    let _ = writeln!(md, "| preset | `{preset}` |");
    let _ = writeln!(md, "| builds fingerprinted | {ok} ({} failed) |", n - ok);
    let _ = writeln!(md, "| unique fingerprints | {unique} |");
    let _ = writeln!(
        md,
        "| duplicate copies | {dup_copies} ({:.1}%) |",
        100.0 * dup_copies as f64 / ok.max(1) as f64
    );
    let _ = writeln!(md, "| duplicate clusters | {} |", dups.len());
    let _ = writeln!(
        md,
        "| runtime | {total:.1}s ({:.0} builds/s) |\n",
        ok as f64 / total.max(1e-9)
    );

    let num_clusters = dups.len();
    let max_clusters = 300usize;
    let max_members = 40usize;
    let _ = writeln!(md, "## Duplicate clusters (largest first)\n");
    if num_clusters > max_clusters {
        let _ = writeln!(
            md,
            "_showing the {max_clusters} largest of {num_clusters} clusters_\n"
        );
    }
    for (i, (fp, mut members)) in dups.into_iter().take(max_clusters).enumerate() {
        members.sort_by(|a, b| a.path.cmp(&b.path));
        let rep = &members[0];
        let (dx, dy, dz) = rep.dims;
        let _ = writeln!(
            md,
            "### {}. {} copies · `{:016x}` · {dx}×{dy}×{dz}, {} blocks",
            i + 1,
            members.len(),
            fp as u64,
            rep.blocks
        );
        for m in members.iter().take(max_members) {
            let rel = m.path.strip_prefix(base).unwrap_or(&m.path);
            let _ = writeln!(md, "- `{}`", rel.display());
        }
        if members.len() > max_members {
            let _ = writeln!(md, "- _…and {} more_", members.len() - max_members);
        }
        let _ = writeln!(md);
    }

    std::fs::write(&out_path, md).expect("write report");
    eprintln!(
        "\n{ok} builds in {total:.1}s · {dup_copies} dup copies in {num_clusters} clusters · wrote {out_path}"
    );
    println!("report: {out_path}");
}
