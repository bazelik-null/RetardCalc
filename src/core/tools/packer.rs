// src/packer.rs
use crate::core::shared::executable::Executable;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Packer {
    executable: Executable,
    output_path: String,
}

impl Packer {
    pub fn new(executable: Executable, output_path: String) -> Self {
        Self {
            executable,
            output_path,
        }
    }

    /// Pack executable into a standalone binary
    /// Format: `[morsel-vm binary][serialized_executable][executable_size:u64]`
    pub fn pack(&self) -> Result<(), String> {
        let vm_binary = self.get_vm_binary()?;
        let packed = self.embed_executable(&vm_binary)?;
        self.write_executable(&packed)?;
        Ok(())
    }

    /// Get the morsel-vm binary
    fn get_vm_binary(&self) -> Result<Vec<u8>, String> {
        let vm_path = self.find_vm_binary()?;
        std::fs::read(&vm_path).map_err(|e| e.to_string())
    }

    /// Find morsel-vm binary
    fn find_vm_binary(&self) -> Result<PathBuf, String> {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = current_exe.parent().unwrap();

        // Platform-specific VM binary name
        let vm_name = if cfg!(windows) {
            "morsel-vm.exe"
        } else {
            "morsel-vm"
        };

        // 1. Same directory as morsel-packer
        let vm_path = exe_dir.join(vm_name);
        if vm_path.exists() {
            return Ok(vm_path);
        }

        // 2. Current working directory
        let vm_path = Path::new(vm_name);
        if vm_path.exists() {
            return Ok(vm_path.to_path_buf());
        }

        // 3. target/release or target/debug
        let target_dirs = vec![
            exe_dir.join("../morsel-vm"),
            exe_dir.join("../../../target/release/morsel-vm"),
            exe_dir.join("../../../target/debug/morsel-vm"),
        ];

        for path in target_dirs {
            let with_ext = if cfg!(windows) {
                PathBuf::from(format!("{}.exe", path.display()))
            } else {
                path
            };

            if with_ext.exists() {
                return Ok(with_ext);
            }
        }

        Err(format!(
            "Could not find morsel-vm binary. Searched in: {:?}, current dir, and target directories",
            exe_dir
        ))
    }

    fn embed_executable(&self, vm_binary: &[u8]) -> Result<Vec<u8>, String> {
        let mut packed = Vec::new();

        // Write the VM binary
        packed.extend_from_slice(vm_binary);

        // Serialize and append the executable
        let exe_bytes = self.executable.serialize();
        packed.extend_from_slice(&exe_bytes);

        // Write executable size at the end
        let size = exe_bytes.len() as u64;
        packed.extend_from_slice(&size.to_le_bytes());

        Ok(packed)
    }

    fn write_executable(&self, data: &[u8]) -> Result<(), String> {
        let mut file = File::create(&self.output_path).map_err(|e| e.to_string())?;
        file.write_all(data).map_err(|e| e.to_string())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&self.output_path, perms).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
