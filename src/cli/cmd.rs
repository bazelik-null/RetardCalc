#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Debug,
    File(String),
    Help,
    Func,
    Exit,
    Unknown,
}

impl Command {
    pub fn from_input(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.first().map(|s| s.to_lowercase()).as_deref() {
            Some("debug") => Command::Debug,
            Some("file") => {
                if let Some(filename) = parts.get(1) {
                    Command::File(filename.to_string())
                } else {
                    Command::Unknown
                }
            }
            Some("help") => Command::Help,
            Some("func") => Command::Func,
            Some("exit") => Command::Exit,
            _ => Command::Unknown,
        }
    }
}
