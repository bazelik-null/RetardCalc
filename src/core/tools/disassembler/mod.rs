use crate::core::shared::bytecode::Opcode;
use crate::core::shared::executable::Executable;
use colored::Colorize;
use std::collections::HashSet;

/// Disassembles an Executable into human-readable format.
pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(executable: &Executable) -> String {
        let mut output = String::new();

        output.push_str(&"\n BYTECODE DISASSEMBLY\n".bold().cyan().to_string());
        output.push_str(&"═".repeat(75).cyan().to_string());
        output.push('\n');

        Self::write_header(&mut output, executable);
        Self::write_data_section(&mut output, executable);
        Self::write_code_section(&mut output, executable);

        output.push('\n');
        output.push_str(&"═".repeat(75).cyan().to_string());
        output.push('\n');
        output.push_str(&" END OF DISASSEMBLY\n".bold().cyan().to_string());

        output
    }

    /// Collect all function entry points from CALL instructions
    fn collect_function_entries(executable: &Executable) -> HashSet<usize> {
        let mut entries = HashSet::new();

        // Entry point is always a function
        entries.insert(executable.header.entry_point as usize);

        // Scan for CALL instructions to find other function entry points
        for offset in 0..executable.instructions.len() {
            if let Some(instruction) = executable.instructions.get(offset)
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
        output.push_str(&"─".repeat(75).yellow().to_string());
        output.push('\n');
        output.push_str(&format!(
            "  {} {}\n",
            "Instructions:".bright_white(),
            executable.instructions.len().to_string().green()
        ));
        output.push_str(&format!(
            "  {} {} bytes\n",
            "Data Size:".bright_white(),
            executable.data.len().to_string().green()
        ));
        output.push_str(&format!(
            "  {} 0x{:06x}\n",
            "Entry Point:".bright_white(),
            executable.header.entry_point
        ));
        output.push('\n');
    }

    fn write_data_section(output: &mut String, executable: &Executable) {
        if executable.data.is_empty() {
            return;
        }

        output.push_str(&" DATA SECTION\n".bold().yellow().to_string());
        output.push_str(&"─".repeat(75).yellow().to_string());
        output.push('\n');

        let data = &executable.data;

        // Max visual width of hex dump (16 bytes = 47 chars)
        let max_hex_width = 47;

        output.push_str(&format!(
            "  {:<8} {:<width$} {}\n",
            "Offset".bright_white().bold(),
            "Hex Dump".bright_white().bold(),
            "ASCII".bright_white().bold(),
            width = max_hex_width
        ));

        for (i, chunk) in data.chunks(16).enumerate() {
            let offset = i * 16;

            // Build hex string with colors
            let hex_colored = chunk
                .iter()
                .map(|b| format!("{:02x}", b).bright_black().to_string())
                .collect::<Vec<_>>()
                .join(" ");

            // Calculate visual length (without ANSI codes)
            let hex_visual_len = chunk.len() * 3 - 1;

            // Pad with spaces to align ASCII column
            let hex_padded = format!(
                "{}{}",
                hex_colored,
                " ".repeat(max_hex_width - hex_visual_len + 2)
            );

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
                "  {}  {}{}\n",
                format!("0x{:04x}", offset).cyan(),
                hex_padded,
                ascii_str
            ));
        }
        output.push('\n');
    }

    fn write_code_section(output: &mut String, executable: &Executable) {
        output.push_str(&" CODE SECTION\n".bold().yellow().to_string());
        output.push_str(&"─".repeat(75).yellow().to_string());
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

        let entry_point = executable.header.entry_point as usize;
        let mut function_labels: std::collections::HashMap<usize, String> =
            std::collections::HashMap::new();

        for entry in sorted_entries.iter() {
            let is_main = *entry == entry_point;
            function_labels.insert(*entry, Self::generate_function_label(*entry, is_main));
        }

        for offset in 0..executable.instructions.len() {
            // Print function label if this address is a function entry point
            if let Some(label) = function_labels.get(&offset) {
                output.push_str(&format!("\n  {}\n", label.bold().green()));
            }

            if let Some(instruction) = executable.instructions.get(offset) {
                output.push_str(&format!(
                    "  {}  {}\n",
                    format!("0x{:06x}", offset).cyan(),
                    instruction
                ));
            }
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
