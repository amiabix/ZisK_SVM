use crate::error::{ZiskExecutionError, TranspilerError};
use crate::types::BpfProgram;
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
name = "bpf-zisk-interpreter"
version = "0.1.0"
edition = "2021"

[dependencies]

[[bin]]
name = "bpf_interpreter"
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

    /// Generate Rust code for BPF interpreter in ZisK
    fn generate_interpreter_code(&self, bpf_program: &BpfProgram) -> Result<String, TranspilerError> {
        let mut code = String::new();
        
        // Add header
        code.push_str(r#"#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// BPF register structure
struct BpfRegisters {
    r0: u64,
    r1: u64,
    r2: u64,
    r3: u64,
    r4: u64,
    r5: u64,
    r6: u64,
    r7: u64,
    r8: u64,
    r9: u64,
    r10: u64,
}

impl BpfRegisters {
    fn new() -> Self {
        Self {
            r0: 0, r1: 0, r2: 0, r3: 0, r4: 0,
            r5: 0, r6: 0, r7: 0, r8: 0, r9: 0, r10: 0,
        }
    }
    
    fn get(&self, reg: u8) -> u64 {
        match reg {
            0 => self.r0, 1 => self.r1, 2 => self.r2, 3 => self.r3, 4 => self.r4,
            5 => self.r5, 6 => self.r6, 7 => self.r7, 8 => self.r8, 9 => self.r9,
            10 => self.r10,
            _ => 0,
        }
    }
    
    fn set(&mut self, reg: u8, value: u64) {
        match reg {
            0 => self.r0 = value, 1 => self.r1 = value, 2 => self.r2 = value,
            3 => self.r3 = value, 4 => self.r4 = value, 5 => self.r5 = value,
            6 => self.r6 = value, 7 => self.r7 = value, 8 => self.r8 = value,
            9 => self.r9 = value, 10 => self.r10 = value,
            _ => {},
        }
    }
}

// Memory space for BPF operations
static mut MEMORY: [u8; 1024] = [0; 1024];

#[no_mangle]
pub extern "C" fn main() -> i32 {
    let mut registers = BpfRegisters::new();
    let mut pc = 0;
    
    // BPF program execution
    "#);

        // Add BPF program execution logic
        code.push_str(&format!("
    // Program has {} instructions
    let program_size = {};
    
    while pc < program_size {{
        match pc {{", bpf_program.instructions.len(), bpf_program.instructions.len()));

        // Generate instruction execution for each instruction
        for (i, instruction) in bpf_program.instructions.iter().enumerate() {
            code.push_str(&format!("\n        {} => {{", i));
            
            match instruction.opcode {
                crate::types::BpfOpcode::Mov64Imm => {
                    code.push_str(&format!(
                        "registers.set({}, {});",
                        instruction.dst_reg, instruction.immediate
                    ));
                }
                crate::types::BpfOpcode::Add64Imm => {
                    code.push_str(&format!(
                        "registers.set({}, registers.get({}) + {});",
                        instruction.dst_reg, instruction.dst_reg, instruction.immediate
                    ));
                }
                crate::types::BpfOpcode::Exit => {
                    code.push_str("return registers.r0 as i32;");
                }
                _ => {
                    code.push_str(&format!(
                        "// TODO: Implement {:?}",
                        instruction.opcode
                    ));
                }
            }
            
            code.push_str("\n        }");
        }

        // Add default case to handle all other PC values
        code.push_str(r#"
        _ => {
            // Invalid program counter - this should not happen
            return -1;
        }
        }
        pc += 1;
    }
    
    // Return success if no exit instruction
    0
}
"#);

        Ok(code)
    }

    /// Build BPF interpreter into ZisK ELF binary
    pub fn build_interpreter(&self, bpf_program: &BpfProgram) -> Result<String, TranspilerError> {
        // Generate Rust code for the BPF interpreter
        let main_rs = self.generate_interpreter_code(bpf_program)?;

        fs::write(format!("{}/src/main.rs", self.project_dir), main_rs)?;

        // Build using cargo-zisk
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
    pub fn execute_bpf_program(&self, bpf_program: &BpfProgram) -> Result<ExecutionResult, TranspilerError> {
        // Build interpreter first
        let elf_path = self.build_interpreter(bpf_program)?;
        let elf_name = "bpf_interpreter";

        // Debug: Print the actual path being used
        let full_elf_path = format!("{}/{}", elf_path, elf_name);
        println!("🔍 Debug: Looking for ELF at: {}", full_elf_path);

        // Check if ELF file exists
        if !Path::new(&full_elf_path).exists() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ExecutionError {
                message: format!("ELF file not found at: {}", full_elf_path),
            }));
        }

        // Execute in ZisK emulator
        let start_time = Instant::now();
        let output = Command::new("ziskemu")
            .args(&["-e", elf_name])
            .current_dir(&self.project_dir)
            .env("PATH", format!("{}:{}", std::env::var("PATH").unwrap_or_default(), "~/.zisk/bin"))
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ExecutionError {
                message: format!("Failed to run ziskemu: {}", e),
            }))?;

        let execution_time = start_time.elapsed();

        if !output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ExecutionError {
                message: format!("Execution failed: {}", String::from_utf8_lossy(&output.stderr)),
            }));
        }

        // Parse exit code from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let exit_code = stdout.trim().parse::<u64>().unwrap_or(0);

        Ok(ExecutionResult {
            exit_code,
            registers: [0; 11], // TODO: Extract actual register values
            instructions_executed: bpf_program.instructions.len(),
            execution_time,
        })
    }

    /// Execute BPF program and generate proof in ZisK
    pub fn execute_with_proof(&self, bpf_program: &BpfProgram) -> Result<(ExecutionResult, Vec<u8>), TranspilerError> {
        // Build interpreter first
        let elf_path = self.build_interpreter(bpf_program)?;
        let elf_name = "bpf_interpreter";

        // Generate ROM setup
        let rom_output = Command::new("cargo-zisk")
            .args(&["rom-setup", "-e", elf_name])
            .current_dir(&self.project_dir)
            .env("PATH", format!("{}:{}", std::env::var("PATH").unwrap_or_default(), "~/.zisk/bin"))
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Failed to run cargo-zisk rom-setup: {}", e),
            }))?;

        if !rom_output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("ROM setup failed: {}", String::from_utf8_lossy(&rom_output.stderr)),
            }));
        }

        // Generate proof
        let proof_output = Command::new("cargo-zisk")
            .args(&["prove", "-e", elf_name, "-o", "proof", "-a", "-y"])
            .current_dir(&self.project_dir)
            .env("PATH", format!("{}:{}", std::env::var("PATH").unwrap_or_default(), "~/.zisk/bin"))
            .output()
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Failed to run cargo-zisk prove: {}", e),
            }))?;

        if !proof_output.status.success() {
            return Err(TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Proof generation failed: {}", String::from_utf8_lossy(&proof_output.stderr)),
            }));
        }

        // Read generated proof
        let proof_path = format!("{}/proof/vadcop_final_proof.bin", self.project_dir);
        let proof = fs::read(&proof_path)
            .map_err(|e| TranspilerError::ZiskExecutionError(ZiskExecutionError::ProofGenerationError {
                message: format!("Failed to read proof file: {}", e),
            }))?;

        // Execute program to get result
        let result = self.execute_bpf_program(bpf_program)?;

        Ok((result, proof))
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
