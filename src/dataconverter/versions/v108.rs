//! V108 (15w32c + 4) — schematic-relevant subset of `V108.java`.
//!
//! VERSION = MCVersions.V15W32C (104) + 4 = 108.
//!
//! Ported: the ENTITY structure converter (V108.java:19-42) that turns a legacy
//! String `UUID` field into the `UUIDMost`/`UUIDLeast` long pair. It parses the
//! string, removes `UUID`, and sets `UUIDMost = mostSignificantBits` /
//! `UUIDLeast = leastSignificantBits`. If `UUID` is absent, or parsing fails,
//! the converter is a no-op (Java returns null without further mutation, though
//! note Java still removes `UUID` before attempting the parse — replicated here).

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 108;

/// Port of `java.util.UUID.fromString` -> (mostSignificantBits, leastSignificantBits).
///
/// Mirrors the JDK behaviour: split on '-' into exactly 5 components, parse each
/// as signed hex (`Long.parseLong(part, 16)`), then recombine. Returns `None`
/// when the shape is wrong or any component fails to parse, which corresponds to
/// Java's `UUID.fromString` throwing (caught -> return null in the converter).
fn parse_uuid(s: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    // java.util.UUID.fromString: group widths are 32/16/16 (most) and 16/48
    // (least); each group is masked to its field width then shifted into place.
    let p = |hex: &str| u64::from_str_radix(hex, 16).ok();
    let mut most = p(parts[0])? & 0xffff_ffff;
    most <<= 16;
    most |= p(parts[1])? & 0xffff;
    most <<= 16;
    most |= p(parts[2])? & 0xffff;

    let mut least = p(parts[3])? & 0xffff;
    least <<= 48;
    least |= p(parts[4])? & 0xffff_ffff_ffff;

    Some((most as i64, least as i64))
}

/// Inverse of `parse_uuid`: port of `java.util.UUID.toString()`.
///
/// JDK formats each field as lowercase hex zero-padded to its width and joins
/// with '-': 8-4-4-4-12 (most: bits 63..32 / 31..16 / 15..0; least: 63..48 /
/// 47..0). This is the exact preimage of the forward split, so the round trip is
/// lossless for any pair that the forward could have produced.
fn format_uuid(most: i64, least: i64) -> String {
    let most = most as u64;
    let least = least as u64;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (most >> 32) & 0xffff_ffff,
        (most >> 16) & 0xffff,
        most & 0xffff,
        (least >> 48) & 0xffff,
        least & 0xffff_ffff_ffff,
    )
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let uuid_string = match data.get_string("UUID") {
                Some(s) => s.to_string(),
                None => return,
            };
            data.take("UUID");

            if let Some((most, least)) = parse_uuid(&uuid_string) {
                data.set_i64("UUIDMost", most);
                data.set_i64("UUIDLeast", least);
            }
        }),
    );

    // Reverse of V108.java:19-42 — recombine the UUIDMost/UUIDLeast long pair
    // back into the canonical String `UUID` (JDK UUID.toString layout). Exact
    // inverse of the forward split, so this is lossless (bucket B). Only acts
    // when both halves are present (mirrors the forward only emitting both).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let most = match data.get_i64("UUIDMost") {
                Some(v) => v,
                None => return,
            };
            let least = match data.get_i64("UUIDLeast") {
                Some(v) => v,
                None => return,
            };
            data.take("UUIDMost");
            data.take("UUIDLeast");
            data.set_string("UUID", format_uuid(most, least));
        }),
    );
}
