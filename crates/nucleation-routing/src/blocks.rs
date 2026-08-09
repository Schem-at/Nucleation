//! Block-state strings and property helpers (port of `rs.py`).
//!
//! Everything is plain vanilla block-state strings so the crate stays free
//! of schematic-format dependencies; the nucleation bridge converts.

/// Plain stone.
pub const STONE: &str = "minecraft:stone";

/// Redstone dust with the default state fully spelled out.
///
/// Authoring bare `minecraft:redstone_wire` interns a property-less state
/// that tick engines never normalise unless the cell catches a placement
/// update — such cells sit inert. Always spell the default out.
pub const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

/// Standing redstone torch (lit).
pub const TORCH: &str = "minecraft:redstone_torch[lit=true]";

/// Redstone lamp (off).
pub const LAMP: &str = "minecraft:redstone_lamp[lit=false]";

/// Floor lever, off.
pub const LEVER_OFF: &str = "minecraft:lever[face=floor,facing=north,powered=false]";

/// Substring hints that mark a block as a full solid conductor. Matches the
/// Python checkers: stone, lamps, and every concrete colour.
pub const SOLID_HINTS: [&str; 3] = ["minecraft:stone", "redstone_lamp", "concrete"];

/// Substring key identifying dust.
pub const DUST_KEY: &str = "redstone_wire";

/// Role colouring for structural blocks (purely cosmetic; concrete conducts
/// exactly like stone, but it makes a large build readable).
pub fn role_block(role: &str) -> &'static str {
    match role {
        "rail_floor" => "minecraft:gray_concrete",
        "lid" => "minecraft:light_blue_concrete",
        "tap" => "minecraft:red_concrete",
        "coll_floor" => "minecraft:lime_concrete",
        "gate" => "minecraft:orange_concrete",
        "lane" => "minecraft:yellow_concrete",
        "route" => "minecraft:magenta_concrete",
        "inv" => "minecraft:purple_concrete",
        _ => STONE,
    }
}

/// Whether a block string is a solid conductor.
///
/// ONE CLASSIFIER, NOT TWO. This used to be its own substring test over
/// [`SOLID_HINTS`], independent of [`crate::transport::classify`], and the two
/// drifted apart in the way that class of duplication always does: when
/// `classify` was taught that wool, planks and terracotta are solid — the fix
/// mattering because the verified library tiles are BUILT of wool, so their own
/// cut cells were reading as air — this copy was not, and every consumer of it
/// (`workspace::solid_at`, `nets`, `lvs`, `audit`, `fabric`, `wire`) kept the
/// old wrong answer. Delegating means a material fact can only be stated once.
pub fn is_solid_block(block: &str) -> bool {
    crate::transport::conducts(Some(block))
}

/// Whether dust/repeaters may SIT on this block (static support legality).
///
/// Sturdiness is not conductivity (the probed material model,
/// `redstone-eda/notes-material-model.md`): glass, stained glass and
/// top-half slabs are full-cube supports that conduct nothing — they are
/// exactly what a bus dip uses where a diagonal below must survive.
pub fn is_sturdy_support(block: &str) -> bool {
    crate::transport::sturdy(Some(block))
}

/// Whether a block string is redstone dust.
pub fn is_dust(block: &str) -> bool {
    block.contains(DUST_KEY)
}

/// Whether a block string is a repeater.
pub fn is_repeater(block: &str) -> bool {
    block.contains("minecraft:repeater")
}

/// Whether a block string is a comparator.
pub fn is_comparator(block: &str) -> bool {
    block.contains("minecraft:comparator")
}

/// Whether a block string is a standing or wall redstone torch.
pub fn is_torch(block: &str) -> bool {
    block.contains("redstone_torch") || block.contains("redstone_wall_torch")
}

/// Whether a block string is a lever.
pub fn is_lever(block: &str) -> bool {
    block.contains("minecraft:lever")
}

/// A repeater whose INPUT side is `input_from`; it outputs the opposite way.
/// Verified: `repeater[facing=west]` conducts toward +X.
pub fn repeater(input_from: &str, delay: u8) -> String {
    format!("minecraft:repeater[facing={input_from},delay={delay},locked=false,powered=false]")
}

/// A comparator whose INPUT (back) side faces `input_from`.
pub fn comparator(input_from: &str, mode: &str) -> String {
    format!("minecraft:comparator[facing={input_from},mode={mode},powered=false]")
}

