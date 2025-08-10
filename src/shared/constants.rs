// ZisK zkVM Configuration Constants
// Shared between host and guest components

/// Maximum cycles allowed per transaction
pub const MAX_CYCLES: u32 = 1_000_000;

/// Maximum memory usage in bytes
pub const MAX_MEMORY: usize = 1024 * 1024; // 1MB

/// Maximum number of accounts per transaction
pub const MAX_ACCOUNTS: usize = 64;

/// Maximum account data size in bytes
pub const MAX_ACCOUNT_DATA: usize = 1024 * 1024; // 1MB

/// ZisK memory layout constants
pub const HEAP_SIZE: usize = 64 * 1024; // 64KB
pub const STACK_SIZE: usize = 8 * 1024; // 8KB

/// Input/output buffer sizes
pub const INPUT_BUFFER_SIZE: usize = 1024 * 1024; // 1MB
pub const OUTPUT_BUFFER_SIZE: usize = 32; // 32 bytes for state root

/// Cycle costs for different operations
pub const OP_CYCLES: [u32; 256] = {
    let mut costs = [1; 256]; // Default cost
    
    // Load operations
    costs[0x30] = 2; // LdAbsB
    costs[0x28] = 2; // LdAbsH
    costs[0x20] = 2; // LdAbsW
    costs[0x18] = 2; // LdAbsDw
    costs[0x50] = 3; // LdIndB
    costs[0x48] = 3; // LdIndH
    costs[0x40] = 3; // LdIndW
    costs[0x38] = 3; // LdIndDw
    
    // Store operations
    costs[0x62] = 2; // StReg
    costs[0x63] = 2; // StRegImm
    
    // Arithmetic operations
    costs[0x0F] = 1; // AddReg
    costs[0x1F] = 1; // SubReg
    costs[0x2F] = 2; // MulReg
    costs[0x3F] = 4; // DivReg
    costs[0x07] = 1; // AddImm
    costs[0x17] = 1; // SubImm
    costs[0x27] = 2; // MulImm
    costs[0x37] = 4; // DivImm
    
    // Bitwise operations
    costs[0x5F] = 1; // AndReg
    costs[0x6F] = 1; // OrReg
    costs[0x7F] = 1; // XorReg
    costs[0x6C] = 1; // LshReg
    costs[0x7C] = 1; // RshReg
    costs[0x54] = 1; // AndImm
    costs[0x64] = 1; // OrImm
    costs[0x74] = 1; // XorImm
    costs[0x6D] = 1; // LshImm
    costs[0x7D] = 1; // RshImm
    
    // Comparison and jumps
    costs[0x1D] = 1; // JeqReg
    costs[0x5D] = 1; // JneReg
    costs[0x2D] = 1; // JgtReg
    costs[0x3D] = 1; // JgeReg
    costs[0xAD] = 1; // JltReg
    costs[0xBD] = 1; // JleReg
    costs[0x15] = 1; // JeqImm
    costs[0x55] = 1; // JneImm
    costs[0x25] = 1; // JgtImm
    costs[0x35] = 1; // JgeImm
    costs[0xA5] = 1; // JltImm
    costs[0xB5] = 1; // JleImm
    costs[0x05] = 1; // Ja
    
    // Control flow
    costs[0x85] = 5; // Call
    costs[0x95] = 1; // Exit
    
    // Solana-specific operations
    costs[0xE0] = 10; // SolCall
    costs[0xE1] = 2;  // SolLog
    costs[0xE2] = 1;  // SolReturn
    
    costs
};

/// ZisK-specific error codes
#[derive(Debug, Clone, PartialEq)]
pub enum ZkError {
    InsufficientCycles,
    MemoryOutOfBounds,
    InvalidInput,
    InvalidOpcode,
    StackOverflow,
    StackUnderflow,
    InvalidAccount,
    InvalidTransaction,
}

impl std::fmt::Display for ZkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkError::InsufficientCycles => write!(f, "Insufficient cycles"),
            ZkError::MemoryOutOfBounds => write!(f, "Memory access out of bounds"),
            ZkError::InvalidInput => write!(f, "Invalid input data"),
            ZkError::InvalidOpcode => write!(f, "Invalid BPF opcode"),
            ZkError::StackOverflow => write!(f, "Stack overflow"),
            ZkError::StackUnderflow => write!(f, "Stack underflow"),
            ZkError::InvalidAccount => write!(f, "Invalid account data"),
            ZkError::InvalidTransaction => write!(f, "Invalid transaction"),
        }
    }
}

impl std::error::Error for ZkError {}

/// ZisK assertion macro for safety checks
#[macro_export]
macro_rules! zk_assert {
    ($cond:expr) => {
        if !$cond {
            return Err($crate::shared::constants::ZkError::InvalidInput);
        }
    };
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}
