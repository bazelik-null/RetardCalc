use crate::core::shared::types::Type;
use std::str::FromStr;

/// Built-in functions for syscalls
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysCallId {
    // I/O
    Print = 0x0,
    Println = 0x1,
    Input = 0x2,
    // Casts
    Int = 0x3,
    Float = 0x4,
    String = 0x5,
    Bool = 0x6,
}

impl FromStr for SysCallId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "print" => Ok(SysCallId::Print),
            "println" => Ok(SysCallId::Println),
            "input" => Ok(SysCallId::Input),
            "int" => Ok(SysCallId::Int),
            "float" => Ok(SysCallId::Float),
            "string" => Ok(SysCallId::String),
            "bool" => Ok(SysCallId::Bool),
            _ => Err(()),
        }
    }
}

impl SysCallId {
    pub fn from_u8(byte: u8) -> Result<Self, String> {
        match byte {
            0x0 => Ok(SysCallId::Print),
            0x1 => Ok(SysCallId::Println),
            0x2 => Ok(SysCallId::Input),
            0x3 => Ok(SysCallId::Int),
            0x4 => Ok(SysCallId::Float),
            0x5 => Ok(SysCallId::String),
            0x6 => Ok(SysCallId::Bool),
            _ => Err(format!("Invalid opcode: 0x{:02X}", byte)),
        }
    }

    pub fn get_return_type(&self) -> Type {
        match self {
            SysCallId::Print => Type::Void,
            SysCallId::Println => Type::Void,
            SysCallId::Input => Type::String,
            SysCallId::Int => Type::Integer,
            SysCallId::Float => Type::Float,
            SysCallId::String => Type::String,
            SysCallId::Bool => Type::Boolean,
        }
    }
}
