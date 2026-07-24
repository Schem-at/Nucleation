//! Peak-memory / output-preservation harness for the `exact` fingerprint.
//!
//! Builds a large dense fill (default ~10M blocks, two-block palette with
//! properties so tokens are non-trivial), then computes the exact fingerprint.
//! Run under `/usr/bin/time -l` (macOS) or `/usr/bin/time -v` (Linux) to read
//! peak RSS. Prints the fingerprint hex so old-vs-new runs can be diffed for
//! byte-identity.
//!
//!   cargo run --release --example fingerprint_mem -- 216
//!   /usr/bin/time -l target/release/examples/fingerprint_mem 216

use nucleation::fingerprint::{fingerprint, FingerprintSpec};
use nucleation::{BlockState, UniversalSchematic};

fn main() {
    let side: i32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(216);

    let a = BlockState::new("minecraft:oak_log")
        .with_properties(vec![("axis".into(), "y".into())]);
    let b = BlockState::new("minecraft:redstone_wire").with_properties(vec![
        ("north".into(), "side".into()),
        ("south".into(), "up".into()),
        ("east".into(), "none".into()),
        ("west".into(), "side".into()),
        ("power".into(), "13".into()),
    ]);

    let mut s = UniversalSchematic::new("mem".to_string());
    let mut n: u64 = 0;
    for x in 0..side {
        for y in 0..side {
            for z in 0..side {
                let blk = if (x * 7 + y * 13 + z) % 4 == 0 { &b } else { &a };
                s.set_block(x, y, z, blk);
                n += 1;
            }
        }
    }
    eprintln!("built {n} blocks (side {side})");

    let fp = fingerprint(&s, &FingerprintSpec::exact());
    println!("exact_fingerprint {}", fp.to_hex());
}
