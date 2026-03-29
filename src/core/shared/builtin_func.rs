use crate::core::compiler::parser::tree::Type;
use std::str::FromStr;

/// Built-in functions for syscalls
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysCallId {
    Print = 0x0,
    Println = 0x1,
    Input = 0x2,
}

impl FromStr for SysCallId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "print" => Ok(SysCallId::Print),
            "println" => Ok(SysCallId::Println),
            "input" => Ok(SysCallId::Input),
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
            _ => Err(format!("Invalid opcode: 0x{:02X}", byte)),
        }
    }

    pub fn get_return_type(&self) -> Type {
        match self {
            SysCallId::Print => Type::Void,
            SysCallId::Println => Type::Void,
            SysCallId::Input => Type::String,
        }
    }
}
