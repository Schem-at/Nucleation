//! Promoting a port must not break EXPORT.
//!
//! Reported from the studio: promoting any port permanently broke
//! `Design::to_litematic`, which raised `NucleationError.Serialize`. The studio
//! worked around it by exporting before any promotion — i.e. you could not ship
//! the very designs promotion exists to make possible.
//!
//! Root cause, and it is a whole class rather than one typo: the `.litematic`
//! manifest is JSON, the promotion patch was carried as `BTreeMap<P3, _>`, and
//! **serde_json cannot serialize a map whose key is not a string** — it fails
//! with "key must be a string". `.nucm` never showed it because that payload is
//! bincode, which does not care. So the rule this file pins: every export path
//! must survive a promoted port, and the JSON ones must not carry
//! position-keyed maps.
//!
//! The cell here is synthetic on purpose — no corpus needed, so this runs in the
//! default gate.

#![cfg(feature = "routing")]

use nucleation::design::{Design, PortMode};
use nucleation::io_contract::{CellContract, IoLayoutBuilder, IoType, LayoutFunction};
use nucleation::UniversalSchematic;

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const WIDTH: usize = 4;

/// A cell with a 4-bit LEVER input `d` and a 4-bit LAMP output `q`: the shape of
/// community hardware, and the shape promotion has to convert.
fn lever_cell() -> (UniversalSchematic, CellContract) {
    let mut s = UniversalSchematic::new("lc".to_string());
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    for k in 0..WIDTH as i32 {
        let z = 2 * k;
        // Input bit: a floor lever on its own support.
        s.set_block_from_string(0, 0, z, STONE).unwrap();
        s.set_block_from_string(0, 1, z, LEVER).unwrap();
        ins.push((0, 1, z));
        // Output bit: a lamp.
        s.set_block_from_string(6, 0, z, STONE).unwrap();
        s.set_block_from_string(6, 1, z, LAMP).unwrap();
        outs.push((6, 1, z));
    }
    let ty = IoType::UnsignedInt { bits: WIDTH };
    let io = IoLayoutBuilder::new()
        .add_input("d", ty.clone(), LayoutFunction::OneToOne, ins)
        .unwrap()
        .add_output("q", ty, LayoutFunction::OneToOne, outs)
        .unwrap()
        .build();
    (s, CellContract::new("lc", io))
}

/// Every non-air block of the flattened design, for byte comparison.
fn blocks(d: &Design) -> std::collections::BTreeMap<(i32, i32, i32), String> {
    let flat = d.flatten().expect("flatten");
    let mut out = std::collections::BTreeMap::new();
    for (bp, bs) in flat.iter_blocks() {
        let s = bs.to_string();
        if s.contains("minecraft:air") {
            continue;
        }
        out.insert((bp.x, bp.y, bp.z), s);
    }
    out
}

fn design() -> Design {
    let mut d = Design::new("exp");
    let (sch, contract) = lever_cell();
    d.add_cell_with_contract("lc", sch, contract);
    d.place("u0", "lc", (0, 0, 0), 0).unwrap();
    d
}

/// Every export path, before and after a promotion, with the reason named. A
/// bare `is_ok()` would tell us nothing about WHY, and the whole bug was a
/// swallowed serializer message.
fn export_all(d: &Design) -> Vec<(&'static str, Result<usize, String>)> {
    vec![
        (
            "litematic",
            d.to_litematic_layered_bytes().map(|b| b.len()),
        ),
        ("nucm", d.to_nucm_bytes().map(|b| b.len())),
        (
            "schem",
            d.flatten().and_then(|f| {
                nucleation::formats::schematic::to_schematic(&f)
                    .map(|b| b.len())
                    .map_err(|e| e.to_string())
            }),
        ),
    ]
}

#[test]
fn promoting_a_port_does_not_break_any_export() {
    let mut d = design();
    for (fmt, r) in export_all(&d) {
        assert!(r.is_ok(), "{fmt} export broken BEFORE promotion: {r:?}");
    }

    d.promote_input("u0", "d").unwrap();
    assert_eq!(d.port_mode("u0", "d"), PortMode::Bus);

    for (fmt, r) in export_all(&d) {
        assert!(
            r.is_ok(),
            "{fmt} export broke AFTER promoting a port: {r:?} — a promoted design must still \
             be shippable"
        );
    }
}

