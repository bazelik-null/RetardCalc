use std::fmt::{Display, Formatter};

pub type Operand = i32;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Stack manipulation
    PUSH_IMM = 0x00,       // Push i3 immediate onto stack
    PUSH_FLOAT_IMM = 0x01, // Push f32 immediate (bitcasted as i32) onto stack
    PUSH_HEAP_REF = 0x02,  // Push heap reference
    PUSH_LOCAL_REF = 0x03, // Push local reference
    POP = 0x04,            // Pop and discard top stack value
    DUP = 0x05,            // Duplicate top stack value
    SWAP = 0x06,           // Swap top two values
    ROT = 0x07,            // Rotate top 3: [a,b,c] → [c,a,b]

    // Arithmetic
    ADD = 0x10, // Pop two values, push their sum. Polymorphic, works with strings.
    SUB = 0x11, // Pop two values, push difference
    MUL = 0x12, // Pop two values, push product
    DIV = 0x13, // Pop two values, push quotient
    REM = 0x14, // Pop two values, push remainder
    POW = 0x15, // Pop two values, push power
    NEG = 0x16, // Negate top stack value

    // Logical
    AND = 0x20, // Pop two values, push bitwise AND
    OR = 0x21,  // Pop two values, push bitwise OR
    XOR = 0x22, // Pop two values, push bitwise XOR
    NOT = 0x23, // Bitwise NOT top stack value

    // Shift
    SLA = 0x30, // Shift left arithmetic
    SRA = 0x31, // Shift right arithmetic

    // Comparison
    EQ = 0x40, // Pop two values, push true if equal
    NE = 0x41, // Pop two values, push true if not equal
    LT = 0x42, // Pop two values, push true if less than
    GT = 0x43, // Pop two values, push true if greater than
    LE = 0x44, // Pop two values, push true if less or equal
    GE = 0x45, // Pop two values, push true if greater or equal

    // Memory
    LOAD = 0x50,        // Load value from memory address on stack
    STORE = 0x51,       // Store stack value to memory address
    LOAD_LOCAL = 0x52,  // Load local variable
    STORE_LOCAL = 0x53, // Store to local variable

    // Control Flow
    JMP = 0x60,  // Unconditional jump to address
    JMPT = 0x61, // Jump if top stack value is true
    JMPF = 0x62, // Jump if top stack value is false
    CALL = 0x63, // Call function at address
    RET = 0x64,  // Return from function

    // Misc
    NOP = 0xFF,     // No operation
    HALT = 0xFE,    // Stop execution
    SYSCALL = 0xFD, // Call system func
}

impl Opcode {
    /// Convert u8 to Opcode, returning error if invalid.
    pub fn from_u8(byte: u8) -> Result<Self, String> {
        match byte {
            0x00 => Ok(Opcode::PUSH_IMM),
            0x01 => Ok(Opcode::PUSH_FLOAT_IMM),
            0x02 => Ok(Opcode::PUSH_HEAP_REF),
            0x03 => Ok(Opcode::PUSH_LOCAL_REF),
            0x04 => Ok(Opcode::POP),
            0x05 => Ok(Opcode::DUP),
            0x06 => Ok(Opcode::SWAP),
            0x07 => Ok(Opcode::ROT),
            0x10 => Ok(Opcode::ADD),
            0x11 => Ok(Opcode::SUB),
            0x12 => Ok(Opcode::MUL),
            0x13 => Ok(Opcode::DIV),
            0x14 => Ok(Opcode::REM),
            0x15 => Ok(Opcode::POW),
            0x16 => Ok(Opcode::NEG),
            0x20 => Ok(Opcode::AND),
            0x21 => Ok(Opcode::OR),
            0x22 => Ok(Opcode::XOR),
            0x23 => Ok(Opcode::NOT),
            0x30 => Ok(Opcode::SLA),
            0x31 => Ok(Opcode::SRA),
            0x40 => Ok(Opcode::EQ),
            0x41 => Ok(Opcode::NE),
            0x42 => Ok(Opcode::LT),
            0x43 => Ok(Opcode::GT),
            0x44 => Ok(Opcode::LE),
            0x45 => Ok(Opcode::GE),
            0x50 => Ok(Opcode::LOAD),
            0x51 => Ok(Opcode::STORE),
            0x52 => Ok(Opcode::LOAD_LOCAL),
            0x53 => Ok(Opcode::STORE_LOCAL),
            0x60 => Ok(Opcode::JMP),
            0x61 => Ok(Opcode::JMPT),
            0x62 => Ok(Opcode::JMPF),
            0x63 => Ok(Opcode::CALL),
            0x64 => Ok(Opcode::RET),
            0xFF => Ok(Opcode::NOP),
            0xFE => Ok(Opcode::HALT),
            0xFD => Ok(Opcode::SYSCALL),
            _ => Err(format!("Invalid opcode: 0x{:02X}", byte)),
        }
    }

