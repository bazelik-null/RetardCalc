use crate::core::shared::bytecode::{Instruction, Opcode};
use crate::core::shared::executable::Executable;
use colored::Colorize;
use std::collections::HashSet;

/// Disassembles an Executable into human-readable format.
pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(executable: &Executable) -> String {
        let mut output = String::new();

        output.push_str(&"\n BYTECODE DISASSEMBLY\n".bold().cyan().to_string());
        output.push_str(&"═".repeat(60).cyan().to_string());
        output.push('\n');

        Self::write_header(&mut output, executable);
        Self::write_data_section(&mut output, executable);
        Self::write_code_section(&mut output, executable);

        output.push('\n');
        output.push_str(&"═".repeat(60).cyan().to_string());
        output.push('\n');
        output.push_str(&" END OF DISASSEMBLY\n".bold().cyan().to_string());

        output
    }

    /// Collect all function entry points from CALL instructions
    fn collect_function_entries(executable: &Executable) -> HashSet<usize> {
        let mut entries = HashSet::new();

        // Entry point is always a function
        entries.insert(executable.entry_point());

        // Scan for CALL instructions to find other function entry points
        for offset in 0..executable.instruction_count() {
            if let Some(instruction) = executable.get_instruction(offset)
                && (instruction.opcode == Opcode::CALL)
            {
                entries.insert(instruction.operand as usize);
            }
        }

        entries
    }

    /// Generate function labels for display
    fn generate_function_label(entry_point: usize, is_main: bool) -> String {
        if is_main {
            ".main".to_string()
        } else {
            format!(".func_{:x}", entry_point)
        }
    }

    fn write_header(output: &mut String, executable: &Executable) {
        output.push_str(&" FILE HEADER\n".bold().yellow().to_string());
        output.push_str(&"─".repeat(60).yellow().to_string());
        output.push('\n');
        output.push_str(&format!(
            "  {} {}\n",
            "Instructions:".bright_white(),
            executable.instruction_count().to_string().green()
        ));
        output.push_str(&format!(
            "  {} {} bytes\n",
            "Data Size:".bright_white(),
            executable.data_size().to_string().green()
        ));
        output.push_str(&format!(
            "  {} 0x{:06x}\n",
            "Entry Point:".bright_white(),
            executable.entry_point()
        ));
        output.push('\n');
    }

    fn write_data_section(output: &mut String, executable: &Executable) {
        if executable.data_size() == 0 {
            return;
        }

        output.push_str(&" DATA SECTION\n".bold().yellow().to_string());
        output.push_str(&"─".repeat(60).yellow().to_string());
        output.push('\n');

        let data = executable.data();

        // Calculate max hex dump width for alignment
        let max_hex_width = data
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .len()
            })
            .max()
            .unwrap_or(0);

        output.push_str(&format!(
            "  {:<8} {:<width$} {}\n",
            "Offset".bright_white().bold(),
            "Hex Dump".bright_white().bold(),
            "ASCII".bright_white().bold(),
            width = max_hex_width
        ));

        for (i, chunk) in data.chunks(16).enumerate() {
            let offset = i * 16;
            let hex_str = chunk
                .iter()
                .map(|b| format!("{:02x}", b).bright_black().to_string())
                .collect::<Vec<_>>()
                .join(" ");

            let ascii_str = chunk
                .iter()
                .map(|&b| {
                    if (32..=126).contains(&b) {
                        (b as char).to_string().bright_black().to_string()
                    } else {
                        ".".bright_black().to_string()
                    }
                })
                .collect::<String>();

            output.push_str(&format!(
                "  {:<8} {:<width$} {}\n",
                format!("0x{:04x}", offset).cyan(),
                hex_str,
                ascii_str,
                width = max_hex_width
            ));
        }
        output.push('\n');
    }

    fn write_code_section(output: &mut String, executable: &Executable) {
        output.push_str(&" CODE SECTION\n".bold().yellow().to_string());
        output.push_str(&"─".repeat(60).yellow().to_string());
        output.push('\n');
        output.push_str(&format!(
            "  {:<10} {}\n",
            "Address".bright_white().bold(),
            "Instruction".bright_white().bold()
        ));
        output.push('\n');

        // Build function entry point map
        let function_entries = Self::collect_function_entries(executable);
        let mut sorted_entries: Vec<usize> = function_entries.into_iter().collect();
        sorted_entries.sort();

        let entry_point = executable.entry_point();
        let mut function_labels: std::collections::HashMap<usize, String> =
            std::collections::HashMap::new();

        for entry in sorted_entries.iter() {
            let is_main = *entry == entry_point;
            function_labels.insert(*entry, Self::generate_function_label(*entry, is_main));
        }

        for offset in 0..executable.instruction_count() {
            // Print function label if this address is a function entry point
            if let Some(label) = function_labels.get(&offset) {
                output.push_str(&format!("\n  {}\n", label.bold().green()));
            }

            if let Some(instruction) = executable.get_instruction(offset) {
                let instruction_display = Self::format_instruction(&instruction);
                output.push_str(&format!(
                    "  {}  {}\n",
                    format!("0x{:06x}", offset).cyan(),
                    instruction_display
                ));
            }
        }
    }

    fn format_instruction(instruction: &Instruction) -> String {
        match instruction.opcode {
            // Stack manipulation
            Opcode::PUSH => format!("PUSH 0x{:02x}", instruction.operand),
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
            Opcode::LOAD_LOCAL => format!("LOAD.LOCAL {}", instruction.operand),
            Opcode::STORE_LOCAL => format!("STORE.LOCAL {}", instruction.operand),

            // Memory access
            Opcode::LOAD => "LOAD".to_string(),
            Opcode::STORE => "STORE".to_string(),

            // Control flow
            Opcode::JMP => format!("JMP 0x{:06x}", instruction.operand),
            Opcode::JMPT => format!("JMPT 0x{:06x}", instruction.operand),
            Opcode::JMPF => format!("JMPF 0x{:06x}", instruction.operand),
            Opcode::CALL => format!("CALL 0x{:06x}", instruction.operand),
            Opcode::RET => "RET".to_string(),

            // Miscellaneous
            Opcode::NOP => "NOP".to_string(),
            Opcode::HALT => "HALT".to_string(),
        }
    }
}

pub trait DisassembleExt {
    fn disassemble(&self) -> String;
}

impl DisassembleExt for Executable {
    fn disassemble(&self) -> String {
        Disassembler::disassemble(self)
    }
}
