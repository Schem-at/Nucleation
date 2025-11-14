//! Comprehensive tests for the typed executor system
//!
//! Tests are organized by component:
//! - Value conversions
//! - IoType binary conversions
//! - LayoutFunction spreading/collecting
//! - IoMapping end-to-end
//! - Full executor (when implemented)

use super::*;

// =============================================================================
// VALUE TESTS
// =============================================================================

#[test]
fn test_value_u32_conversions() {
    let v = Value::U32(42);
    assert_eq!(v.as_u32().unwrap(), 42);
    assert_eq!(v.as_u64().unwrap(), 42);
    assert!(v.as_i32().is_err());
    assert!(v.as_bool().is_err());
}

#[test]
fn test_value_i32_conversions() {
    let v = Value::I32(-42);
    assert_eq!(v.as_i32().unwrap(), -42);
    assert_eq!(v.as_i64().unwrap(), -42);
    assert!(v.as_u32().is_err());
}

#[test]
fn test_value_overflow() {
    let v = Value::U64(u64::MAX);
    assert!(v.as_u32().is_err()); // Too large for u32
}

// =============================================================================
// IOTYPE TESTS
// =============================================================================

#[test]
fn test_iotype_bit_counts() {
    assert_eq!(IoType::UnsignedInt { bits: 1 }.bit_count(), 1);
    assert_eq!(IoType::UnsignedInt { bits: 8 }.bit_count(), 8);
    assert_eq!(IoType::UnsignedInt { bits: 32 }.bit_count(), 32);
    assert_eq!(IoType::SignedInt { bits: 16 }.bit_count(), 16);
    assert_eq!(IoType::Float32.bit_count(), 32);
    assert_eq!(IoType::Boolean.bit_count(), 1);
    assert_eq!(IoType::Ascii { chars: 1 }.bit_count(), 8);
    assert_eq!(IoType::Ascii { chars: 10 }.bit_count(), 80);
}

#[test]
fn test_unsigned_int_binary_conversion() {
    let io_type = IoType::UnsignedInt { bits: 8 };

    // Test 0
    let bits = io_type.to_binary(&Value::U32(0)).unwrap();
    assert_eq!(bits, vec![false; 8]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::U32(0));

    // Test 1
    let bits = io_type.to_binary(&Value::U32(1)).unwrap();
    assert_eq!(bits[0], true);
    assert_eq!(bits[1..], vec![false; 7]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::U32(1));

    // Test 255 (max for 8 bits)
    let bits = io_type.to_binary(&Value::U32(255)).unwrap();
    assert_eq!(bits, vec![true; 8]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::U32(255));

    // Test 42
    let bits = io_type.to_binary(&Value::U32(42)).unwrap();
    // 42 = 0b00101010
    assert_eq!(
        bits,
        vec![false, true, false, true, false, true, false, false]
    );
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::U32(42));
}

#[test]
fn test_signed_int_binary_conversion() {
    let io_type = IoType::SignedInt { bits: 8 };

    // Test 0
    let bits = io_type.to_binary(&Value::I32(0)).unwrap();
    assert_eq!(bits, vec![false; 8]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::I32(0));

    // Test 1
    let bits = io_type.to_binary(&Value::I32(1)).unwrap();
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::I32(1));

    // Test -1 (all bits set in two's complement)
    let bits = io_type.to_binary(&Value::I32(-1)).unwrap();
    assert_eq!(bits, vec![true; 8]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::I32(-1));

    // Test 127 (max positive for 8-bit signed)
    let bits = io_type.to_binary(&Value::I32(127)).unwrap();
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::I32(127));

    // Test -128 (min negative for 8-bit signed)
    let bits = io_type.to_binary(&Value::I32(-128)).unwrap();
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::I32(-128));

    // Test -42
    let bits = io_type.to_binary(&Value::I32(-42)).unwrap();
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::I32(-42));
}

#[test]
fn test_boolean_binary_conversion() {
    let io_type = IoType::Boolean;

    let bits = io_type.to_binary(&Value::Bool(true)).unwrap();
    assert_eq!(bits, vec![true]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::Bool(true));

    let bits = io_type.to_binary(&Value::Bool(false)).unwrap();
    assert_eq!(bits, vec![false]);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::Bool(false));
}

