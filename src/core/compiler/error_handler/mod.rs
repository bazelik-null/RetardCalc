use core::fmt;
use std::fmt::Formatter;

#[derive(Debug, Default, Clone)]
pub struct CompilerError {
    message: String,             // Error message
    from: String,                // From which part of the compiler
    line: u16,                   // Line with error
    column: u16,                 // Character offset from line start
    source_line: Option<String>, // Line contents
    filename: Option<String>,
}

impl CompilerError {
    pub fn new(
        message: String,
        from: String,
        line: u16,
        column: u16,
        source_line: Option<String>,
        filename: Option<String>,
    ) -> CompilerError {
        CompilerError {
            message,
            from,
            line,
            column,
            source_line,
            filename,
        }
    }

    fn display(&self) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("[ERROR]: {}: {}\n", self.from, self.message));

        // Location line
        output.push_str(&format!(
            "  --> {}:{}:{}\n",
            self.filename.as_deref().unwrap_or(""),
            self.line,
            self.column
        ));

        // Separator
        output.push_str("   |\n");

        // Source line with number
        output.push_str(&format!(
            " {} | {}\n",
            self.line,
            self.source_line.as_deref().unwrap_or("")
        ));

        // Pointer to error
        let char_pos = self.column;
        output.push_str(&format!("   |{}^ here\n", " ".repeat(char_pos as usize)));

        output
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}
