//! Layout functions for mapping bits to physical positions
//!
//! Defines how logical bits are spread across physical redstone positions (nibbles 0-15).

/// Defines how bits are mapped to physical redstone positions
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutFunction {
    /// One bit per position: false=0, true=15
    /// Most common for traditional redstone circuits
    /// Example: 8 bits → 8 positions
    OneToOne,

    /// Four bits per position (hex nibbles)
    /// More efficient packing
    /// Example: 8 bits → 2 positions
    Packed4,

    /// Custom bit-to-position mapping
    /// mapping[bit_index] = position_index
    /// Allows non-contiguous or reordered bits
    Custom(Vec<usize>),

    /// 2D row-major layout (for matrices/screens)
    /// Bits are laid out row by row
    RowMajor {
        rows: usize,
        cols: usize,
        bits_per_element: usize,
    },

    /// 2D column-major layout
    /// Bits are laid out column by column
    ColumnMajor {
        rows: usize,
        cols: usize,
        bits_per_element: usize,
    },

    /// Scanline layout (for screens)
    /// Pixels are laid out row by row, left to right
    Scanline {
        width: usize,
        height: usize,
        bits_per_pixel: usize,
    },

    /// Tiled layout (for large screens)
    /// Screen is divided into tiles, each tile is laid out in scanline order
    Tiled {
        tile_width: usize,
        tile_height: usize,
        tiles_x: usize,
        tiles_y: usize,
        bits_per_pixel: usize,
    },

    /// Chunked layout (for streaming data)
    /// Data is divided into fixed-size chunks
    Chunked {
        chunk_size: usize,
        num_chunks: usize,
    },
}

impl LayoutFunction {
    /// Calculate the number of physical positions (nibbles) needed
    pub fn position_count(&self, bit_count: usize) -> usize {
        match self {
            LayoutFunction::OneToOne => bit_count,
            LayoutFunction::Packed4 => (bit_count + 3) / 4,
            LayoutFunction::Custom(mapping) => mapping.len(),
            LayoutFunction::RowMajor {
                rows,
                cols,
                bits_per_element,
            } => {
                let total_bits = rows * cols * bits_per_element;
                (total_bits + 3) / 4 // Assuming packed4 for elements
            }
            LayoutFunction::ColumnMajor {
                rows,
                cols,
                bits_per_element,
            } => {
                let total_bits = rows * cols * bits_per_element;
                (total_bits + 3) / 4
            }
            LayoutFunction::Scanline {
                width,
                height,
                bits_per_pixel,
            } => {
                let total_bits = width * height * bits_per_pixel;
                (total_bits + 3) / 4
            }
            LayoutFunction::Tiled {
                tile_width,
                tile_height,
                tiles_x,
                tiles_y,
                bits_per_pixel,
            } => {
                let total_pixels = tile_width * tile_height * tiles_x * tiles_y;
                let total_bits = total_pixels * bits_per_pixel;
                (total_bits + 3) / 4
            }
            LayoutFunction::Chunked {
                chunk_size,
                num_chunks,
            } => {
                let total_bits = chunk_size * num_chunks;
                (total_bits + 3) / 4
            }
        }
    }

    /// Spread bits across physical positions (as nibbles 0-15)
    pub fn spread_bits(&self, bits: &[bool]) -> Result<Vec<u8>, String> {
        match self {
            LayoutFunction::OneToOne => {
                // 1 bit per nibble: false=0, true=15
                Ok(bits.iter().map(|&b| if b { 15 } else { 0 }).collect())
            }

            LayoutFunction::Packed4 => {
                // 4 bits per nibble
                let mut nibbles = Vec::with_capacity((bits.len() + 3) / 4);
                for chunk in bits.chunks(4) {
                    let nibble = chunk
                        .iter()
                        .enumerate()
                        .fold(0u8, |acc, (i, &bit)| acc | (if bit { 1u8 << i } else { 0 }));
                    nibbles.push(nibble);
                }
                Ok(nibbles)
            }

            LayoutFunction::Custom(mapping) => {
                // Custom bit-to-position mapping
                if mapping.len() != bits.len() {
                    return Err(format!(
                        "Custom mapping length ({}) doesn't match bit count ({})",
                        mapping.len(),
                        bits.len()
                    ));
                }

                let max_pos = mapping.iter().max().copied().unwrap_or(0);
                let mut nibbles = vec![0u8; max_pos + 1];

                for (bit_idx, &pos_idx) in mapping.iter().enumerate() {
                    nibbles[pos_idx] = if bits[bit_idx] { 15 } else { 0 };
                }

                Ok(nibbles)
            }

            LayoutFunction::RowMajor { .. }
            | LayoutFunction::ColumnMajor { .. }
            | LayoutFunction::Scanline { .. }
            | LayoutFunction::Tiled { .. }
            | LayoutFunction::Chunked { .. } => {
                // All 2D/complex layouts use packed4 encoding
                // The layout just defines the logical structure
                self.spread_bits_packed4(bits)
            }
        }
    }

