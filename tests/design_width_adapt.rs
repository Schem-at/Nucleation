//! WIDTH ADAPTATION: a narrower word driving a wider port is a layout
//! question, not an error.
//!
//! User report, verbatim: "width mismatch: u1.bcd_hundreds[2] → u3.bcd[4] Can
//! we make that work and align for the msb and maybe support shifting?" Yes:
//! 2 or 3 hundreds bits genuinely fit inside a 4-bit `bcd` input, and the only
//! decision is which of the 4 they drive.
//!
//! The destination bits nothing drives need NO HARDWARE — undriven dust is
//! logical 0. That is asserted in-sim here
//! (`an_undriven_promoted_input_reads_zero`) rather than assumed, because the
//! whole narrow-to-wide story rests on it.

#![cfg(feature = "routing")]

use nucleation::design::{BusAlign, BusState, BusStyle, Design, WidthAdapt};
use nucleation::io_contract::IoType;
use nucleation::UniversalSchematic;

type P3 = (i32, i32, i32);

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

fn ty(w: u8) -> IoType {
    IoType::UnsignedInt { bits: w as usize }
}

fn lever_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, w: u8) -> P3 {
    for i in 0..w as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + 1, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x + 1, y, z, DUST).unwrap();
    }
    (x + 1, y0, z)
}

fn lamp_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, w: u8) -> P3 {
    for i in 0..w as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, y0, z)
}

/// A design with a `wd`-bit driver and a `ws`-bit sink, 32 apart in x.
fn mismatched(wd: u8, ws: u8) -> Design {
    let mut s = UniversalSchematic::new("wa".to_string());
    let drv = lever_bank(&mut s, 0, 2, 8, wd);
    let snk = lamp_bank(&mut s, 32, 2, 8, ws);
    let mut d = Design::for_schematic("wa", s);
    d.declare_input("din", drv, (0, 2, 0), wd, ty(wd)).unwrap();
    d.declare_output("dout", snk, (0, 2, 0), ws, ty(ws))
        .unwrap();
    d
}

fn map_of(d: &Design, bus: &str) -> nucleation::design::WidthMap {
    d.bus(bus)
        .unwrap()
        .width_map
        .clone()
        .unwrap_or_else(|| panic!("bus `{bus}` recorded no width map"))
}

/// Which sink bit each driver bit reaches, read back out of the FRAGMENT: the
/// routed stack must actually land on the mapped cells.
fn assert_route_lands_on_mapped_bits(d: &Design, bus: &str) {
    let m = map_of(d, bus);
    let layer = d.bus(bus).unwrap();
    let drv = d.resolve_port(&layer.driver).unwrap();
    let snk = d.resolve_port(&layer.sinks[0]).unwrap();
    let is_dust = |p: &P3| {
        layer
            .fragment
            .get(p)
            .is_some_and(|b| b.contains("redstone_wire") || b.contains("repeater"))
    };
    for i in 0..m.bits {
        let dbit = m.from_bit + i;
        let sbit = (dbit as i32 + m.shift) as u8;
        // The planner lays cells strictly BETWEEN anchors, so each end's mapped
        // wire must have a fragment cell orthogonally beside it.
        for (label, anchor) in [("driver", drv.wire(dbit)), ("sink", snk.wire(sbit))] {
            let touched = [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)]
                .iter()
                .any(|(dx, dy, dz)| is_dust(&(anchor.0 + dx, anchor.1 + dy, anchor.2 + dz)));
            assert!(
                touched,
                "{label} bit {dbit}->{sbit}: nothing in the fragment reaches {anchor:?}"
            );
        }
    }
    // A tied-zero sink bit must have NOTHING routed to it.
    for z in &m.tied_zero {
        let anchor = snk.wire(*z);
        let touched = [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)]
            .iter()
            .any(|(dx, dy, dz)| is_dust(&(anchor.0 + dx, anchor.1 + dy, anchor.2 + dz)));
        assert!(
            !touched,
            "sink bit {z} is reported as undriven but the bus reaches it at {anchor:?}"
        );
    }
}

