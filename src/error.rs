use thiserror::Error;

/// BPF parsing errors
#[derive(Error, Debug)]
pub enum BpfParseError {
    #[error("Program too large: {size} bytes (max: {max_size})")]
    ProgramTooLarge { size: usize, max_size: usize },
    
    #[error("Unexpected end of input at offset {offset}")]
    UnexpectedEndOfInput { offset: usize },
    
    #[error("Invalid opcode: {opcode}")]
    InvalidOpcode { opcode: u8 },
    
    #[error("Invalid register index: {register}")]
    InvalidRegister { register: u8 },
    
    #[error("Invalid instruction format at offset {offset}")]
    InvalidInstructionFormat { offset: usize },
}

/// BPF interpreter errors
#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Invalid register: {register}")]
    InvalidRegister { register: u8 },
    
    #[error("Memory access violation at address {address} (size: {size}, max: {max_address})")]
    MemoryAccessViolation { address: usize, size: usize, max_address: usize },
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Unsupported opcode: {opcode}")]
    UnsupportedOpcode { opcode: u8 },
    
    #[error("Execution limit exceeded (max: 100,000 instructions)")]
    ExecutionLimitExceeded,
    
    #[error("Invalid jump target: {target}")]
    InvalidJumpTarget { target: usize },
    
    #[error("Stack overflow")]
    StackOverflow,
    
    #[error("Stack underflow")]
    StackUnderflow,
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
    
    #[error("ZisK toolchain not found")]
    ToolchainNotFound,
    
    #[error("Project initialization failed: {message}")]
    InitializationError { message: String },
}

/// Main transpiler error type
#[derive(Error, Debug)]
pub enum TranspilerError {
    #[error("BPF parsing error: {0}")]
    BpfParseError(#[from] BpfParseError),
    
    #[error("Interpreter error: {0}")]
    InterpreterError(#[from] InterpreterError),
    
    #[error("ZisK execution error: {0}")]
    ZiskExecutionError(#[from] ZiskExecutionError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Generic error: {message}")]
    Generic { message: String },
}

impl From<String> for TranspilerError {
    fn from(message: String) -> Self {
        TranspilerError::Generic { message }
    }
}

impl From<&str> for TranspilerError {
    fn from(message: &str) -> Self {
        TranspilerError::Generic { message: message.to_string() }
    }
}
