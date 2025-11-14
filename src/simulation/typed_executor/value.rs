//! Value types for typed executor
//!
//! Represents the high-level values that can be passed to/from circuits.

use std::fmt;

/// A value that can be passed to or returned from a circuit
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Unsigned integer (any size up to 64 bits)
    U32(u32),
    U64(u64),

    /// Signed integer (any size up to 64 bits)
    I32(i32),
    I64(i64),

    /// 32-bit IEEE 754 float
    F32(f32),

    /// Boolean
    Bool(bool),

    /// ASCII string
    String(String),

    /// Raw bit array
    BitArray(Vec<bool>),

    /// Raw byte array (for nibbles)
    Bytes(Vec<u8>),

    /// Array of values
    Array(Vec<Value>),

    /// Struct/object with named fields
    Struct(Vec<(String, Value)>),
}

impl Value {
    /// Try to get as u32
    pub fn as_u32(&self) -> Result<u32, String> {
        match self {
            Value::U32(v) => Ok(*v),
            Value::U64(v) => {
                if *v <= u32::MAX as u64 {
                    Ok(*v as u32)
                } else {
                    Err(format!("Value {} too large for u32", v))
                }
            }
            _ => Err(format!("Cannot convert {:?} to u32", self)),
        }
    }

    /// Try to get as u64
    pub fn as_u64(&self) -> Result<u64, String> {
        match self {
            Value::U32(v) => Ok(*v as u64),
            Value::U64(v) => Ok(*v),
            _ => Err(format!("Cannot convert {:?} to u64", self)),
        }
    }

    /// Try to get as i32
    pub fn as_i32(&self) -> Result<i32, String> {
        match self {
            Value::I32(v) => Ok(*v),
            Value::I64(v) => {
                if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
                    Ok(*v as i32)
                } else {
                    Err(format!("Value {} out of range for i32", v))
                }
            }
            _ => Err(format!("Cannot convert {:?} to i32", self)),
        }
    }

    /// Try to get as i64
    pub fn as_i64(&self) -> Result<i64, String> {
        match self {
            Value::I32(v) => Ok(*v as i64),
            Value::I64(v) => Ok(*v),
            _ => Err(format!("Cannot convert {:?} to i64", self)),
        }
    }

    /// Try to get as f32
    pub fn as_f32(&self) -> Result<f32, String> {
        match self {
            Value::F32(v) => Ok(*v),
            _ => Err(format!("Cannot convert {:?} to f32", self)),
        }
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(v) => Ok(*v),
            _ => Err(format!("Cannot convert {:?} to bool", self)),
        }
    }

    /// Try to get as string
    pub fn as_str(&self) -> Result<&str, String> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(format!("Cannot convert {:?} to string", self)),
        }
    }

    /// Try to get as bit array
    pub fn as_bit_array(&self) -> Result<&Vec<bool>, String> {
        match self {
            Value::BitArray(v) => Ok(v),
            _ => Err(format!("Cannot convert {:?} to bit array", self)),
        }
    }

    /// Try to get as byte array
    pub fn as_bytes(&self) -> Result<&Vec<u8>, String> {
        match self {
            Value::Bytes(v) => Ok(v),
            _ => Err(format!("Cannot convert {:?} to bytes", self)),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
            Value::BitArray(v) => write!(f, "[{} bits]", v.len()),
            Value::Bytes(v) => write!(f, "[{} bytes]", v.len()),
            Value::Array(v) => write!(f, "[{} elements]", v.len()),
            Value::Struct(v) => write!(f, "{{ {} fields }}", v.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversions() {
        let v = Value::U32(42);
        assert_eq!(v.as_u32().unwrap(), 42);
        assert_eq!(v.as_u64().unwrap(), 42);

        let v = Value::I32(-42);
        assert_eq!(v.as_i32().unwrap(), -42);
        assert_eq!(v.as_i64().unwrap(), -42);

        let v = Value::Bool(true);
        assert_eq!(v.as_bool().unwrap(), true);

        let v = Value::String("hello".to_string());
        assert_eq!(v.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_value_conversion_errors() {
        let v = Value::Bool(true);
        assert!(v.as_u32().is_err());

        let v = Value::U64(u64::MAX);
        assert!(v.as_u32().is_err()); // Too large
    }
}