#[test]
fn test_ascii_binary_conversion() {
    let io_type = IoType::Ascii { chars: 5 };

    // Test "Hello"
    let bits = io_type
        .to_binary(&Value::String("Hello".to_string()))
        .unwrap();
    assert_eq!(bits.len(), 40); // 5 chars * 8 bits
    let decoded = io_type.from_binary(&bits).unwrap();
    assert_eq!(decoded, Value::String("Hello".to_string()));

    // Test shorter string (should pad)
    let bits = io_type.to_binary(&Value::String("Hi".to_string())).unwrap();
    assert_eq!(bits.len(), 40);
    let decoded = io_type.from_binary(&bits).unwrap();
    assert_eq!(decoded, Value::String("Hi".to_string()));

    // Test empty string
    let bits = io_type.to_binary(&Value::String("".to_string())).unwrap();
    assert_eq!(bits.len(), 40);
    let decoded = io_type.from_binary(&bits).unwrap();
    assert_eq!(decoded, Value::String("".to_string()));
}

#[test]
fn test_float32_binary_conversion() {
    let io_type = IoType::Float32;

    // Test 0.0
    let bits = io_type.to_binary(&Value::F32(0.0)).unwrap();
    assert_eq!(bits.len(), 32);
    assert_eq!(io_type.from_binary(&bits).unwrap(), Value::F32(0.0));

    // Test 1.0
    let bits = io_type.to_binary(&Value::F32(1.0)).unwrap();
    if let Value::F32(f) = io_type.from_binary(&bits).unwrap() {
        assert!((f - 1.0).abs() < 0.00001);
    } else {
        panic!("Expected F32");
    }

    // Test pi
    let bits = io_type.to_binary(&Value::F32(3.14159)).unwrap();
    if let Value::F32(f) = io_type.from_binary(&bits).unwrap() {
        assert!((f - 3.14159).abs() < 0.00001);
    } else {
        panic!("Expected F32");
    }
}

// =============================================================================
// LAYOUTFUNCTION TESTS
// =============================================================================

#[test]
fn test_layout_position_counts() {
    assert_eq!(LayoutFunction::OneToOne.position_count(8), 8);
    assert_eq!(LayoutFunction::OneToOne.position_count(32), 32);

    assert_eq!(LayoutFunction::Packed4.position_count(4), 1);
    assert_eq!(LayoutFunction::Packed4.position_count(8), 2);
    assert_eq!(LayoutFunction::Packed4.position_count(32), 8);
    assert_eq!(LayoutFunction::Packed4.position_count(5), 2); // Rounds up
}

#[test]
fn test_one_to_one_layout() {
    let layout = LayoutFunction::OneToOne;

    // Test all zeros
    let bits = vec![false; 8];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles, vec![0; 8]);
    assert_eq!(layout.collect_bits(&nibbles).unwrap(), bits);

    // Test all ones
    let bits = vec![true; 8];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles, vec![15; 8]);
    assert_eq!(layout.collect_bits(&nibbles).unwrap(), bits);

    // Test alternating
    let bits = vec![true, false, true, false, true, false, true, false];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles, vec![15, 0, 15, 0, 15, 0, 15, 0]);
    assert_eq!(layout.collect_bits(&nibbles).unwrap(), bits);
}

#[test]
fn test_packed4_layout() {
    let layout = LayoutFunction::Packed4;

    // Test 0x00
    let bits = vec![false; 8];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles, vec![0, 0]);

    // Test 0xFF
    let bits = vec![true; 8];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles, vec![15, 15]);

    // Test 0xAA = 0b10101010
    let bits = vec![false, true, false, true, false, true, false, true];
    let nibbles = layout.spread_bits(&bits).unwrap();
    // Low nibble: 0b1010 = 10
    // High nibble: 0b1010 = 10
    assert_eq!(nibbles, vec![10, 10]);

    // Roundtrip
    let collected = layout.collect_bits(&nibbles).unwrap();
    assert_eq!(collected[0..8], bits[..]);
}