// ----------------------------------------------------------------------
// The four policies
// ----------------------------------------------------------------------

/// LSB alignment is the DEFAULT, and it is what plain `route_bus` now does
/// instead of refusing: 3 bits into 4, bit 0 to bit 0, top bit reads 0.
#[test]
fn narrow_to_wide_routes_lsb_aligned_by_default() {
    let mut d = mismatched(3, 4);
    let st = d
        .route_bus("b", "din", &["dout"], vec![], BusStyle::default())
        .expect("a width mismatch must no longer be refused");
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("b"));
    let m = map_of(&d, "b");
    assert_eq!((m.driver_width, m.sink_width, m.shift), (3, 4, 0));
    assert_eq!((m.from_bit, m.bits), (0, 3));
    assert_eq!(m.tied_zero, vec![3], "the unused sink bit is the TOP one");
    assert!(m.dropped.is_empty());
    assert_route_lands_on_mapped_bits(&d, "b");
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);
}

/// MSB alignment: the user's `bcd_hundreds[2] -> bcd[4]` case. Top bit to top
/// bit, so the source is shifted up by the width difference and the LOW bits
/// read 0.
#[test]
fn msb_alignment_shifts_the_word_up_by_the_width_difference() {
    let mut d = mismatched(2, 4);
    let st = d
        .route_bus_adapted(
            "b",
            "din",
            &["dout"],
            vec![],
            BusStyle::default(),
            WidthAdapt::msb(),
        )
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("b"));
    let m = map_of(&d, "b");
    assert_eq!(m.shift, 2, "msb alignment of 2 into 4 shifts by 2");
    assert_eq!((m.from_bit, m.bits), (0, 2));
    assert_eq!(
        m.tied_zero,
        vec![0, 1],
        "the LOW sink bits are the spare ones"
    );
    // driver bit 0 -> sink bit 2, driver bit 1 -> sink bit 3
    assert!(
        m.to_json().contains("[0,2]") && m.to_json().contains("[1,3]"),
        "{}",
        m.to_json()
    );
    assert_route_lands_on_mapped_bits(&d, "b");
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);

    // And the reported sentence says what it did, in the user's terms.
    let note = d.bus_width_map_json("b").unwrap();
    assert!(note.contains("msb") || note.contains("shifted 2"), "{note}");
    assert!(note.contains("left undriven"), "{note}");
}

/// An explicit shift places the word anywhere in the destination.
#[test]
fn an_explicit_shift_places_the_word_where_asked() {
    for (wd, ws, sh, from, bits) in [(2u8, 6u8, 1i32, 0u8, 2u8), (3, 8, 4, 0, 3), (2, 5, 3, 0, 2)] {
        let mut d = mismatched(wd, ws);
        let st = d
            .route_bus_adapted(
                "b",
                "din",
                &["dout"],
                vec![],
                BusStyle::default(),
                WidthAdapt::shift(sh),
            )
            .unwrap_or_else(|e| panic!("shift {sh} of {wd} into {ws}: {e}"));
        assert_eq!(st, BusState::Routed, "shift {sh}: {:?}", d.bus_state("b"));
        let m = map_of(&d, "b");
        assert_eq!((m.shift, m.from_bit, m.bits), (sh, from, bits));
        assert_eq!(
            m.tied_zero.len(),
            (ws - bits) as usize,
            "every unmapped sink bit must be reported"
        );
        assert_route_lands_on_mapped_bits(&d, "b");
        assert!(
            d.check().unwrap().clean,
            "shift {sh}: {}",
            d.check().unwrap().json
        );
    }
}