    /// Collect nibbles back into bits
    pub fn collect_bits(&self, nibbles: &[u8]) -> Result<Vec<bool>, String> {
        match self {
            LayoutFunction::OneToOne => {
                // Any non-zero nibble = true
                Ok(nibbles.iter().map(|&n| n > 0).collect())
            }

            LayoutFunction::Packed4 => {
                // 4 bits per nibble
                let mut bits = Vec::with_capacity(nibbles.len() * 4);
                for &nibble in nibbles {
                    for i in 0..4 {
                        bits.push((nibble >> i) & 1 == 1);
                    }
                }
                Ok(bits)
            }

            LayoutFunction::Custom(mapping) => {
                // Reverse the custom mapping
                let mut bits = vec![false; mapping.len()];
                for (bit_idx, &pos_idx) in mapping.iter().enumerate() {
                    if pos_idx < nibbles.len() {
                        bits[bit_idx] = nibbles[pos_idx] > 0;
                    }
                }
                Ok(bits)
            }

            LayoutFunction::RowMajor { .. }
            | LayoutFunction::ColumnMajor { .. }
            | LayoutFunction::Scanline { .. }
            | LayoutFunction::Tiled { .. }
            | LayoutFunction::Chunked { .. } => {
                // All 2D/complex layouts use packed4 encoding
                self.collect_bits_packed4(nibbles)
            }
        }
    }

    // Helper methods
    fn spread_bits_packed4(&self, bits: &[bool]) -> Result<Vec<u8>, String> {
        let mut nibbles = Vec::with_capacity((bits.len() + 3) / 4);
        for chunk in bits.chunks(4) {
            let nibble = chunk
                .iter()
                .enumerate()
                .fold(0u8, |acc, (i, &bit)| acc | (if bit { 1u8 << i } else { 0 }));
            nibbles.push(nibble);
        }
        Ok(nibbles)
    }

    fn collect_bits_packed4(&self, nibbles: &[u8]) -> Result<Vec<bool>, String> {
        let mut bits = Vec::with_capacity(nibbles.len() * 4);
        for &nibble in nibbles {
            for i in 0..4 {
                bits.push((nibble >> i) & 1 == 1);
            }
        }
        Ok(bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_to_one_spread() {
        let layout = LayoutFunction::OneToOne;
        let bits = vec![true, false, true, false];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles, vec![15, 0, 15, 0]);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_packed4_spread() {
        let layout = LayoutFunction::Packed4;

        // Test 8 bits: 0b10101010 = 0xAA
        let bits = vec![false, true, false, true, false, true, false, true];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 2);
        // First nibble: bits 0-3 = 0b1010 = 10
        assert_eq!(nibbles[0], 0b1010);
        // Second nibble: bits 4-7 = 0b1010 = 10
        assert_eq!(nibbles[1], 0b1010);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_packed4_partial_nibble() {
        let layout = LayoutFunction::Packed4;

        // Test 5 bits (not a multiple of 4)
        let bits = vec![true, false, true, false, true];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 2);
        // First nibble: bits 0-3 = 0b0101 = 5
        assert_eq!(nibbles[0], 0b0101);
        // Second nibble: bit 4 only = 0b0001 = 1
        assert_eq!(nibbles[1], 0b0001);

        let collected = layout.collect_bits(&nibbles).unwrap();
        // Note: collected will have 8 bits (padded), not 5
        assert_eq!(collected[0..5], bits[..]);
    }

    #[test]
    fn test_custom_mapping() {
        // Map bits in reverse order: bit 0 → pos 3, bit 1 → pos 2, etc.
        let layout = LayoutFunction::Custom(vec![3, 2, 1, 0]);
        let bits = vec![true, false, true, false];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 4);
        assert_eq!(nibbles, vec![0, 15, 0, 15]); // Reversed

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_custom_mapping_sparse() {
        // Map to non-contiguous positions: bit 0 → pos 0, bit 1 → pos 5
        let layout = LayoutFunction::Custom(vec![0, 5]);
        let bits = vec![true, true];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 6); // 0 to 5
        assert_eq!(nibbles[0], 15);
        assert_eq!(nibbles[5], 15);
        assert_eq!(nibbles[1..5], vec![0, 0, 0, 0]); // Gaps are zeros
    }

