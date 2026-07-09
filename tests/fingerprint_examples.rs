//! Fingerprint over example schematics on hand. Uses the transparent open API.

use nucleation::fingerprint::{fingerprint, FingerprintSpec};
use nucleation::UniversalSchematic;

fn open(path: &str) -> Option<UniversalSchematic> {
    UniversalSchematic::open(path).ok()
}

#[test]
fn fingerprint_is_stable_and_distinguishes() {
    let spec = FingerprintSpec::redstone_computational();
    let adder = open("tests/fixtures/4bit_adder.litematic");
    let cube = open("simple_cube.litematic");
    match (adder, cube) {
        (Some(a), Some(c)) => {
            // deterministic across two computations
            assert_eq!(fingerprint(&a, &spec), fingerprint(&a, &spec));
            // distinct builds differ
            assert_ne!(fingerprint(&a, &spec), fingerprint(&c, &spec));
        }
        _ => eprintln!("skipping: example schematics not found at repo root"),
    }
}
