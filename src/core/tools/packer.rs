use crate::core::shared::executable::Executable;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
    /// Format: `[morsel_vm binary][serialized_executable][executable_size:u64]`
    pub fn pack(&self) -> Result<(), String> {
        let vm_binary = self.get_vm_binary()?;
        let packed = self.embed_executable(&vm_binary)?;
        self.write_executable(&packed)?;
        Ok(())
    }

    /// Get the morsel_vm binary
    fn get_vm_binary(&self) -> Result<Vec<u8>, String> {
        let vm_path = self.find_vm_binary()?;
        std::fs::read(&vm_path).map_err(|e| e.to_string())
    }

    /// Find morsel_vm binary
    fn find_vm_binary(&self) -> Result<PathBuf, String> {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = current_exe.parent().ok_or("failed to get exe dir")?;

        // Decide target from output_path extension
        let wants_windows = self.output_path.to_lowercase().ends_with(".exe");
        let vm_name = if wants_windows {
            "morsel_vm.exe"
        } else {
            "morsel_vm"
        };

        // Common toolchain dirs
        let triples = [
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-linux-musl",
            "x86_64-pc-windows-gnu",
            "x86_64-pc-windows-msvc",
        ];

        // Check exe_dir relative target folders
        let exe_dir_rel = exe_dir.to_path_buf();
        let up = |levels: usize| {
            let mut p = exe_dir_rel.clone();
            for _ in 0..levels {
                p = p.join("..");
            }
            p
        };

        let mut candidates = Vec::new();

        // ../.. /target/{release,debug}
        for variant in &["release", "debug"] {
            candidates.push(up(2).join("target").join(variant).join(vm_name));
        }

        // ../.. /target/{triple}/{release,debug}
        for triple in &triples {
            for variant in &["release", "debug"] {
                candidates.push(
                    up(2)
                        .join("target")
                        .join(triple)
                        .join(variant)
                        .join(vm_name),
                );
            }
        }

        // Check
        for cand in candidates
            .into_iter()
            .map(|c| c.canonicalize().unwrap_or(c))
        {
            if cand.exists() {
                return Ok(cand);
            }
        }

        Err("Could not find morsel_vm binary.".to_string())
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
