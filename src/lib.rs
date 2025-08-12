//! BPF to RISC-V Transpiler for ZisK Integration
//! 
//! This library provides a complete transpiler that converts BPF (Berkeley Packet Filter)
//! bytecode to RISC-V assembly, enabling native execution in ZisK zkVM.
//! 
//! ## Architecture
//! 
//! 1. **BPF Parser**: Parses BPF bytecode into structured instructions
//! 2. **RISC-V Generator**: Converts BPF instructions to RISC-V assembly
//! 3. **ZisK Integration**: Executes RISC-V code natively with proof generation
//! 
//! ## Benefits
//! 
//! - ✅ **Native RISC-V execution** (no interpretation overhead)
//! - ✅ **Better performance** (direct instruction execution)
//! - ✅ **True zkVM value** (native program execution + proofs)
//! - ✅ **Production-ready** (real BPF → RISC-V compilation)

pub mod bpf_parser;
pub mod riscv_generator;
pub mod zisk_integration;
pub mod types;
pub mod error;

pub use bpf_parser::BpfParser;
pub use riscv_generator::RiscvGenerator;
pub use zisk_integration::ZiskIntegration;
pub use types::*;
pub use error::*;

/// Main transpiler that converts BPF to RISC-V
pub struct BpfTranspiler {
    parser: BpfParser,
    generator: RiscvGenerator,
}

impl BpfTranspiler {
    /// Create a new BPF transpiler
    pub fn new() -> Self {
        Self {
            parser: BpfParser::new(),
            generator: RiscvGenerator::new(),
        }
    }
    
    /// Transpile BPF bytecode to RISC-V assembly
    pub fn transpile(&mut self, bpf_bytecode: &[u8]) -> Result<Vec<u8>, TranspilerError> {
        // Parse BPF bytecode
        let bpf_program = self.parser.parse(bpf_bytecode)?;
        
        // Generate RISC-V assembly
        let riscv_code = self.generator.generate(&bpf_program)?;
        
        Ok(riscv_code)
    }
    
    /// Execute BPF program directly in ZisK
    pub fn execute_in_zisk(&mut self, bpf_bytecode: &[u8]) -> Result<ExecutionResult, TranspilerError> {
        // Transpile to RISC-V assembly
        let riscv_assembly = self.transpile_to_assembly(bpf_bytecode)?;

        // Execute in ZisK
        let mut zisk = ZiskIntegration::new();
        zisk.initialize()?;
        zisk.execute(&riscv_assembly)
    }

    /// Transpile BPF to RISC-V assembly (text format)
    pub fn transpile_to_assembly(&mut self, bpf_bytecode: &[u8]) -> Result<String, TranspilerError> {
        // Parse BPF bytecode
        let bpf_program = self.parser.parse(bpf_bytecode)?;

        // Generate RISC-V program structure
        let riscv_program = self.generator.generate_program(&bpf_program)?;

        // Convert to assembly text format
        self.generator.program_to_assembly(&riscv_program)
    }

    /// Execute BPF program and generate proof in ZisK
    pub fn execute_with_proof(&mut self, bpf_bytecode: &[u8]) -> Result<(ExecutionResult, Vec<u8>), TranspilerError> {
        // Transpile to RISC-V assembly
        let riscv_assembly = self.transpile_to_assembly(bpf_bytecode)?;

        // Execute and generate proof in ZisK
        let mut zisk = ZiskIntegration::new();
        zisk.initialize()?;
        zisk.execute_with_proof(&riscv_assembly)
    }
}

/// Result of BPF program execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: u64,
    pub registers: [u64; 11],
    pub instructions_executed: usize,
    pub execution_time: std::time::Duration,
}

impl Default for BpfTranspiler {
    fn default() -> Self {
        Self::new()
    }
}
