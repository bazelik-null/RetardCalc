pub struct SourceCode {
    pub filename: String,
    pub source: Vec<char>, // Input string
}

impl SourceCode {
    pub fn new(input: String, filename: String) -> Self {
        let source: Vec<char> = input.trim().chars().collect();
        Self { filename, source }
    }

    /// Returns character position in the current line
    pub fn get_column(pos: usize, line_start: usize) -> u16 {
        (pos - line_start) as u16
    }

    /// Returns line as string
    pub fn get_line(&self, line: usize) -> Option<String> {
        let mut current_line = 0;

        for (i, &c) in self.source.iter().enumerate() {
            if current_line == line {
                // Found the start of the requested line
                let end = self.source[i..]
                    .iter()
                    .position(|&ch| ch == '\n')
                    .map(|p| i + p)
                    .unwrap_or(self.source.len());

                return Some(self.source[i..end].iter().collect::<String>());
            }

            if c == '\n' {
                current_line += 1;
            }
        }

        None // Line number out of bounds
    }
}