/// WIDE -> NARROW is lossy, so it is refused unless asked for explicitly — and
/// the refusal names the bits that would be lost.
#[test]
fn wide_to_narrow_needs_truncate_and_says_which_bits_go() {
    let mut d = mismatched(8, 4);
    let err = d
        .route_bus("b", "din", &["dout"], vec![], BusStyle::default())
        .unwrap_err();
    assert!(err.contains("drop bits 4..7"), "{err}");
    assert!(
        err.contains("truncate"),
        "the refusal must name the way out: {err}"
    );
    assert!(d.bus("b").is_none(), "the refused bus was created anyway");

    // With truncate: the low 4 bits connect and the high 4 are dropped.
    let st = d
        .route_bus_adapted(
            "b",
            "din",
            &["dout"],
            vec![],
            BusStyle::default(),
            WidthAdapt::lsb().truncating(),
        )
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("b"));
    let m = map_of(&d, "b");
    assert_eq!((m.from_bit, m.bits), (0, 4));
    assert_eq!(m.dropped, vec![4, 5, 6, 7]);
    assert!(m.tied_zero.is_empty(), "a 4-bit sink is fully driven");
    assert_route_lands_on_mapped_bits(&d, "b");
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);
    assert!(
        d.bus_width_map_json("b").unwrap().contains("TRUNCATED"),
        "truncation must be reported, not silent"
    );
}

/// A word shifted entirely off the destination is refused with the reason.
#[test]
fn a_word_shifted_off_the_end_is_refused() {
    let mut d = mismatched(4, 4);
    let err = d
        .route_bus_adapted(
            "b",
            "din",
            &["dout"],
            vec![],
            BusStyle::default(),
            WidthAdapt::shift(9).truncating(),
        )
        .unwrap_err();
    assert!(err.contains("entirely outside"), "{err}");
}

/// Adaptation is single-driver/single-sink: several sinks sharing one trunk
/// would each want their own mapping, so those still need a common width, and
/// the refusal says so.
#[test]
fn fanout_still_requires_one_common_width() {
    let mut s = UniversalSchematic::new("fan".to_string());
    let drv = lever_bank(&mut s, 0, 2, 8, 3);
    let s1 = lamp_bank(&mut s, 32, 2, 8, 4);
    let s2 = lamp_bank(&mut s, 20, 2, 28, 3);
    let mut d = Design::for_schematic("fan", s);
    d.declare_input("din", drv, (0, 2, 0), 3, ty(3)).unwrap();
    d.declare_output("o1", s1, (0, 2, 0), 4, ty(4)).unwrap();
    d.declare_output("o2", s2, (0, 2, 0), 3, ty(3)).unwrap();
    let err = d
        .route_bus("b", "din", &["o2", "o1"], vec![], BusStyle::default())
        .unwrap_err();
    assert!(err.contains("single driver and a single sink"), "{err}");
}

/// The mapping is part of the bus's INTENT, so it survives a reroute and a
/// document round trip — otherwise a reload would silently re-pair the bits.
#[test]
fn the_mapping_survives_a_reroute_and_a_round_trip() {
    let mut d = mismatched(2, 4);
    d.route_bus_adapted(
        "b",
        "din",
        &["dout"],
        vec![],
        BusStyle::default(),
        WidthAdapt::msb(),
    )
    .unwrap();
    let before = map_of(&d, "b");
    let geom = d.bus("b").unwrap().fragment.clone();

    d.rip("b").unwrap();
    assert_eq!(d.reroute("b").unwrap(), BusState::Routed);
    assert_eq!(map_of(&d, "b"), before, "the reroute lost the bit mapping");
    assert_eq!(
        d.bus("b").unwrap().fragment,
        geom,
        "the reroute moved the wiring"
    );

    let bytes = d.to_nucm_bytes().unwrap();
    let back = Design::from_nucm_bytes(&bytes).unwrap();
    assert_eq!(
        map_of(&back, "b"),
        before,
        "the round trip lost the bit mapping"
    );
    assert!(back.check().unwrap().clean);
}

// ----------------------------------------------------------------------
// In-sim: the premise, and one aligned case end to end
// ----------------------------------------------------------------------