#[test]
fn test_packed4_partial_nibble() {
    let layout = LayoutFunction::Packed4;

    // 5 bits: should produce 2 nibbles (second one partial)
    let bits = vec![true, false, true, false, true];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles.len(), 2);
    assert_eq!(nibbles[0], 0b0101); // First 4 bits
    assert_eq!(nibbles[1], 0b0001); // Last bit only
}

#[test]
fn test_custom_layout() {
    // Reverse mapping: bit 0 → pos 3, bit 1 → pos 2, bit 2 → pos 1, bit 3 → pos 0
    let layout = LayoutFunction::Custom(vec![3, 2, 1, 0]);

    let bits = vec![true, false, true, false];
    let nibbles = layout.spread_bits(&bits).unwrap();
    assert_eq!(nibbles, vec![0, 15, 0, 15]); // Reversed!

    let collected = layout.collect_bits(&nibbles).unwrap();
    assert_eq!(collected, bits);
}

// =============================================================================
// IOMAPPING TESTS
// =============================================================================

fn make_positions(count: usize) -> Vec<(i32, i32, i32)> {
    (0..count).map(|i| (i as i32, 0, 0)).collect()
}

#[test]
fn test_io_mapping_validation() {
    // Valid: 8-bit uint + OneToOne = 8 positions
    let result = IoMapping::new(
        IoType::UnsignedInt { bits: 8 },
        LayoutFunction::OneToOne,
        make_positions(8),
    );
    assert!(result.is_ok());

    // Invalid: wrong position count
    let result = IoMapping::new(
        IoType::UnsignedInt { bits: 8 },
        LayoutFunction::OneToOne,
        make_positions(4),
    );
    assert!(result.is_err());

    // Valid: 32-bit uint + Packed4 = 8 positions
    let result = IoMapping::new(
        IoType::UnsignedInt { bits: 32 },
        LayoutFunction::Packed4,
        make_positions(8),
    );
    assert!(result.is_ok());
}

#[test]
fn test_io_mapping_encode_decode_uint() {
    let mapping = IoMapping::new(
        IoType::UnsignedInt { bits: 16 },
        LayoutFunction::Packed4,
        make_positions(4),
    )
    .unwrap();

    let test_values = vec![0, 1, 255, 1000, 65535];

    for val in test_values {
        let value = Value::U32(val);
        let nibbles = mapping.encode(&value).unwrap();
        assert_eq!(nibbles.len(), 4);

        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value, "Failed for value {}", val);
    }
}

#[test]
fn test_io_mapping_encode_decode_signed() {
    let mapping = IoMapping::new(
        IoType::SignedInt { bits: 16 },
        LayoutFunction::Packed4,
        make_positions(4),
    )
    .unwrap();

    let test_values = vec![-32768, -1000, -1, 0, 1, 1000, 32767];

    for val in test_values {
        let value = Value::I32(val);
        let nibbles = mapping.encode(&value).unwrap();
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value, "Failed for value {}", val);
    }
}