/// A wall torch whose attachment block is one step OPPOSITE `face_dir`.
pub fn wall_torch(face_dir: &str, lit: bool) -> String {
    format!("minecraft:redstone_wall_torch[facing={face_dir},lit={lit}]")
}

/// Extract a `facing=` property value from a block string.
pub fn facing_of(block: &str) -> Option<&str> {
    let idx = block.find("facing=")?;
    let rest = &block[idx + "facing=".len()..];
    let end = rest.find([',', ']']).unwrap_or(rest.len());
    Some(&rest[..end])
}

/// The unit vector a facing name denotes: north = -z, south = +z,
/// east = +x, west = -x.
pub fn facing_vec(name: &str) -> Option<(i32, i32, i32)> {
    match name {
        "north" => Some((0, 0, -1)),
        "south" => Some((0, 0, 1)),
        "east" => Some((1, 0, 0)),
        "west" => Some((-1, 0, 0)),
        _ => None,
    }
}

/// Facing name for a horizontal step vector pointing AT the input (used when
/// the emitter inserts a repeater: the input side is the previous cell).
pub fn facing_name(dx: i32, dz: i32) -> Option<&'static str> {
    match (dx, dz) {
        (-1, 0) => Some("west"),
        (1, 0) => Some("east"),
        (0, -1) => Some("north"),
        (0, 1) => Some("south"),
        _ => None,
    }
}

/// The repeater delay in redstone ticks parsed from its state string.
pub fn repeater_delay(block: &str) -> u32 {
    block
        .find("delay=")
        .and_then(|i| {
            let rest = &block[i + "delay=".len()..];
            let end = rest.find([',', ']']).unwrap_or(rest.len());
            rest[..end].parse::<u32>().ok()
        })
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_parse_and_vectors() {
        let r = repeater("west", 1);
        assert_eq!(facing_of(&r), Some("west"));
        assert_eq!(facing_vec("west"), Some((-1, 0, 0)));
        assert_eq!(repeater_delay(&r), 1);
        assert_eq!(repeater_delay(&repeater("north", 3)), 3);
        assert!(is_repeater(&r) && !is_dust(&r));
        assert!(is_dust(DUST));
        assert!(is_solid_block("minecraft:magenta_concrete"));
        assert!(!is_solid_block(DUST));
    }

    /// The divergences an audit of this file against `materials.py` turned up.
    /// Every one was SILENT — a plausible wrong answer, never an error.
    #[test]
    fn the_two_classifiers_agree_on_what_a_support_is() {
        // The wool bug, which the tiles themselves are built out of. This file
        // knew three families; `classify` knew six and was fixed without it.
        for b in [
            "minecraft:red_wool",
            "minecraft:oak_planks",
            "minecraft:white_terracotta",
            "minecraft:smooth_stone_slab[type=double]",
        ] {
            assert!(is_solid_block(b), "{b} must be a solid conductor");
            assert!(is_sturdy_support(b), "{b} must support dust");
        }
        // Full cubes that used to fall through to "not a support, conducts
        // nothing" — the default that made a polished-andesite wall invisible
        // and sent `O03_flat_obstacle` on a detour around a block it could have
        // run straight over.
        for b in [
            "minecraft:polished_andesite",
            "minecraft:cobblestone",
            "minecraft:smooth_stone",
            "minecraft:stone_bricks",
            "minecraft:deepslate",
            "minecraft:iron_block",
            "minecraft:obsidian",
            "minecraft:oak_log[axis=y]",
        ] {
            assert!(is_solid_block(b), "{b} is a full cube and conducts");
        }
        // `contains("minecraft:stone")` called all of these solid conductors.
        for b in [
            "minecraft:stone_button[face=wall,facing=north,powered=false]",
            "minecraft:stone_pressure_plate[powered=false]",
            "minecraft:stone_stairs[facing=north,half=bottom,shape=straight]",
            "minecraft:stone_slab[type=bottom]",
        ] {
            assert!(!is_solid_block(b), "{b} is not a full cube");
            assert!(!is_sturdy_support(b), "{b} cannot host dust");
        }
        // Sturdy but NOT conducting: the distinction the crossing tiles run on.
        for b in ["minecraft:glass", "minecraft:lime_stained_glass"] {
            assert!(is_sturdy_support(b), "{b} supports dust");
            assert!(!is_solid_block(b), "{b} conducts nothing");
        }
    }
}
