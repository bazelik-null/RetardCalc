use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::preprocessor::token::{LiteralValue, OperatorValue};
use crate::core::shared::builtin_func::SysCallId;
use lasso::Spur;
use std::fmt;
use std::fmt::Formatter;

pub struct ParserOutput {
    pub nodes: Vec<Node>,
    pub errors: Vec<CompilerError>,
}

impl Default for ParserOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserOutput {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            errors: Vec::new(),
        }
    }
}

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
}

impl Type {
    /// Serializes the Type into a byte vector
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.serialize_into(&mut bytes);
        bytes
    }

    /// Recursively serialize into a buffer
    fn serialize_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Type::Integer => bytes.push(0x0),
            Type::Float => bytes.push(0x1),
            Type::Boolean => bytes.push(0x2),
            Type::String => bytes.push(0x3),
            Type::Array(inner) => {
                bytes.push(0x4);
                inner.serialize_into(bytes);
            }
            Type::FixedArray(inner, size) => {
                bytes.push(0x5);
                inner.serialize_into(bytes);
                // Serialize the size as 8 bytes (u64)
                bytes.extend_from_slice(&size.to_le_bytes());
            }
            Type::Void => bytes.push(0x6),
            Type::Reference(inner) => {
                bytes.push(0x7);
                inner.serialize_into(bytes);
            }
        }
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
                if bytes.len() < 9 {
                    return Err("Not enough bytes for FixedArray size".to_string());
                }
                let (inner, consumed) = Type::from_bytes(&bytes[1..])?;
                let size_start = 1 + consumed;
                if bytes.len() < size_start + 8 {
                    return Err("Not enough bytes for FixedArray size".to_string());
                }
                let size_bytes: [u8; 8] = bytes[size_start..size_start + 8]
                    .try_into()
                    .map_err(|_| "Failed to parse size".to_string())?;
                let size = usize::from_le_bytes(size_bytes);
                Ok((Type::FixedArray(Box::new(inner), size), size_start + 8))
            }
            0x6 => Ok((Type::Void, 1)),
            0x7 => {
                let (inner, consumed) = Type::from_bytes(&bytes[1..])?;
                Ok((Type::Reference(Box::new(inner)), 1 + consumed))
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
                vec![0] // String is a single pointer at offset 0
            }
            Type::Reference(_) => {
                vec![0] // Reference itself is the pointer
            }
            Type::Array(_) => {
                // Array layout: [length: u32][capacity: u32][ptr: u64]
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
            Type::Array(inner) | Type::FixedArray(inner, _) => inner.contains_references(),
            _ => false,
        }
    }

    pub fn size_in_bytes(&self) -> usize {
        match self {
            Type::Integer => 4,
            Type::Float => 4,
            Type::Boolean => 1,
            Type::String => 8, // pointer
            Type::Reference(_) => 8,
            Type::Array(_) => 16, // [len:4][cap:4][ptr:8]
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Spur,
    pub type_annotation: Type,
}

#[derive(Debug, Clone)]
pub enum Node {
    // Expressions
    Literal(LiteralValue),
    ArrayLiteral(Vec<Node>),
    Identifier(Spur),
    Unary {
        op: OperatorValue,
        rhs: Box<Node>,
    },
    Binary {
        lhs: Box<Node>,
        op: OperatorValue,
        rhs: Box<Node>,
    },
    Assignment {
        target: Box<Node>,
        value: Box<Node>,
    },

    // Statements
    Block(Vec<Node>),
    If {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    While {
        condition: Box<Node>,
        body: Box<Node>,
    },
    VariableDecl {
        name: Spur,
        mutable: bool,
        type_annotation: Option<Type>,
        value: Box<Node>,
    },
    FunctionDecl {
        name: Spur,
        params: Vec<Parameter>,
        body: Box<Node>,
        return_type: Option<Type>,
    },
    FunctionCall {
        name: Box<Node>,
        args: Vec<Node>,
    },
    SysCall {
        id: SysCallId,
        args: Vec<Node>,
    },
    ArrayAccess {
        array: Box<Node>,
        index: Box<Node>,
    },
    Return(Option<Box<Node>>),
}