    #[test]
    fn test_position_count() {
        assert_eq!(LayoutFunction::OneToOne.position_count(8), 8);
        assert_eq!(LayoutFunction::Packed4.position_count(8), 2);
        assert_eq!(LayoutFunction::Packed4.position_count(5), 2); // Rounds up
        assert_eq!(LayoutFunction::Custom(vec![0, 1, 2]).position_count(3), 3);
    }

    #[test]
    fn test_roundtrip_various_patterns() {
        let test_patterns = vec![
            vec![true, true, true, true],
            vec![false, false, false, false],
            vec![true, false, true, false],
            vec![false, true, false, true],
            vec![true, true, false, false],
        ];

        for bits in test_patterns {
            // Test OneToOne
            let layout = LayoutFunction::OneToOne;
            let nibbles = layout.spread_bits(&bits).unwrap();
            let collected = layout.collect_bits(&nibbles).unwrap();
            assert_eq!(collected, bits, "OneToOne roundtrip failed");

            // Test Packed4
            let layout = LayoutFunction::Packed4;
            let nibbles = layout.spread_bits(&bits).unwrap();
            let collected = layout.collect_bits(&nibbles).unwrap();
            assert_eq!(
                collected[0..bits.len()],
                bits[..],
                "Packed4 roundtrip failed"
            );
        }
    }

    #[test]
    fn test_efficiency_comparison() {
        let bit_count = 32;

        let one_to_one = LayoutFunction::OneToOne;
        let packed4 = LayoutFunction::Packed4;

        assert_eq!(one_to_one.position_count(bit_count), 32);
        assert_eq!(packed4.position_count(bit_count), 8);

        // Packed4 uses 4x fewer positions!
    }

    // =============================================================================
    // 2D LAYOUT TESTS
    // =============================================================================

    #[test]
    fn test_row_major_position_count() {
        let layout = LayoutFunction::RowMajor {
            rows: 3,
            cols: 4,
            bits_per_element: 4,
        };

        // 3 rows * 4 cols * 4 bits = 48 bits
        // Packed4: 48 / 4 = 12 nibbles
        assert_eq!(layout.position_count(48), 12);
    }

    #[test]
    fn test_row_major_spread_collect() {
        let layout = LayoutFunction::RowMajor {
            rows: 2,
            cols: 2,
            bits_per_element: 4,
        };

        // 2x2 matrix with 4 bits per element = 16 bits total
        // Elements: [0,0]=0xA, [0,1]=0xB, [1,0]=0xC, [1,1]=0xD
        let bits = vec![
            // Element [0,0] = 0xA = 0b1010
            false, true, false, true, // Element [0,1] = 0xB = 0b1011
            true, true, false, true, // Element [1,0] = 0xC = 0b1100
            false, false, true, true, // Element [1,1] = 0xD = 0b1101
            true, false, true, true,
        ];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 4); // 16 bits / 4 = 4 nibbles
        assert_eq!(nibbles, vec![0xA, 0xB, 0xC, 0xD]);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_column_major_position_count() {
        let layout = LayoutFunction::ColumnMajor {
            rows: 3,
            cols: 4,
            bits_per_element: 4,
        };

        // Same total bits as row major, different order
        assert_eq!(layout.position_count(48), 12);
    }

    #[test]
    fn test_column_major_spread_collect() {
        let layout = LayoutFunction::ColumnMajor {
            rows: 2,
            cols: 2,
            bits_per_element: 4,
        };

        // 2x2 matrix with 4 bits per element = 16 bits total
        // Column-major order: [0,0], [1,0], [0,1], [1,1]
        let bits = vec![
            // Element [0,0] = 0xA
            false, true, false, true, // Element [1,0] = 0xB
            true, true, false, true, // Element [0,1] = 0xC
            false, false, true, true, // Element [1,1] = 0xD
            true, false, true, true,
        ];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 4);
        assert_eq!(nibbles, vec![0xA, 0xB, 0xC, 0xD]);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_scanline_position_count() {
        let layout = LayoutFunction::Scanline {
            width: 16,
            height: 16,
            bits_per_pixel: 4,
        };

        // 16*16 = 256 pixels * 4 bits = 1024 bits
        // Packed4: 1024 / 4 = 256 nibbles
        assert_eq!(layout.position_count(1024), 256);
    }

