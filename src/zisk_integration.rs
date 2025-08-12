use crate::error::{ZiskExecutionError, TranspilerError};
use crate::ExecutionResult;
use std::process::Command;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub struct ZiskIntegration {
    project_dir: String,
    target_dir: String,
}

impl ZiskIntegration {
    pub fn new() -> Self {
        Self {
            project_dir: "zisk_bpf_project".to_string(),
            target_dir: "target/riscv64ima-zisk-zkvm-elf/release".to_string(),
        }
    }

    /// Initialize ZisK project structure
    pub fn initialize(&mut self) -> Result<(), TranspilerError> {
        // Create project directory if it doesn't exist
        if !Path::new(&self.project_dir).exists() {
            fs::create_dir_all(&self.project_dir)?;
        }

        // Create Cargo.toml for ZisK project
        let cargo_toml = r#"
[package]
name = "bpf-zisk-program"
version = "0.1.0"
edition = "2021"

[dependencies]

[[bin]]
name = "bpf_program"
path = "src/main.rs"

[target.riscv64ima-zisk-zkvm-elf]
rustflags = [
    "-C", "target-feature=+m,+a,+c",
    "-C", "link-arg=--strip-all",
]
"#;
        fs::write(format!("{}/Cargo.toml", self.project_dir), cargo_toml)?;

        // Create src directory
        fs::create_dir_all(format!("{}/src", self.project_dir))?;

        Ok(())
    }

    /// Build RISC-V assembly into ZisK ELF binary
    pub fn build_elf(&self, riscv_assembly: &str) -> Result<String, TranspilerError> {
        // Write RISC-V assembly to main.rs as pure Rust code
        let main_rs = format!(r#"
#![no_main]

#[no_mangle]
pub extern "C" fn main() -> i32 {{
    // RISC-V assembly converted to Rust
    {}
    
    // Return success
    0
}}
"#, riscv_assembly);

        fs::write(format!("{}/src/main.rs", self.project_dir), main_rs)?;

        // Build using cargo-zisk with the correct toolchain
        let output = Command::new("cargo-zisk")
            .args(&["build", "--release"])
            .current_dir(&self.project_dir)
            .env("PATH", format!("{}:{}", std::env::var("PATH").unwrap_or_default(), "~/.zisk/bin"))
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::BuildError {
                message: format!("Failed to run cargo-zisk: {}", e),
            }))?;

        if !output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::BuildError {
                message: format!("Build failed: {}", String::from_utf8_lossy(&output.stderr)),
            }));
        }

        Ok(format!("{}/{}", self.project_dir, self.target_dir))
    }

    /// Execute BPF program in ZisK emulator
    pub fn execute(&self, riscv_assembly: &str) -> Result<ExecutionResult, TranspilerError> {
        // Build ELF first
        let elf_path = self.build_elf(riscv_assembly)?;
        let elf_name = "bpf_program";

        // Debug: Print the actual path being used
        let full_elf_path = format!("{}/{}", elf_path, elf_name);
        println!("🔍 Debug: Looking for ELF at: {}", full_elf_path);
        println!("🔍 Debug: Current directory: {}", std::env::current_dir().unwrap().display());

        // Execute with ZisK emulator - use relative path from project directory
        let output = Command::new("ziskemu")
            .args(&["-e", "target/riscv64ima-zisk-zkvm-elf/release/bpf_program"])
            .current_dir(&self.project_dir)
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ExecutionError {
                message: format!("Failed to run ziskemu: {}", e),
            }))?;

        if !output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ExecutionError {
                message: format!("Execution failed: {}", String::from_utf8_lossy(&output.stderr)),
            }));
        }

        // Parse output to get execution result
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        Ok(ExecutionResult {
            exit_code: output.status.code().unwrap_or(0) as u64,
            registers: [0; 11], // Will be enhanced to parse actual register values
            instructions_executed: 0, // Will be enhanced to parse actual cycle count
            execution_time: std::time::Duration::from_millis(100), // Placeholder
        })
    }

    /// Generate cryptographic proof using ZisK
    pub fn generate_proof(&self, riscv_assembly: &str) -> Result<Vec<u8>, TranspilerError> {
        // Build ELF first
        let elf_path = self.build_elf(riscv_assembly)?;
        let elf_name = "bpf_program";

        // Generate ROM setup
        let setup_output = Command::new("cargo-zisk")
            .args(&["rom-setup", "-e", &format!("{}/{}", elf_path, elf_name)])
            .current_dir(&self.project_dir)
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Failed to run rom-setup: {}", e),
            }))?;

        if !setup_output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("ROM setup failed: {}", String::from_utf8_lossy(&setup_output.stderr)),
            }));
        }

        // Generate proof
        let proof_output = Command::new("cargo-zisk")
            .args(&["prove", "-e", &format!("{}/{}", elf_path, elf_name), "-o", "proof", "-a", "-y"])
            .current_dir(&self.project_dir)
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Failed to generate proof: {}", e),
            }))?;

        if !proof_output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Proof generation failed: {}", String::from_utf8_lossy(&proof_output.stderr)),
            }));
        }

        // Read the generated proof
        let proof_path = format!("{}/proof/vadcop_final_proof.bin", self.project_dir);
        fs::read(&proof_path).map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
            message: format!("Failed to read proof file: {}", e),
        }))
    }

    /// Execute BPF program and generate proof
    pub fn execute_with_proof(&mut self, riscv_assembly: &str) -> Result<(ExecutionResult, Vec<u8>), TranspilerError> {
        // Execute first
        let result = self.execute(riscv_assembly)?;
        
        // Generate proof
        let proof = self.generate_proof(riscv_assembly)?;
        
        Ok((result, proof))
    }

    /// Get ZisK project information
    pub fn get_info(&self) -> ZiskInfo {
        ZiskInfo {
            project_dir: self.project_dir.clone(),
            target_dir: self.target_dir.clone(),
            zisk_version: self.get_zisk_version().unwrap_or_else(|_| "Unknown".to_string()),
        }
    }

    /// Get ZisK version
    fn get_zisk_version(&self) -> Result<String, TranspilerError> {
        let output = Command::new("cargo-zisk")
            .arg("--version")
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::VersionError {
                message: format!("Failed to get ZisK version: {}", e),
            }))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Validate RISC-V code for ZisK compatibility
    pub fn validate_code(&self, riscv_assembly: &str) -> Result<(), TranspilerError> {
        // Basic validation - check for required structure
        if !riscv_assembly.contains("main") {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ValidationError {
                message: "RISC-V code must include main function".to_string(),
            }));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ZiskInfo {
    pub project_dir: String,
    pub target_dir: String,
    pub zisk_version: String,
}

impl Default for ZiskIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zisk_integration_creation() {
        let zisk = ZiskIntegration::new();
        assert_eq!(zisk.project_dir, "zisk_bpf_project");
        assert_eq!(zisk.target_dir, "target/riscv64ima-zisk-zkvm-elf/release");
    }

    #[test]
    fn test_zisk_initialization() {
        let mut zisk = ZiskIntegration::new();
        let result = zisk.initialize();
        assert!(result.is_ok());
        
        // Cleanup
        let _ = fs::remove_dir_all("zisk_bpf_project");
    }

    #[test]
    fn test_zisk_info() {
        let zisk = ZiskIntegration::new();
        let info = zisk.get_info();
        assert_eq!(info.project_dir, "zisk_bpf_project");
        assert_eq!(info.target_dir, "target/riscv64ima-zisk-zkvm-elf/release");
    }
}