#[test]
fn test_io_mapping_ascii() {
    let mapping = IoMapping::new(
        IoType::Ascii { chars: 10 },
        LayoutFunction::Packed4,
        make_positions(20), // 10 chars * 8 bits / 4 = 20 nibbles
    )
    .unwrap();

    let test_strings = vec!["Hello", "Test", "A", "1234567890"];

    for s in test_strings {
        let value = Value::String(s.to_string());
        let nibbles = mapping.encode(&value).unwrap();
        let decoded = mapping.decode(&nibbles).unwrap();
        assert_eq!(decoded, value, "Failed for string '{}'", s);
    }
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_full_pipeline_32bit_adder() {
    // Simulate a 32-bit adder circuit IO
    let input_a = IoMapping::new(
        IoType::UnsignedInt { bits: 32 },
        LayoutFunction::OneToOne,
        make_positions(32),
    )
    .unwrap();

    let input_b = IoMapping::new(
        IoType::UnsignedInt { bits: 32 },
        LayoutFunction::OneToOne,
        make_positions(32),
    )
    .unwrap();

    let output = IoMapping::new(
        IoType::UnsignedInt { bits: 32 },
        LayoutFunction::OneToOne,
        make_positions(32),
    )
    .unwrap();

    // Test encoding inputs
    let a_val = Value::U32(1000);
    let b_val = Value::U32(2000);

    let a_nibbles = input_a.encode(&a_val).unwrap();
    let b_nibbles = input_b.encode(&b_val).unwrap();

    assert_eq!(a_nibbles.len(), 32);
    assert_eq!(b_nibbles.len(), 32);

    // Simulate output (in real circuit, this would be computed)
    let result_val = Value::U32(3000);
    let result_nibbles = output.encode(&result_val).unwrap();

    // Decode output
    let decoded = output.decode(&result_nibbles).unwrap();
    assert_eq!(decoded, result_val);
}

#[test]
fn test_efficiency_comparison() {
    // Compare OneToOne vs Packed4 for same data
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

    let value = Value::U32(0xDEADBEEF);

    let nibbles1 = one_to_one.encode(&value).unwrap();
    let nibbles2 = packed4.encode(&value).unwrap();

    // Packed4 uses 4x fewer positions!
    assert_eq!(nibbles1.len(), 32);
    assert_eq!(nibbles2.len(), 8);

    // Both decode to same value
    assert_eq!(one_to_one.decode(&nibbles1).unwrap(), value);
    assert_eq!(packed4.decode(&nibbles2).unwrap(), value);
}

// =============================================================================
// EXECUTION MODE TESTS
// =============================================================================

#[test]
fn test_output_condition_equals() {
    let condition = OutputCondition::Equals(Value::U32(42));

    assert!(condition.check(&Value::U32(42)));
    assert!(!condition.check(&Value::U32(43)));
    assert!(!condition.check(&Value::Bool(true)));
}

#[test]
fn test_output_condition_not_equals() {
    let condition = OutputCondition::NotEquals(Value::U32(42));

    assert!(!condition.check(&Value::U32(42)));
    assert!(condition.check(&Value::U32(43)));
    assert!(condition.check(&Value::Bool(true)));
}

#[test]
fn test_output_condition_greater_than() {
    let condition = OutputCondition::GreaterThan(Value::U32(10));

    assert!(condition.check(&Value::U32(11)));
    assert!(condition.check(&Value::U32(100)));
    assert!(!condition.check(&Value::U32(10)));
    assert!(!condition.check(&Value::U32(9)));
    assert!(!condition.check(&Value::Bool(true))); // Type mismatch
}

#[test]
fn test_output_condition_less_than() {
    let condition = OutputCondition::LessThan(Value::I32(0));

    assert!(condition.check(&Value::I32(-1)));
    assert!(condition.check(&Value::I32(-100)));
    assert!(!condition.check(&Value::I32(0)));
    assert!(!condition.check(&Value::I32(1)));
}

#[test]
fn test_output_condition_bitwise_and() {
    // Check if bit 0 is set (mask = 0x1)
    let condition = OutputCondition::BitwiseAnd(0x1);

    assert!(condition.check(&Value::U32(1))); // 0b0001
    assert!(condition.check(&Value::U32(3))); // 0b0011
    assert!(condition.check(&Value::U32(5))); // 0b0101
    assert!(!condition.check(&Value::U32(0))); // 0b0000
    assert!(!condition.check(&Value::U32(2))); // 0b0010

    // Check if bit 7 is set (mask = 0x80)
    let condition = OutputCondition::BitwiseAnd(0x80);
    assert!(condition.check(&Value::U32(0x80)));
    assert!(condition.check(&Value::U32(0xFF)));
    assert!(!condition.check(&Value::U32(0x7F)));
}

#[test]
fn test_execution_mode_fixed_ticks() {
    let mode = ExecutionMode::FixedTicks { ticks: 100 };

    // Just verify it can be created
    match mode {
        ExecutionMode::FixedTicks { ticks } => assert_eq!(ticks, 100),
        _ => panic!("Wrong mode"),
    }
}

#[test]
fn test_execution_mode_until_condition() {
    let mode = ExecutionMode::UntilCondition {
        output_name: "done".to_string(),
        condition: OutputCondition::Equals(Value::Bool(true)),
        max_ticks: 1000,
        check_interval: 10,
    };

    match mode {
        ExecutionMode::UntilCondition {
            output_name,
            max_ticks,
            check_interval,
            ..
        } => {
            assert_eq!(output_name, "done");
            assert_eq!(max_ticks, 1000);
            assert_eq!(check_interval, 10);
        }
        _ => panic!("Wrong mode"),
    }
}

#[test]
fn test_state_mode_values() {
    assert_eq!(StateMode::Stateless, StateMode::Stateless);
    assert_ne!(StateMode::Stateless, StateMode::Stateful);
    assert_ne!(StateMode::Stateful, StateMode::Manual);
}

// =============================================================================
// IOLAYOUT BUILDER TESTS
// =============================================================================

#[test]
fn test_io_layout_builder_basic() {
    let layout = IoLayoutBuilder::new()
        .add_input(
            "a",
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        )
        .unwrap()
        .add_output(
            "result",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            make_positions(1),
        )
        .unwrap()
        .build();

    assert_eq!(layout.inputs.len(), 1);
    assert_eq!(layout.outputs.len(), 1);
    assert!(layout.get_input("a").is_some());
    assert!(layout.get_output("result").is_some());
}

#[test]
fn test_io_layout_builder_auto_inference() {
    // OneToOne inference (8 bits, 8 positions)
    let layout = IoLayoutBuilder::new()
        .add_input_auto("a", IoType::UnsignedInt { bits: 8 }, make_positions(8))
        .unwrap()
        .build();

    let mapping = layout.get_input("a").unwrap();
    assert!(matches!(mapping.layout, LayoutFunction::OneToOne));

    // Packed4 inference (8 bits, 2 positions)
    let layout = IoLayoutBuilder::new()
        .add_input_auto("b", IoType::UnsignedInt { bits: 8 }, make_positions(2))
        .unwrap()
        .build();

    let mapping = layout.get_input("b").unwrap();
    assert!(matches!(mapping.layout, LayoutFunction::Packed4));
}

#[test]
fn test_io_layout_builder_merge() {
    let builder1 = IoLayoutBuilder::new()
        .add_input(
            "a",
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        )
        .unwrap();

    let builder2 = IoLayoutBuilder::new()
        .add_input(
            "b",
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        )
        .unwrap();

    let layout = builder1.merge(builder2).unwrap().build();

    assert_eq!(layout.inputs.len(), 2);
    assert!(layout.get_input("a").is_some());
    assert!(layout.get_input("b").is_some());
}

#[test]
fn test_io_layout_builder_duplicate_error() {
    let result = IoLayoutBuilder::new()
        .add_input(
            "a",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            make_positions(1),
        )
        .unwrap()
        .add_input(
            "a", // Duplicate!
            IoType::Boolean,
            LayoutFunction::OneToOne,
            make_positions(1),
        );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Duplicate input name"));
}

#[test]
fn test_io_layout_validation() {
    let layout = IoLayoutBuilder::new()
        .add_input(
            "a",
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            make_positions(8),
        )
        .unwrap()
        .build();

    assert!(layout.validate().is_ok());
}

#[test]
fn test_io_layout_names() {
    let layout = IoLayoutBuilder::new()
        .add_input(
            "input_a",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            make_positions(1),
        )
        .unwrap()
        .add_input(
            "input_b",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            make_positions(1),
        )
        .unwrap()
        .add_output(
            "output",
            IoType::Boolean,
            LayoutFunction::OneToOne,
            make_positions(1),
        )
        .unwrap()
        .build();

    let input_names = layout.input_names();
    assert_eq!(input_names.len(), 2);
    assert!(input_names.contains(&"input_a"));
    assert!(input_names.contains(&"input_b"));

    let output_names = layout.output_names();
    assert_eq!(output_names.len(), 1);
    assert!(output_names.contains(&"output"));
}
