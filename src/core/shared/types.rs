use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,                      // 0x0
    Float,                        // 0x1
    Boolean,                      // 0x2
    String,                       // 0x3
    Array(Box<Type>),             // 0x4
    FixedArray(Box<Type>, usize), // 0x5
    Void,                         // 0x6
    Reference(Box<Type>),         // 0x7
    MutableReference(Box<Type>),  // 0x8
}

impl Type {
    /// Serializes the Type into an 8-byte RTTI
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        self.serialize_into(&mut bytes, 0);
        bytes
    }

    /// Recursively serialize into a buffer at a given offset
    /// Returns the number of bytes written
    fn serialize_into(&self, bytes: &mut [u8], mut offset: usize) -> usize {
        if offset >= bytes.len() {
            return 0;
        }

        let start_offset = offset;

        match self {
            Type::Integer => {
                bytes[offset] = 0x0;
                offset += 1;
            }
            Type::Float => {
                bytes[offset] = 0x1;
                offset += 1;
            }
            Type::Boolean => {
                bytes[offset] = 0x2;
                offset += 1;
            }
            Type::String => {
                bytes[offset] = 0x3;
                offset += 1;
            }
            Type::Array(inner) => {
                bytes[offset] = 0x4;
                offset += 1;
                offset += inner.serialize_into(bytes, offset);
            }
            Type::FixedArray(inner, _size) => {
                bytes[offset] = 0x5;
                offset += 1;
                // Don't store size—serialize only the element type
                offset += inner.serialize_into(bytes, offset);
            }
            Type::Void => {
                bytes[offset] = 0x6;
                offset += 1;
            }
            Type::Reference(inner) => {
                bytes[offset] = 0x7;
                offset += 1;
                offset += inner.serialize_into(bytes, offset);
            }
            Type::MutableReference(inner) => {
                bytes[offset] = 0x8;
                offset += 1;
                offset += inner.serialize_into(bytes, offset);
            }
        }

        offset - start_offset
    }

    /// Deserializes bytes back into a Type
    /// Returns (Type, bytes_consumed)
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), String> {
        if bytes.is_empty() {
            return Err("Empty byte slice".to_string());
        }

        match bytes[0] {
            0x0 => Ok((Type::Integer, 1)),
            0x1 => Ok((Type::Float, 1)),
            0x2 => Ok((Type::Boolean, 1)),
            0x3 => Ok((Type::String, 1)),
            0x4 => {
                let (inner, consumed) = Type::from_bytes(&bytes[1..])?;
                Ok((Type::Array(Box::new(inner)), 1 + consumed))
            }
            0x5 => {
                let (inner, consumed) = Type::from_bytes(&bytes[1..])?;
                // FixedArray without size—you'll need to track size separately
                Ok((Type::FixedArray(Box::new(inner), 0), 1 + consumed))
            }
            0x6 => Ok((Type::Void, 1)),
            0x7 => {
                let (inner, consumed) = Type::from_bytes(&bytes[1..])?;
                Ok((Type::Reference(Box::new(inner)), 1 + consumed))
            }
            0x8 => {
                let (inner, consumed) = Type::from_bytes(&bytes[1..])?;
                Ok((Type::MutableReference(Box::new(inner)), 1 + consumed))
            }
            _ => Err(format!("Unknown type tag: {}", bytes[0])),
        }
    }

    /// Get byte offsets where pointers are stored in serialized data
    pub fn pointer_offsets(&self) -> Vec<usize> {
        match self {
            Type::Integer | Type::Float | Type::Boolean | Type::Void => {
                vec![] // No pointers
            }
            Type::String => {
                vec![] // String is just data
            }
            Type::Reference(_) => {
                vec![0] // Reference itself is the pointer
            }
            Type::MutableReference(_) => {
                vec![0] // Reference itself is the pointer
            }
            Type::Array(_) => {
                // Array layout: [length: u32][capacity: u32][ptr: u32]
                vec![8] // Pointer at offset 8
            }
            Type::FixedArray(element_type, size) => {
                // Fixed array: elements laid out sequentially
                let element_size = element_type.size_in_bytes();
                let mut offsets = Vec::new();
                if element_type.contains_references() {
                    for i in 0..*size {
                        offsets.extend(
                            element_type
                                .pointer_offsets()
                                .iter()
                                .map(|o| i * element_size + o),
                        );
                    }
                }
                offsets
            }
        }
    }

    pub fn contains_references(&self) -> bool {
        match self {
            Type::Reference(_) => true,
            Type::MutableReference(_) => true,
            Type::Array(inner) | Type::FixedArray(inner, _) => inner.contains_references(),
            _ => false,
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        match self {
            Type::Integer => 4,
            Type::Float => 4,
            Type::Boolean => 4,
            Type::String => 0, // Variable length, stored as data not in type
            Type::Reference(_) => 4,
            Type::MutableReference(_) => 4,
            Type::Array(_) => 12, // [len:4][cap:4][ptr:4]
            Type::FixedArray(element_type, size) => element_type.size_in_bytes() * size,
            Type::Void => 0,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Boolean => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Array(inner) => write!(f, "[{}]", inner),
            Type::FixedArray(inner, size) => write!(f, "[{}: {}]", inner, size),
            Type::Void => write!(f, "void"),
            Type::Reference(inner) => write!(f, "ref {}", inner),
            Type::MutableReference(inner) => write!(f, "mut ref {}", inner),
        }
    }
}
