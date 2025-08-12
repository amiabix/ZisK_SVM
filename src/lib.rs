//! BPF Interpreter for ZisK Integration
//! 
//! This library provides a complete BPF interpreter that runs natively in ZisK zkVM,
//! enabling direct execution of Solana BPF programs with zero-knowledge proof generation.
//! 
//! ## Architecture
//! 
//! 1. **BPF Parser**: Parses BPF bytecode into structured instructions
//! 2. **BPF Interpreter**: Executes BPF instructions natively in ZisK
//! 3. **ZisK Integration**: Runs interpreter and generates cryptographic proofs
//! 
//! ## Benefits
//! 
//! - ✅ **Full BPF compatibility** (no transpilation overhead)
//! - ✅ **Native ZisK execution** (direct interpretation in zkVM)
//! - ✅ **Complete Solana support** (all BPF instruction categories)
//! - ✅ **Production-ready** (real BPF execution + proofs)

pub mod bpf_parser;
pub mod bpf_interpreter;
pub mod zisk_integration;
pub mod types;
pub mod error;

pub use bpf_parser::BpfParser;
pub use bpf_interpreter::BpfInterpreter;
pub use zisk_integration::ZiskIntegration;
pub use types::*;
pub use error::*;

/// Main BPF interpreter for ZisK execution
pub struct BpfZiskExecutor {
    parser: BpfParser,
    interpreter: BpfInterpreter,
}

impl BpfZiskExecutor {
    /// Create a new BPF ZisK executor
    pub fn new() -> Self {
        Self {
            parser: BpfParser::new(),
            interpreter: BpfInterpreter::new(),
        }
    }
    
    /// Execute BPF program directly in ZisK
    pub fn execute_in_zisk(&mut self, bpf_bytecode: &[u8]) -> Result<ExecutionResult, TranspilerError> {
        // Parse BPF bytecode
        let bpf_program = self.parser.parse(bpf_bytecode)?;
        
        // Execute in ZisK
        let mut zisk = ZiskIntegration::new();
        zisk.initialize()?;
        zisk.execute_bpf_program(&bpf_program)
    }

    /// Execute BPF program and generate proof in ZisK
    pub fn execute_with_proof(&mut self, bpf_bytecode: &[u8]) -> Result<(ExecutionResult, Vec<u8>), TranspilerError> {
        // Parse BPF bytecode
        let bpf_program = self.parser.parse(bpf_bytecode)?;

        // Execute and generate proof in ZisK
        let mut zisk = ZiskIntegration::new();
        zisk.initialize()?;
        zisk.execute_with_proof(&bpf_program)
    }

    /// Parse BPF bytecode without execution
    pub fn parse_bpf(&self, bpf_bytecode: &[u8]) -> Result<BpfProgram, TranspilerError> {
        self.parser.parse(bpf_bytecode)
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

impl Default for BpfZiskExecutor {
    fn default() -> Self {
        Self::new()
    }
}
