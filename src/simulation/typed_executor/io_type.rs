//! IO type definitions and conversions
//!
//! Defines the semantic types that can be used for circuit IO and provides
//! conversion functions to/from binary representations.

use super::value::Value;

/// Semantic type for circuit IO
#[derive(Debug, Clone, PartialEq)]
pub enum IoType {
    /// Unsigned integer with specified bit width
    UnsignedInt { bits: usize },

    /// Signed integer with specified bit width (two's complement)
    SignedInt { bits: usize },

    /// 32-bit IEEE 754 floating point
    Float32,

    /// Single boolean value
    Boolean,

    /// ASCII string with fixed character count
    Ascii { chars: usize },

    /// Array of elements of the same type
    Array {
        element_type: Box<IoType>,
        length: usize,
    },

    /// 2D matrix
    Matrix {
        element_type: Box<IoType>,
        rows: usize,
        cols: usize,
    },

    /// Pixel buffer for screens
    PixelBuffer {
        width: usize,
        height: usize,
        bits_per_pixel: usize,
    },

    /// Struct with named fields
    Struct { fields: Vec<(String, IoType)> },

    /// Raw bit array (no interpretation)
    BitArray { bits: usize },
}

impl IoType {
    /// Calculate the total number of bits needed for this type
    pub fn bit_count(&self) -> usize {
        match self {
            IoType::UnsignedInt { bits } => *bits,
            IoType::SignedInt { bits } => *bits,
            IoType::Float32 => 32,
            IoType::Boolean => 1,
            IoType::Ascii { chars } => chars * 8,
            IoType::Array {
                element_type,
                length,
            } => element_type.bit_count() * length,
            IoType::Matrix {
                element_type,
                rows,
                cols,
            } => element_type.bit_count() * rows * cols,
            IoType::PixelBuffer {
                width,
                height,
                bits_per_pixel,
            } => width * height * bits_per_pixel,
            IoType::Struct { fields } => fields.iter().map(|(_, t)| t.bit_count()).sum(),
            IoType::BitArray { bits } => *bits,
        }
    }

    /// Convert a value to a binary representation (bit array)
    pub fn to_binary(&self, value: &Value) -> Result<Vec<bool>, String> {
        match self {
            IoType::UnsignedInt { bits } => {
                let num = value.as_u64()?;
                Ok((0..*bits).map(|i| (num >> i) & 1 == 1).collect())
            }

            IoType::SignedInt { bits } => {
                let num = value.as_i64()?;
                let unsigned = num as u64;
                Ok((0..*bits).map(|i| (unsigned >> i) & 1 == 1).collect())
            }

            IoType::Float32 => {
                let f = value.as_f32()?;
                let bits_u32 = f.to_bits();
                Ok((0..32).map(|i| (bits_u32 >> i) & 1 == 1).collect())
            }

            IoType::Boolean => Ok(vec![value.as_bool()?]),

            IoType::Ascii { chars } => {
                let text = value.as_str()?;
                let mut bits = Vec::with_capacity(chars * 8);

                for ch in text.chars().take(*chars) {
                    let byte = ch as u8;
                    for i in 0..8 {
                        bits.push((byte >> i) & 1 == 1);
                    }
                }

                // Pad with zeros if string is shorter than expected
                bits.resize(chars * 8, false);
                Ok(bits)
            }

            IoType::BitArray {
                bits: expected_bits,
            } => {
                let bit_array = value.as_bit_array()?;
                if bit_array.len() != *expected_bits {
                    return Err(format!(
                        "Bit array length mismatch: expected {}, got {}",
                        expected_bits,
                        bit_array.len()
                    ));
                }
                Ok(bit_array.clone())
            }

            _ => Err(format!("to_binary not yet implemented for {:?}", self)),
        }
    }

