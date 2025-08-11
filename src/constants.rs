//! ZisK Constants and Error Types
//! 
//! This module provides centralized error handling and constants
//! for the ZisK Solana integration system.

use thiserror::Error;
use solana_sdk::pubkey::Pubkey;

/// Comprehensive error types for ZisK operations
#[derive(Debug, Clone, Error)]
pub enum ZisKError {
    // BPF execution errors
    #[error("BPF execution error: {0}")]
    BpfExecutionError(String),
    
    #[error("BPF load error: {0}")]
    BpfLoadError(String),
    
    #[error("BPF verification error: {0}")]
    BpfVerificationError(String),
    
    // Memory and system errors
    #[error("Memory mapping error: {0}")]
    MemoryMappingError(String),
    
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    
    #[error("Stack overflow")]
    StackOverflow,
    
    #[error("Account data too large: {0} bytes")]
    AccountDataTooLarge(usize),
    
    #[error("Account data bounds error")]
    AccountDataBoundsError,
    
    // Transaction and account errors
    #[error("Transaction parse error: {0}")]
    TransactionParseError(String),
    
    #[error("Account parse error: {0}")]
    AccountParseError(String),
    
    #[error("Account validation error: {0}")]
    AccountValidationError(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(Pubkey),
    
    #[error("Account already exists: {0}")]
    AccountAlreadyExists(Pubkey),
    
    // State management errors
    #[error("No checkpoint available for rollback")]
    NoCheckpointAvailable,
    
    #[error("Compute budget exceeded")]
    ComputeBudgetExceeded,
    
    #[error("Insufficient rent balance: required {required}, provided {provided}")]
    InsufficientRentBalance { required: u64, provided: u64 },
    
    // Generic errors
    #[error("Generic error: {0}")]
    Generic(String),
}

/// Global cycle counter for ZisK operations
pub static mut OP_CYCLES: u64 = 0;

/// ZisK memory constraints
pub const ZISK_MAX_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64MB
pub const ZISK_MAX_STACK_SIZE: usize = 8 * 1024 * 1024;  // 8MB
pub const ZISK_MAX_ACCOUNT_DATA: usize = 10 * 1024 * 1024; // 10MB

/// Solana compute unit limits
pub const SOLANA_MAX_COMPUTE_UNITS: u64 = 1_400_000;
pub const SOLANA_MIN_COMPUTE_UNITS: u64 = 200_000;

/// BPF program limits
pub const BPF_MAX_INSTRUCTION_COUNT: u64 = 1_000_000;
pub const BPF_MAX_CALL_DEPTH: u32 = 64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = ZisKError::BpfExecutionError("Test error".to_string());
        assert!(error.to_string().contains("Test error"));
    }

    #[test]
    fn test_pubkey_error() {
        let pubkey = Pubkey::new_unique();
        let error = ZisKError::AccountNotFound(pubkey);
        assert!(error.to_string().contains("Account not found"));
    }

    #[test]
    fn test_rent_balance_error() {
        let error = ZisKError::InsufficientRentBalance { required: 1000, provided: 500 };
        assert!(error.to_string().contains("required 1000"));
        assert!(error.to_string().contains("provided 500"));
    }
}
