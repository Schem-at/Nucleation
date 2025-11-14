//! IO mapping - combines type and layout
//!
//! Maps a semantic type to physical redstone positions through a layout function.

use super::{IoType, LayoutFunction, Value};

/// Maps a semantic IO type to physical positions
#[derive(Debug, Clone)]
pub struct IoMapping {
    /// The semantic type (what the user sees)
    pub io_type: IoType,

    /// How bits are spread across positions
    pub layout: LayoutFunction,

    /// Physical redstone positions (x, y, z)
    pub positions: Vec<(i32, i32, i32)>,
}

impl IoMapping {
    /// Create a new IO mapping
    pub fn new(
        io_type: IoType,
        layout: LayoutFunction,
        positions: Vec<(i32, i32, i32)>,
    ) -> Result<Self, String> {
        let mapping = Self {
            io_type,
            layout,
            positions,
        };
        mapping.validate()?;
        Ok(mapping)
    }

    /// Validate that the mapping is consistent
    pub fn validate(&self) -> Result<(), String> {
        let bit_count = self.io_type.bit_count();
        let expected_positions = self.layout.position_count(bit_count);
        let actual_positions = self.positions.len();

        if expected_positions != actual_positions {
            return Err(format!(
                "Position count mismatch: type needs {} bits, layout needs {} positions, but {} positions provided",
                bit_count,
                expected_positions,
                actual_positions
            ));
        }

        Ok(())
    }

    /// Encode a value to nibbles (ready to set on physical positions)
    /// Pipeline: Value → Binary → Spread → Nibbles
    pub fn encode(&self, value: &Value) -> Result<Vec<u8>, String> {
        // Stage 1: Type → Binary
        let bits = self.io_type.to_binary(value)?;

        // Stage 2: Binary → Spread → Nibbles
        let nibbles = self.layout.spread_bits(&bits)?;

        // Validate
        if nibbles.len() != self.positions.len() {
            return Err(format!(
                "Nibble count mismatch: got {}, expected {}",
                nibbles.len(),
                self.positions.len()
            ));
        }

        Ok(nibbles)
    }

    /// Decode nibbles back to a value
    /// Pipeline: Nibbles → Collect → Binary → Value
    pub fn decode(&self, nibbles: &[u8]) -> Result<Value, String> {
        // Validate
        if nibbles.len() != self.positions.len() {
            return Err(format!(
                "Nibble count mismatch: got {}, expected {}",
                nibbles.len(),
                self.positions.len()
            ));
        }

        // Stage 1: Nibbles → Collect → Binary
        let bits = self.layout.collect_bits(nibbles)?;

        // Stage 2: Binary → Type
        let value = self.io_type.from_binary(&bits)?;

        Ok(value)
    }

    /// Get the number of positions needed
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_positions(count: usize) -> Vec<(i32, i32, i32)> {
        (0..count).map(|i| (i as i32, 0, 0)).collect()
    }

    #[test]
    fn test_mapping_validation() {
        // Valid mapping: 8-bit uint with OneToOne needs 8 positions
        let mapping = IoMapping::new(
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        );
        assert!(mapping.is_ok());

        // Invalid: wrong number of positions
        let mapping = IoMapping::new(
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(4), // Should be 8!
        );
        assert!(mapping.is_err());

        // Valid: 8-bit uint with Packed4 needs 2 positions
        let mapping = IoMapping::new(
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::Packed4,
            make_positions(2),
        );
        assert!(mapping.is_ok());
    }

    #[test]
    fn test_encode_decode_one_to_one() {
        let mapping = IoMapping::new(
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        )
        .unwrap();

        // Encode 42
        let value = Value::U32(42);
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles.len(), 8);

        // 42 = 0b00101010
        assert_eq!(nibbles, vec![0, 15, 0, 15, 0, 15, 0, 0]);

        // Decode back
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_packed4() {
        let mapping = IoMapping::new(
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::Packed4,
            make_positions(2),
        )
        .unwrap();

        // Encode 42 = 0x2A
        let value = Value::U32(42);
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles.len(), 2);

        // 42 = 0x2A = 0b00101010
        // Low nibble: 0b1010 = 10
        // High nibble: 0b0010 = 2
        assert_eq!(nibbles, vec![10, 2]);

        // Decode back
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_boolean() {
        let mapping =
            IoMapping::new(IoType::Boolean, LayoutFunction::OneToOne, make_positions(1)).unwrap();

        // Test true
        let value = Value::Bool(true);
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles, vec![15]);
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);

        // Test false
        let value = Value::Bool(false);
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles, vec![0]);
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_ascii() {
        let mapping = IoMapping::new(
            IoType::Ascii { chars: 5 },
            LayoutFunction::Packed4,
            make_positions(10), // 5 chars * 8 bits = 40 bits / 4 = 10 nibbles
        )
        .unwrap();

        let value = Value::String("Hello".to_string());
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles.len(), 10);

        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_signed_int_negative() {
        let mapping = IoMapping::new(
            IoType::SignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        )
        .unwrap();

        // Test -1 (all bits set)
        let value = Value::I32(-1);
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles, vec![15; 8]); // All ones

        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);

        // Test -42
        let value = Value::I32(-42);
        let nibbles = mapping.encode(&value).unwrap();
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_roundtrip_various_types() {
        let test_cases = vec![
            (
                IoType::UnsignedInt { bits: 16 },
                LayoutFunction::OneToOne,
                16,
                Value::U32(1234),
            ),
            (
                IoType::UnsignedInt { bits: 32 },
                LayoutFunction::Packed4,
                8,
                Value::U32(0xDEADBEEF),
            ),
            (
                IoType::SignedInt { bits: 16 },
                LayoutFunction::Packed4,
                4,
                Value::I32(-1000),
            ),
            (
                IoType::Boolean,
                LayoutFunction::OneToOne,
                1,
                Value::Bool(true),
            ),
            (
                IoType::Ascii { chars: 10 },
                LayoutFunction::Packed4,
                20,
                Value::String("Test123".to_string()),
            ),
        ];

        for (io_type, layout, pos_count, value) in test_cases {
            let mapping =
                IoMapping::new(io_type.clone(), layout, make_positions(pos_count)).unwrap();

            let nibbles = mapping.encode(&value).unwrap();
            let decoded = mapping.decode(&nibbles).unwrap();

            assert_eq!(decoded, value, "Roundtrip failed for {:?}", io_type);
        }
    }

    #[test]
    fn test_efficiency_comparison() {
        // 32-bit integer with different layouts
        let one_to_one = IoMapping::new(
            IoType::UnsignedInt { bits: 32 },
            LayoutFunction::OneToOne,
            make_positions(32),
        )
        .unwrap();

        let packed4 = IoMapping::new(
            IoType::UnsignedInt { bits: 32 },
            LayoutFunction::Packed4,
            make_positions(8),
        )
        .unwrap();

        let value = Value::U32(0x12345678);

        // Both should produce the same logical result
        let nibbles1 = one_to_one.encode(&value).unwrap();
        let nibbles2 = packed4.encode(&value).unwrap();

        assert_eq!(nibbles1.len(), 32);
        assert_eq!(nibbles2.len(), 8);

        // Decode should give same value
        let decoded1 = one_to_one.decode(&nibbles1).unwrap();
        let decoded2 = packed4.decode(&nibbles2).unwrap();

        assert_eq!(decoded1, value);
        assert_eq!(decoded2, value);
    }
}