#[test]
fn a_promoted_design_survives_a_litematic_round_trip() {
    let mut d = design();
    d.promote_input("u0", "d").unwrap();
    let bytes = d.to_litematic_layered_bytes().expect("export");

    // Opens as a plain multi-region litematic...
    let plain = nucleation::formats::litematic::from_litematic(&bytes).expect("plain import");
    assert!(
        plain.get_region_names().iter().any(|r| r == "inst:u0"),
        "instance layer missing: {:?}",
        plain.get_region_names()
    );

    // ...and reimports as a usable design. What "survived" means here is set by
    // the TIER, not by convenience: `.litematic` is the INTERCHANGE tier, where
    // a cell reference degrades to an embedded copy of the body as exported —
    // already promoted. So the promotion arrives BAKED IN, not remembered:
    // the port reads as `Executor` because its hardware is simply dust now, and
    // there is no library cell left to revert to. Reversibility is a `.nucm`
    // (project tier) guarantee; see the nucm test below.
    let back = Design::from_litematic_layered_bytes(&bytes).expect("design reimport");
    assert_eq!(
        back.port_mode("u0", "d"),
        PortMode::Executor,
        "interchange tier bakes the promotion into the embedded body; it does not carry the \
         reversible patch"
    );
    // The point of the round trip: the port is still ROUTABLE, so the reimported
    // design can still be wired up.
    let port = back
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u0.d")
        .expect("u0.d present after round trip");
    assert!(
        port.routable(),
        "a promoted port must still be routable after a round trip: {:?}",
        port.blocked
    );
    assert_eq!(port.width, WIDTH as u8);
}

/// The per-layer fast paths must agree with `flatten()` CELL FOR CELL.
///
/// They exist to skip a full flatten during live re-routing, so the only way
/// they can hurt is by disagreeing with what actually gets exported — a viewer
/// showing a bus somewhere the artifact does not have it. Pinned here rather
/// than trusted.
#[test]
fn per_layer_block_json_agrees_with_flatten() {
    let mut d = design();
    d.promote_input("u0", "d").unwrap();

    // What flatten says region `inst:u0` holds.
    let flat = d.flatten().unwrap();
    let region = flat.get_region("inst:u0").expect("instance layer");
    let bb = region.get_tight_bounds().expect("the layer holds blocks");
    let mut from_flatten: Vec<(i32, i32, i32, String)> = Vec::new();
    for x in bb.min.0..=bb.max.0 {
        for y in bb.min.1..=bb.max.1 {
            for z in bb.min.2..=bb.max.2 {
                if let Some(b) = region.get_block(x, y, z) {
                    let s = b.to_string();
                    if s.contains("minecraft:air") {
                        continue;
                    }
                    from_flatten.push((x, y, z, s));
                }
            }
        }
    }
    from_flatten.sort();

    // What the fast path says.
    let json = d.instance_blocks_json("u0").unwrap();
    let parsed: Vec<(i32, i32, i32, String)> = serde_json::from_str::<Vec<serde_json::Value>>(&json)
        .expect("valid JSON")
        .into_iter()
        .map(|v| {
            (
                v[0].as_i64().unwrap() as i32,
                v[1].as_i64().unwrap() as i32,
                v[2].as_i64().unwrap() as i32,
                v[3].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let mut parsed = parsed;
    parsed.sort();

    assert_eq!(
        parsed, from_flatten,
        "instance_blocks_json disagrees with flatten's inst:u0 region"
    );
    assert!(!parsed.is_empty(), "the instance must have blocks");

    // Unknown names are errors; an unrouted bus is legitimately empty.
    assert!(d.instance_blocks_json("nope").is_err());
    assert!(d.bus_blocks_json("nope").is_err());
}

#[test]
fn a_promoted_design_survives_a_nucm_round_trip_and_stays_reversible() {
    let mut d = design();
    let shipped = blocks(&d);
    d.promote_input("u0", "d").unwrap();

    let bytes = d.to_nucm_bytes().expect("nucm export");
    let mut back = Design::from_nucm_bytes(&bytes).expect("nucm import");
    assert_eq!(back.port_mode("u0", "d"), PortMode::Bus);

    // The reversible half has to survive the file too: demoting a reloaded
    // design must restore the cell exactly as shipped.
    back.set_port_mode("u0", "d", PortMode::Executor).unwrap();
    assert_eq!(
        blocks(&back),
        shipped,
        "demoting after a .nucm round trip must restore the shipped hardware"
    );
}
