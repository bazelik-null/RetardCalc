//
// EXECUTABLE BINARY FORMAT
//

use crate::core::shared::bytecode::Instruction;

const MAGIC_NUMBER: [u8; 4] = [0x4D, 0x53, 0x4C, 0x45]; // MSLE
const VERSION: u16 = 0;

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub magic_number: [u8; 4],
    pub version_number: u16,
    pub instructions_size: u32,
    pub data_offset: u32,
    pub data_size: u32,
    pub entry_point: u32,
}

impl Header {
    fn new(instructions_size: usize, data_size: usize, entry_point: usize) -> Self {
        let header_size = size_of::<Header>();
        Header {
            magic_number: MAGIC_NUMBER,
            version_number: VERSION,
            instructions_size: instructions_size as u32,
            data_offset: (header_size + instructions_size) as u32,
            data_size: data_size as u32,
            entry_point: entry_point as u32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Executable {
    pub header: Header,
    pub instructions: Vec<Instruction>,
    pub data: Vec<u8>,
}

impl Executable {
    pub fn new(instructions: Vec<Instruction>, entry_point: usize, data: Vec<u8>) -> Self {
        let instructions_size = Instruction::serialized_size(&instructions);
        let data_size = data.len();

        Executable {
            header: Header::new(instructions_size, data_size, entry_point),
            instructions,
            data,
        }
    }

    /// Serialize to binary format.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Write header fields in order
        buffer.extend_from_slice(&self.header.magic_number);
        buffer.extend_from_slice(&self.header.version_number.to_le_bytes());
        buffer.extend_from_slice(&self.header.instructions_size.to_le_bytes());
        buffer.extend_from_slice(&self.header.data_offset.to_le_bytes());
        buffer.extend_from_slice(&self.header.data_size.to_le_bytes());
        buffer.extend_from_slice(&self.header.entry_point.to_le_bytes());

        // Pad to match struct size (2 bytes of padding after version_number)
        let header_size = size_of::<Header>();
        let written = 4 + 2 + 4 + 4 + 4 + 4; // 22 bytes
        buffer.extend_from_slice(&vec![0u8; header_size - written]);

        // Write instructions
        buffer.extend(Instruction::serialize(&self.instructions));

        // Write data
        buffer.extend_from_slice(&self.data);

        buffer
    }

    /// Deserialize from binary format.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        let header_size = size_of::<Header>();

        if bytes.len() < header_size {
            return Err("Binary too small for header".to_string());
        }

        // Parse header
        let magic = &bytes[0..4];
        if magic != MAGIC_NUMBER {
            return Err("Invalid magic number".to_string());
        }

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != VERSION {
            return Err(format!("Unsupported version: {}", version));
        }

        let instructions_size = u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;
        let data_offset = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
        let data_size = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
        let entry_point = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;

        // Validate
        if header_size + instructions_size > bytes.len() {
            return Err(format!(
                "Instructions section extends beyond buffer: need {} bytes, have {}",
                header_size + instructions_size,
                bytes.len()
            ));
        }

        if data_offset + data_size > bytes.len() {
            return Err(format!(
                "Data section extends beyond buffer: need {} bytes, have {}",
                data_offset + data_size,
                bytes.len()
            ));
        }

        // Parse instructions
        let instr_bytes = &bytes[header_size..header_size + instructions_size];
        let instructions = Instruction::deserialize(instr_bytes)?;

        // Parse data
        let data = bytes[data_offset..data_offset + data_size].to_vec();

        Ok(Executable {
            header: Header::new(instructions_size, data_size, entry_point),
            instructions,
            data,
        })
    }
}