    /// Whether this opcode carries an operand after the opcode.
    pub fn has_operand(self) -> bool {
        matches!(
            self,
            Opcode::PUSH_IMM
                | Opcode::PUSH_FLOAT_IMM
                | Opcode::PUSH_HEAP_REF
                | Opcode::PUSH_LOCAL_REF
                | Opcode::LOAD_LOCAL
                | Opcode::STORE_LOCAL
                | Opcode::JMP
                | Opcode::JMPT
                | Opcode::JMPF
                | Opcode::CALL
                | Opcode::SYSCALL
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Operand,
}

impl Instruction {
    /// Create a new instruction with the given opcode and operand.
    pub fn new(opcode: Opcode, operand: Operand) -> Self {
        Instruction { opcode, operand }
    }

    /// Size in bytes when encoded (1 byte opcode + 4 bytes if opcode has operand).
    pub fn size(&self) -> usize {
        1 + if self.opcode.has_operand() { 4 } else { 0 }
    }

    /// f32 -> i32
    pub fn bitcast_float(value: f32) -> i32 {
        value.to_bits() as i32 // u32 -> i32
    }

    /// i32 -> f32
    pub fn bitcast_int(value: i32) -> f32 {
        let float_bits = value as u32; // i32 -> u32
        f32::from_bits(float_bits)
    }

    /// Encode instruction
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + if self.opcode.has_operand() { 4 } else { 0 });
        out.push(self.opcode as u8);
        if self.opcode.has_operand() {
            out.extend_from_slice(&self.operand.to_le_bytes());
        }
        out
    }

    /// Decode instruction from 5-byte fixed format.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), String> {
        if bytes.is_empty() {
            return Err("Empty byte slice".to_string());
        }
        let opcode = Opcode::from_u8(bytes[0])?;
        let mut consumed = 1usize;
        let operand = if opcode.has_operand() {
            if bytes.len() < consumed + 4 {
                return Err(format!(
                    "Insufficient bytes for operand: need 4, have {}",
                    bytes.len() - consumed
                ));
            }
            let b = [
                bytes[consumed],
                bytes[consumed + 1],
                bytes[consumed + 2],
                bytes[consumed + 3],
            ];
            consumed += 4;
            i32::from_le_bytes(b)
        } else {
            0
        };
        Ok((Instruction { opcode, operand }, consumed))
    }

    /// Serialize a vector of Instructions to a byte vector.
    pub fn serialize(instructions: &[Instruction]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instr in instructions {
            bytes.extend_from_slice(&instr.encode());
        }
        bytes
    }

    /// Deserialize a byte vector into a vector of Instructions.
    pub fn deserialize(mut bytes: &[u8]) -> Result<Vec<Instruction>, String> {
        let mut instructions = Vec::new();
        while !bytes.is_empty() {
            let (instr, consumed) = Instruction::decode(bytes)?;
            instructions.push(instr);
            bytes = &bytes[consumed..];
        }
        Ok(instructions)
    }

    /// Total serialized size for a slice of instructions.
    pub fn serialized_size(instructions: &[Instruction]) -> usize {
        instructions.iter().map(|instr| instr.size()).sum()
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self.opcode {
            // Stack manipulation
            Opcode::PUSH_IMM => format!("PUSH.IMM {}", self.operand),
            Opcode::PUSH_FLOAT_IMM => {
                format!("PUSH.FLOAT.IMM {}", Instruction::bitcast_int(self.operand))
            }
            Opcode::PUSH_HEAP_REF => format!("PUSH.HEAP.REF 0x{:06x}", self.operand),
            Opcode::PUSH_LOCAL_REF => format!("PUSH.LOCAL.REF 0x{:06x}", self.operand),
            Opcode::POP => "POP".to_string(),
            Opcode::DUP => "DUP".to_string(),
            Opcode::SWAP => "SWAP".to_string(),
            Opcode::ROT => "ROT".to_string(),

            // Arithmetic operations
            Opcode::ADD => "ADD".to_string(),
            Opcode::SUB => "SUB".to_string(),
            Opcode::MUL => "MUL".to_string(),
            Opcode::DIV => "DIV".to_string(),
            Opcode::REM => "REM".to_string(),
            Opcode::POW => "POW".to_string(),
            Opcode::NEG => "NEG".to_string(),

            // Logical operations
            Opcode::AND => "AND".to_string(),
            Opcode::OR => "OR".to_string(),
            Opcode::XOR => "XOR".to_string(),
            Opcode::NOT => "NOT".to_string(),

            // Bitwise shift operations
            Opcode::SLA => "SLA".to_string(),
            Opcode::SRA => "SRA".to_string(),

            // Comparison operations
            Opcode::EQ => "EQ".to_string(),
            Opcode::NE => "NE".to_string(),
            Opcode::LT => "LT".to_string(),
            Opcode::GT => "GT".to_string(),
            Opcode::LE => "LE".to_string(),
            Opcode::GE => "GE".to_string(),

            // Local variable access
            Opcode::LOAD_LOCAL => format!("LOAD.LOCAL {}", self.operand),
            Opcode::STORE_LOCAL => format!("STORE.LOCAL {}", self.operand),

            // Memory access
            Opcode::LOAD => "LOAD".to_string(),
            Opcode::STORE => "STORE".to_string(),

            // Control flow
            Opcode::JMP => format!("JMP 0x{:06x}", self.operand),
            Opcode::JMPT => format!("JMPT 0x{:06x}", self.operand),
            Opcode::JMPF => format!("JMPF 0x{:06x}", self.operand),
            Opcode::CALL => format!("CALL 0x{:06x}", self.operand),
            Opcode::RET => "RET".to_string(),

            // Miscellaneous
            Opcode::NOP => "NOP".to_string(),
            Opcode::HALT => "HALT".to_string(),
            Opcode::SYSCALL => format!("SYSCALL 0x{:02x}", self.operand),
        };
        write!(f, "{}", string)
    }
}
