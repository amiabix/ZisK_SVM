//! ZisK zkVM Integration Constants
//! 
//! This module contains constants and types used for integrating Solana programs
//! with the ZisK zero-knowledge virtual machine.





impl std::fmt::Display for ZkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkError::CycleLimitExceeded => write!(f, "Cycle limit exceeded"),
            ZkError::InvalidMemoryAccess => write!(f, "Invalid memory access"),
            ZkError::UnsupportedOperation => write!(f, "Unsupported operation"),
            ZkError::ConstraintViolation => write!(f, "ZisK constraint violation"),
        }
    }
}

impl std::error::Error for ZkError {}

/// BPF instruction cycle costs for ZisK integration
/// These values are optimized for ZisK zkVM constraints
pub const OP_CYCLES: [u32; 256] = {
    let mut cycles = [1; 256]; // Default to 1 cycle
    
    // Load/Store operations - higher cost due to memory access
    cycles[0x30] = 3; // LdAbsB
    cycles[0x28] = 3; // LdAbsH
    cycles[0x20] = 3; // LdAbsW
    cycles[0x18] = 3; // LdAbsDw
    cycles[0x50] = 4; // LdIndB
    cycles[0x48] = 4; // LdIndH
    cycles[0x40] = 4; // LdIndW
    cycles[0x38] = 4; // LdIndDw
    
    // Register operations
    cycles[0x61] = 1; // LdReg
    cycles[0x62] = 1; // StReg
    cycles[0x63] = 1; // StRegImm
    
    // Arithmetic operations
    cycles[0x0F] = 2; // AddReg
    cycles[0x1F] = 2; // SubReg
    cycles[0x2F] = 3; // MulReg
    cycles[0x3F] = 4; // DivReg
    cycles[0x9F] = 4; // ModReg
    cycles[0x07] = 2; // AddImm
    cycles[0x17] = 2; // SubImm
    cycles[0x27] = 3; // MulImm
    cycles[0x37] = 4; // DivImm
    cycles[0x97] = 4; // ModImm
    
    // Bitwise operations
    cycles[0x5F] = 1; // AndReg
    cycles[0x6F] = 1; // OrReg
    cycles[0x7F] = 1; // XorReg
    cycles[0x6C] = 2; // LshReg
    cycles[0x7C] = 2; // RshReg
    cycles[0x54] = 1; // AndImm
    cycles[0x64] = 1; // OrImm
    cycles[0x74] = 1; // XorImm
    cycles[0x6D] = 2; // LshImm
    cycles[0x7D] = 2; // RshImm
    
    // Comparison operations
    cycles[0x1D] = 1; // JeqReg
    cycles[0x5D] = 1; // JneReg
    cycles[0x2D] = 1; // JgtReg
    cycles[0x3D] = 1; // JgeReg
    cycles[0xAD] = 1; // JltReg
    cycles[0xBD] = 1; // JleReg
    cycles[0x15] = 1; // JeqImm
    cycles[0x55] = 1; // JneImm
    cycles[0x25] = 1; // JgtImm
    cycles[0x35] = 1; // JgeImm
    cycles[0xA5] = 1; // JltImm
    cycles[0xB5] = 1; // JleImm
    
    // Control flow
    cycles[0x05] = 1; // Ja
    cycles[0x85] = 2; // Call
    cycles[0x95] = 1; // Exit
    
    // Solana-specific operations
    cycles[0xE0] = 5; // SolCall
    cycles[0xE1] = 2; // SolLog
    cycles[0xE2] = 2; // SolReturn
    
    cycles
};
