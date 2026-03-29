use std::fmt;

/// Error type for memory operations
#[derive(Debug)]
pub enum VmError {
    OutOfBounds {
        addr: usize,
        len: usize,
        heap_size: usize,
    },
    HeapExhausted {
        requested: usize,
        available: usize,
    },
    InvalidAllocation(usize),
    InvalidReference(usize),
    ZeroAllocation,
    StackUnderflow,
    CallStackUnderflow,
    NoActiveFrame,
    LocalOutOfBounds(usize),
    RTTITooLarge(usize),
    InvalidRTTI(String),
    TypeMismatch(&'static str, String),
    Runtime(String),
    InvalidExecutable,
}

impl VmError {
    pub fn type_mismatch(expected: &'static str, found: impl Into<String>) -> Self {
        VmError::TypeMismatch(expected, found.into())
    }
    pub fn runtime(reason: impl Into<String>) -> Self {
        VmError::Runtime(reason.into())
    }
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::OutOfBounds {
                addr,
                len,
                heap_size,
            } => {
                write!(
                    f,
                    "Heap access out of bounds: addr={}, len={}, heap_size={}",
                    addr, len, heap_size
                )
            }
            VmError::HeapExhausted {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Heap exhausted: requested {}, available {}",
                    requested, available
                )
            }
            VmError::InvalidAllocation(addr) => write!(f, "Invalid address to free: {}", addr),
            VmError::InvalidReference(addr) => write!(f, "Invalid address to reference: {}", addr),
            VmError::ZeroAllocation => write!(f, "Cannot allocate zero bytes"),
            VmError::StackUnderflow => write!(f, "Stack underflow"),
            VmError::CallStackUnderflow => write!(f, "Call stack underflow"),
            VmError::NoActiveFrame => write!(f, "No active frame"),
            VmError::LocalOutOfBounds(idx) => write!(f, "Local index out of bounds: {}", idx),
            VmError::RTTITooLarge(len) => write!(f, "RTTI too large: {} bytes (max 255)", len),
            VmError::InvalidRTTI(reason) => write!(f, "Invalid RTTI: {}", reason),
            VmError::TypeMismatch(expected, found) => {
                write!(f, "Cannot treat {} as {}", found, expected)
            }
            VmError::Runtime(err) => write!(f, "Runtime error: {}", err),
            VmError::InvalidExecutable => write!(f, "Invalid executable"),
        }
    }
}
