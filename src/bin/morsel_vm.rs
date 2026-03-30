use morsel_core::core::shared::executable::Executable;
use morsel_core::core::vm::VirtualMachine;
use std::fs;

const HEAP_SIZE: usize = 8000000; // 8MB

fn main() {
    let exe_path = std::env::current_exe().expect("Failed to get current executable path");

    match run_packed_executable(&exe_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_packed_executable(path: &std::path::Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;

    // Extract embedded executable from the binary
    let executable = extract_executable_from_packed(&data)
        .ok_or_else(|| "No embedded executable found in binary".to_string())?;

    execute(executable)
}

fn execute(executable: Executable) -> Result<(), String> {
    let mut vm = VirtualMachine::new(HEAP_SIZE);
    vm.load_executable(&executable).map_err(|e| e.to_string())?;
    vm.run().map_err(|e| e.to_string())?;
    Ok(())
}

fn extract_executable_from_packed(data: &[u8]) -> Option<Executable> {
    if data.len() < 8 {
        return None;
    }

    // Read size from last 8 bytes (little-endian)
    let size_bytes = &data[data.len() - 8..];
    let size = u64::from_le_bytes([
        size_bytes[0],
        size_bytes[1],
        size_bytes[2],
        size_bytes[3],
        size_bytes[4],
        size_bytes[5],
        size_bytes[6],
        size_bytes[7],
    ]) as usize;

    // Validate size is reasonable
    if size == 0 || size > data.len() - 8 {
        return None;
    }

    let exe_start = data.len() - 8 - size;
    let exe_bytes = &data[exe_start..data.len() - 8];

    Executable::deserialize(exe_bytes).ok()
}