    /// Convert a binary representation back to a value
    pub fn from_binary(&self, bits: &[bool]) -> Result<Value, String> {
        // Validate bit count
        let expected = self.bit_count();
        if bits.len() != expected {
            return Err(format!(
                "Bit count mismatch: expected {}, got {}",
                expected,
                bits.len()
            ));
        }

        match self {
            IoType::UnsignedInt { .. } => {
                let num = bits.iter().enumerate().fold(0u64, |acc, (i, &bit)| {
                    acc | (if bit { 1u64 << i } else { 0 })
                });

                // Return appropriate size
                if num <= u32::MAX as u64 {
                    Ok(Value::U32(num as u32))
                } else {
                    Ok(Value::U64(num))
                }
            }

            IoType::SignedInt { bits: bit_count } => {
                let unsigned = bits.iter().enumerate().fold(0u64, |acc, (i, &bit)| {
                    acc | (if bit { 1u64 << i } else { 0 })
                });

                // Sign extend if negative
                let sign_bit = 1u64 << (bit_count - 1);
                let num = if unsigned & sign_bit != 0 {
                    // Negative: sign extend
                    let mask = !((1u64 << bit_count) - 1);
                    (unsigned | mask) as i64
                } else {
                    unsigned as i64
                };

                // Return appropriate size
                if num >= i32::MIN as i64 && num <= i32::MAX as i64 {
                    Ok(Value::I32(num as i32))
                } else {
                    Ok(Value::I64(num))
                }
            }

            IoType::Float32 => {
                let bits_u32 = bits.iter().enumerate().fold(0u32, |acc, (i, &bit)| {
                    acc | (if bit { 1u32 << i } else { 0 })
                });
                Ok(Value::F32(f32::from_bits(bits_u32)))
            }

            IoType::Boolean => Ok(Value::Bool(bits.get(0).copied().unwrap_or(false))),

            IoType::Ascii { chars } => {
                let mut text = String::with_capacity(*chars);
                for chunk in bits.chunks(8) {
                    let byte = chunk
                        .iter()
                        .enumerate()
                        .fold(0u8, |acc, (i, &bit)| acc | (if bit { 1u8 << i } else { 0 }));
                    if byte != 0 {
                        text.push(byte as char);
                    }
                }
                Ok(Value::String(text))
            }

            IoType::BitArray { .. } => Ok(Value::BitArray(bits.to_vec())),

            _ => Err(format!("from_binary not yet implemented for {:?}", self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_count() {
        assert_eq!(IoType::UnsignedInt { bits: 32 }.bit_count(), 32);
        assert_eq!(IoType::SignedInt { bits: 16 }.bit_count(), 16);
        assert_eq!(IoType::Float32.bit_count(), 32);
        assert_eq!(IoType::Boolean.bit_count(), 1);
        assert_eq!(IoType::Ascii { chars: 10 }.bit_count(), 80);
    }

    #[test]
    fn test_unsigned_int_conversion() {
        let io_type = IoType::UnsignedInt { bits: 8 };

        // Test encoding
        let value = Value::U32(42);
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits.len(), 8);

        // 42 = 0b00101010
        assert_eq!(
            bits,
            vec![false, true, false, true, false, true, false, false]
        );

        // Test decoding
        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::U32(42));
    }

    #[test]
    fn test_signed_int_conversion() {
        let io_type = IoType::SignedInt { bits: 8 };

        // Test positive number
        let value = Value::I32(42);
        let bits = io_type.to_binary(&value).unwrap();
        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::I32(42));

        // Test negative number (-1 = 0xFF in 8-bit two's complement)
        let value = Value::I32(-1);
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits, vec![true; 8]); // All ones
        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::I32(-1));

