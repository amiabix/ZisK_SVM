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

/// BPF parsing errors
#[derive(Error, Debug)]
pub enum BpfParseError {
    #[error("Invalid instruction at offset {offset}: {message}")]
    InvalidInstruction { offset: usize, message: String },

    #[error("Program too large: {size} bytes (max {max_size})")]
    ProgramTooLarge { size: usize, max_size: usize },

    #[error("Invalid opcode: {opcode:#x}")]
    InvalidOpcode { opcode: u8 },

    #[error("Unexpected end of input at offset {offset}")]
    UnexpectedEndOfInput { offset: usize },
}

/// RISC-V generation errors
#[derive(Error, Debug)]
pub enum RiscvGenerationError {
    #[error("Failed to allocate register: {message}")]
    RegisterAllocationError { message: String },

    #[error("Failed to generate instruction: {instruction}")]
    InstructionGenerationFailed { instruction: String },

    #[error("Invalid immediate value: {value}")]
    InvalidImmediate { value: i64 },

    #[error("Invalid offset value: {value}")]
    InvalidOffset { value: i16 },

    #[error("Assembly failed: {message}")]
    AssemblyFailed { message: String },
}

/// ZisK execution errors
#[derive(Error, Debug)]
pub enum ZiskExecutionError {
    #[error("Build error: {message}")]
    BuildError { message: String },

    #[error("Execution error: {message}")]
    ExecutionError { message: String },

    #[error("Proof generation error: {message}")]
    ProofGenerationError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Version error: {message}")]
    VersionError { message: String },

    #[error("File I/O error: {message}")]
    FileError { message: String },
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
