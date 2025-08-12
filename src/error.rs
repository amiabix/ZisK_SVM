use thiserror::Error;

/// Errors that can occur during BPF to RISC-V transpilation
#[derive(Error, Debug)]
pub enum TranspilerError {
    #[error("BPF parsing error: {0}")]
    BpfParseError(#[from] BpfParseError),
    
    #[error("RISC-V generation error: {0}")]
    RiscvGenerationError(#[from] RiscvGenerationError),
    
    #[error("ZisK execution error: {0}")]
    ZiskExecutionError(#[from] ZiskExecutionError),
    
    #[error("Invalid BPF program: {message}")]
    InvalidBpfProgram { message: String },
    
    #[error("Unsupported BPF opcode: {opcode:#x}")]
    UnsupportedOpcode { opcode: u8 },
    
    #[error("Memory allocation failed: {message}")]
    MemoryError { message: String },
}

/// BPF parsing specific errors
#[derive(Error, Debug)]
pub enum BpfParseError {
    #[error("Invalid instruction format at offset {offset}: {message}")]
    InvalidInstruction { offset: usize, message: String },
    
    #[error("Unexpected end of program at offset {offset}")]
    UnexpectedEnd { offset: usize },
    
    #[error("Invalid register index: {register} (must be 0-10)")]
    InvalidRegister { register: u8 },
    
    #[error("Invalid immediate value: {value}")]
    InvalidImmediate { value: i64 },
    
    #[error("Program too large: {size} bytes (max: {max})")]
    ProgramTooLarge { size: usize, max: usize },
}

/// RISC-V generation specific errors
#[derive(Error, Debug)]
pub enum RiscvGenerationError {
    #[error("Failed to generate RISC-V for instruction: {instruction:?}")]
    InstructionGenerationFailed { instruction: String },
    
    #[error("Invalid RISC-V assembly: {message}")]
    InvalidAssembly { message: String },
    
    #[error("Label generation failed: {message}")]
    LabelError { message: String },
    
    #[error("Register allocation failed: {message}")]
    RegisterAllocationError { message: String },
}

/// ZisK execution specific errors
#[derive(Error, Debug)]
pub enum ZiskExecutionError {
    #[error("ZisK initialization failed: {message}")]
    InitializationError { message: String },
    
    #[error("RISC-V compilation failed: {message}")]
    CompilationError { message: String },
    
    #[error("Execution failed: {message}")]
    ExecutionError { message: String },
    
    #[error("Proof generation failed: {message}")]
    ProofError { message: String },
}

impl From<std::io::Error> for TranspilerError {
    fn from(err: std::io::Error) -> Self {
        TranspilerError::MemoryError {
            message: err.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for TranspilerError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        TranspilerError::BpfParseError(BpfParseError::InvalidInstruction {
            offset: 0,
            message: format!("UTF-8 error: {}", err),
        })
    }
}