/// THE PREMISE OF NARROW-TO-WIDE: a destination bit nothing drives reads 0, so
/// tying the spare bits costs no hardware at all. Verified, not assumed.
#[cfg(all(feature = "simulation", feature = "bridge", feature = "mc-tick"))]
#[test]
fn an_undriven_promoted_input_reads_zero() {
    use nucleation::io_contract::Value;
    use nucleation::simulation::typed_executor::BackendCircuitExecutor;

    // A 4-bit sink whose TOP bit is deliberately left unwired.
    let mut d = mismatched(3, 4);
    assert_eq!(
        d.route_bus("b", "din", &["dout"], vec![], BusStyle::default())
            .unwrap(),
        BusState::Routed
    );
    assert_eq!(map_of(&d, "b").tied_zero, vec![3]);

    let baked = d.bake(4000).unwrap();
    let contract = baked.embedded_cell_contract().unwrap().unwrap();
    let extra = nucleation::design::executor_extra_states();
    let refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    let mut cell = BackendCircuitExecutor::for_cell(baked, &contract, &refs).unwrap();
    cell.settle(4000);
    let read = |c: &mut BackendCircuitExecutor| match c.read_output("dout").unwrap() {
        Value::U32(v) => v,
        other => panic!("unexpected {other:?}"),
    };
    // Every 3-bit input value: the top destination bit must stay 0 throughout,
    // so the 4-bit read equals the 3-bit word exactly.
    for v in 0..8u32 {
        cell.set_input("din", &Value::U32(v)).unwrap();
        cell.settle(800);
        let got = read(&mut cell);
        assert_eq!(
            got, v,
            "din={v}: the undriven sink bit 3 did not read 0 (got {got:#x}) — narrow-to-wide \
             adaptation would need real tie-down hardware"
        );
    }
}

/// One ALIGNED case end to end: MSB alignment of 2 into 4 must multiply by 4.
#[cfg(all(feature = "simulation", feature = "bridge", feature = "mc-tick"))]
#[test]
fn an_msb_aligned_bus_delivers_the_shifted_word_in_sim() {
    use nucleation::io_contract::Value;
    use nucleation::simulation::typed_executor::BackendCircuitExecutor;

    let mut d = mismatched(2, 4);
    assert_eq!(
        d.route_bus_adapted(
            "b",
            "din",
            &["dout"],
            vec![],
            BusStyle::default(),
            WidthAdapt::msb()
        )
        .unwrap(),
        BusState::Routed
    );
    let baked = d.bake(4000).unwrap();
    let contract = baked.embedded_cell_contract().unwrap().unwrap();
    let extra = nucleation::design::executor_extra_states();
    let refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    let mut cell = BackendCircuitExecutor::for_cell(baked, &contract, &refs).unwrap();
    cell.settle(4000);
    for v in 0..4u32 {
        cell.set_input("din", &Value::U32(v)).unwrap();
        cell.settle(800);
        let got = match cell.read_output("dout").unwrap() {
            Value::U32(x) => x,
            other => panic!("unexpected {other:?}"),
        };
        assert_eq!(
            got,
            v << 2,
            "din={v}: msb-aligning 2 bits into 4 is a shift up by 2, so the sink must read {}",
            v << 2
        );
    }
}

/// The `align` values the bridge takes, exercised through the Rust enum so the
/// numbering the bindings document cannot drift from the behaviour.
#[test]
fn the_three_alignments_are_distinct() {
    let (wd, ws) = (2u8, 5u8);
    let mut shifts = Vec::new();
    for adapt in [
        WidthAdapt {
            align: BusAlign::Lsb,
            truncate: false,
        },
        WidthAdapt {
            align: BusAlign::Msb,
            truncate: false,
        },
        WidthAdapt {
            align: BusAlign::Shift(1),
            truncate: false,
        },
    ] {
        let mut d = mismatched(wd, ws);
        d.route_bus_adapted("b", "din", &["dout"], vec![], BusStyle::default(), adapt)
            .unwrap();
        shifts.push(map_of(&d, "b").shift);
    }
    assert_eq!(shifts, vec![0, 3, 1], "lsb / msb / shift(1) must differ");
}