    #[test]
    fn test_scanline_spread_collect() {
        let layout = LayoutFunction::Scanline {
            width: 4,
            height: 2,
            bits_per_pixel: 4,
        };

        // 4x2 pixels with 4 bits per pixel = 32 bits
        // Row 0: pixels 0,1,2,3
        // Row 1: pixels 4,5,6,7
        let bits = vec![
            // Row 0
            false, false, false, false, // Pixel 0 = 0x0
            true, false, false, false, // Pixel 1 = 0x1
            false, true, false, false, // Pixel 2 = 0x2
            true, true, false, false, // Pixel 3 = 0x3
            // Row 1
            false, false, true, false, // Pixel 4 = 0x4
            true, false, true, false, // Pixel 5 = 0x5
            false, true, true, false, // Pixel 6 = 0x6
            true, true, true, false, // Pixel 7 = 0x7
        ];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 8);
        assert_eq!(nibbles, vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_tiled_position_count() {
        let layout = LayoutFunction::Tiled {
            tile_width: 4,
            tile_height: 4,
            tiles_x: 2,
            tiles_y: 2,
            bits_per_pixel: 4,
        };

        // 2x2 tiles of 4x4 pixels each = 8x8 total pixels
        // 64 pixels * 4 bits = 256 bits
        // Packed4: 256 / 4 = 64 nibbles
        assert_eq!(layout.position_count(256), 64);
    }

    #[test]
    fn test_tiled_spread_collect() {
        let layout = LayoutFunction::Tiled {
            tile_width: 2,
            tile_height: 2,
            tiles_x: 2,
            tiles_y: 1,
            bits_per_pixel: 4,
        };

        // 2 tiles horizontally, each 2x2 = 8 pixels total
        // 8 pixels * 4 bits = 32 bits
        let bits = vec![
            // Tile 0 (left)
            false, false, false, false, // Pixel [0,0] = 0x0
            true, false, false, false, // Pixel [0,1] = 0x1
            false, true, false, false, // Pixel [1,0] = 0x2
            true, true, false, false, // Pixel [1,1] = 0x3
            // Tile 1 (right)
            false, false, true, false, // Pixel [0,0] = 0x4
            true, false, true, false, // Pixel [0,1] = 0x5
            false, true, true, false, // Pixel [1,0] = 0x6
            true, true, true, false, // Pixel [1,1] = 0x7
        ];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 8);
        assert_eq!(nibbles, vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_chunked_position_count() {
        let layout = LayoutFunction::Chunked {
            chunk_size: 16,
            num_chunks: 4,
        };

        // 4 chunks * 16 bits = 64 bits
        // Packed4: 64 / 4 = 16 nibbles
        assert_eq!(layout.position_count(64), 16);
    }

    #[test]
    fn test_chunked_spread_collect() {
        let layout = LayoutFunction::Chunked {
            chunk_size: 8,
            num_chunks: 2,
        };

        // 2 chunks of 8 bits each = 16 bits
        let bits = vec![
            // Chunk 0
            false, false, false, false, false, false, false, false, // 0x00
            // Chunk 1
            true, true, true, true, true, true, true, true, // 0xFF
        ];

        let nibbles = layout.spread_bits(&bits).unwrap();
        assert_eq!(nibbles.len(), 4); // 16 bits / 4 = 4 nibbles
        assert_eq!(nibbles, vec![0x0, 0x0, 0xF, 0xF]);

        let collected = layout.collect_bits(&nibbles).unwrap();
        assert_eq!(collected, bits);
    }

    #[test]
    fn test_2d_layouts_roundtrip() {
        // Test that all 2D layouts preserve data through roundtrip
        let test_bits = vec![
            true, false, true, false, true, false, true, false, false, true, false, true, false,
            true, false, true,
        ];

        let layouts = vec![
            LayoutFunction::RowMajor {
                rows: 2,
                cols: 2,
                bits_per_element: 4,
            },
            LayoutFunction::ColumnMajor {
                rows: 2,
                cols: 2,
                bits_per_element: 4,
            },
            LayoutFunction::Scanline {
                width: 4,
                height: 1,
                bits_per_pixel: 4,
            },
            LayoutFunction::Tiled {
                tile_width: 2,
                tile_height: 1,
                tiles_x: 2,
                tiles_y: 1,
                bits_per_pixel: 4,
            },
            LayoutFunction::Chunked {
                chunk_size: 8,
                num_chunks: 2,
            },
        ];

        for layout in layouts {
            let nibbles = layout.spread_bits(&test_bits).unwrap();
            let collected = layout.collect_bits(&nibbles).unwrap();
            assert_eq!(collected, test_bits, "Roundtrip failed for {:?}", layout);
        }
    }
}
