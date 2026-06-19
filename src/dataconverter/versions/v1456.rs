//! V1456 (17w49b+1) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V1456.java`.
//!
//! Item frames stored a 2D `Facing` value (0..3) that must be remapped to the
//! 3D direction enum used afterwards: 0->3, 1->4, 2->2, 3->5 (default 2)
//! (V1456.java:12-33). Nothing non-schematic is present in this version.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1456;

/// `direction2dTo3d` (V1456.java:12-24): 0->3, 1->4, 2->2 (default), 3->5.
fn direction_2d_to_3d(old: i8) -> i8 {
    match old {
        0 => 3,
        1 => 4,
        3 => 5,
        _ => 2, // case 2 and default
    }
}

/// Inverse of `direction_2d_to_3d`: 3->0, 4->1, 2->2 (default), 5->3.
///
/// The forward maps the four 2D values {0,1,2,3} onto the horizontal subset of
/// the 3D direction enum {3(south),4(west),2(north),5(east)} bijectively, so a
/// genuine item-frame downgrade always carries a horizontal `Facing` and the
/// inverse is exact (the vertical 0(down)/1(up) values can never appear on a
/// frame that originated from a 2D source). Lossless.
fn direction_3d_to_2d(new: i8) -> (i8, bool) {
    match new {
        3 => (0, false),
        4 => (1, false),
        2 => (2, false),
        5 => (3, false),
        _ => (2, true),
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:item_frame",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // getByte("Facing") defaults to 0 when absent.
            let old = data.get_i64("Facing").unwrap_or(0) as i8;
            data.set_byte("Facing", direction_2d_to_3d(old));
        }),
    );

    // Reverse: restore the 2D `Facing` value the older format used.
    // Matches the NEW id ("minecraft:item_frame"; no id-rename here).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:item_frame",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let new = data.get_i64("Facing").unwrap_or(0) as i8;
            let (old, approximated) = direction_3d_to_2d(new);
            if approximated {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Approximated,
                    format!(
                        "item_frame Facing={new} is not representable in the pre-V1456 2D direction enum; restoring Facing=2"
                    ),
                );
            }
            data.set_byte("Facing", old);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_vertical_item_frame_facing_reports_approximation() {
        let mut frame = crate::nbt::NbtMap::new();
        frame.set_string("id", "minecraft:item_frame");
        frame.set_byte("Facing", 1);

        let report = convert_entity_reverse(&mut frame, 1456, 1455);

        assert_eq!(frame.get_i64("Facing"), Some(2));
        assert_eq!(report.len(), 1);
        assert_eq!(report.loss_count(), 0);
    }
}
