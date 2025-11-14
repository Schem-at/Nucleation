//! Standard Unicode Palettes for Redstone Circuits
//!
//! This module provides pre-defined palettes using Unicode characters
//! to represent common redstone components, making circuit design more
//! visual and intuitive.

use std::collections::HashMap;

/// Standard redstone component palette using Unicode characters
pub struct StandardPalette;

impl StandardPalette {
    /// Get the complete standard palette as a HashMap
    pub fn get() -> HashMap<char, String> {
        let mut palette = HashMap::new();

        // ============================================================================
        // REDSTONE WIRE (Box Drawing Characters)
        // ============================================================================

        // Cross (all 4 directions)
        palette.insert(
            '╋',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=side,south=side]"
                .to_string(),
        );
        palette.insert(
            '┼',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=side,south=side]"
                .to_string(),
        );

        // Horizontal wire
        palette.insert(
            '─',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]"
                .to_string(),
        );
        palette.insert(
            '═',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]"
                .to_string(),
        );

        // Vertical wire (in XZ plane, north-south)
        palette.insert(
            '│',
            "minecraft:redstone_wire[power=0,east=none,west=none,north=side,south=side]"
                .to_string(),
        );
        palette.insert(
            '║',
            "minecraft:redstone_wire[power=0,east=none,west=none,north=side,south=side]"
                .to_string(),
        );

        // Corners
        palette.insert(
            '┌',
            "minecraft:redstone_wire[power=0,east=side,west=none,north=none,south=side]"
                .to_string(),
        );
        palette.insert(
            '┐',
            "minecraft:redstone_wire[power=0,east=none,west=side,north=none,south=side]"
                .to_string(),
        );
        palette.insert(
            '└',
            "minecraft:redstone_wire[power=0,east=side,west=none,north=side,south=none]"
                .to_string(),
        );
        palette.insert(
            '┘',
            "minecraft:redstone_wire[power=0,east=none,west=side,north=side,south=none]"
                .to_string(),
        );

        // T-junctions
        palette.insert(
            '├',
            "minecraft:redstone_wire[power=0,east=side,west=none,north=side,south=side]"
                .to_string(),
        );
        palette.insert(
            '┤',
            "minecraft:redstone_wire[power=0,east=none,west=side,north=side,south=side]"
                .to_string(),
        );
        palette.insert(
            '┬',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=side]"
                .to_string(),
        );
        palette.insert(
            '┴',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=side,south=none]"
                .to_string(),
        );

        // ============================================================================
        // REPEATERS (Arrows)
        // ============================================================================
        // Note: Arrow direction shows signal flow, but 'facing' is opposite (where repeater points)

        // Repeaters (unlocked, 1 tick delay)
        palette.insert(
            '→',
            "minecraft:repeater[facing=west,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '←',
            "minecraft:repeater[facing=east,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '↑',
            "minecraft:repeater[facing=south,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '↓',
            "minecraft:repeater[facing=north,delay=1,locked=false,powered=false]".to_string(),
        );

        // Repeaters (2 tick delay) - using double arrows
        palette.insert(
            '⇒',
            "minecraft:repeater[facing=west,delay=2,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⇐',
            "minecraft:repeater[facing=east,delay=2,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⇑',
            "minecraft:repeater[facing=south,delay=2,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⇓',
            "minecraft:repeater[facing=north,delay=2,locked=false,powered=false]".to_string(),
        );

        // Repeaters (3 tick delay) - using heavy arrows
        palette.insert(
            '➡',
            "minecraft:repeater[facing=west,delay=3,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⬅',
            "minecraft:repeater[facing=east,delay=3,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⬆',
            "minecraft:repeater[facing=south,delay=3,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⬇',
            "minecraft:repeater[facing=north,delay=3,locked=false,powered=false]".to_string(),
        );

        // Repeaters (4 tick delay) - using outlined arrows
        palette.insert(
            '⇨',
            "minecraft:repeater[facing=west,delay=4,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⇦',
            "minecraft:repeater[facing=east,delay=4,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⇧',
            "minecraft:repeater[facing=south,delay=4,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '⇩',
            "minecraft:repeater[facing=north,delay=4,locked=false,powered=false]".to_string(),
        );

        // ============================================================================
        // COMPARATORS (Triangle/Chevron Arrows)
        // ============================================================================
        // Note: Arrow direction shows signal flow, but 'facing' is opposite (where comparator points)

        // Comparators (compare mode)
        palette.insert(
            '▷',
            "minecraft:comparator[facing=west,mode=compare,powered=false]".to_string(),
        );
        palette.insert(
            '◁',
            "minecraft:comparator[facing=east,mode=compare,powered=false]".to_string(),
        );
        palette.insert(
            '△',
            "minecraft:comparator[facing=south,mode=compare,powered=false]".to_string(),
        );
        palette.insert(
            '▽',
            "minecraft:comparator[facing=north,mode=compare,powered=false]".to_string(),
        );

        // Comparators (subtract mode) - using filled triangles
        palette.insert(
            '▶',
            "minecraft:comparator[facing=west,mode=subtract,powered=false]".to_string(),
        );
        palette.insert(
            '◀',
            "minecraft:comparator[facing=east,mode=subtract,powered=false]".to_string(),
        );
        palette.insert(
            '▲',
            "minecraft:comparator[facing=south,mode=subtract,powered=false]".to_string(),
        );
        palette.insert(
            '▼',
            "minecraft:comparator[facing=north,mode=subtract,powered=false]".to_string(),
        );

        // ============================================================================
        // TORCHES
        // ============================================================================

        // Redstone torch (on block)
        palette.insert('🔥', "minecraft:redstone_torch[lit=true]".to_string());
        palette.insert('⚡', "minecraft:redstone_torch[lit=true]".to_string());
        palette.insert('*', "minecraft:redstone_torch[lit=true]".to_string());

        // Redstone torch (off)
        palette.insert('○', "minecraft:redstone_torch[lit=false]".to_string());

        // Wall torches
        palette.insert(
            '🡆',
            "minecraft:redstone_wall_torch[facing=east,lit=true]".to_string(),
        );
        palette.insert(
            '🡄',
            "minecraft:redstone_wall_torch[facing=west,lit=true]".to_string(),
        );
        palette.insert(
            '🡅',
            "minecraft:redstone_wall_torch[facing=north,lit=true]".to_string(),
        );
        palette.insert(
            '🡇',
            "minecraft:redstone_wall_torch[facing=south,lit=true]".to_string(),
        );

        // ============================================================================
        // BLOCKS
        // ============================================================================

        // Solid blocks (common materials)
        palette.insert('█', "minecraft:gray_concrete".to_string());
        palette.insert('▓', "minecraft:stone".to_string());
        palette.insert('▒', "minecraft:cobblestone".to_string());
        palette.insert('░', "minecraft:glass".to_string());

        // Specific colors
        palette.insert('■', "minecraft:black_concrete".to_string());
        palette.insert('□', "minecraft:white_concrete".to_string());
        palette.insert('▪', "minecraft:gray_concrete".to_string());
        palette.insert('▫', "minecraft:light_gray_concrete".to_string());

        // Common shorthand
        palette.insert('c', "minecraft:gray_concrete".to_string()); // 'c' for concrete (very common)

        // ============================================================================
        // SPECIAL
        // ============================================================================

        // Air/empty
        palette.insert('_', "minecraft:air".to_string());
        palette.insert(' ', "minecraft:air".to_string());
        palette.insert('·', "minecraft:air".to_string());

        // Redstone block
        palette.insert('⬛', "minecraft:redstone_block".to_string());
        palette.insert('R', "minecraft:redstone_block".to_string());

        // Lever (off)
        palette.insert(
            '⎓',
            "minecraft:lever[face=floor,facing=north,powered=false]".to_string(),
        );

        // Button (stone, unpressed)
        palette.insert(
            '●',
            "minecraft:stone_button[face=floor,facing=north,powered=false]".to_string(),
        );

        palette
    }

    /// Get a minimal palette with just the essentials
    pub fn minimal() -> HashMap<char, String> {
        let mut palette = HashMap::new();

        // Wire
        palette.insert(
            '─',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]"
                .to_string(),
        );
        palette.insert(
            '│',
            "minecraft:redstone_wire[power=0,east=none,west=none,north=side,south=side]"
                .to_string(),
        );
        palette.insert(
            '╋',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=side,south=side]"
                .to_string(),
        );

        // Repeaters (arrow shows signal flow, facing is opposite)
        palette.insert(
            '→',
            "minecraft:repeater[facing=west,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '←',
            "minecraft:repeater[facing=east,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '↑',
            "minecraft:repeater[facing=south,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '↓',
            "minecraft:repeater[facing=north,delay=1,locked=false,powered=false]".to_string(),
        );

        // Comparators (arrow shows signal flow, facing is opposite)
        palette.insert(
            '▷',
            "minecraft:comparator[facing=west,mode=compare,powered=false]".to_string(),
        );
        palette.insert(
            '◁',
            "minecraft:comparator[facing=east,mode=compare,powered=false]".to_string(),
        );
        palette.insert(
            '△',
            "minecraft:comparator[facing=south,mode=compare,powered=false]".to_string(),
        );
        palette.insert(
            '▽',
            "minecraft:comparator[facing=north,mode=compare,powered=false]".to_string(),
        );

        // Torch
        palette.insert('*', "minecraft:redstone_torch[lit=true]".to_string());

        // Block
        palette.insert('█', "minecraft:gray_concrete".to_string());

        // Air
        palette.insert('_', "minecraft:air".to_string());

        palette
    }

    /// Get a palette optimized for compact circuits
    pub fn compact() -> HashMap<char, String> {
        let mut palette = HashMap::new();

        // Use single-width characters only
        palette.insert(
            '-',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]"
                .to_string(),
        );
        palette.insert(
            '|',
            "minecraft:redstone_wire[power=0,east=none,west=none,north=side,south=side]"
                .to_string(),
        );
        palette.insert(
            '+',
            "minecraft:redstone_wire[power=0,east=side,west=side,north=side,south=side]"
                .to_string(),
        );

        // Repeaters (arrow shows signal flow, facing is opposite)
        palette.insert(
            '>',
            "minecraft:repeater[facing=west,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '<',
            "minecraft:repeater[facing=east,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            '^',
            "minecraft:repeater[facing=south,delay=1,locked=false,powered=false]".to_string(),
        );
        palette.insert(
            'v',
            "minecraft:repeater[facing=north,delay=1,locked=false,powered=false]".to_string(),
        );

        palette.insert('*', "minecraft:redstone_torch[lit=true]".to_string());
        palette.insert('#', "minecraft:gray_concrete".to_string());
        palette.insert('_', "minecraft:air".to_string());

        palette
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_palette() {
        let palette = StandardPalette::get();

        // Check wire characters
        assert!(palette.contains_key(&'╋'));
        assert!(palette.contains_key(&'─'));
        assert!(palette.contains_key(&'│'));

        // Check repeaters
        assert!(palette.contains_key(&'→'));
        assert!(palette.contains_key(&'←'));
        assert!(palette.contains_key(&'↑'));
        assert!(palette.contains_key(&'↓'));

        // Check comparators
        assert!(palette.contains_key(&'▷'));
        assert!(palette.contains_key(&'◁'));
        assert!(palette.contains_key(&'△'));
        assert!(palette.contains_key(&'▽'));

        // Verify they contain correct block types
        assert!(palette.get(&'╋').unwrap().contains("redstone_wire"));
        assert!(palette.get(&'→').unwrap().contains("repeater"));
        assert!(palette.get(&'▷').unwrap().contains("comparator"));
    }

    #[test]
    fn test_minimal_palette() {
        let palette = StandardPalette::minimal();
        assert!(palette.len() < 20);
        assert!(palette.contains_key(&'─'));
        assert!(palette.contains_key(&'→'));
    }

    #[test]
    fn test_compact_palette() {
        let palette = StandardPalette::compact();

        // All characters should be ASCII
        for ch in palette.keys() {
            assert!(ch.is_ascii(), "Character '{}' is not ASCII", ch);
        }
    }
}