        // Test -42
        let value = Value::I32(-42);
        let bits = io_type.to_binary(&value).unwrap();
        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::I32(-42));
    }

    #[test]
    fn test_boolean_conversion() {
        let io_type = IoType::Boolean;

        let value = Value::Bool(true);
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits, vec![true]);

        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::Bool(true));

        let value = Value::Bool(false);
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits, vec![false]);

        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::Bool(false));
    }

    #[test]
    fn test_ascii_conversion() {
        let io_type = IoType::Ascii { chars: 5 };

        // Test "Hello"
        let value = Value::String("Hello".to_string());
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits.len(), 40); // 5 chars * 8 bits

        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::String("Hello".to_string()));

        // Test shorter string (should pad with zeros)
        let value = Value::String("Hi".to_string());
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits.len(), 40);

        let decoded = io_type.from_binary(&bits).unwrap();
        assert_eq!(decoded, Value::String("Hi".to_string()));
    }

    #[test]
    fn test_float32_conversion() {
        let io_type = IoType::Float32;

        let value = Value::F32(3.14159);
        let bits = io_type.to_binary(&value).unwrap();
        assert_eq!(bits.len(), 32);

        let decoded = io_type.from_binary(&bits).unwrap();
        if let Value::F32(f) = decoded {
            assert!((f - 3.14159).abs() < 0.00001);
        } else {
            panic!("Expected F32");
        }
    }

    #[test]
    fn test_roundtrip_various_values() {
        // Test various bit widths
        for bits in [1, 4, 8, 16, 32] {
            let io_type = IoType::UnsignedInt { bits };
            let max_value = if bits == 32 {
                u32::MAX
            } else {
                (1u32 << bits) - 1
            };

            for test_val in [0, 1, max_value / 2, max_value] {
                let value = Value::U32(test_val);
                let binary = io_type.to_binary(&value).unwrap();
                let decoded = io_type.from_binary(&binary).unwrap();
                assert_eq!(
                    decoded, value,
                    "Failed roundtrip for {} bits, value {}",
                    bits, test_val
                );
            }
        }
    }

    // =============================================================================
    // COMPLEX TYPE TESTS
    // =============================================================================

    #[test]
    fn test_array_bit_count() {
        let io_type = IoType::Array {
            element_type: Box::new(IoType::UnsignedInt { bits: 8 }),
            length: 4,
        };
        assert_eq!(io_type.bit_count(), 32); // 4 elements * 8 bits each

        let io_type = IoType::Array {
            element_type: Box::new(IoType::Boolean),
            length: 16,
        };
        assert_eq!(io_type.bit_count(), 16); // 16 booleans
    }

    #[test]
    fn test_matrix_bit_count() {
        let io_type = IoType::Matrix {
            element_type: Box::new(IoType::UnsignedInt { bits: 4 }),
            rows: 3,
            cols: 4,
        };
        assert_eq!(io_type.bit_count(), 48); // 3*4 = 12 elements * 4 bits each

        let io_type = IoType::Matrix {
            element_type: Box::new(IoType::Float32),
            rows: 2,
            cols: 2,
        };
        assert_eq!(io_type.bit_count(), 128); // 4 floats * 32 bits each
    }

    #[test]
    fn test_pixel_buffer_bit_count() {
        let io_type = IoType::PixelBuffer {
            width: 16,
            height: 16,
            bits_per_pixel: 4,
        };
        assert_eq!(io_type.bit_count(), 1024); // 16*16 = 256 pixels * 4 bits each

        let io_type = IoType::PixelBuffer {
            width: 8,
            height: 8,
            bits_per_pixel: 1,
        };
        assert_eq!(io_type.bit_count(), 64); // 64 pixels * 1 bit (monochrome)
    }

    #[test]
    fn test_struct_bit_count() {
        let io_type = IoType::Struct {
            fields: vec![
                ("x".to_string(), IoType::UnsignedInt { bits: 16 }),
                ("y".to_string(), IoType::UnsignedInt { bits: 16 }),
                ("active".to_string(), IoType::Boolean),
            ],
        };
        assert_eq!(io_type.bit_count(), 33); // 16 + 16 + 1
    }

    #[test]
    fn test_bit_array_bit_count() {
        let io_type = IoType::BitArray { bits: 100 };
        assert_eq!(io_type.bit_count(), 100);
    }

    #[test]
    fn test_bit_array_conversion() {
        let io_type = IoType::BitArray { bits: 8 };

        // Test encoding
        let bits_in = vec![true, false, true, false, true, false, true, false];
        let value = Value::BitArray(bits_in.clone());
        let bits_out = io_type.to_binary(&value).unwrap();
        assert_eq!(bits_out, bits_in);

        // Test decoding
        let decoded = io_type.from_binary(&bits_out).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_nested_array() {
        // Array of arrays: [[u8; 2]; 3]
        let io_type = IoType::Array {
            element_type: Box::new(IoType::Array {
                element_type: Box::new(IoType::UnsignedInt { bits: 8 }),
                length: 2,
            }),
            length: 3,
        };

        // Total: 3 arrays * 2 elements * 8 bits = 48 bits
        assert_eq!(io_type.bit_count(), 48);
    }

    #[test]
    fn test_complex_struct() {
        // Struct with mixed types
        let io_type = IoType::Struct {
            fields: vec![
                ("id".to_string(), IoType::UnsignedInt { bits: 32 }),
                ("name".to_string(), IoType::Ascii { chars: 16 }),
                (
                    "position".to_string(),
                    IoType::Struct {
                        fields: vec![
                            ("x".to_string(), IoType::Float32),
                            ("y".to_string(), IoType::Float32),
                            ("z".to_string(), IoType::Float32),
                        ],
                    },
                ),
                ("flags".to_string(), IoType::BitArray { bits: 8 }),
            ],
        };

        // Total: 32 + (16*8) + (3*32) + 8 = 32 + 128 + 96 + 8 = 264 bits
        assert_eq!(io_type.bit_count(), 264);
    }
}
